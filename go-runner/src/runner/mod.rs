use crate::cli::Cli;
use crate::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

mod overlay;

fn check_c_compiler(go_binary: &Path) -> anyhow::Result<()> {
    let output = Command::new(go_binary)
        .args(["env", "CC"])
        .output()
        .context("Failed to run `go env CC`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to determine C compiler via `go env CC`: {stderr}");
    }

    let cc = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if cc.is_empty() {
        bail!(
            "No C compiler found. The CodSpeed Go runner requires a C compiler (gcc/cc) \
             to build the instrumentation hooks. Install `build-essential` on Ubuntu/Debian \
             or the equivalent for your platform."
        );
    }

    Ok(())
}

fn run_cmd<P: AsRef<Path>>(
    profile_dir: P,
    dir: P,
    cli: &Cli,
) -> anyhow::Result<(TempDir, Command)> {
    // Execute the `go test` command using the go binary, rather than the one in the PATH
    // to avoid running into infinite loops with the runner which tries to intercept `go test`.
    let go_binary = find_go_binary()?;

    // Check early, before downloading instrument-hooks and generating the overlay.
    check_c_compiler(&go_binary)?;

    let (_dir, overlay_file) = overlay::get_overlay_file(profile_dir.as_ref())?;

    // Convert the CLI struct into a command:
    let mut cmd = Command::new(go_binary);
    cmd.args([
        "test",
        // Disable stripping of symbols and debug information
        "-ldflags=-s=false -w=false",
        // Keep the test binary on disk after executing it. This is required by
        // the runner to properly parse the symbols and debug info.
        "-work",
        "-overlay",
        &overlay_file.to_string_lossy(),
        "-bench",
        &cli.bench,
        "-benchtime",
        &cli.benchtime,
        // Dont' run tests, only benchmarks
        "-run=^$",
    ]);
    cmd.args(&cli.packages);
    cmd.current_dir(dir);

    // Create isolated Go caches to avoid conflicts when tests run concurrently
    cmd.env("GOCACHE", _dir.path().join("gocache"));
    cmd.env("GOMODCACHE", _dir.path().join("gomodcache"));

    // The overlay includes instrument-hooks.go which uses cgo (`import "C"`).
    // If CGO_ENABLED=0 (e.g. no C compiler on a bare metal runner), Go silently
    // excludes the file, causing "undefined: InstrumentHooks" build errors.
    cmd.env("CGO_ENABLED", "1");

    Ok((_dir, cmd))
}

fn check_success(output: &std::process::Output) -> anyhow::Result<String> {
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        warn!("Command output: {stdout}");
        warn!("Command error output: {stderr}");

        bail!(
            "Failed to run benchmark. Exit status: {}\n\nStdout:\n{}\n\nStderr:\n{}",
            output.status,
            stdout,
            stderr
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Runs the cmd and returns the output.
pub fn run_with_stdout<P: AsRef<Path>>(
    profile_dir: P,
    dir: P,
    cli: &Cli,
) -> anyhow::Result<String> {
    let (_dir, mut cmd) = run_cmd(profile_dir, dir, cli)?;
    let output = cmd.output().context("Failed to execute go build command")?;
    check_success(&output)
}

/// Runs the cmd and forwards the output to stdout/stderr.
pub fn run<P: AsRef<Path>>(profile_dir: P, dir: P, cli: &Cli) -> anyhow::Result<()> {
    let (_dir, mut cmd) = run_cmd(profile_dir, dir, cli)?;
    let output = cmd
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("Failed to execute go build command")?;

    check_success(&output).map(|_| ())
}

fn find_go_binary() -> anyhow::Result<PathBuf> {
    let go_binary = overlay::find_goroot()?.join("bin").join("go");
    if !go_binary.exists() {
        bail!("Go binary doesn't exist at: {:?}", go_binary);
    }

    Ok(go_binary)
}
