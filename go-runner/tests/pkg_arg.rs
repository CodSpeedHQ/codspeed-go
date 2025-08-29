use std::path::Path;

use go_runner::cli::Cli;

pub fn run_with_cli<P: AsRef<Path>>(dir: P, cli: &Cli) -> anyhow::Result<String> {
    assert!(dir.as_ref().exists());

    let packages = go_runner::filtered_packages_from_project(dir.as_ref(), &cli.packages)?;
    assert_eq!(
        packages.len(),
        1,
        "Currently only single package is supported"
    );

    let (_dir, runner_path) = go_runner::builder::templater::run(&packages[0])?;

    let args = ["-test.bench", &cli.bench, "-test.benchtime", &cli.benchtime];
    go_runner::runner::run_with_stdout(&runner_path, &args)
}

#[test]
pub fn test_pkg_arg_filters_correctly() {
    let cli = Cli {
        bench: "BenchmarkBar1".to_string(),
        benchtime: "1x".to_string(),
        packages: vec!["bar".to_string()],
    };
    let stdout = run_with_cli("tests/pkg_arg.in", &cli).unwrap();

    // Should contain output from the targeted benchmark
    assert!(stdout.contains("bar_bench_1_should_be_in_stdout"));

    // Should NOT contain output from other benchmarks
    assert!(!stdout.contains("foo_bench_1_should_not_be_in_stdout"));
    assert!(!stdout.contains("foo_bench_2_should_not_be_in_stdout"));
    assert!(!stdout.contains("bar_bench_2_should_not_be_in_stdout"));
}
