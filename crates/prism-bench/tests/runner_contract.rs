use std::path::Path;

use prism_bench::dataset::Dataset;
use prism_bench::percentiles::percentile_nearest_rank_thousandths;
use prism_bench::runner::{run_level, Level, RunConfig};

#[test]
fn nearest_rank_supports_p99_9() {
    let mut samples = [1, 2, 3, 4, 5];
    assert_eq!(percentile_nearest_rank_thousandths(&mut samples, 99_900), 5);
}

#[test]
fn runner_emits_five_repetitions_and_complete_metrics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/p0-smoke");
    let dataset = Dataset::load(&root).unwrap();
    let config = RunConfig {
        warmup: 0,
        samples: 10,
        warmup_frames: 0,
        convergence_window: 1_000,
        convergence_threshold_percent: 5,
        max_warmup_frames: 0,
        measured_frames: 10,
        repetitions: 5,
        concurrency: 1,
    };
    let run = run_level(Level::B0, &dataset, &config).unwrap();
    assert_eq!(run.repetitions.len(), 5);
    assert_eq!(run.correctness_total, 50);
    assert_eq!(run.correctness_matches, 50);
    assert!(run.p99_9_ns >= run.p99_ns);
    assert!(run.max_ns >= run.p99_9_ns);
    assert!(run.frames_per_sec > 0.0);
    assert!(run.mb_per_sec > 0.0);
}
