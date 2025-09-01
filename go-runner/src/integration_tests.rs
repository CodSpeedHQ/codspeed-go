use itertools::Itertools;
use rstest::rstest;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::results::walltime_results::WalltimeResults;

fn assert_results_snapshots(profile_dir: &Path, project_name: &str) {
    let glob_pattern = profile_dir.join("results");
    if !glob_pattern.exists() {
        eprintln!("No results found for project: {project_name}");
        return;
    }

    let files = std::fs::read_dir(&glob_pattern)
        .expect("Failed to read results directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .map(|path| {
            let file = std::fs::File::open(&path).unwrap();
            serde_json::from_reader::<_, WalltimeResults>(file).unwrap()
        })
        // Ensure we have the correct order for multiple test executables
        .sorted_by_cached_key(|r| {
            r.benchmarks
                .iter()
                .map(|b| b.metadata.name.clone())
                .sorted()
                .join(";")
        })
        .collect::<Vec<_>>();

    for (i, mut content) in files.into_iter().enumerate() {
        content
            .benchmarks
            .sort_by_cached_key(|b| b.metadata.name.clone());

        let _guard = {
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_suffix(format!("{project_name}_{i}"));
            settings.bind_to_scope()
        };

        insta::assert_json_snapshot!(content, {
            ".creator.pid" => "[pid]",
            ".benchmarks[].stats" => "[stats]",
        });
    }
}

#[rstest]
// // #[case::caddy("caddy")]
#[case::fzf("fzf")]
#[case::opentelemetry_go("opentelemetry-go")]
#[case::golang_benchmarks("golang-benchmarks")]
#[case::zerolog("zerolog")]
#[case::zap("zap")]
#[case::hugo("hugo")]
// Currently not producing results:
#[case::fuego("fuego")]
#[case::cli_runtime("cli-runtime")]
#[case::example("example")]
fn test_build_and_run(#[case] project_name: &str) {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/projects")
        .join(project_name);

    let temp_dir = TempDir::new().unwrap();
    let profile_dir = temp_dir.path().join("profile");
    let cli = crate::cli::Cli {
        benchtime: "1x".into(),
        ..Default::default()
    };
    if let Err(error) = crate::run_benchmarks(&profile_dir, project_dir.as_path(), &cli) {
        panic!("Benchmarks couldn't run: {error}");
    }

    assert_results_snapshots(&profile_dir, project_name);
}
