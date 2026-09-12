use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::sha256::{canonical_json_bytes, sha256_hex};

#[derive(Debug)]
pub enum DatasetError {
    Io(io::Error),
    MissingCorpus(PathBuf),
    CountMismatch { fixtures: usize, expected: usize },
    DigestMismatch { expected: String, actual: String },
    InvalidManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadCounts {
    pub class_105: usize,
    pub class_249: usize,
    pub class_501: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidityCounts {
    pub valid: usize,
    pub invalid: usize,
    pub edge_complex: usize,
}

impl From<io::Error> for DatasetError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct Dataset {
    pub fixtures: Vec<Vec<u8>>,
    pub expected: Vec<String>,
    pub manifest_digest: String,
    pub dataset_id: String,
    pub fixture_count: usize,
    pub payload_counts: PayloadCounts,
    pub validity_counts: ValidityCounts,
}

fn json_string(source: &str, key: &str) -> String {
    let marker = format!("\"{key}\":\"");
    let start = source.find(&marker).expect("manifest string field missing") + marker.len();
    let end = source[start..]
        .find('"')
        .expect("manifest string field incomplete")
        + start;
    source[start..end].to_owned()
}

fn json_number(source: &str, key: &str) -> usize {
    let marker = format!("\"{key}\":");
    let start = source
        .find(&marker)
        .expect("manifest numeric field missing")
        + marker.len();
    source[start..]
        .split(|ch: char| !ch.is_ascii_digit())
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn expected_json(line: &str) -> String {
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
                    return line[start..=index].to_owned();
                }
            }
            _ => {}
        }
    }
    panic!("incomplete expected JSON")
}

impl Dataset {
    pub fn load(path: &Path) -> Result<Self, DatasetError> {
        if !path.join("manifest.json").is_file() || !path.join("expected-results.jsonl").is_file() {
            return Err(DatasetError::MissingCorpus(path.to_owned()));
        }
        let manifest = fs::read(path.join("manifest.json"))?;
        let manifest_text = String::from_utf8_lossy(&manifest);
        let workload_text = manifest_text.split("\"workload\":{").nth(1).and_then(|text| text.split('}').next()).unwrap_or("");
        let expected_digest = fs::read_to_string(path.join("manifest.sha256"))?
            .trim()
            .to_owned();
        let canonical_manifest =
            canonical_json_bytes(&manifest).map_err(|_| DatasetError::InvalidManifest)?;
        let actual_digest = sha256_hex(&canonical_manifest);
        if expected_digest != actual_digest {
            return Err(DatasetError::DigestMismatch {
                expected: expected_digest,
                actual: actual_digest,
            });
        }
        let mut fixture_paths = fs::read_dir(path.join("fixtures"))?
            .map(|entry| entry.map(|item| item.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fixture_paths.sort();
        let fixtures = fixture_paths
            .into_iter()
            .map(|path| fs::read(path))
            .collect::<Result<Vec<_>, _>>()?;
        let expected = fs::read_to_string(path.join("expected-results.jsonl"))?
            .lines()
            .map(expected_json)
            .collect::<Vec<_>>();
        let manifest_digest = fs::read_to_string(path.join("manifest.sha256"))?
            .trim()
            .to_owned();
        if fixtures.len() != expected.len() {
            return Err(DatasetError::CountMismatch {
                fixtures: fixtures.len(),
                expected: expected.len(),
            });
        }
        let fixture_count = fixtures.len();
        Ok(Self {
            fixtures,
            expected,
            manifest_digest,
            dataset_id: json_string(&manifest_text, "dataset_id"),
            fixture_count,
            payload_counts: PayloadCounts {
                class_105: manifest_text.matches("\"payload_class\":105").count(),
                class_249: manifest_text.matches("\"payload_class\":249").count(),
                class_501: manifest_text.matches("\"payload_class\":501").count(),
            },
            validity_counts: ValidityCounts {
                valid: json_number(workload_text, "valid"),
                invalid: json_number(workload_text, "invalid"),
                edge_complex: json_number(workload_text, "edge_complex"),
            },
        })
    }
}
