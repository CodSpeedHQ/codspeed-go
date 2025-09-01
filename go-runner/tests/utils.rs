use codspeed_go_runner::{builder, builder::BenchmarkPackage, cli::Cli, runner};
use std::path::Path;

/// Helper function to run a single package with arguments
pub fn run_package_with_args(package: &BenchmarkPackage, args: &[&str]) -> anyhow::Result<String> {
    let profile_dir = tempfile::TempDir::new()?;
    let (_dir, runner_path) = builder::templater::run(package, profile_dir.as_ref())?;
    let binary_path = builder::build_binary(&runner_path)?;
    runner::run_with_stdout(&binary_path, args)
}

/// Helper function to run tests in a directory with specific arguments
pub fn run_with_args<P: AsRef<Path>>(dir: P, args: &[&str]) -> anyhow::Result<String> {
    assert!(dir.as_ref().exists());

    let packages = BenchmarkPackage::from_project(dir.as_ref(), &["./...".to_string()])?;
    assert_eq!(packages.len(), 1);

    run_package_with_args(&packages[0], args)
}

/// Helper function to run a single package using CLI configuration
pub fn run_with_cli<P: AsRef<Path>>(dir: P, cli: &Cli) -> anyhow::Result<String> {
    assert!(dir.as_ref().exists());

    let packages = BenchmarkPackage::from_project(dir.as_ref(), &cli.packages)?;
    assert_eq!(
        packages.len(),
        1,
        "Currently only single package is supported"
    );

    let args = ["-test.bench", &cli.bench, "-test.benchtime", &cli.benchtime];
    run_package_with_args(&packages[0], &args)
}

/// Helper function to run multiple packages using CLI configuration
pub fn run_with_cli_multi<P: AsRef<Path>>(dir: P, cli: &Cli) -> anyhow::Result<String> {
    assert!(dir.as_ref().exists());

    let packages = BenchmarkPackage::from_project(dir.as_ref(), &cli.packages)?;

    let mut all_stdout = String::new();
    for package in &packages {
        let args = ["-test.bench", &cli.bench, "-test.benchtime", &cli.benchtime];
        let stdout = run_package_with_args(package, &args)?;
        all_stdout.push_str(&stdout);
    }

    Ok(all_stdout)
}
