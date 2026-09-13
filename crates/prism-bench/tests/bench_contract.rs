use std::fs;

use prism_bench::cli::{parse_args, CliError};
use prism_bench::dataset::Dataset;
use prism_bench::output::write_once;
use prism_bench::percentiles::percentile_nearest_rank;
use prism_bench::runner::{run_level, Level, RunConfig};

#[test]
fn nearest_rank_percentiles_are_deterministic() {
    let mut samples = [1, 2, 3, 4, 5];
    assert_eq!(percentile_nearest_rank(&mut samples, 50), 3);
    assert_eq!(percentile_nearest_rank(&mut samples, 95), 5);
}

#[test]
fn cli_requires_explicit_environment_metadata() {
    let error = parse_args(["bench", "--dataset", "smoke"]).unwrap_err();
    assert_eq!(error, CliError::MissingMetadata);
}

#[test]
fn raw_output_is_never_overwritten() {
    let path = std::env::temp_dir().join(format!("prism-bench-{}.jsonl", std::process::id()));
    fs::write(&path, "existing\n").unwrap();
    let error = write_once(&path, "replacement\n").unwrap_err();
    assert_eq!(error.to_string(), "output already exists");
    assert_eq!(fs::read_to_string(&path).unwrap(), "existing\n");
    fs::remove_file(path).unwrap();
}

#[test]
fn s1_runner_keeps_correctness_outside_timing() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/p0-smoke");
    let dataset = Dataset::load(&root).unwrap();
    let result = run_level(
        Level::B0,
        &dataset,
        &RunConfig {
            warmup: 0,
            samples: 1,
            warmup_frames: 0,
            convergence_window: 1_000,
            convergence_threshold_percent: 5,
            max_warmup_frames: 0,
            measured_frames: 1,
            repetitions: 1,
            concurrency: 1,
        },
    )
    .unwrap();
    assert_eq!(result.correctness_total, 1);
    assert_eq!(result.correctness_matches, 1);
    assert!(result.p95_ns >= result.p50_ns);
}

#[test]
fn external_100k_dataset_contract_is_opt_in() {
    let Some(path) = std::env::var_os("PRISM_100K_DATASET") else {
        eprintln!("skipped: PRISM_100K_DATASET is not set");
        return;
    };
    let dataset = Dataset::load(std::path::Path::new(&path)).unwrap();
    assert_eq!(dataset.fixture_count, 100_000);
    assert_eq!(dataset.fixtures.len(), dataset.expected.len());
}
