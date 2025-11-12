#[cfg(test)]
use crate::prelude::*;
use std::io;
use std::path::{Path, PathBuf};

pub fn copy_dir_recursively(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let excludes = vec!["node_modules".into(), "target".into()];
    let includes = vec![];
    dircpy::copy_dir_advanced(src, dst, true, true, true, excludes, includes)?;
    Ok(())
}

pub fn get_parent_git_repo_path(abs_path: &Path) -> io::Result<PathBuf> {
    if abs_path.join(".git").exists() {
        Ok(abs_path.to_path_buf())
    } else {
        get_parent_git_repo_path(
            abs_path
                .parent()
                .ok_or(io::Error::from(io::ErrorKind::NotFound))?,
        )
    }
}

pub fn get_git_relative_path<P>(abs_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    if let Ok(canonicalized_abs_path) = abs_path.as_ref().canonicalize() {
        // `repo_path` is still canonicalized as it is a subpath of `canonicalized_abs_path`
        if let Ok(repo_path) = get_parent_git_repo_path(&canonicalized_abs_path) {
            canonicalized_abs_path
                .strip_prefix(repo_path)
                .expect("Repository path is malformed.")
                .to_path_buf()
        } else {
            canonicalized_abs_path
        }
    } else {
        abs_path.as_ref().to_path_buf()
    }
}

#[cfg(test)]
fn go_version() -> anyhow::Result<semver::Version> {
    use anyhow::Context;

    let output = std::process::Command::new("go").arg("version").output()?;
    if !output.status.success() {
        panic!("Failed to get Go version");
    }
    let output = String::from_utf8_lossy(&output.stdout);

    // Example output: go version go1.24.9 linux/amd64
    let go_version_str = output
        .split_whitespace()
        .nth(2)
        .context("Failed to parse Go version")?
        .strip_prefix("go")
        .context("Failed to strip 'go' prefix")?;
    Ok(semver::Version::parse(go_version_str)?)
}

// Check whether the go.mod expects a specific Go version, and skip the test if not met
#[cfg(test)]
fn project_go_version(project_dir: &Path) -> Option<semver::VersionReq> {
    let go_mod_path = project_dir.join("go.mod");
    if !go_mod_path.exists() {
        return None;
    }

    let go_mod_content = std::fs::read_to_string(go_mod_path).unwrap_or_default();
    for line in go_mod_content.lines() {
        let Some(version_str) = line.strip_prefix("go ") else {
            continue;
        };

        return semver::VersionReq::parse(version_str).ok();
    }

    None
}

#[cfg(test)]
pub fn can_build_project(project_dir: &Path) -> bool {
    if let Some(required_version) = project_go_version(project_dir) {
        let current_version = go_version().unwrap();
        info!(
            "Project requires Go {}, current version is {}",
            required_version, current_version
        );
        required_version.matches(&current_version)
    } else {
        true
    }
}
