use std::collections::BTreeMap;
use std::fs;

use prism_bench::output::{workflow_tax_percent, write_once_atomic, OutputError, RawRecord};

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
        dataset_digest: "abc".into(),
        workflow_tax_percent: -10.0,
        metadata: BTreeMap::new(),
    }
}

#[test]
fn workflow_tax_retains_negative_values() {
    assert_eq!(workflow_tax_percent(100, 90), -10.0);
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
    ] {
        assert!(json.contains(&format!("\"{key}\"")), "missing {key}");
    }
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
