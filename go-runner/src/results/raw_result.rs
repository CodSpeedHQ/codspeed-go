use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::results::walltime_results::WalltimeBenchmark;

// WARN: Keep in sync with Golang "testing" fork (benchmark.go)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawResult {
    pub benchmark_name: String,
    pub pid: u32,
    pub codspeed_time_per_round_ns: Vec<u64>,

    #[serde(default)]
    pub codspeed_iters_per_round: Vec<u64>,
}

impl RawResult {
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        serde_json::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse raw result: {}", e))
    }

    pub fn parse_folder<P: AsRef<Path>>(folder: P) -> anyhow::Result<Vec<Self>> {
        let glob_pattern = folder.as_ref().join("raw_results").join("*.json");
        Ok(glob::glob(&glob_pattern.to_string_lossy())?
            .filter_map(Result::ok)
            .filter_map(|path| {
                let content = std::fs::read_to_string(&path).ok()?;
                Self::parse(&content).ok()
            })
            .collect())
    }

    pub fn into_walltime_benchmark(self, file_path: Option<String>) -> WalltimeBenchmark {
        let name = self.benchmark_name;

        let file = file_path.as_deref().unwrap_or("unknown");
        let uri = format!("{file}::{name}");

        let times_per_round_ns = self
            .codspeed_time_per_round_ns
            .iter()
            .map(|t| *t as u128)
            .collect::<Vec<_>>();
        let iters_per_round = if self.codspeed_iters_per_round.is_empty() {
            vec![1; times_per_round_ns.len()]
        } else {
            self.codspeed_iters_per_round
                .iter()
                .map(|i| *i as u128)
                .collect()
        };

        WalltimeBenchmark::from_runtime_data(name, uri, iters_per_round, times_per_round_ns, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_result_deserialization() {
        let json_data = include_str!("../../testdata/raw_results/BenchmarkFibonacci20-16.json");
        let result: RawResult = serde_json::from_str(json_data).unwrap();

        assert_eq!(result.benchmark_name, "BenchmarkFibonacci20-16");
        assert_eq!(result.pid, 777767);
        assert_eq!(result.codspeed_time_per_round_ns.len(), 3);
        assert_eq!(result.codspeed_iters_per_round.len(), 0); // Default: 1 per round
    }

    #[test]
    fn test_into_walltime_benchmark_with_file_path() {
        let raw_result = RawResult {
            benchmark_name: "BenchmarkFibonacci20-16".to_string(),
            pid: 777767,
            codspeed_time_per_round_ns: vec![1000, 2000, 3000],
            codspeed_iters_per_round: vec![],
        };

        // Test with file path - should not panic and create successfully
        let _walltime_bench = raw_result
            .clone()
            .into_walltime_benchmark(Some("pkg/foo/fib_test.go".to_string()));

        // Test without file path (should default to TODO) - should not panic and create successfully
        let _walltime_bench_no_path = raw_result.into_walltime_benchmark(None);
    }
}
