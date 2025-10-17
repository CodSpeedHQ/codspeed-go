pub mod discovery;
pub mod patcher;
pub mod templater;

pub use discovery::*;
use thiserror::Error;

use crate::prelude::*;
use std::{path::Path, process::Command};

#[derive(Error, Debug)]
pub enum BuildError {
    #[error(
        "Function {function} in '{file_line}' ('{package}') is using testing.TB which is not supported because CodSpeed \
            uses a custom 'testing' package. Please modify the file to not use 'testing.TB' or contact support if you need help."
    )]
    TestingTBUsage {
        package: String,
        file_line: String,
        function: String,
    },
}

/// Check if the output contained an error about using testing.TB. Find those lines:
/// ```no_run,ignore
/// Build command error output: # go.opentelemetry.io/otel/internal/global
/// ./benchmark_codspeed.go:14:15: cannot use b (variable of type *codspeed.B) as "testing".TB value in argument to ResetForTest: *codspeed.B does not implement "testing".TB (unexported method private)
/// ```
///
/// Returns (package_name, filename:line_number, func)
fn parse_testing_type_error(stderr: &str) -> Option<(String, String, String)> {
    use regex::Regex;
    use std::sync::LazyLock;

    // Static regexes compiled once
    static PACKAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^# (.+)$").unwrap());

    // Regex to match the error line with codspeed.B and testing.B/TB
    // - ^\.\/                      - Starts with ./ (current directory prefix)
    // - ([^:]+)                    - Capture group 1: Filename (everything until first colon)
    // - :(\d+):                    - Capture group 2: Line number between colons
    // - value in argument to (\w+) - Capture group 3: Function name that caused the error
    //
    // Example: ./fib_codspeed.go:11:14: cannot use b (variable of type *codspeed.B) as *"testing".B value in argument to Whatever
    static ERROR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"^\./([^:]+):(\d+):\d+: cannot use \w+ \(variable of type \*codspeed\.B\) as (?:\*"testing"\.B|"testing"\.TB) value in argument to (\w+)"#
        ).unwrap()
    });

    let mut it = stderr.lines();
    loop {
        let Some(line) = it.next() else {
            break;
        };

        // Look for package line
        if let Some(captures) = PACKAGE_REGEX.captures(line.trim()) {
            let Some(package_name) = captures.get(1).map(|m| m.as_str().to_string()) else {
                continue;
            };

            // Look for error line
            let Some(line) = it.next() else {
                continue;
            };
            if let Some(captures) = ERROR_REGEX.captures(line.trim()) {
                let filename = captures
                    .get(1)?
                    .as_str()
                    .replace("_codspeed.go", "_test.go");
                let line_number = captures.get(2)?.as_str();
                let function = captures.get(3)?.as_str();

                let file_line = format!("{filename}:{line_number}");

                return Some((package_name, file_line, function.to_string()));
            }
        }
    }

    None
}

/// Builds a Go runner file into an executable binary
pub fn build_binary<P: AsRef<Path>>(runner_go_path: P) -> anyhow::Result<std::path::PathBuf> {
    let runner_go_path = runner_go_path.as_ref();
    let file_dir = runner_go_path.parent().unwrap();
    let module_root = file_dir.parent().unwrap();

    // This will be the relative path from the module root to the codspeed directory, containing
    // the runner.go file. This is needed so that we compile _all_ the files within that package.
    //
    // This is important when we have external test packages, which are moved to the codspeed folder.
    let relative_dir_path = file_dir.strip_prefix(module_root).unwrap();

    debug!(
        "Building codspeed runner binary: {:?} (root = {:?})",
        module_root.join(relative_dir_path),
        module_root
    );

    let binary_path = runner_go_path.with_extension("bin");

    // Set the integration version in our testing library and include debug symbols
    let ldflags = format!(
        "-X github.com/CodSpeedHQ/codspeed-go/testing/capi.integrationVersion={} -s=false -w=false",
        env!("CARGO_PKG_VERSION")
    );

    // Go doesn't support absolute paths, so we have to convert it to a relative path starting
    // with `./{relative_dir_path}`.
    let dot_slash_path = {
        let p = relative_dir_path.to_str().unwrap();
        format!("./{p}")
    };
    let args = vec![
        "build",
        "-mod=mod",
        "-tags=codspeed",
        "-ldflags",
        &ldflags,
        "-o",
        binary_path.to_str().unwrap(),
        &dot_slash_path,
    ];

    let output = Command::new("go")
        .args(&args)
        .current_dir(module_root)
        .output()
        .context("Failed to execute go build command")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        warn!("Build command output: {stdout}");
        warn!("Build command error output: {stderr}");

        if let Some((package, file_line, func)) = parse_testing_type_error(&stderr) {
            bail!(BuildError::TestingTBUsage {
                package,
                file_line,
                function: func
            })
        }

        bail!(
            "Failed to build benchmark binary. Exit status: {}",
            output.status
        );
    }

    debug!("Successfully built binary: {binary_path:?}");
    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_testing_type_error_basic() {
        let stderr = r#"
# example
./fib_codspeed.go:11:14: cannot use b (variable of type *codspeed.B) as *"testing".B value in argument to Whatever
"#;

        let result = parse_testing_type_error(stderr);
        assert_eq!(
            result,
            Some((
                "example".to_string(),
                "fib_test.go:11".to_string(),
                "Whatever".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_testing_type_error_with_package() {
        let stderr = r#"
# go.opentelemetry.io/otel/internal/global
./benchmark_codspeed.go:14:15: cannot use b (variable of type *codspeed.B) as "testing".TB value in argument to ResetForTest: *codspeed.B does not implement "testing".TB (unexported method private)
"#;

        let result = parse_testing_type_error(stderr);
        assert_eq!(
            result,
            Some((
                "go.opentelemetry.io/otel/internal/global".to_string(),
                "benchmark_test.go:14".to_string(),
                "ResetForTest".to_string()
            ))
        );
    }
}
