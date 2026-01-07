//! Patches the imports to use codspeed rather than the official "testing" package.

use crate::prelude::*;
use itertools::Itertools;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Patcher is responsible for patching Go source files to replace imports and package names.
///
/// It also reverts all changes on drop to avoid breaking tests. This can happen when we
/// rename or move a file, which makes it available in other packages which could lead to duplicate
/// symbols or flag definitions.
pub struct Patcher {
    git_repo: PathBuf,
}

impl Patcher {
    pub fn new(git_root: &Path) -> Self {
        Self {
            git_repo: git_root.to_path_buf(),
        }
    }

    /// Replace `package main` with `package main_compat` to allow importing it from other packages.
    /// Also replace `package foo_test` with `package main` for external test packages.
    ///
    /// Returns the previous package name, or None if no replacement was made.
    fn patch_package(
        source: &mut String,
        replacement: Option<String>,
    ) -> anyhow::Result<Option<String>> {
        let parsed = gosyn::parse_source(&source)?;
        let prev_pkg_name = parsed.pkg_name.name;

        let replacement = replacement.or_else(|| {
            if prev_pkg_name == "main" {
                Some("main_compat".into())
            } else if prev_pkg_name.ends_with("_test") {
                // For external test packages (package foo_test), convert to package main
                // They will be placed in the codspeed/ subdirectory and built as standalone executables
                Some("main".into())
            } else {
                None
            }
        });

        if let Some(new_name) = replacement {
            let name_start = parsed.pkg_name.pos;
            let name_end = name_start + prev_pkg_name.len();
            source.replace_range(name_start..name_end, &new_name);

            Ok(Some(prev_pkg_name))
        } else {
            Ok(None)
        }
    }

    /// Patches imports and package in specific test files
    ///
    /// This ensures we only modify the test files that belong to the current test package,
    /// avoiding conflicts when multiple test packages exist in the same directory
    pub fn patch_packages_for_files(&mut self, files: &[PathBuf]) -> anyhow::Result<()> {
        for go_file in files {
            if !go_file.is_file() {
                continue;
            }

            let mut content = fs::read_to_string(go_file)
                .context(format!("Failed to read Go file: {go_file:?}"))?;
            if Self::patch_package(&mut content, None)?.is_some() {
                fs::write(go_file, content)
                    .context(format!("Failed to write patched Go file: {go_file:?}"))?;
            }
        }

        Ok(())
    }

    /// Patches all .go files in a directory to rename "package main" to "package main_compat"
    ///
    /// This is needed when we have a "package main" with benchmarks that need to be imported.
    /// By renaming all files in the package to "main_compat", we make it importable.
    pub fn patch_all_packages_in_dir<P: AsRef<Path>>(&mut self, dir: P) -> anyhow::Result<()> {
        self.patch_packages_for_files(
            &glob::glob(&dir.as_ref().join("*.go").to_string_lossy())?
                .filter_map(Result::ok)
                .collect::<Vec<_>>(),
        )?;

        Ok(())
    }

    fn patch_imports_for_source(source: &mut String) -> bool {
        let mut modified = false;

        // If we can't parse the source, skip this replacement
        // This can happen with template files or malformed Go code
        let parsed = match gosyn::parse_source(&source) {
            Ok(p) => p,
            Err(_) => return modified,
        };

        let mut replacements = vec![];
        let mut find_replace_range = |import_path: &str, replacement: &str| {
            // Optimization: check if the import path exists in the source before parsing
            if !source.contains(import_path) {
                return;
            }

            if let Some(import) = parsed
                .imports
                .iter()
                .find(|import| import.path.value == format!("\"{import_path}\""))
            {
                let start_pos = import.path.pos;
                let end_pos = start_pos + import.path.value.len();
                modified = true;

                replacements.push((start_pos..end_pos, replacement.to_string()));
            }
        };

        // Then replace sub-packages like "testing/synctest"
        for testing_pkg in &["fstest", "iotest", "quick", "slogtest", "synctest"] {
            find_replace_range(
                &format!("testing/{}", testing_pkg),
                &format!(
                    "{testing_pkg} \"github.com/CodSpeedHQ/codspeed-go/testing/testing/{testing_pkg}\""
                ),
            );
        }

        find_replace_range(
            "testing",
            "testing \"github.com/CodSpeedHQ/codspeed-go/testing/testing\"",
        );
        find_replace_range(
            "github.com/thejerf/slogassert",
            "\"github.com/CodSpeedHQ/codspeed-go/pkg/slogassert\"",
        );
        find_replace_range(
            "github.com/frankban/quicktest",
            "\"github.com/CodSpeedHQ/codspeed-go/pkg/quicktest\"",
        );

        // Replace logr + subpackages
        for logr_pkg in &["testr", "funcr", "slogr", "benchmark", "testing"] {
            find_replace_range(
                &format!("github.com/go-logr/logr/{}", logr_pkg),
                &format!("\"github.com/CodSpeedHQ/codspeed-go/pkg/logr/{logr_pkg}\""),
            );
        }
        find_replace_range(
            "github.com/go-logr/logr",
            "\"github.com/CodSpeedHQ/codspeed-go/pkg/logr\"",
        );
        find_replace_range(
            "github.com/go-logr/stdr",
            "\"github.com/CodSpeedHQ/codspeed-go/pkg/stdr\"",
        );

        // Replace testify + subpackages
        for testify_pkg in &["assert", "require", "mock", "suite", "http"] {
            find_replace_range(
                &format!("github.com/stretchr/testify/{}", testify_pkg),
                &format!("\"github.com/CodSpeedHQ/codspeed-go/pkg/testify/{testify_pkg}\""),
            );
        }
        find_replace_range(
            "github.com/stretchr/testify",
            "\"github.com/CodSpeedHQ/codspeed-go/pkg/testify\"",
        );

        // Apply replacements in reverse order to avoid shifting positions
        for (range, replacement) in replacements
            .into_iter()
            .sorted_by_key(|(range, _)| range.start)
            .rev()
        {
            source.replace_range(range, &replacement);
        }

        modified
    }

