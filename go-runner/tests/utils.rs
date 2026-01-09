use codspeed_go_runner::{cli::Cli, runner};
use std::path::Path;

/// Helper function to run a single package using CLI configuration
pub fn run_with_cli<P: AsRef<Path>>(dir: P, cli: &Cli) -> anyhow::Result<String> {
    assert!(dir.as_ref().exists());
    let stdout = runner::run_with_stdout(Path::new("/tmp"), dir.as_ref(), cli)?;
    Ok(stdout)
}
