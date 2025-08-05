use crate::{builder::BenchmarkPackage, prelude::*};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod builder;
pub mod cli;
pub mod prelude;
mod results;
pub(crate) mod utils;

#[cfg(test)]
mod integration_tests;

/// Builds and runs the specified Go project benchmarks, writing results to the .codspeed folder.
pub fn run_benchmarks(project_dir: &Path, bench: &str) -> anyhow::Result<()> {
    let profile_dir = std::env::var("CODSPEED_PROFILE_FOLDER")
        .context("CODSPEED_PROFILE_FOLDER env var not set")?;
    std::fs::remove_dir_all(&profile_dir).ok();

    // 1. Build phase - Benchmark and package discovery
    let packages = BenchmarkPackage::from_project(project_dir)?;
    info!("Discovered {} packages", packages.len());

    let mut bench_name_to_path = HashMap::new();
    for package in &packages {
        for benchmark in &package.benchmarks {
            bench_name_to_path.insert(benchmark.name.clone(), benchmark.file_path.clone());
        }
    }

    let total_benchmarks: usize = packages.iter().map(|p| p.benchmarks.len()).sum();
    info!("Total benchmarks discovered: {total_benchmarks}");
    for (name, path) in &bench_name_to_path {
        info!("Found {name:30} in {path:?}");
    }

    // 2. Generate codspeed runners and execute them
    for package in &packages {
        info!("Generating custom runner for package: {}", package.name);
        let (_target_dir, runner_path) = builder::templater::run(package)?;

        let args = [
            "-test.bench",
            bench,
            // Use a single iteration in tests to speed up execution, otherwise use 5 seconds
            "-test.benchtime",
            if cfg!(test) || std::env::var("CODSPEED_ENV").is_err() {
                "1x"
            } else {
                "5s"
            },
        ];

        info!("Running benchmarks for package: {}", package.name);
        builder::runner::run(&runner_path, &args)?;
    }

    // 3. Collect the results
    collect_walltime_results(bench_name_to_path)?;

    Ok(())
}

// TODO: This should be merged with codspeed-rust/codspeed/walltime_results.rs
fn collect_walltime_results(bench_name_to_path: HashMap<String, PathBuf>) -> anyhow::Result<()> {
    let profile_dir = std::env::var("CODSPEED_PROFILE_FOLDER")
        .context("CODSPEED_PROFILE_FOLDER env var not set")?;
    let profile_dir = PathBuf::from(&profile_dir);
    let raw_results = results::raw_result::RawResult::parse_folder(&profile_dir)?;
    info!("Parsed {} raw results", raw_results.len());

    let mut benchmarks_by_pid: HashMap<u32, Vec<results::walltime_results::WalltimeBenchmark>> =
        HashMap::new();
    for raw in raw_results {
        let file_path = bench_name_to_path
            .get(&raw.benchmark_name)
            .map(|p| p.to_string_lossy().to_string());
        benchmarks_by_pid
            .entry(raw.pid)
            .or_default()
            .push(raw.into_walltime_benchmark(file_path));
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
