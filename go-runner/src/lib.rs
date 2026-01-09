use crate::{
    prelude::*,
    results::{raw_result::RawResult, walltime_results::WalltimeBenchmark},
};
use std::{collections::HashMap, path::Path};

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
        let walltime_results =
            results::walltime_results::WalltimeResults::new(walltime_benchmarks, creator)?;
        std::fs::write(&results_file, serde_json::to_string(&walltime_results)?)?;
        info!("Results written to {results_file:?}");
    }

    Ok(())
}
