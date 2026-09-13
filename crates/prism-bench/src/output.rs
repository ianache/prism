use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct RawRecord {
    pub run_id: String,
    pub implementation: String,
    pub level: String,
    pub scenario: String,
    pub concurrency: usize,
    pub repetition: usize,
    pub p50_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
    pub p99_9_ns: u128,
    pub max_ns: u128,
    pub frames_per_sec: f64,
    pub mb_per_sec: f64,
    pub correctness_total: usize,
    pub correctness_matches: usize,
    pub dataset_digest: String,
    pub workflow_tax_percent: f64,
    pub metadata: BTreeMap<String, String>,
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

impl RawRecord {
    pub fn to_json(&self) -> String {
        let metadata = self
            .metadata
            .iter()
            .map(|(key, value)| format!("\"{}\":\"{}\"", escape(key), escape(value)))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"concurrency\":{},\"correctness_matches\":{},\"correctness_total\":{},\"dataset_digest\":\"{}\",\"frames_per_sec\":{},\"implementation\":\"{}\",\"level\":\"{}\",\"max_ns\":{},\"mb_per_sec\":{},\"metadata\":{{{}}},\"p50_ns\":{},\"p95_ns\":{},\"p99_9_ns\":{},\"p99_ns\":{},\"repetition\":{},\"run_id\":\"{}\",\"scenario\":\"{}\",\"workflow_tax_percent\":{}}}", self.concurrency, self.correctness_matches, self.correctness_total, escape(&self.dataset_digest), self.frames_per_sec, escape(&self.implementation), escape(&self.level), self.max_ns, self.mb_per_sec, metadata, self.p50_ns, self.p95_ns, self.p99_9_ns, self.p99_ns, self.repetition, escape(&self.run_id), escape(&self.scenario), self.workflow_tax_percent)
    }
}

#[derive(Debug)]
pub enum OutputError {
    AlreadyExists,
    Io(io::Error),
}

pub fn workflow_tax_percent(base_p99: u128, compared_p99: u128) -> f64 {
    if base_p99 == 0 {
        return 0.0;
    }
    (compared_p99 as f64 - base_p99 as f64) / base_p99 as f64 * 100.0
}

pub fn write_once_atomic(path: &Path, records: &[RawRecord]) -> Result<(), OutputError> {
    if path.exists() {
        return Err(OutputError::AlreadyExists);
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let contents = records
        .iter()
        .map(|record| format!("{}\n", record.to_json()))
        .collect::<String>();
    fs::write(&temporary, contents).map_err(OutputError::Io)?;
    fs::rename(&temporary, path).map_err(OutputError::Io)
}

pub fn write_once(path: &Path, contents: &str) -> io::Result<()> {
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "output already exists",
        ));
    }
    fs::write(path, contents)
}
