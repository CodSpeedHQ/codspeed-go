use std::path::Path;

use go_runner::builder::BenchmarkPackage;

pub fn run_with_args<P: AsRef<Path>>(dir: P, args: &[&str]) -> anyhow::Result<String> {
    assert!(dir.as_ref().exists());

    let packages = BenchmarkPackage::from_project(dir.as_ref())?;
    assert_eq!(packages.len(), 1);

    let (_dir, runner_path) = go_runner::builder::templater::run(&packages[0])?;
    go_runner::runner::run_with_stdout(&runner_path, args)
}

#[test]
pub fn test_error_has_test_filename() {
    let stdout = run_with_args(
        "tests/error_file.in",
        &["-test.bench", "BenchmarkErrorFile", "-test.benchtime", "1x"],
    )
    .unwrap();

    assert!(stdout.contains("this_should_be_in_stdout"));
    assert!(stdout.contains("error_test.go"));
    assert!(!stdout.contains("error_codspeed.go"));
}
