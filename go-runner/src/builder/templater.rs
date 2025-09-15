use std::fs;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

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

pub fn run<P: AsRef<Path>>(
    package: &BenchmarkPackage,
    profile_dir: P,
) -> anyhow::Result<(TempDir, PathBuf)> {
    // 1. Copy the whole module to a build directory
    let target_dir = TempDir::new()?;
    std::fs::create_dir_all(&target_dir).context("Failed to create target directory")?;
    utils::copy_dir_recursively(&package.module.dir, &target_dir)?;

    // Create a new go-runner.metadata file in the root of the project
    //
    // The package path will be prepended to the URI. The benchmark will
    // find the path relative to the root of the `target_dir`.
    //
    // This is needed because we could execute a Go project that is a sub-folder
    // within a Git repository, then we won't copy the .git folder. Therefore, we
    // have to resolve the .git relative path in go-runner and then combine it.
    let relative_package_path = utils::get_git_relative_path(&package.dir)
        .to_string_lossy()
        .into();
    debug!("Relative package path: {relative_package_path}");

    let metadata = GoRunnerMetadata {
        profile_folder: profile_dir.as_ref().to_string_lossy().into(),
        relative_package_path,
    };
    fs::write(
        target_dir.path().join("go-runner.metadata"),
        serde_json::to_string_pretty(&metadata)?,
    )
    .context("Failed to write go-runner.metadata file")?;

    // Get files that need to be renamed first
    let files = package
        .test_go_files
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No test files found for package: {}", package.name))?;

    // Calculate the relative path from module root to package directory
    let package_dir = Path::new(&package.dir);
    let module_dir = Path::new(&package.module.dir);
    let relative_package_path = package_dir.strip_prefix(module_dir).context(format!(
        "Package dir {:?} is not within module dir {:?}",
        package.dir, package.module.dir
    ))?;
    debug!("Relative package path: {relative_package_path:?}");

    // 2. Patch the imports of all files with our own versions
    patcher::patch_imports(&target_dir)?;

    // 3. Rename the _test.go files to _codspeed.go
    for file in files {
        let old_path = target_dir.path().join(relative_package_path).join(file);
        let new_path = old_path.with_file_name(
            old_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .replace("_test", "_codspeed"),
        );

        fs::rename(&old_path, &new_path)
            .context(format!("Failed to rename {old_path:?} to {new_path:?}"))?;
    }

    // 4. Generate the codspeed/runner.go file using the template
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

    let runner_path = target_dir
        .path()
        .join(relative_package_path)
        .join("codspeed/runner.go");
    fs::create_dir_all(
        target_dir
            .path()
            .join(relative_package_path)
            .join("codspeed"),
    )
    .context("Failed to create codspeed directory")?;
    fs::write(&runner_path, rendered).context("Failed to write runner.go file")?;

    Ok((target_dir, runner_path))
}
