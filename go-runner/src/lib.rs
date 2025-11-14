use crate::{
    builder::{BenchmarkPackage, templater::Templater},
    prelude::*,
    results::{raw_result::RawResult, walltime_results::WalltimeBenchmark},
};
use std::{collections::HashMap, path::Path};

pub mod builder;
pub mod cli;
pub mod prelude;
pub mod results;
pub mod runner;
pub(crate) mod utils;

#[cfg(test)]
mod integration_tests;

/// Builds and runs the specified Go project benchmarks, writing results to the .codspeed folder.
pub fn run_benchmarks<P: AsRef<Path>>(
    profile_dir: P,
    project_dir: &Path,
    cli: &crate::cli::Cli,
) -> anyhow::Result<()> {
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
    let templater = Templater::new();
    for package in &packages {
        info!("Generating custom runner for package: {}", package.name);
        let runner_path = templater.run(package, &profile_dir)?;

        info!("Building binary for package: {}", package.name);

        let binary_path = match builder::build_binary(&runner_path) {
            Ok(binary_path) => binary_path,
            Err(e) => {
                if cfg!(test) {
                    panic!("Failed to build {}: {e}", package.name);
                } else {
                    error!("Failed to build {}: {e}", package.name);
                    continue;
                }
            }
        };

        if !cli.dry_run {
            if let Err(error) = runner::run(
                &binary_path,
                &["-test.bench", &cli.bench, "-test.benchtime", &cli.benchtime],
            ) {
                error!("Failed to run benchmarks for {}: {error}", package.name);
                continue;
            }
        } else {
            info!("Skipping benchmark execution (dry-run mode)");
        }
    }

    // 3. Collect the results
    if !cli.dry_run {
        collect_walltime_results(profile_dir.as_ref())?;
    }

    Ok(())
}

// TODO: This should be merged with codspeed-rust/codspeed/walltime_results.rs
pub fn collect_walltime_results(profile_dir: &Path) -> anyhow::Result<()> {
    let mut benchmarks_by_pid: HashMap<u32, Vec<WalltimeBenchmark>> = HashMap::new();

    for (pid, walltime_result) in RawResult::parse_folder(profile_dir)?.into_iter() {
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
