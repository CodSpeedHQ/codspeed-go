use std::fs;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use crate::builder::patcher::Patcher;
use crate::builder::{BenchmarkPackage, GoBenchmark};
use crate::utils;
use crate::{builder::patcher, prelude::*};

#[derive(Debug, Serialize)]
struct GoRunnerMetadata {
    profile_folder: String,
    relative_package_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TemplateData {
    benchmarks: Vec<GoBenchmark>,
    module_name: String,
}

pub struct CodspeedContext {
    package: BenchmarkPackage,
    profile_dir: PathBuf,
    git_root: PathBuf,
    target_dir: PathBuf,

    // Artifacts that have to be cleaned up later
    patcher: Patcher,
    metadata_path: Option<PathBuf>,
    runner_path: Option<PathBuf>,
}

impl CodspeedContext {
    pub fn for_package<P: AsRef<Path>>(
        package: &BenchmarkPackage,
        profile_dir: P,
        git_root: PathBuf,
        target_dir: PathBuf,
    ) -> Self {
        Self {
            patcher: Patcher::new(&target_dir),
            package: package.clone(),
            profile_dir: profile_dir.as_ref().to_path_buf(),
            git_root,
            target_dir,
            metadata_path: None,
            runner_path: None,
        }
    }

    fn setup_runner_metadata(&mut self) -> anyhow::Result<()> {
        // Create a new go-runner.metadata file in the root of the project
        //
        // The package path will be prepended to the URI. The benchmark will
        // find the path relative to the root of the `target_dir`.
        //
        // This is needed because we could execute a Go project that is a sub-folder
        // within a Git repository, then we won't copy the .git folder. Therefore, we
        // have to resolve the .git relative path in go-runner and then combine it.
        let relative_package_path = utils::get_git_relative_path(&self.package.dir)
            .to_string_lossy()
            .into();
        debug!("Relative package path: {relative_package_path}");

        let metadata = GoRunnerMetadata {
            profile_folder: self.profile_dir.to_string_lossy().into(),
            relative_package_path,
        };
        let metadata_path = self.target_dir.join("go-runner.metadata");
        fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .context("Failed to write go-runner.metadata file")?;
        self.metadata_path = Some(metadata_path);

        Ok(())
    }

    fn patch_files(&mut self) -> anyhow::Result<()> {
        let package = &self.package;
        let target_dir = &self.target_dir;

        // Get files that need to be renamed first
        let files = package.test_files().with_context(|| {
            anyhow::anyhow!("No test files found for package: {}", package.name)
        })?;

        // Patch the imports and package of the test files
        // - Renames package declarations (to support main package tests and external tests)
        // - Fixes imports to use our compat packages (e.g., testing/quicktest/testify)
        let package_path = target_dir.join(self.relative_package_path()?);
        let test_file_paths: Vec<PathBuf> = files.iter().map(|f| package_path.join(f)).collect();

        // If we have external tests (e.g. "package {pkg}_test") they have to be
        // changed to "package main" so they can be built within the codspeed/ sub-package.
        if package.is_external_test_package() {
            self.patcher.patch_packages_for_files(&test_file_paths)?;
        } else if package.name == "main" {
            // If this is a "package main" (not external test), we need to patch ALL .go files in the package directory
            // so they all become "package main_compat" and can be imported by the runner.

            info!("Package is 'main' - patching all .go files in package directory");
            self.patcher.patch_all_packages_in_dir(&package_path)?;
        }
        self.patcher.patch_imports(target_dir)?;

        // Handle test files differently based on whether they're external or internal tests
        let codspeed_dir = self.codspeed_dir()?;
        fs::create_dir_all(&codspeed_dir).context("Failed to create codspeed directory")?;

        if package.is_external_test_package() {
            // For external test packages: copy test files to codspeed/ subdirectory AND rename them
            // (remove _test suffix so Go will compile them with `go build`)
            // They're now package main and will be built from the subdirectory
            self.patcher
                .rename_and_move_test_files(&test_file_paths, &codspeed_dir)?;

            // Also rename internal test files in place so they are accessible in
            // the tests during `go build`. This allows external tests to call
            // functions defined in internal test files (e.g., mylib.SetTestState()).
            let internal_test_paths: Vec<PathBuf> = package
                .internal_test_files()
                .iter()
                .map(|f| package_path.join(f))
                .collect();
            self.patcher.rename_test_files(&internal_test_paths)?;
            info!(
                "Renamed {} internal test files for external test package",
                internal_test_paths.len()
            );
        } else {
            // For internal test packages: rename _test.go to _codspeed.go in place
            self.patcher.rename_test_files(&test_file_paths)?;
        }

        Ok(())
    }

    fn install_codspeed_dependency(&mut self) -> anyhow::Result<()> {
        let package = &self.package;
        let git_root = &self.git_root;
        let target_dir = &self.target_dir;

        // Install codspeed-go dependency at the package module level
        // Find the module directory by getting the relative path from git root
        let module_dir = Path::new(&package.module.dir)
            .strip_prefix(git_root)
            .map(|relative_module_path| target_dir.join(relative_module_path))
            .unwrap_or_else(|_| {
                // Fall back to target_dir if we can't calculate relative path
                target_dir.to_path_buf()
            });
        patcher::install_codspeed_dependency(&module_dir)?;

        Ok(())
    }

