use crate::prelude::*;
use std::{path::Path, process::Command};

pub fn run<P: AsRef<Path>>(runner_go_path: P, run_args: &[&str]) -> anyhow::Result<()> {
    // Extract the directory containing runner.go to use as working directory
    let runner_go_path = runner_go_path.as_ref();
    let file_dir = runner_go_path.parent().unwrap();
    let module_root = file_dir.parent().unwrap();
    let relative_path = runner_go_path.strip_prefix(module_root).unwrap();
    debug!(
        "Building codspeed runner: {:?} (root = {:?})",
        module_root.join(relative_path),
        module_root
    );

    // Run go run -tags=codspeed <path> {args}
    let output = Command::new("go")
        .arg("run")
        .arg("-tags=codspeed")
        .arg(relative_path)
        .args(run_args)
        .current_dir(module_root)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("Failed to execute go build command")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        warn!("Command output: {stdout}");
        warn!("Command error output: {stderr}");

        bail!("Failed to run benchmark. Exit status: {}", output.status);
    }

    Ok(())
}
