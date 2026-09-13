use std::path::Path;

use prism_bench::dataset::Dataset;
use prism_bench::percentiles::percentile_nearest_rank_thousandths;
use prism_bench::burst::{Calibration, Phase, PhaseSchedule};
use prism_bench::runner::{run_level, run_phase, Level, RunConfig};

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
    assert_eq!(run.warmup_target, 0);
    assert_eq!(run.warmup_frames, 0);
    assert_eq!(run.convergence_window, 1_000);
    assert_eq!(run.convergence_threshold_percent, 5);
    assert!(run.converged);
    assert_eq!(run.measured_frames, 10);
    assert_eq!(run.typed_rejections, 0);
    assert_eq!(run.execution_failures, 0);
    assert!(run.p99_9_ns >= run.p99_ns);
    assert!(run.max_ns >= run.p99_9_ns);
    assert!(run.frames_per_sec > 0.0);
    assert!(run.mb_per_sec > 0.0);
}

fn parallel_config(concurrency: usize) -> RunConfig {
    RunConfig {
        warmup: 0,
        samples: 16,
        warmup_frames: 0,
        convergence_window: 1_000,
        convergence_threshold_percent: 5,
        max_warmup_frames: 0,
        measured_frames: 16,
        repetitions: 1,
        concurrency,
    }
}

#[test]
fn runner_processes_complete_frame_set_with_two_workers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/p0-smoke");
    let dataset = Dataset::load(&root).unwrap();
    let run = run_level(Level::B1, &dataset, &parallel_config(2)).unwrap();
    assert_eq!(run.repetitions[0].frames, 16);
    assert_eq!(run.correctness_total, 16);
    assert_eq!(run.correctness_matches, 16);
    assert!(run.repetitions[0].frames_per_sec > 0.0);
}

#[test]
fn sustained_config_requires_five_100k_repetitions() {
    use prism_bench::sustained::SustainedConfig;
    assert_eq!(SustainedConfig { duration_seconds: 900, window_frames: 100_000, repetitions: 5 }.window_frames, 100_000);
}

#[test]
fn resource_snapshot_preserves_optional_cpu_metrics_and_monotonic_order() {
    use prism_bench::sustained::ResourceSnapshot;
    let before = ResourceSnapshot { rss_bytes: Some(10), sampled_at_ns: 100, monotonic_at_ns: 1, process_cpu_ns: Some(20), system_cpu_ns: Some(30) };
    let after = ResourceSnapshot { rss_bytes: Some(11), sampled_at_ns: 101, monotonic_at_ns: 2, process_cpu_ns: Some(25), system_cpu_ns: Some(35) };
    assert!(after.monotonic_at_ns > before.monotonic_at_ns);
    assert_eq!(after.process_cpu_ns.unwrap() - before.process_cpu_ns.unwrap(), 5);
    assert_eq!(after.system_cpu_ns.unwrap() - before.system_cpu_ns.unwrap(), 5);
}

#[test]
fn b2_parallel_run_returns_complete_owned_filter_evidence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/p0-smoke");
    let dataset = Dataset::load(&root).unwrap();
    let run = run_level(Level::B2, &dataset, &parallel_config(2)).unwrap();
    assert_eq!(run.correctness_matches, 16);
    assert_eq!(run.filter_invocations.len(), 6);
    assert!(run.filter_invocations.values().sum::<usize>() >= 16);
}

#[test]
fn phase_runner_preserves_order_correctness_and_lateness_accounting() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/p0-smoke");
    let dataset = Dataset::load(&root).unwrap();
    let config = parallel_config(1);
    let calibration = Calibration::from_throughputs(&[1_000_000_000.0; 5]).unwrap();
    let schedule = PhaseSchedule::new(&calibration, Phase::Burst, 16).unwrap();
    let phase = run_phase(Level::B2, &dataset, &config, &schedule).unwrap();
    assert_eq!(phase.phase, Phase::Burst);
    assert_eq!(phase.frames, 16);
    assert_eq!(phase.correctness_matches, 16);
    assert_eq!(phase.late_frames + phase.on_time_frames, 16);
    assert!(phase.late_frames > 0);
    assert_eq!(phase.filter_invocations.len(), 6);
}
