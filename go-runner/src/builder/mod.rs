pub mod discovery;
pub mod patcher;
pub mod templater;
pub mod verifier;

pub use discovery::*;

use crate::prelude::*;
use std::{path::Path, process::Command};

/// Builds a Go runner file into an executable binary
pub fn build_binary<P: AsRef<Path>>(runner_go_path: P) -> anyhow::Result<std::path::PathBuf> {
    let runner_go_path = runner_go_path.as_ref();
    let file_dir = runner_go_path.parent().unwrap();
    let module_root = file_dir.parent().unwrap();
    let relative_path = runner_go_path.strip_prefix(module_root).unwrap();

    debug!(
        "Building codspeed runner binary: {:?} (root = {:?})",
        module_root.join(relative_path),
        module_root
    );

    let binary_path = runner_go_path.with_extension("bin");

    // Set the integration version in our testing library and include debug symbols
    let ldflags = format!(
        "-X github.com/CodSpeedHQ/codspeed-go/testing/capi.integrationVersion={} -s=false -w=false",
        env!("CARGO_PKG_VERSION")
    );

    let args = vec![
        "build",
        "-tags=codspeed",
        "-ldflags",
        &ldflags,
        "-o",
        binary_path.to_str().unwrap(),
        relative_path.to_str().unwrap(),
    ];

    let output = Command::new("go")
        .args(&args)
        .current_dir(module_root)
        .output()
        .context("Failed to execute go build command")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        warn!("Build command output: {stdout}");
        warn!("Build command error output: {stderr}");

        bail!(
            "Failed to build benchmark binary. Exit status: {}",
            output.status
        );
    }

    debug!("Successfully built binary: {binary_path:?}");
    Ok(binary_path)
}
