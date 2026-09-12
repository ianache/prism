use std::fs;
use std::path::{Path, PathBuf};

use prism_runtime::b0;
use prism_runtime::b1::Pipeline;

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/p0-smoke")
}

fn expected_json(line: &str) -> &str {
    let marker = "\"expected\":";
    let start = line.find(marker).expect("expected field missing") + marker.len();
    let bytes = line.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for index in start..bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &line[start..=index];
                }
            }
            _ => {}
        }
    }
    panic!("expected JSON object is incomplete")
}

#[test]
fn rust_b0_and_b1_match_python_smoke_oracle() {
    let root = corpus_root();
    assert!(
        root.join("manifest.json").is_file(),
        "missing smoke corpus manifest: {}",
        root.display()
    );
    let expected = fs::read_to_string(root.join("expected-results.jsonl")).unwrap();
    let mut fixtures = fs::read_dir(root.join("fixtures"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    fixtures.sort();
    let expected_lines = expected.lines().collect::<Vec<_>>();
    assert_eq!(fixtures.len(), expected_lines.len());
    let pipeline = Pipeline::new().unwrap();
    for (fixture, line) in fixtures.iter().zip(expected_lines) {
        let payload = fs::read(fixture).unwrap();
        let b0_outcome = b0::process(&payload);
        let b1_outcome = pipeline.process(&payload);
        assert_eq!(
            b0_outcome,
            b1_outcome,
            "B0/B1 mismatch for {}",
            fixture.display()
        );
        assert_eq!(
            b0::serialize_outcome_json(&b0_outcome),
            expected_json(line),
            "oracle mismatch for {}",
            fixture.display()
        );
    }
}
