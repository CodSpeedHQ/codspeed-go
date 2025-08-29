use crate::prelude::*;
use std::{path::Path, process::Command};

fn run_cmd<P: AsRef<Path>>(binary_path: P, run_args: &[&str]) -> anyhow::Result<Command> {
    let binary_path = binary_path.as_ref();
    debug!("Running codspeed benchmark binary: {binary_path:?}");

    // Execute it from the folder with the benchmarks:
    // ```
    // benches/                 <-- module_root
    //   codspeed/
    //      runner.go
    //      runner.bin          <-- binary_path
    //   foo_test.go
    //   fib_test.go
    // ```
    let module_root = binary_path.parent().unwrap().parent().unwrap();
    let mut cmd = Command::new(binary_path);
    cmd.args(run_args).current_dir(module_root);
    Ok(cmd)
}

fn check_success(output: &std::process::Output) -> anyhow::Result<String> {
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        warn!("Command output: {stdout}");
        warn!("Command error output: {stderr}");

        bail!("Failed to run benchmark. Exit status: {}", output.status);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Runs the cmd and returns the output.
pub fn run_with_stdout<P: AsRef<Path>>(
    binary_path: P,
    run_args: &[&str],
) -> anyhow::Result<String> {
    let mut cmd = run_cmd(binary_path, run_args)?;
    let output = cmd.output().context("Failed to execute go build command")?;
    check_success(&output)
}

/// Runs the cmd and forwards the output to stdout/stderr.
pub fn run<P: AsRef<Path>>(binary_path: P, run_args: &[&str]) -> anyhow::Result<()> {
    let mut cmd = run_cmd(binary_path, run_args)?;
    let output = cmd
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("Failed to execute go build command")?;

    check_success(&output).map(|_| ())
}
