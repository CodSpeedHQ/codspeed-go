use crate::prelude::*;
use std::{path::Path, process::Command};

pub fn run<P: AsRef<Path>>(binary_path: P, run_args: &[&str]) -> anyhow::Result<()> {
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

    let output = Command::new(binary_path)
        .args(run_args)
        .current_dir(module_root)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("Failed to execute benchmark binary")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        warn!("Command output: {stdout}");
        warn!("Command error output: {stderr}");

        bail!("Failed to run benchmark. Exit status: {}", output.status);
    }

    Ok(())
}
