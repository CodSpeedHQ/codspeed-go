use crate::{builder::BenchmarkPackage, prelude::*};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub mod builder;
pub mod cli;
pub mod prelude;
mod results;
pub mod runner;
pub(crate) mod utils;

#[cfg(test)]
mod integration_tests;

/// Builds and runs the specified Go project benchmarks, writing results to the .codspeed folder.
pub fn run_benchmarks(project_dir: &Path, cli: &crate::cli::Cli) -> anyhow::Result<()> {
    let profile_dir = std::env::var("CODSPEED_PROFILE_FOLDER")
        .context("CODSPEED_PROFILE_FOLDER env var not set")?;
    std::fs::remove_dir_all(&profile_dir).ok();

    // 1. Build phase - Benchmark and package discovery
    let packages = BenchmarkPackage::from_project(project_dir, &cli.packages)?;
    info!("Discovered {} packages", packages.len());

    let total_benchmarks: usize = packages.iter().map(|p| p.benchmarks.len()).sum();
    info!("Total benchmarks discovered: {total_benchmarks}");

    for package in &packages {
        for benchmark in &package.benchmarks {
            info!("Found {:30} in {:?}", benchmark.name, benchmark.file_path);
        }
    }

    // 2. Generate codspeed runners, build binaries, and execute them
    for package in &packages {
        info!("Generating custom runner for package: {}", package.name);
        let (_target_dir, runner_path) = builder::templater::run(package)?;

        info!("Building binary for package: {}", package.name);
        let binary_path = builder::build_binary(&runner_path)?;

        runner::run(
            &binary_path,
            &["-test.bench", &cli.bench, "-test.benchtime", &cli.benchtime],
        )?;
    }

    // 3. Collect the results
    collect_walltime_results()?;

    Ok(())
}

// TODO: This should be merged with codspeed-rust/codspeed/walltime_results.rs
fn collect_walltime_results() -> anyhow::Result<()> {
    let profile_dir = std::env::var("CODSPEED_PROFILE_FOLDER")
        .context("CODSPEED_PROFILE_FOLDER env var not set")?;
    let profile_dir = PathBuf::from(&profile_dir);
    let raw_results = results::raw_result::RawResult::parse_folder(&profile_dir)?;
    info!("Parsed {} raw results", raw_results.len());

    let mut benchmarks_by_pid: HashMap<u32, Vec<results::walltime_results::WalltimeBenchmark>> =
        HashMap::new();
    for raw in raw_results {
        benchmarks_by_pid
            .entry(raw.pid)
            .or_default()
            .push(raw.into_walltime_benchmark());
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
