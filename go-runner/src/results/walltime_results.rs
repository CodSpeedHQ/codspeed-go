// NOTE: This file was taken from `codspeed-rust` and modified a bit to fit this project.
// TODO(COD-4241): call the `codspeed` crate's percentile and round sampling code instead of
// copying it.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use statrs::statistics::{Data, Distribution, Max, Min};
use std::collections::BTreeSet;

const IQR_OUTLIER_FACTOR: f64 = 1.5;
const STDEV_OUTLIER_FACTOR: f64 = 3.0;
/// Above this, the rounds file keeps a uniformly random subsample of the rounds, in execution order.
const MAX_WALLTIME_ROUNDS_PER_BENCHMARK: usize = 100_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkStats {
    min_ns: f64,
    max_ns: f64,
    mean_ns: f64,
    stdev_ns: f64,

    q1_ns: f64,
    median_ns: f64,
    q3_ns: f64,
    /// p1..p99 of the time per iteration: `[24]`, `[49]` and `[74]` equal `q1_ns`, `median_ns` and `q3_ns`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    percentiles_ns: Option<Vec<f64>>,

    rounds: u64,
    total_time: f64,
    iqr_outlier_rounds: u64,
    stdev_outlier_rounds: u64,
    iter_per_round: u64,
    warmup_iters: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct BenchmarkConfig {
    warmup_time_ns: Option<f64>,
    min_round_time_ns: Option<f64>,
    max_time_ns: Option<f64>,
    max_rounds: Option<u64>,
}

#[derive(Debug, Serialize, Default)]
struct BenchmarkRounds {
    times_per_round_ns: Vec<u64>,
    iters_per_round: Vec<u64>,
}

#[derive(Debug, Serialize)]
struct UriRounds {
    uri: String,
    #[serde(flatten)]
    rounds: BenchmarkRounds,
}

/// Content of `walltime_rounds/<pid>.json.gz` in the profile folder
#[derive(Debug, Serialize)]
pub struct WalltimeRounds {
    benchmarks: Vec<UriRounds>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalltimeBenchmark {
    #[serde(flatten)]
    pub metadata: BenchmarkMetadata,

    config: BenchmarkConfig,
    stats: BenchmarkStats,

    /// Rounds in execution order, capped at [`MAX_WALLTIME_ROUNDS_PER_BENCHMARK`]: fewer than
    /// `stats.rounds` when subsampled
    #[serde(skip)]
    walltime_rounds: BenchmarkRounds,
}

impl WalltimeBenchmark {
    pub fn from_runtime_data(
        name: String,
        uri: String,
        iters_per_round: Vec<u64>,
        times_per_round_ns: Vec<u64>,
        max_time_ns: Option<u128>,
    ) -> Self {
        let total_time =
            times_per_round_ns.iter().map(|&t| t as u128).sum::<u128>() as f64 / 1_000_000_000.0;
        let mut time_per_iteration_per_round_ns: Vec<_> = times_per_round_ns
            .iter()
            .zip(&iters_per_round)
            .map(|(time_per_round, iter_per_round)| {
                *time_per_round as u128 / *iter_per_round as u128
            })
            .map(|t| t as f64)
            .collect::<Vec<f64>>();

        // statrs' `quantile` formula (R-8) on rounds sorted once: statrs itself runs an O(n) selection per call
        time_per_iteration_per_round_ns.sort_unstable_by(f64::total_cmp);
        let sorted = &time_per_iteration_per_round_ns;
        let percentiles_ns: Vec<f64> = (1..100)
            .map(|p| {
                let h = (sorted.len() as f64 + 1.0 / 3.0) * (p as f64 / 100.0) + 1.0 / 3.0;
                let hf = h as usize;
                if hf == 0 {
                    return sorted[0];
                }
                if hf >= sorted.len() {
                    return sorted[sorted.len() - 1];
                }
                let (a, b) = (sorted[hf - 1], sorted[hf]);
                a + (h - hf as f64) * (b - a)
            })
            .collect();
        let q1_ns = percentiles_ns[24];
        let median_ns = percentiles_ns[49];
        let q3_ns = percentiles_ns[74];

        let data = Data::new(time_per_iteration_per_round_ns);
        let rounds: u64 = data.len() as u64;

        let mean_ns = data.mean().unwrap();

        let stdev_ns = if data.len() < 2 {
            // std_dev() returns f64::NAN if data has less than two entries, so we have to
            // manually handle this case.
            0.0
        } else {
            data.std_dev().unwrap()
        };

        let iqr_ns = q3_ns - q1_ns;
        let iqr_outlier_rounds = data
            .iter()
            .filter(|&&t| {
                t < q1_ns - IQR_OUTLIER_FACTOR * iqr_ns || t > q3_ns + IQR_OUTLIER_FACTOR * iqr_ns
            })
            .count() as u64;

        let stdev_outlier_rounds = data
            .iter()
            .filter(|&&t| {
                t < mean_ns - STDEV_OUTLIER_FACTOR * stdev_ns
                    || t > mean_ns + STDEV_OUTLIER_FACTOR * stdev_ns
            })
            .count() as u64;

        let min_ns = data.min();
        let max_ns = data.max();

        // TODO(COD-1056): We currently only support single iteration count per round
        let iter_per_round = (iters_per_round.iter().map(|&t| t as u128).sum::<u128>()
            / iters_per_round.len() as u128) as u64;
        let warmup_iters = 0; // FIXME: add warmup detection

        let stats = BenchmarkStats {
            min_ns,
            max_ns,
            mean_ns,
            stdev_ns,
            q1_ns,
            median_ns,
            q3_ns,
            percentiles_ns: Some(percentiles_ns),
            rounds,
            total_time,
            iqr_outlier_rounds,
            stdev_outlier_rounds,
            iter_per_round,
            warmup_iters,
        };

        let walltime_rounds = if times_per_round_ns.len() <= MAX_WALLTIME_ROUNDS_PER_BENCHMARK {
            BenchmarkRounds {
                times_per_round_ns,
                iters_per_round,
            }
        } else {
            // Floyd's sampling of distinct indices; the set iterates them in execution order
            let mut random = vec![0u8; 8 * MAX_WALLTIME_ROUNDS_PER_BENCHMARK];
            getrandom::fill(&mut random).expect("Failed to generate random bytes");
            let mut kept_indices = BTreeSet::new();
            let n = times_per_round_ns.len();
            for (j, r) in (n - MAX_WALLTIME_ROUNDS_PER_BENCHMARK..n).zip(random.chunks_exact(8)) {
                let t = (u64::from_ne_bytes(r.try_into().unwrap()) % (j as u64 + 1)) as usize;
                if !kept_indices.insert(t) {
                    kept_indices.insert(j);
                }
            }
            BenchmarkRounds {
                times_per_round_ns: kept_indices
                    .iter()
                    .map(|&i| times_per_round_ns[i])
                    .collect(),
                iters_per_round: kept_indices.iter().map(|&i| iters_per_round[i]).collect(),
            }
        };

        WalltimeBenchmark {
            metadata: BenchmarkMetadata { name, uri },
            config: BenchmarkConfig {
                max_time_ns: max_time_ns.map(|t| t as f64),
                ..Default::default()
            },
            stats,
            walltime_rounds,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Instrument {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Creator {
    pub name: String,
    pub version: String,
    pub pid: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalltimeResults {
    creator: Creator,
    instrument: Instrument,
    pub benchmarks: Vec<WalltimeBenchmark>,
}

impl WalltimeResults {
    pub fn new(benchmarks: Vec<WalltimeBenchmark>, creator: Creator) -> Result<Self> {
        Ok(WalltimeResults {
            instrument: Instrument {
                type_: "walltime".to_string(),
            },
            creator,
            benchmarks,
        })
    }

    pub fn take_walltime_rounds(&mut self) -> WalltimeRounds {
        WalltimeRounds {
            benchmarks: self
                .benchmarks
                .iter_mut()
                .map(|bench| UriRounds {
                    uri: bench.metadata.uri.clone(),
                    rounds: std::mem::take(&mut bench.walltime_rounds),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "benchmark";
    const URI: &str = "test::benchmark";

    #[test]
    fn test_parse_single_benchmark() {
        let benchmark = WalltimeBenchmark::from_runtime_data(
            NAME.to_string(),
            URI.to_string(),
            vec![1],
            vec![42],
            None,
        );
        assert_eq!(benchmark.stats.stdev_ns, 0.);
        assert_eq!(benchmark.stats.min_ns, 42.);
        assert_eq!(benchmark.stats.max_ns, 42.);
        assert_eq!(benchmark.stats.mean_ns, 42.);
    }

    #[test]
    fn test_parse_bench_with_variable_iterations() {
        let iters_per_round = vec![1, 2, 3, 4, 5, 6];
        let total_rounds = iters_per_round.iter().sum::<u64>() as f64;

        let benchmark = WalltimeBenchmark::from_runtime_data(
            NAME.to_string(),
            URI.to_string(),
            iters_per_round,
            vec![42, 42 * 2, 42 * 3, 42 * 4, 42 * 5, 42 * 6],
            None,
        );

        assert_eq!(benchmark.stats.stdev_ns, 0.);
        assert_eq!(benchmark.stats.min_ns, 42.);
        assert_eq!(benchmark.stats.max_ns, 42.);
        assert_eq!(benchmark.stats.mean_ns, 42.);
        assert_eq!(
            benchmark.stats.total_time,
            42. * total_rounds / 1_000_000_000.0
        );
    }

    #[test]
    fn test_percentiles_match_statrs() {
        for rounds in [1u64, 2, 3, 10, 100] {
            // 1..=rounds in a shuffled execution order
            let times: Vec<u64> = (1..=rounds).map(|i| i * 37 % (rounds + 1)).collect();
            let benchmark = WalltimeBenchmark::from_runtime_data(
                NAME.to_string(),
                URI.to_string(),
                vec![1; times.len()],
                times.clone(),
                None,
            );

            let percentiles = benchmark.stats.percentiles_ns.unwrap();
            let mut data = Data::new(times.iter().map(|&t| t as f64).collect::<Vec<_>>());
            let expected: Vec<f64> = (1..100)
                .map(|p| statrs::statistics::OrderStatistics::percentile(&mut data, p))
                .collect();
            assert_eq!(percentiles, expected, "{rounds} rounds");
            assert!(percentiles.windows(2).all(|w| w[0] <= w[1]));
            assert_eq!(benchmark.stats.q1_ns, percentiles[24]);
            assert_eq!(benchmark.stats.median_ns, percentiles[49]);
            assert_eq!(benchmark.stats.q3_ns, percentiles[74]);
        }
    }

    #[test]
    fn test_walltime_rounds_capped_in_execution_order() {
        let rounds = MAX_WALLTIME_ROUNDS_PER_BENCHMARK + 50_000;
        let iters: Vec<u64> = (1..=rounds as u64).collect();
        let times: Vec<u64> = iters.iter().map(|i| i * 10).collect();
        let benchmark = WalltimeBenchmark::from_runtime_data(
            NAME.to_string(),
            URI.to_string(),
            iters,
            times,
            None,
        );

        assert_eq!(benchmark.stats.rounds, rounds as u64);
        let kept = benchmark.walltime_rounds;
        assert_eq!(
            kept.times_per_round_ns.len(),
            MAX_WALLTIME_ROUNDS_PER_BENCHMARK
        );
        assert_eq!(
            kept.iters_per_round.len(),
            MAX_WALLTIME_ROUNDS_PER_BENCHMARK
        );
        // A random subsample, not the first rounds
        assert!(*kept.iters_per_round.last().unwrap() > MAX_WALLTIME_ROUNDS_PER_BENCHMARK as u64);
        assert!(kept.times_per_round_ns.windows(2).all(|w| w[0] < w[1]));
        assert!(
            kept.times_per_round_ns
                .iter()
                .zip(&kept.iters_per_round)
                .all(|(&t, &i)| t == i * 10 && (1..=rounds as u64).contains(&i))
        );
    }

    #[test]
    fn test_take_walltime_rounds() {
        let mut results = WalltimeResults::new(
            vec![WalltimeBenchmark::from_runtime_data(
                NAME.to_string(),
                URI.to_string(),
                vec![1, 2],
                vec![30, 50],
                None,
            )],
            Creator {
                name: "codspeed-go".to_string(),
                version: "0.0.0".to_string(),
                pid: 1,
            },
        )
        .unwrap();

        let rounds = serde_json::to_value(results.take_walltime_rounds()).unwrap();
        assert_eq!(
            rounds,
            serde_json::json!({"benchmarks": [
                {"uri": URI, "times_per_round_ns": [30, 50], "iters_per_round": [1, 2]}
            ]})
        );
        let results = serde_json::to_value(&results).unwrap();
        assert!(results["benchmarks"][0].get("walltime_rounds").is_none());
        assert_eq!(
            results["benchmarks"][0]["stats"]["percentiles_ns"]
                .as_array()
                .unwrap()
                .len(),
            99
        );
    }
}
