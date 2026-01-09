use anyhow::{bail, ensure};
use semver::Version;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

mod instrument_hooks;

const OVERLAY_TEMPLATES: &[(&str, &str)] = &[
    ("codspeed.go", include_str!("../../../overlay/codspeed.go")),
    (
        "instrument-hooks.go",
        include_str!("../../../overlay/instrument-hooks.go"),
    ),
];

fn get_overlay_files(
    profile_dir: &Path,
    temp_dir: &TempDir,
) -> anyhow::Result<HashMap<String, String>> {
    let instrument_hooks_dir = instrument_hooks::download_instrument_hooks(temp_dir)?;

    let mut files = HashMap::new();

    // Select the appropriate benchmark file based on Go version
    let content = if detect_go_version()? >= Version::new(1, 25, 0) {
        include_str!("../../../overlay/benchmark1.25.0.go")
    } else {
        include_str!("../../../overlay/benchmark1.24.0.go")
    };
    files.insert("benchmark.go".to_string(), content.to_string());

    // Add other overlay files
    for (file_name, content) in OVERLAY_TEMPLATES {
        let content = content
            .replace(
                "@@INSTRUMENT_HOOKS_DIR@@",
                &instrument_hooks_dir.to_string_lossy(),
            )
            .replace("@@CODSPEED_PROFILE_DIR@@", &profile_dir.to_string_lossy())
            .replace("@@GO_RUNNER_VERSION@@", env!("CARGO_PKG_VERSION"));
        files.insert(file_name.to_string(), content);
    }
    Ok(files)
}

pub fn get_overlay_file(profile_dir: &Path) -> anyhow::Result<(TempDir, PathBuf)> {
    let overlay_dir = TempDir::new()?;
    let goroot_dir = find_goroot()?.join("src").join("testing");
    ensure!(goroot_dir.exists(), "GOROOT/src/testing does not exist");

    // Put all the overlay files into $GOROOT/src/testing
    let mut replaces = HashMap::new();
    for (file_name, content) in get_overlay_files(profile_dir, &overlay_dir)? {
        let real_path = goroot_dir.join(&file_name);
        let patch_path = overlay_dir.path().join(&file_name);

        std::fs::write(&patch_path, content)?;
        replaces.insert(real_path, patch_path);
    }

    // Construct the JSON string with the replaces
    let replaces = replaces
        .iter()
        .map(|(k, v)| (k.to_string_lossy(), v.to_string_lossy()))
        .collect::<HashMap<_, _>>();
    let json = serde_json::json!(
        {
            "Replace": replaces
        }
    )
    .to_string();

    let overlay_file = overlay_dir.path().join("overlay.json");
    std::fs::write(&overlay_file, json)?;
    Ok((overlay_dir, overlay_file))
}

pub fn find_goroot() -> anyhow::Result<PathBuf> {
    let output = Command::new("go").args(["env", "GOROOT"]).output()?;
    if !output.status.success() {
        bail!("Failed to find $GOROOT");
    }

    let goroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = PathBuf::from(goroot);
    if !path.exists() {
        bail!("GOROOT doesn't exist: {:?}", path);
    }

    Ok(path)
}

fn detect_go_version() -> anyhow::Result<Version> {
    let output = Command::new("go").args(["env", "GOVERSION"]).output()?;
    if !output.status.success() {
        bail!("Failed to get Go version");
    }
    let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Remove "go" prefix (e.g., "go1.25.2" -> "1.25.2")
    let version_str = version_str
        .strip_prefix("go")
        .ok_or_else(|| anyhow::anyhow!("Invalid Go version format: {}", version_str))?;

    Ok(Version::parse(version_str)?)
}
