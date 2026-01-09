use anyhow::{bail, ensure};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

mod instrument_hooks;

const OVERLAY_TEMPLATES: &[(&str, &str)] = &[
    (
        "benchmark.go",
        include_str!("../../../overlay/benchmark.go"),
    ),
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

fn find_goroot() -> anyhow::Result<PathBuf> {
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
