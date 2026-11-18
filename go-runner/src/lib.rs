use crate::{
    prelude::*,
    results::{raw_result::RawResult, walltime_results::WalltimeBenchmark},
};
use flate2::{Compression, write::GzEncoder};
use std::{
    collections::HashMap,
    io::{BufWriter, IntoInnerError},
    path::Path,
};

pub mod cli;
pub mod prelude;
pub mod results;
pub mod runner;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(test)]
mod integration_tests;

/// Builds and runs the specified Go project benchmarks, writing results to the .codspeed folder.
pub fn run_benchmarks<P: AsRef<Path>>(
    profile_dir: P,
    project_dir: P,
    cli: &crate::cli::Cli,
) -> anyhow::Result<()> {
    if let Err(error) = runner::run(&profile_dir, &project_dir, cli) {
        bail!("Failed to run benchmarks: {error}");
    }

    let profile_dir = profile_dir.as_ref().to_path_buf();
    collect_walltime_results(&profile_dir).unwrap();

    Ok(())
}

// TODO: This should be merged with codspeed-rust/codspeed/walltime_results.rs
pub fn collect_walltime_results(profile_dir: &Path) -> anyhow::Result<()> {
    let mut benchmarks_by_pid: HashMap<u32, Vec<WalltimeBenchmark>> = HashMap::new();

    let raw_results_dir = profile_dir.join("raw_results");
    for (pid, walltime_result) in RawResult::parse_folder(&raw_results_dir)?.into_iter() {
        benchmarks_by_pid
            .entry(pid)
            .or_default()
            .push(walltime_result);
    }

    for (pid, walltime_benchmarks) in benchmarks_by_pid {
        let creator = results::walltime_results::Creator {
            name: "codspeed-go".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            pid,
        };
        let results_dir = profile_dir.join("results");
        std::fs::create_dir_all(&results_dir)?;

        let results_file = results_dir.join(format!("{pid}.json"));
        let mut walltime_results =
            results::walltime_results::WalltimeResults::new(walltime_benchmarks, creator)?;
        std::fs::write(&results_file, serde_json::to_string(&walltime_results)?)?;
        info!("Results written to {results_file:?}");

        let rounds_dir = profile_dir.join("walltime_rounds");
        std::fs::create_dir_all(&rounds_dir)?;
        let rounds_file = std::fs::File::create(rounds_dir.join(format!("{pid}.json.gz")))?;
        let mut rounds_writer = BufWriter::new(GzEncoder::new(rounds_file, Compression::fast()));
        serde_json::to_writer(&mut rounds_writer, &walltime_results.take_walltime_rounds())?;
        rounds_writer
            .into_inner()
            .map_err(IntoInnerError::into_error)?
            .finish()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;

    #[test]
    fn test_collect_walltime_results_writes_gzipped_rounds() {
        let profile_dir = tempfile::TempDir::new().unwrap();
        let raw_results_dir = profile_dir.path().join("raw_results");
        std::fs::create_dir_all(&raw_results_dir).unwrap();
        std::fs::write(
            raw_results_dir.join("bench.json"),
            serde_json::json!({
                "name": "BenchmarkFoo",
                "uri": "pkg/foo_test.go::BenchmarkFoo",
                "pid": 42,
                "codspeed_time_per_round_ns": [3000, 1000, 2000],
                "codspeed_iters_per_round": [3, 1, 2],
            })
            .to_string(),
        )
        .unwrap();

        collect_walltime_results(profile_dir.path()).unwrap();

        let rounds_file =
            std::fs::File::open(profile_dir.path().join("walltime_rounds/42.json.gz")).unwrap();
        let rounds: serde_json::Value =
            serde_json::from_reader(GzDecoder::new(rounds_file)).unwrap();
        assert_eq!(
            rounds,
            serde_json::json!({"benchmarks": [{
                "uri": "pkg/foo_test.go::BenchmarkFoo",
                "times_per_round_ns": [3000, 1000, 2000],
                "iters_per_round": [3, 1, 2],
            }]})
        );
    }
}
