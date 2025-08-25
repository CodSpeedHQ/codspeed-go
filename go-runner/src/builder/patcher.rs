//! Patches the imports to use codspeed rather than the official "testing" package.

use crate::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn patch_imports<P: AsRef<Path>>(
    folder: P,
    files_to_patch: Vec<PathBuf>,
) -> anyhow::Result<()> {
    let folder = folder.as_ref();
    debug!("Patching imports in folder: {folder:?}");

    // 2. Find all imports that match "testing" and replace them with codspeed equivalent
    let mut patched_files = 0;
    for go_file in files_to_patch {
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

    // 3. Update the go module to use the codspeed package
    let mut cmd: Command = Command::new("go");
    cmd.arg("get")
        .arg("github.com/CodSpeedHQ/codspeed-go@cod-1172-create-codspeed-go-repository-with-the-compat-layer")
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

    Ok(())
}

/// Internal function to apply import patterns to Go source code
pub fn patch_go_source(source: &str) -> anyhow::Result<String> {
    let parsed = gosyn::parse_source(source)?;

    let mut modified_content = source.to_string();
    if let Some(import) = parsed
        .imports
        .iter()
        .find(|import| import.path.value == "\"testing\"")
    {
        let start_pos = import.path.pos;
        let end_pos = start_pos + import.path.value.len();

        let replacement = "testing \"github.com/CodSpeedHQ/codspeed-go/compat/testing\"";
        modified_content.replace_range(start_pos..end_pos, replacement);
    }

    Ok(modified_content)
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
    #[case(
        "multiline_import_with_testing_string",
        MULTILINE_IMPORT_WITH_TESTING_STRING
    )]
    fn test_patch_go_source(#[case] test_name: &str, #[case] source: &str) {
        let result = patch_go_source(source).unwrap();
        assert_snapshot!(test_name, result);
    }
}
