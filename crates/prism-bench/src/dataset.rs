use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum DatasetError {
    Io(io::Error),
    MissingCorpus(PathBuf),
    CountMismatch { fixtures: usize, expected: usize },
}

impl From<io::Error> for DatasetError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct Dataset {
    pub fixtures: Vec<Vec<u8>>,
    pub expected: Vec<String>,
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
        if fixtures.len() != expected.len() {
            return Err(DatasetError::CountMismatch {
                fixtures: fixtures.len(),
                expected: expected.len(),
            });
        }
        Ok(Self { fixtures, expected })
    }
}
