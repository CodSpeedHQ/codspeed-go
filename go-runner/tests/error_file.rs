pub mod utils;

use codspeed_go_runner::cli::Cli;
use utils::run_with_cli;

#[test]
pub fn test_error_has_test_filename() {
    let cli = Cli {
        bench: "BenchmarkErrorFile".to_string(),
        benchtime: "1x".to_string(),
        packages: vec!["./...".to_string()],
        dry_run: false,
    };
    let result = run_with_cli("tests/error_file.in", &cli);
    assert!(result.is_err(), "Expected an error but got success");

    let error_msg = result.unwrap_err().to_string();
    eprintln!("Error output: {error_msg}");
    assert!(error_msg.contains("this_should_be_in_stdout"));
    assert!(error_msg.contains("error_test.go"));
    assert!(!error_msg.contains("error_codspeed.go"));
}