    pub fn patch_imports<P: AsRef<Path>>(&mut self, folder: P) -> anyhow::Result<()> {
        let folder = folder.as_ref();
        debug!("Patching imports in folder: {folder:?}");

        // 1. Find all imports that match "testing" and replace them with codspeed equivalent
        let pattern = folder.join("**/*.go");
        let patched_files = glob::glob(pattern.to_str().unwrap())?
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|go_file| {
                // Skip directories - glob can match directories ending in .go (e.g., vendor/github.com/nats-io/nats.go)
                if !go_file.is_file() {
                    return None;
                }

                let Ok(mut content) = fs::read_to_string(&go_file) else {
                    error!("Failed to read Go file: {go_file:?}");
                    return None;
                };

                if Self::patch_imports_for_source(&mut content) {
                    let Ok(_) = fs::write(&go_file, &content) else {
                        error!("Failed to write patched Go file: {go_file:?}");
                        return None;
                    };

                    debug!("Patched imports in: {go_file:?}");
                }
                Some(())
            })
            .count();
        debug!("Patched {} files", patched_files);

        Ok(())
    }

    pub fn rename_test_files<P: AsRef<Path>>(&mut self, files: &[P]) -> anyhow::Result<()> {
        for file in files {
            let src_path = file.as_ref();
            let new_path = src_path.with_file_name(
                src_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .replace("_test", "_codspeed"),
            );
            fs::rename(src_path, new_path.clone())?;
        }

        Ok(())
    }

    pub fn rename_and_move_test_files<P: AsRef<Path>>(
        &mut self,
        files: &[P],
        dst_dir: &P,
    ) -> anyhow::Result<()> {
        for src_path in files {
            let dst_filename = src_path
                .as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .replace("_test.go", "_codspeed.go");
            let dst_path = dst_dir.as_ref().join(&dst_filename);
            fs::rename(src_path.as_ref(), &dst_path)?;
        }

        Ok(())
    }
}

impl Drop for Patcher {
    fn drop(&mut self) {
        let repo_path = &self.git_repo;

        let execute_cmd = |cmd: &str| {
            debug!("Executing {cmd:?} in {repo_path:?}");

            let output = std::process::Command::new("bash")
                .args(["-c", cmd])
                .current_dir(repo_path)
                .output();
            if let Ok(output) = output {
                if !output.status.success() {
                    error!(
                        "Failed to execute cmd: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            } else {
                panic!("Failed to execute command: {cmd:?}");
            }
        };

        execute_cmd("git reset --hard");
        execute_cmd("git clean -fd");
        execute_cmd("git submodule foreach git reset --hard");
        execute_cmd("git submodule foreach git clean -fd");
    }
}

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

/// Installs the codspeed-go dependency in the module
pub fn install_codspeed_dependency<P: AsRef<Path>>(module_dir: P) -> anyhow::Result<()> {
    let folder = module_dir.as_ref();
    debug!("Installing codspeed-go dependency in module: {folder:?}");

    // 1. Update the go module to use the codspeed package
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
    if std::env::var("CODSPEED_GO_PKG_VERSION").is_ok() || cfg!(test) {
        replace_pkg(folder)?;
    }

    Ok(())
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

    const MANY_TESTING_IMPORTS: &str = r#"package subpackages
import (
	"bytes"
	"io"
	"testing"
	"testing/fstest"
	"testing/iotest"
	"testing/synctest"
)
"#;

    const IMPORT_TESTIFY: &str = r#"package main
import (
    "testing"

    "github.com/stretchr/testify/assert"
    "github.com/stretchr/testify/require"
)
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
    #[case("many_testing_imports", MANY_TESTING_IMPORTS)]
    #[case("import_testify", IMPORT_TESTIFY)]
    fn test_patch_go_source(#[case] test_name: &str, #[case] source: &str) {
        let mut result = source.to_string();

        Patcher::patch_imports_for_source(&mut result);
        Patcher::patch_package(&mut result, None).unwrap();
        assert_snapshot!(test_name, result);
    }
}
