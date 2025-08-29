pub mod utils;

use utils::run_with_args;

#[test]
pub fn test_error_has_test_filename() {
    let stdout = run_with_args(
        "tests/error_file.in",
        &["-test.bench", "BenchmarkErrorFile", "-test.benchtime", "1x"],
    )
    .unwrap();

    eprintln!("Error output: {stdout}");
    assert!(stdout.contains("this_should_be_in_stdout"));
    assert!(stdout.contains("error_test.go"));
    assert!(!stdout.contains("error_codspeed.go"));
}
