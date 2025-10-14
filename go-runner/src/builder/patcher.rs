//! Patches the imports to use codspeed rather than the official "testing" package.

use crate::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn replace_pkg<P: AsRef<Path>>(folder: P) -> anyhow::Result<()> {
    let codspeed_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let replace_arg = format!(
        "github.com/CodSpeedHQ/codspeed-go={}",
        codspeed_root.display()
    );
    debug!("Replacing codspeed-go with {}", codspeed_root.display());

    let output = Command::new("go")
        .args(["mod", "edit", "-replace", &replace_arg])
        .current_dir(folder.as_ref())
        .output()
        .context("Failed to execute 'go mod edit' command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to add replace directive: {}", stderr);
    }

    debug!("Added local replace directive to go.mod");

    Ok(())
}

pub fn patch_imports<P: AsRef<Path>>(folder: P) -> anyhow::Result<()> {
    let folder = folder.as_ref();
    debug!("Patching imports in folder: {folder:?}");

    // 1. Find all imports that match "testing" and replace them with codspeed equivalent
    let mut patched_files = 0;

    let pattern = folder.join("**/*.go");
    for go_file in glob::glob(pattern.to_str().unwrap())?.filter_map(Result::ok) {
        // Skip directories - glob can match directories ending in .go (e.g., vendor/github.com/nats-io/nats.go)
        if !go_file.is_file() {
            continue;
        }

        let content =
            fs::read_to_string(&go_file).context(format!("Failed to read Go file: {go_file:?}"))?;

        let patched_content = patch_go_source(&content)?;
        if patched_content != content {
            fs::write(&go_file, patched_content)
                .context(format!("Failed to write patched Go file: {go_file:?}"))?;

            debug!("Patched imports in: {go_file:?}");
            patched_files += 1;
        }
    }
    debug!("Patched {patched_files} files");

    // 2. Update the go module to use the codspeed package
    let version = std::env::var("CODSPEED_GO_PKG_VERSION")
        .unwrap_or_else(|_| format!("v{}", env!("CARGO_PKG_VERSION")));
    let pkg = format!("github.com/CodSpeedHQ/codspeed-go@{}", version);
    debug!("Installing {pkg}");

    let mut cmd: Command = Command::new("go");
    cmd.arg("get")
        .arg(pkg)
        // Bypass Go proxy cache to fetch directly from source - prevents issues with
        // cached versions that may have incorrect module paths or outdated content
        .env("GOPROXY", "direct")
        .current_dir(folder);

    let output = cmd.output().context("Failed to execute 'go get' command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install codspeed-go dependency: {}", stderr);
    }
    debug!("Successfully installed codspeed-go dependency");

    // Run 'go mod tidy' to resolve transitive dependencies
    let output = Command::new("go")
        .args(["mod", "tidy"])
        .current_dir(folder)
        .output()
        .context("Failed to execute 'go mod tidy' command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to run 'go mod tidy': {}", stderr);
    }
    debug!("Ran 'go mod tidy' successfully");

    // Ensure we have the latest codspeed-go package installed. Just
    // use the local one which might contain uncommitted changes.
    if std::env::var("CODSPEED_LOCAL_GO_PKG").is_ok() || cfg!(test) {
        replace_pkg(folder)?;
    }

    Ok(())
}

/// Internal function to apply import patterns to Go source code
pub fn patch_go_source(source: &str) -> anyhow::Result<String> {
    let replace_import =
        |mut source: String, import_path: &str, replacement: &str| -> anyhow::Result<String> {
            let parsed = gosyn::parse_source(&source)?;

            if let Some(import) = parsed
                .imports
                .iter()
                .find(|import| import.path.value == format!("\"{import_path}\""))
            {
                let start_pos = import.path.pos;
                let end_pos = start_pos + import.path.value.len();

                source.replace_range(start_pos..end_pos, replacement);
            }

            Ok(source)
        };

    let source = replace_package_main(source.into())?;
    let source = replace_import(
        source,
        "testing",
        "testing \"github.com/CodSpeedHQ/codspeed-go/compat/testing\"",
    )?;
    let source = replace_import(
        source,
        "github.com/thejerf/slogassert",
        "\"github.com/CodSpeedHQ/codspeed-go/pkg/slogassert\"",
    )?;
    let source = replace_import(
        source,
        "github.com/frankban/quicktest",
        "\"github.com/CodSpeedHQ/codspeed-go/pkg/quicktest\"",
    )?;

    Ok(source)
}

