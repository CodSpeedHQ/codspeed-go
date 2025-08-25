//! Finds all the benchmarks and packages in a given Go project.

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
    path::{Path, PathBuf},
    process::Command,
};

use crate::builder::verifier;
use crate::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// Represents a Go package, deserialized from `go list -json` output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoPackage {
    /// The path to the package (e.g., "github.com/user/project/pkg/foo").
    #[serde(rename = "Dir")]
    pub dir: PathBuf,

    /// The name of the package (e.g., "foo").
    #[serde(rename = "Name")]
    pub name: String,

    /// The import path of the package with a package identifier (e.g., "local.dev/example-complex/internal/config [local.dev/example-complex/internal/config.test]").
    #[serde(rename = "ImportPath")]
    pub import_path: String,

    #[serde(rename = "TestGoFiles")]
    pub test_go_files: Option<Vec<String>>,

    #[serde(rename = "TestImports")]
    pub test_imports: Option<Vec<String>>,

    #[serde(rename = "CompiledGoFiles")]
    pub compiled_go_files: Option<Vec<String>>,

    #[serde(rename = "Module")]
    pub module: GoModule,
}

/// Contains information about the Go module, which contains one or more Go packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoModule {
    /// The module path (e.g., "local.dev/example-complex").
    #[serde(rename = "Path")]
    pub path: String,

    /// The module directory (e.g., "/home/user/go/src/local.dev/example-complex").
    #[serde(rename = "Dir")]
    pub dir: PathBuf,

    /// The module go.mod file (e.g., "/home/user/go/src/local.dev/example-complex/go.mod").
    #[serde(rename = "GoMod")]
    pub go_mod: PathBuf,

    /// The module version (e.g., "v1.0.0").
    #[serde(rename = "GoVersion")]
    pub version: String,

    /// Whether this is the main module.
    #[serde(rename = "Main")]
    pub main: bool,
}

impl GoPackage {
    pub fn from_go_list_output(output: &str) -> anyhow::Result<Vec<Self>> {
        // Replace all \n, then find '}{' and replace with '},{' to convert the output into a valid JSON array
        let output = output.replace("\n", "");
        let output = output.replace("}{", "},{");

        serde_json::from_str(&format!("[{output}]")).context("Failed to parse Go list output")
    }

    fn benchmarks(&self) -> anyhow::Result<Vec<GoBenchmark>> {
        let Some(test_go_files) = &self.test_go_files else {
            bail!("No test files found for package: {}", self.name);
        };

        let mut benchmarks = Vec::new();
        'file_loop: for file in test_go_files.iter().sorted() {
            assert!(file.ends_with("_test.go"));

            let file_path = self.dir.join(file);
            let content = std::fs::read_to_string(&file_path)
                .context(format!("Failed to read test file: {file_path:?}"))?;

            let file = match gosyn::parse_source(&content) {
                Ok(ast) => ast,
                Err(e) => {
                    warn!("Failed to parse Go file {file_path:?}: {e}");
                    continue;
                }
            };

            // Check for unsupported imports
            const UNSUPPORTED_IMPORTS: &[(&str, &str)] = &[
                ("github.com/frankban/quicktest", "quicktest"),
                ("github.com/stretchr/testify", "testify"),
            ];
            for (import_path, import_name) in UNSUPPORTED_IMPORTS {
                if file
                    .imports
                    .iter()
                    .any(|import| import.path.value.contains(import_path))
                {
                    warn!("Skipping file with {import_name} import: {file_path:?}");
                    continue 'file_loop;
                }
            }

            // We can't import packages that are declared as `main`
            if file.pkg_name.name == "main" {
                warn!("Skipping file with main package: {file_path:?}");
                continue;
            }

            // First, collect all benchmark function names from this file
            let mut found_benchmarks = Vec::new();
            for decl in &file.decl {
                let gosyn::ast::Declaration::Function(func_decl) = decl else {
                    continue;
                };

                let func_name = &func_decl.name.name;

                // Check if function name starts with "Benchmark"
                if !func_name.starts_with("Benchmark") {
                    continue;
                }

                found_benchmarks.push(func_name.clone());
            }

            // Extract the actual package import path from the full import_path
            // The import_path format is like "local.dev/example-complex/pkg/auth [local.dev/example-complex/pkg/auth.test]"
            let package_import_path = self
                .import_path
                .split_whitespace()
                .next()
                .unwrap_or(&self.import_path)
                .to_string();

            // Remove the module dir parent from the file path
            let root_relative_file_path = file_path.strip_prefix(&self.module.dir).context(
                format!("Couldn't strip the module dir from file path: {file_path:?}"),
            )?;

            let valid_benchmarks =
                verifier::FuncVisitor::verify_source_code(&content, &found_benchmarks)?;
            if valid_benchmarks.len() != found_benchmarks.len() {
                warn!(
                    "Only {} out of {} are valid, skipping file",
                    valid_benchmarks.len(),
                    found_benchmarks.len()
                );
                warn!("Valid benchmarks: {valid_benchmarks:?}");
                warn!(
                    "Invalid benchmarks: {:?}",
                    found_benchmarks
                        .iter()
                        .filter(|f| !valid_benchmarks.contains(f))
                        .collect::<Vec<_>>()
                );

                continue;
            }

            for func in valid_benchmarks.into_iter().sorted() {
                benchmarks.push(GoBenchmark::new(
                    package_import_path.clone(),
                    func,
                    root_relative_file_path.to_path_buf(),
                ));
            }
        }

