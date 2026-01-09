use crate::prelude::*;
use anyhow::{Result, ensure};
use flate2::read::GzDecoder;
use std::path::PathBuf;
use tar::Archive;
use tempfile::TempDir;

const INSTRUMENT_HOOKS_REPO: &str = "CodSpeedHQ/instrument-hooks";
const INSTRUMENT_HOOKS_COMMIT: &str = "1752e9e4eae585e26703932d0055a1473dd77048";

/// Get the instrument-hooks directory, downloading if necessary
/// Downloads to /tmp/codspeed-instrument-hooks-{commit}/
pub fn download_instrument_hooks(temp_dir: &TempDir) -> Result<PathBuf> {
    let hooks_dir = temp_dir
        .path()
        .join(format!("instrument-hooks-{}", INSTRUMENT_HOOKS_COMMIT));

    if !hooks_dir.exists() {
        debug!("Downloading instrument-hooks to {:?}", hooks_dir);
        let url = format!(
            "https://github.com/{}/archive/{}.tar.gz",
            INSTRUMENT_HOOKS_REPO, INSTRUMENT_HOOKS_COMMIT
        );
        let content = reqwest::blocking::get(&url)?.bytes()?;

        // This will unpack to /tmp/codspeed-instrument-hooks-{commit}/
        let tar = GzDecoder::new(&*content);
        let mut archive = Archive::new(tar);
        archive.unpack(temp_dir.path())?;

        ensure!(hooks_dir.exists(), "Failed to download instrument-hooks");
    } else {
        debug!("Using existing instrument-hooks at {:?}", hooks_dir);
    }

    Ok(hooks_dir)
}