    fn setup_runner(&mut self) -> anyhow::Result<()> {
        let package = &self.package;

        // Generate the codspeed/runner.go file using the template
        let mut handlebars = Handlebars::new();
        let template_content = include_str!("template.go");
        handlebars.register_template_string("main", template_content)?;

        // import <alias> <mod_path>
        // { "<name>", <qualified_path> },
        let data = TemplateData {
            benchmarks: package.benchmarks.clone(),
            module_name: "codspeed_runner".into(),
        };
        let rendered = handlebars.render("main", &data)?;

        let runner_path = self.codspeed_dir()?.join("runner.go");
        fs::write(&runner_path, rendered).context("Failed to write runner.go file")?;
        self.runner_path = Some(runner_path);

        Ok(())
    }

    pub fn runner_path(&self) -> &PathBuf {
        self.runner_path
            .as_ref()
            .expect("Runner path not set up yet")
    }

    fn relative_package_path(&self) -> anyhow::Result<PathBuf> {
        let package = &self.package;
        let git_root = &self.git_root;

        // Calculate the relative path from git root to package directory
        let package_dir = Path::new(&package.dir);
        let relative_package_path = package_dir.strip_prefix(git_root).context(format!(
            "Package dir {:?} is not within git root {:?}",
            package.dir, git_root
        ))?;
        Ok(relative_package_path.to_path_buf())
    }

    fn codspeed_dir(&self) -> anyhow::Result<PathBuf> {
        let package = &self.package;
        let target_dir = &self.target_dir;
        let git_root = &self.git_root;

        let package_dir = Path::new(&package.dir);
        let relative_package_path = package_dir.strip_prefix(git_root).unwrap_or_else(|_| {
            panic!(
                "Package dir {:?} is not within git root {:?}",
                package.dir, git_root
            )
        });
        Ok(target_dir.join(relative_package_path).join("codspeed"))
    }
}

pub struct Templater {
    target_dir: OnceCell<TempDir>,
}

impl Default for Templater {
    fn default() -> Self {
        Self::new()
    }
}

impl Templater {
    pub fn new() -> Self {
        Self {
            target_dir: OnceCell::new(),
        }
    }

    /// Runs the templater which sets up a temporary Go project with patched test files and a custom runner.
    ///
    /// # Returns
    ///
    /// The path to the generated runner.go file. This should be passed to the `build_binary` function to build
    /// the binary that will execute the benchmarks.
    pub fn run<P: AsRef<Path>>(
        &self,
        package: &BenchmarkPackage,
        profile_dir: P,
    ) -> anyhow::Result<CodspeedContext> {
        // Copy the whole git repository to a build directory
        let git_root = if let Ok(git_dir) = utils::get_parent_git_repo_path(&package.module.dir) {
            git_dir
        } else {
            warn!("Could not find git repository root. Falling back to module directory as root");
            PathBuf::from(&package.module.dir)
        };

        // Because we added the projects as git submodules, they'll have a symlink to
        // the actual git repository. There is a .git file which only contains the path to
        // the actual .git folder.
        // $ cat .git
        //   gitdir: ../../../../.git/modules/go-runner/testdata/projects/hugo
        //
        // In those cases, we'll have to copy the parent directory rather than the submodule.
        // NOTE: This is not the case for all tests, so we have to only do it for submodules.
        let is_submodule = fs::read_to_string(git_root.join(".git"))
            .map(|content| content.starts_with("gitdir:"))
            .unwrap_or(false);
        let git_root: PathBuf = if cfg!(test) && is_submodule {
            utils::get_parent_git_repo_path(git_root.parent().unwrap()).unwrap()
        } else {
            git_root
        };
        info!("Found git root at {git_root:?}");

        let target_dir = self
            .target_dir
            .get_or_try_init(|| -> anyhow::Result<TempDir> {
                // Create a temporary target directory for building the modified Go project.
                let mut target_dir = TempDir::new()?;

                // We don't want to spend time cleanup any temporary files since the code is only
                // run on CI servers which clean up themselves.
                // However, when running tests we don't want to fill the disk with temporary files, which
                // can cause the tests to fail due to lack of disk space.
                if cfg!(not(test)) {
                    target_dir.disable_cleanup(true);
                }

                utils::copy_dir_recursively(&git_root, target_dir.path())?;

                Ok(target_dir)
            })?;

        let mut ctx = CodspeedContext::for_package(
            package,
            &profile_dir,
            git_root,
            target_dir.path().to_path_buf(),
        );
        ctx.install_codspeed_dependency()?;
        ctx.setup_runner_metadata()?;
        ctx.patch_files()?;
        ctx.setup_runner()?;

        Ok(ctx)
    }
}