/// Replace `package main` with `package main_compat` to allow importing it from other packages.
fn replace_package_main(source: String) -> anyhow::Result<String> {
    let parsed = gosyn::parse_source(&source)?;

    // Only replace if package name is "main"
    if parsed.pkg_name.name != "main" {
        return Ok(source);
    }

    // pkg_name.pos is the position of the identifier "main" in the source
    let name_start = parsed.pkg_name.pos;
    let name_end = name_start + parsed.pkg_name.name.len();

    let mut result = source;
    result.replace_range(name_start..name_end, "main_compat");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use rstest::rstest;

    const SINGLE_IMPORT: &str = r#"package main

import "testing"

func TestExample(t *testing.T) {
    // test code
}
"#;

    const MULTILINE_IMPORT: &str = r#"package main

import (
    "fmt"
    "testing"
    "strings"
)

func TestExample(t *testing.T) {
    // test code
}
"#;

    const MULTILINE_IMPORT_WITH_TABS: &str = r#"package main

import (
	"fmt"
	"testing"
	"strings"
)
"#;

    const IMPORT_WITH_COMMENTS: &str = r#"package main

import (
    "fmt"
    "testing" // for unit tests
    "strings"
)
"#;

    const ALREADY_PATCHED_IMPORT: &str = r#"package main

import testing "github.com/CodSpeedHQ/codspeed-go/compat/testing"

func BenchmarkExample(b *testing.B) {
    // benchmark code
}
"#;

    const MIXED_IMPORT_STYLES: &str = r#"package main

import "testing"

import (
    "fmt"
    "something"
)
"#;

    const IMPORT_AT_END_OF_BLOCK: &str = r#"package main

import (
    "fmt"
    "strings"
    "testing"
)
"#;

    const IMPORT_WITH_EXTRA_WHITESPACE: &str = r#"package main

import (
    "fmt"

    "testing"

    "strings"
)
"#;

    const MULTILINE_IMPORT_WITH_TESTING_STRING: &str = r#"package main
import (
    "fmt"
    "testing"
)

func TestExample(t *testing.T) {
    fmt.Println("testing")
}
"#;

    const IMPORT_WITH_TESTING_STRING: &str = r#"package main
import "testing"
import "fmt"

func TestExample(t *testing.T) {
    fmt.Println("testing")
}
"#;

    const IMPORT_TESTING_AND_SLOGASSERT: &str = r#"package main
import (
    "testing"
    "fmt"
    "github.com/thejerf/slogassert"
)
"#;

    const PACKAGE_MAIN: &str = r#"package main

import "testing"

func BenchmarkExample(b *testing.B) {
    // benchmark code
}

func TestExample(t *testing.T) {
    s := "package main"
}
"#;

    #[rstest]
    #[case("single_import_replacement", SINGLE_IMPORT)]
    #[case("multiline_import_replacement", MULTILINE_IMPORT)]
    #[case("multiline_import_with_tabs", MULTILINE_IMPORT_WITH_TABS)]
    #[case("import_with_comments", IMPORT_WITH_COMMENTS)]
    #[case("already_patched_import", ALREADY_PATCHED_IMPORT)]
    #[case("mixed_import_styles", MIXED_IMPORT_STYLES)]
    #[case("import_at_end_of_block", IMPORT_AT_END_OF_BLOCK)]
    #[case("import_with_extra_whitespace", IMPORT_WITH_EXTRA_WHITESPACE)]
    #[case("import_with_testing_string", IMPORT_WITH_TESTING_STRING)]
    #[case("import_testing_and_slogassert", IMPORT_TESTING_AND_SLOGASSERT)]
    #[case(
        "multiline_import_with_testing_string",
        MULTILINE_IMPORT_WITH_TESTING_STRING
    )]
    #[case("package_main", PACKAGE_MAIN)]
    fn test_patch_go_source(#[case] test_name: &str, #[case] source: &str) {
        let result = patch_go_source(source).unwrap();
        assert_snapshot!(test_name, result);
    }
}
