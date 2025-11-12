use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::results::walltime_results::WalltimeBenchmark;

// WARN: Keep in sync with Golang "testing" fork (benchmark.go)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawResult {
    pub name: String,
    pub uri: String,
    pub pid: u32,
    pub codspeed_time_per_round_ns: Vec<u64>,
    pub codspeed_iters_per_round: Vec<u64>,
}

impl RawResult {
    pub fn parse_folder<P: AsRef<Path>>(
        folder: P,
    ) -> anyhow::Result<Vec<(u32, WalltimeBenchmark)>> {
        let glob_pattern = folder.as_ref().join("raw_results").join("*.json");
        let result = glob::glob(&glob_pattern.to_string_lossy())?
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|path| {
                let file = std::fs::File::open(&path).ok()?;
                let reader = std::io::BufReader::new(file);
                let json: Self = serde_json::from_reader(reader).ok()?;
                Some((
                    json.pid,
                    WalltimeBenchmark::from_runtime_data(
                        json.name,
                        json.uri,
                        &json.codspeed_iters_per_round,
                        &json.codspeed_time_per_round_ns,
                        None,
                    ),
                ))
            })
            .collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_result_deserialization() {
        let json_data = r#"{
    "name": "BenchmarkFibonacci20-16",
    "uri": "pkg/foo/fib_test.go::BenchmarkFibonacci20-16",
    "pid": 777767,
    "codspeed_time_per_round_ns": [1000, 2000, 3000],
    "codspeed_iters_per_round": [1, 2, 3]
}"#;
        let result: RawResult = serde_json::from_str(json_data).unwrap();

        assert_eq!(result.name, "BenchmarkFibonacci20-16");
        assert_eq!(result.pid, 777767);
        assert_eq!(result.codspeed_time_per_round_ns.len(), 3);
        assert_eq!(result.codspeed_iters_per_round.len(), 3);
    }
}