        Ok(benchmarks)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoBenchmark {
    /// The name of the benchmark (e.g. `BenchmarkFoo`).
    pub name: String,

    /// The path to the module (e.g. `github.com/user/foo`).
    module_path: String,

    /// The import alias (e.g. `foo_test_49212941`).
    import_alias: String,

    /// The name with the package (e.g. `foo_test.BenchmarkFoo`).
    qualified_name: String,

    /// The file path relative to the module directory (e.g. `pkg/foo/foo_test.go`).
    pub file_path: PathBuf,
}

impl GoBenchmark {
    pub fn new(package_import_path: String, name: String, file_path: PathBuf) -> Self {
        let hash = {
            let mut hasher = DefaultHasher::new();
            package_import_path.hash(&mut hasher);
            hasher.finish()
        };

        let import_alias = format!("{}_{}", name.to_lowercase(), hash);
        let qualified_name = format!("{}.{}", import_alias, &name);
        Self {
            module_path: package_import_path,
            import_alias,
            name,
            qualified_name,
            file_path,
        }
    }
}

/// Represents a package with its benchmarks.
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkPackage {
    raw_package: GoPackage,
    pub benchmarks: Vec<GoBenchmark>,
}

impl BenchmarkPackage {
    fn new(package: GoPackage, benchmarks: Vec<GoBenchmark>) -> Self {
        Self {
            raw_package: package,
            benchmarks,
        }
    }

    pub fn from_project(go_project_path: &Path) -> anyhow::Result<Vec<BenchmarkPackage>> {
        let raw_packages = Self::run_go_list(go_project_path)?;
        let has_test_files =
            |files: &Vec<String>| files.iter().any(|name| name.ends_with("_test.go"));
        let has_test_imports = |imports: &Vec<String>| {
            imports.iter().any(|import| {
                // import "testing"
                import.contains("testing")
            })
        };

        let mut packages = Vec::new();
        for package in raw_packages {
            // Skip packages without test files
            let has_tests = package
                .test_go_files
                .as_ref()
                .map(has_test_files)
                .unwrap_or_default();
            if !has_tests {
                debug!("Skipping package without test files: {}", package.name);
                continue;
            }

            // Skip packages without test imports
            let has_test_imports = package
                .test_imports
                .as_ref()
                .map(has_test_imports)
                .unwrap_or_default();
            if !has_test_imports {
                debug!("Skipping package without test imports: {}", package.name);
                continue;
            }

            // Only include test executables, since we want to generate them manually.
            // Example format: `local.dev/example-complex [local.dev/example-complex.test]`
            if !package.import_path.ends_with(".test]") {
                debug!("Skipping package without test executable: {}", package.name);
                continue;
            }

            // Skip packages that don't have benchmarks
            let benchmarks = match package.benchmarks() {
                Ok(benchmarks) => benchmarks,
                Err(e) => {
                    warn!(
                        "Failed to get benchmarks for package {}: {}",
                        package.name, e
                    );
                    continue;
                }
            };
            if benchmarks.is_empty() {
                debug!("Skipping package without benchmarks: {}", package.name);
                continue;
            }

            packages.push(BenchmarkPackage::new(package, benchmarks));
        }

        Ok(packages)
    }

    fn run_go_list(go_project_path: &Path) -> anyhow::Result<Vec<GoPackage>> {
        // Execute 'go list -test -compiled -json ./...' to get package information
        let output = Command::new("go")
            .args(["list", "-test", "-compiled", "-json", "./..."])
            .current_dir(go_project_path)
            .output()?;

        if !output.status.success() {
            bail!(
                "Failed to execute 'go list': {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Wrap it in '[{output}]' and parse it with serde_json
        let output_str = String::from_utf8(output.stdout)?;
        trace!("Go list output: {output_str}");

        GoPackage::from_go_list_output(&output_str)
    }
}

impl Deref for BenchmarkPackage {
    type Target = GoPackage;

    fn deref(&self) -> &Self::Target {
        &self.raw_package
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_benchmarks() {
        let packages =
            BenchmarkPackage::from_project(Path::new("testdata/projects/golang-benchmarks"))
                .unwrap();

        insta::assert_json_snapshot!(packages, {
            ".**[\"Dir\"]" => "[package_dir]",
            ".**[\"Module\"][\"Dir\"]" => "[module_dir]",
            ".**[\"Module\"][\"GoMod\"]" => "[go_mod_path]"
        });
    }
}
