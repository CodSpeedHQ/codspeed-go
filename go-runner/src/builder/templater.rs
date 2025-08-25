use std::fs;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use crate::builder::{BenchmarkPackage, GoBenchmark};
use crate::utils;
use crate::{builder::patcher, prelude::*};

#[derive(Debug, Serialize, Deserialize)]
struct TemplateData {
    benchmarks: Vec<GoBenchmark>,
    module_name: String,
}

pub fn run(package: &BenchmarkPackage) -> anyhow::Result<(TempDir, PathBuf)> {
    // 1. Copy the whole module to a build directory
    let target_dir = TempDir::new()?;
    std::fs::create_dir_all(&target_dir).context("Failed to create target directory")?;
    utils::copy_dir_all(&package.module.dir, &target_dir)?;

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

    // 2. Find benchmark files and patch their imports
    let files_to_patch = package
        .benchmarks
        .iter()
        .map(|bench| {
            target_dir
                .path()
                // .join(relative_package_path)
                .join(&bench.file_path)
        })
        .collect::<Vec<_>>();
    patcher::patch_imports(&target_dir, files_to_patch)?;

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
    //
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
