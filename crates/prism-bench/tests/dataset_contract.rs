use std::fs;
use std::path::{Path, PathBuf};

use prism_bench::dataset::{Dataset, DatasetError};
use prism_bench::sha256::sha256_hex;

fn smoke_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/p0-smoke")
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target.join("fixtures")).unwrap();
    for name in ["manifest.json", "manifest.sha256", "expected-results.jsonl"] {
        fs::copy(source.join(name), target.join(name)).unwrap();
    }
    for entry in fs::read_dir(source.join("fixtures")).unwrap() {
        let entry = entry.unwrap();
        fs::copy(
            entry.path(),
            target.join("fixtures").join(entry.file_name()),
        )
        .unwrap();
    }
}

#[test]
fn sha256_matches_nist_ascii_vector() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn dataset_rejects_manifest_digest_mismatch() {
    let root = std::env::temp_dir().join(format!("prism-digest-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&smoke_path(), &root);
    fs::write(
        root.join("manifest.sha256"),
        "0000000000000000000000000000000000000000000000000000000000000000\n",
    )
    .unwrap();
    assert!(matches!(
        Dataset::load(&root),
        Err(DatasetError::DigestMismatch { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn dataset_exposes_protocol_counts() {
    let dataset = Dataset::load(&smoke_path()).unwrap();
    assert_eq!(dataset.dataset_id, "prism.telemetry.p0.v1");
    assert_eq!(dataset.fixture_count, 300);
    assert_eq!(dataset.payload_counts.class_105, 100);
    assert_eq!(dataset.payload_counts.class_249, 100);
    assert_eq!(dataset.payload_counts.class_501, 100);
    assert_eq!(dataset.validity_counts.valid, 240);
    assert_eq!(dataset.validity_counts.invalid, 15);
    assert_eq!(dataset.validity_counts.edge_complex, 45);
}
