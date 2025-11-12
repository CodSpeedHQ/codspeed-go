use codspeed_go_runner::results::raw_result::RawResult;
use std::time::Duration;
use tempfile::TempDir;

#[divan::bench(max_time = std::time::Duration::from_secs(5))]
fn bench_go_runner(bencher: divan::Bencher) {
    use std::path::PathBuf;
    use tempfile::TempDir;

    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/projects/example");

    bencher
        .with_inputs(|| {
            let temp_dir = TempDir::new().unwrap();
            let profile_dir = temp_dir.path().join("profile");
            let cli = codspeed_go_runner::cli::Cli {
                packages: vec!["./...".into()],
                dry_run: true,
                ..Default::default()
            };

            (profile_dir, cli)
        })
        .bench_refs(|(profile_dir, cli)| {
            if let Err(error) =
                codspeed_go_runner::run_benchmarks(profile_dir, project_dir.as_path(), cli)
            {
                panic!("Benchmarks couldn't run: {error}");
            }
        });
}

const TIME_ENTRIES: [usize; 5] = [100_000, 500_000, 1_000_000, 5_000_000, 10_000_000];
const FILE_COUNT: [usize; 3] = [5, 10, 25];

#[divan::bench(args = FILE_COUNT, consts = TIME_ENTRIES, max_time = Duration::from_secs(5))]
fn bench_collect_results<const N: usize>(bencher: divan::Bencher, file_count: usize) {
    use rand::prelude::*;

    fn random_raw_result<const N: usize>(rng: &mut StdRng) -> RawResult {
        let times_per_round = (0..N).map(|_| rng.random::<u64>() % 1_000_000).collect();
        let iters_per_round = (0..N).map(|_| rng.random::<u64>() % 1_000 + 1).collect();
        RawResult {
            name: "foo".into(),
            uri: "foo".into(),
            pid: 42,
            codspeed_time_per_round_ns: times_per_round,
            codspeed_iters_per_round: iters_per_round,
        }
    }

    bencher
        .with_inputs(|| {
            let mut rng = StdRng::seed_from_u64(42);

            let profile_dir = TempDir::new().unwrap();
            let raw_results = profile_dir.path().join("raw_results");
            std::fs::create_dir(&raw_results).unwrap();

            for (i, raw_result) in (0..file_count)
                .map(|_| random_raw_result::<N>(&mut rng))
                .enumerate()
            {
                let json = serde_json::to_string(&raw_result).unwrap();
                std::fs::write(raw_results.join(format!("{i}.json")), json).unwrap();
            }

            profile_dir
        })
        .bench_refs(|profile_dir| {
            if let Err(error) = codspeed_go_runner::collect_walltime_results(profile_dir.path()) {
                panic!("Collecting results failed: {error}");
            }

            // Ensure that we have a results folder with the pid
            let results_dir: std::path::PathBuf = profile_dir.path().join("results");
            assert!(results_dir.exists());
            let result_file = results_dir.join("42.json");
            assert!(result_file.exists());
        });
}

fn main() {
    divan::main();
}
