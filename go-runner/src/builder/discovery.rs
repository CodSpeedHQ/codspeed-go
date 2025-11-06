//! Finds all the benchmarks and packages in a given Go project.

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
    path::{Path, PathBuf},
    process::Command,
};

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

    /// The Go source files included in this package (e.g. `[fib.go, fib_test.go]`)
    #[serde(rename = "GoFiles")]
    pub go_files: Option<Vec<String>>,

    /// The Go test files included in this package (e.g. `[fib_test.go]`).
    /// This is `None` for external test packages.
    #[serde(rename = "TestGoFiles")]
    pub test_go_files: Option<Vec<String>>,

    #[serde(rename = "Imports")]
    pub imports: Option<Vec<String>>,

    #[serde(rename = "TestImports")]
    pub test_imports: Option<Vec<String>>,

    #[serde(rename = "CompiledGoFiles")]
    pub compiled_go_files: Option<Vec<String>>,

    /// For external test packages, this is the package being tested
    #[serde(rename = "ForTest")]
    pub for_test: Option<String>,

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

    /// Check if this package is an external test package (package name ends with _test).
    /// External test packages have names ending with "_test".
    pub fn is_external_test_package(&self) -> bool {
        self.name.ends_with("_test")
    }

    /// Returns the appropriate test files list based on whether this is an external test package.
    pub fn test_files(&self) -> Option<&Vec<String>> {
        if self.is_external_test_package() {
            self.go_files.as_ref()
        } else {
            self.test_go_files.as_ref()
        }
    }

    /// Returns the appropriate imports list based on whether this is an external test package.
    fn test_imports_list(&self) -> &Option<Vec<String>> {
        if self.is_external_test_package() {
            &self.imports
        } else {
            &self.test_imports
        }
    }

    /// Extracts the clean package import path for benchmarks.
    ///
    /// The import_path format is like "local.dev/example-complex/pkg/auth [local.dev/example-complex/pkg/auth.test]"
    /// For external test packages (_test suffix), use the ForTest field which contains the package being tested
    fn package_import_path(&self) -> anyhow::Result<String> {
        if self.is_external_test_package() {
            self.for_test
                .as_ref()
                .ok_or_else(|| {
                    anyhow::anyhow!("External test package {} missing ForTest field", self.name)
                })
                .cloned()
        } else {
            Ok(self
                .import_path
                .split_whitespace()
                .next()
                .unwrap_or(&self.import_path)
                .to_string())
        }
    }

    fn benchmarks(&self) -> anyhow::Result<Vec<GoBenchmark>> {
        let Some(test_go_files) = self.test_files() else {
            bail!("No test files found for package: {}", self.name);
        };

        let package_import_path = self.package_import_path()?;
        let is_external = self.is_external_test_package();

        let mut benchmarks = Vec::new();
        for file in test_go_files.iter().sorted() {
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

            // Remove the module dir parent from the file path
            let root_relative_file_path = file_path.strip_prefix(&self.module.dir).context(
                format!("Couldn't strip the module dir from file path: {file_path:?}"),
            )?;

            for func in found_benchmarks.into_iter().sorted() {
                benchmarks.push(GoBenchmark::new(
                    package_import_path.clone(),
                    func,
                    root_relative_file_path.to_path_buf(),
                    is_external,
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
    pub qualified_name: String,

    /// The file path relative to the module directory (e.g. `pkg/foo/foo_test.go`).
    pub file_path: PathBuf,

    /// Whether this benchmark is from an external test package (package foo_test).
    pub is_external: bool,
}

impl GoBenchmark {
    pub fn new(
        package_import_path: String,
        name: String,
        file_path: PathBuf,
        is_external: bool,
    ) -> Self {
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
            is_external,
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

    pub fn from_project(
        go_project_path: &Path,
        packages: &[String],
    ) -> anyhow::Result<Vec<BenchmarkPackage>> {
        let mut raw_packages = Self::run_go_list(go_project_path, packages)?;

        // Sort packages by import path to ensure deterministic order
        raw_packages.sort_by(|a, b| a.import_path.cmp(&b.import_path));

        let mut packages = Vec::new();
        for package in raw_packages {
            // Filter 1: Must have test files
            let Some(test_files) = package.test_files() else {
                debug!("Skipping package without test files: {}", package.name);
                continue;
            };
            if !test_files.iter().any(|name| name.ends_with("_test.go")) {
                debug!(
                    "Skipping package with files, but without test files: {}",
                    package.name
                );
                continue;
            }

            // Filter 2: Must have testing imports
            let Some(test_imports) = package.test_imports_list() else {
                debug!("Skipping package without test imports: {}", package.name);
                continue;
            };
            if !test_imports.iter().any(|import| import.contains("testing")) {
                debug!("Skipping package without test imports: {}", package.name);
                continue;
            }

            // Filter 3: Must be a test executable
            // Example format: `local.dev/example-complex [local.dev/example-complex.test]`
            if !package.import_path.ends_with(".test]") {
                debug!(
                    "Skipping package without test executable: {}",
                    package.import_path
                );
                continue;
            }

            // Filter 4: Must have benchmarks
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

    fn run_go_list(go_project_path: &Path, packages: &[String]) -> anyhow::Result<Vec<GoPackage>> {
        // Execute 'go list -test -compiled -json <packages>' to get package information
        let mut args = vec!["list", "-test", "-compiled", "-json"];
        args.extend(packages.iter().map(|s| s.as_str()));

        let output = Command::new("go")
            .args(args)
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
    use crate::utils::can_build_project;

    #[rstest::rstest]
    #[case::caddy("caddy")]
    #[case::fzf("fzf")]
    #[case::opentelemetry_go("opentelemetry-go")]
    #[case::golang_benchmarks("golang-benchmarks")]
    #[case::zerolog("zerolog")]
    #[case::zap("zap")]
    #[case::hugo("hugo")]
    #[case::fuego("fuego")]
    #[case::cli_runtime("cli-runtime")]
    #[case::quic_go("quic-go")]
    #[case::example("example")]
    #[case::example_with_helper("example-with-helper")]
    #[case::example_with_main("example-with-main")]
    #[case::example_with_dot_go_folder("example-with-dot-go-folder")]
    #[case::example_with_test_package("example-with-test-package")]
    #[test_log::test]
    fn test_discover_benchmarks(#[case] project_name: &str) {
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/projects")
            .join(project_name);

        if !can_build_project(&project_dir) {
            eprintln!("Skipping test for project {project_name} due to Go version constraints.");
            return;
        }

        let mut packages =
            BenchmarkPackage::from_project(&project_dir, &["./...".to_string()]).unwrap();

        // Sort packages by dir to ensure deterministic order
        packages.sort_by_cached_key(|pkg| pkg.dir.clone());

        let _guard = {
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_suffix(project_name.to_string());
            settings.bind_to_scope()
        };
        insta::assert_json_snapshot!(packages, {
            ".**[\"raw_package\"]" => insta::dynamic_redaction(|_value, _path| "[raw_package]"),
        });
    }
}
