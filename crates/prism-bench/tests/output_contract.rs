use std::collections::BTreeMap;
use std::fs;

use prism_bench::output::{
    workflow_tax_for_concurrency_repetition, workflow_tax_for_repetition, workflow_tax_percent,
    workflow_tax_for_phase_repetition, write_once_atomic, OutputError, RawRecord,
};

fn record() -> RawRecord {
    RawRecord {
        run_id: "run-1".into(),
        implementation: "rust".into(),
        level: "b1".into(),
        scenario: "S1".into(),
        concurrency: 1,
        repetition: 1,
        p50_ns: 10,
        p95_ns: 20,
        p99_ns: 30,
        p99_9_ns: 40,
        max_ns: 50,
        frames_per_sec: 100.0,
        mb_per_sec: 1.0,
        correctness_total: 10,
        correctness_matches: 10,
        timestamp_utc: "2026-09-12T00:00:00Z".into(),
        dataset_id: "prism.telemetry.p0.v1".into(),
        fixture_count: 10,
        payload_class_105: 4,
        payload_class_249: 3,
        payload_class_501: 3,
        valid_count: 8,
        invalid_count: 1,
        edge_complex_count: 1,
        warmup_target: 10_000,
        warmup_frames: 12_000,
        convergence_window: 1_000,
        convergence_threshold_percent: 5,
        converged: true,
        measured_frames: 10,
        typed_rejections: 1,
        execution_failures: 0,
        command: "bench --scenario S1".into(),
        dataset_digest: "abc".into(),
        workflow_tax_percent: -10.0,
        metadata: BTreeMap::new(),
        protocol_version: "1.1".into(),
        observability_variant: "local_metrics".into(),
        execution_id: "exec-1".into(),
        filter_timings_ns: BTreeMap::new(),
        filter_invocations: BTreeMap::new(),
        filter_rejections: BTreeMap::new(),
        filter_execution_failures: BTreeMap::new(),
        observability_tax_percent: 0.0,
        phase: "burst".into(),
        offered_frames_per_sec: 100.0,
        processed_frames_per_sec: 90.0,
        late_frames: 2,
        on_time_frames: 8,
        lateness_p50_ns: 0,
        lateness_p95_ns: 1,
        lateness_p99_ns: 2,
        calibration_median_frames_per_sec: 100.0,
        baseline_frames_per_sec: 80.0,
        burst_frames_per_sec: 800.0,
    }
}

#[test]
fn workflow_tax_retains_negative_values() {
    assert_eq!(workflow_tax_percent(100, 90), -10.0);
}

#[test]
fn workflow_tax_uses_the_matching_baseline_repetition() {
    let baselines = [(1, 100), (2, 200)];
    assert_eq!(workflow_tax_for_repetition(&baselines, 2, 220), 10.0);
}

#[test]
fn workflow_tax_uses_matching_concurrency_and_repetition() {
    let baselines = [((1, 1), 100), ((8, 1), 200), ((8, 2), 300)];
    assert_eq!(workflow_tax_for_concurrency_repetition(&baselines, 8, 2, 330), 10.0);
    assert_eq!(workflow_tax_for_concurrency_repetition(&baselines, 1, 1, 120), 20.0);
}

#[test]
fn workflow_tax_uses_matching_phase_and_repetition() {
    let baselines = [(("baseline".to_owned(), 1), 100), (("burst".to_owned(), 1), 200), (("burst".to_owned(), 2), 300)];
    assert_eq!(workflow_tax_for_phase_repetition(&baselines, "burst", 2, 330), 10.0);
    assert_eq!(workflow_tax_for_phase_repetition(&baselines, "baseline", 1, 120), 20.0);
}

#[test]
fn raw_record_serializes_required_metrics() {
    let json = record().to_json();
    for key in [
        "run_id",
        "p99_9_ns",
        "max_ns",
        "frames_per_sec",
        "mb_per_sec",
        "dataset_digest",
        "workflow_tax_percent",
        "timestamp_utc",
        "dataset_id",
        "fixture_count",
        "warmup_target",
        "converged",
        "typed_rejections",
        "execution_failures",
        "command",
        "protocol_version",
        "observability_variant",
        "execution_id",
        "filter_timings_ns",
        "filter_invocations",
        "filter_rejections",
        "filter_execution_failures",
        "observability_tax_percent",
        "phase",
        "offered_frames_per_sec",
        "processed_frames_per_sec",
        "late_frames",
        "on_time_frames",
        "lateness_p99_ns",
        "calibration_median_frames_per_sec",
        "baseline_frames_per_sec",
        "burst_frames_per_sec",
    ] {
        assert!(json.contains(&format!("\"{key}\"")), "missing {key}");
    }
}

#[test]
fn raw_record_serializes_s3_identity_fields() {
    let mut s3 = record();
    s3.scenario = "S3".into();
    s3.concurrency = 8;
    s3.run_id = "S3-b2-c8-r3".into();
    let json = s3.to_json();
    assert!(json.contains("\"scenario\":\"S3\""));
    assert!(json.contains("\"concurrency\":8"));
    assert!(json.contains("\"run_id\":\"S3-b2-c8-r3\""));
}

#[test]
fn atomic_writer_refuses_existing_output() {
    let path = std::env::temp_dir().join(format!("prism-atomic-{}.jsonl", std::process::id()));
    fs::write(&path, "existing\n").unwrap();
    assert!(matches!(
        write_once_atomic(&path, &[record()]),
        Err(OutputError::AlreadyExists)
    ));
    assert_eq!(fs::read_to_string(&path).unwrap(), "existing\n");
    fs::remove_file(path).unwrap();
}
