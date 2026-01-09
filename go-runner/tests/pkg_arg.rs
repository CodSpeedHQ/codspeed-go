use codspeed_go_runner::cli::Cli;
use utils::run_with_cli;

pub mod utils;

#[test]
pub fn test_pkg_arg_filters_correctly() {
    let cli = Cli {
        bench: "BenchmarkBar1".to_string(),
        benchtime: "1x".to_string(),
        packages: vec!["./bar".to_string()],
        dry_run: false,
    };
    let stdout = run_with_cli("tests/pkg_arg.in", &cli).unwrap();

    // Should contain output from the targeted benchmark
    assert!(stdout.contains("bar_bench_1"));

    // Should NOT contain output from other benchmarks
    assert!(!stdout.contains("foo_bench_1"));
    assert!(!stdout.contains("foo_bench_2"));
    assert!(!stdout.contains("bar_bench_2"));
}

#[test]
pub fn test_pkg_arg_all_packages() {
    let cli = Cli {
        bench: ".".to_string(),
        benchtime: "1x".to_string(),
        packages: vec!["./...".to_string()],
        dry_run: false,
    };
    let stdout = run_with_cli("tests/pkg_arg.in", &cli).unwrap();

    // Should contain output from all benchmarks when using ./...
    assert!(stdout.contains("foo_bench_1"));
    assert!(stdout.contains("foo_bench_2"));
    assert!(stdout.contains("bar_bench_1"));
    assert!(stdout.contains("bar_bench_2"));
}

#[test]
pub fn test_pkg_arg_multiple_packages() {
    let cli = Cli {
        bench: ".".to_string(),
        benchtime: "1x".to_string(),
        packages: vec!["./foo".to_string(), "./bar".to_string()],
        dry_run: false,
    };
    let stdout = run_with_cli("tests/pkg_arg.in", &cli).unwrap();

    // Should contain output from both foo and bar packages
    assert!(stdout.contains("foo_bench_1"));
    assert!(stdout.contains("foo_bench_2"));
    assert!(stdout.contains("bar_bench_1"));
    assert!(stdout.contains("bar_bench_2"));
}
