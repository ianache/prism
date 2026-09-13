use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct RawRecord {
    pub run_id: String, pub implementation: String, pub level: String, pub scenario: String,
    pub concurrency: usize, pub repetition: usize, pub p50_ns: u128, pub p95_ns: u128,
    pub p99_ns: u128, pub p99_9_ns: u128, pub max_ns: u128, pub frames_per_sec: f64,
    pub mb_per_sec: f64, pub correctness_total: usize, pub correctness_matches: usize,
    pub timestamp_utc: String, pub dataset_id: String, pub fixture_count: usize,
    pub payload_class_105: usize, pub payload_class_249: usize, pub payload_class_501: usize,
    pub valid_count: usize, pub invalid_count: usize, pub edge_complex_count: usize,
    pub warmup_target: usize, pub warmup_frames: usize, pub convergence_window: usize,
    pub convergence_threshold_percent: u32, pub converged: bool, pub measured_frames: usize,
    pub typed_rejections: usize, pub execution_failures: usize, pub command: String,
    pub dataset_digest: String, pub workflow_tax_percent: f64, pub metadata: BTreeMap<String, String>,
    pub protocol_version: String, pub observability_variant: String, pub execution_id: String,
    pub filter_timings_ns: BTreeMap<String, u128>, pub filter_invocations: BTreeMap<String, usize>,
    pub filter_rejections: BTreeMap<String, usize>, pub filter_execution_failures: BTreeMap<String, usize>,
    pub observability_tax_percent: f64,
    pub phase: String, pub offered_frames_per_sec: f64, pub processed_frames_per_sec: f64,
    pub late_frames: usize, pub on_time_frames: usize, pub lateness_p50_ns: u128,
    pub lateness_p95_ns: u128, pub lateness_p99_ns: u128,
    pub calibration_median_frames_per_sec: f64, pub baseline_frames_per_sec: f64,
    pub burst_frames_per_sec: f64,
    pub window_index: usize, pub duration_seconds: f64,
    pub rss_before_bytes: Option<u64>, pub rss_after_bytes: Option<u64>,
    pub incomplete_tail_frames: usize,
    pub window_started_ns: u128, pub window_finished_ns: u128, pub repetition_elapsed_ns: u128,
    pub process_cpu_before_ns: Option<u128>, pub process_cpu_after_ns: Option<u128>,
    pub system_cpu_before_ns: Option<u128>, pub system_cpu_after_ns: Option<u128>,
}

fn escape(value: &str) -> String { value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n") }
fn map_strings(map: &BTreeMap<String, String>) -> String { map.iter().map(|(k,v)| format!("\"{}\":\"{}\"", escape(k), escape(v))).collect::<Vec<_>>().join(",") }
fn map_u128(map: &BTreeMap<String, u128>) -> String { map.iter().map(|(k,v)| format!("\"{}\":{}", escape(k), v)).collect::<Vec<_>>().join(",") }
fn map_usize(map: &BTreeMap<String, usize>) -> String { map.iter().map(|(k,v)| format!("\"{}\":{}", escape(k), v)).collect::<Vec<_>>().join(",") }

impl RawRecord {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"command\":\"{}\",\"concurrency\":{},\"correctness_matches\":{},\"correctness_total\":{},\"converged\":{},\"convergence_threshold_percent\":{},\"convergence_window\":{},\"dataset_digest\":\"{}\",\"dataset_id\":\"{}\",\"edge_complex_count\":{},\"execution_failures\":{},\"fixture_count\":{},\"frames_per_sec\":{},\"implementation\":\"{}\",\"invalid_count\":{},\"level\":\"{}\",\"max_ns\":{},\"mb_per_sec\":{},\"measured_frames\":{},\"metadata\":{{{}}},\"observability_tax_percent\":{},\"observability_variant\":\"{}\",\"execution_id\":\"{}\",\"filter_timings_ns\":{{{}}},\"filter_invocations\":{{{}}},\"filter_rejections\":{{{}}},\"filter_execution_failures\":{{{}}},\"protocol_version\":\"{}\",\"p50_ns\":{},\"p95_ns\":{},\"p99_9_ns\":{},\"p99_ns\":{},\"payload_class_105\":{},\"payload_class_249\":{},\"payload_class_501\":{},\"repetition\":{},\"run_id\":\"{}\",\"scenario\":\"{}\",\"timestamp_utc\":\"{}\",\"typed_rejections\":{},\"valid_count\":{},\"warmup_frames\":{},\"warmup_target\":{},\"workflow_tax_percent\":{},\"phase\":\"{}\",\"offered_frames_per_sec\":{},\"processed_frames_per_sec\":{},\"late_frames\":{},\"on_time_frames\":{},\"lateness_p50_ns\":{},\"lateness_p95_ns\":{},\"lateness_p99_ns\":{},\"calibration_median_frames_per_sec\":{},\"baseline_frames_per_sec\":{},\"burst_frames_per_sec\":{},\"window_index\":{},\"duration_seconds\":{},\"rss_before_bytes\":{},\"rss_after_bytes\":{},\"incomplete_tail_frames\":{},\"window_started_ns\":{},\"window_finished_ns\":{},\"repetition_elapsed_ns\":{},\"process_cpu_before_ns\":{},\"process_cpu_after_ns\":{},\"system_cpu_before_ns\":{},\"system_cpu_after_ns\":{}}}",
            escape(&self.command), self.concurrency, self.correctness_matches, self.correctness_total, self.converged,
            self.convergence_threshold_percent, self.convergence_window, escape(&self.dataset_digest), escape(&self.dataset_id),
            self.edge_complex_count, self.execution_failures, self.fixture_count, self.frames_per_sec, escape(&self.implementation),
            self.invalid_count, escape(&self.level), self.max_ns, self.mb_per_sec, self.measured_frames, map_strings(&self.metadata),
            self.observability_tax_percent, escape(&self.observability_variant), escape(&self.execution_id), map_u128(&self.filter_timings_ns),
            map_usize(&self.filter_invocations), map_usize(&self.filter_rejections), map_usize(&self.filter_execution_failures),
            escape(&self.protocol_version), self.p50_ns, self.p95_ns, self.p99_9_ns, self.p99_ns, self.payload_class_105,
            self.payload_class_249, self.payload_class_501, self.repetition, escape(&self.run_id), escape(&self.scenario),
            escape(&self.timestamp_utc), self.typed_rejections, self.valid_count, self.warmup_frames, self.warmup_target,
            self.workflow_tax_percent, escape(&self.phase), self.offered_frames_per_sec, self.processed_frames_per_sec,
            self.late_frames, self.on_time_frames, self.lateness_p50_ns, self.lateness_p95_ns, self.lateness_p99_ns,
            self.calibration_median_frames_per_sec, self.baseline_frames_per_sec, self.burst_frames_per_sec,
            self.window_index, self.duration_seconds, self.rss_before_bytes.map_or("\"N/D\"".to_owned(), |v| v.to_string()),
            self.rss_after_bytes.map_or("\"N/D\"".to_owned(), |v| v.to_string()), self.incomplete_tail_frames,
            self.window_started_ns, self.window_finished_ns, self.repetition_elapsed_ns,
            self.process_cpu_before_ns.map_or("\"N/D\"".to_owned(), |v| v.to_string()), self.process_cpu_after_ns.map_or("\"N/D\"".to_owned(), |v| v.to_string()),
            self.system_cpu_before_ns.map_or("\"N/D\"".to_owned(), |v| v.to_string()), self.system_cpu_after_ns.map_or("\"N/D\"".to_owned(), |v| v.to_string()))
    }
}

#[derive(Debug)]
pub enum OutputError { AlreadyExists, Io(io::Error) }
pub fn workflow_tax_percent(base_p99: u128, compared_p99: u128) -> f64 { if base_p99 == 0 { 0.0 } else { (compared_p99 as f64 - base_p99 as f64) / base_p99 as f64 * 100.0 } }
pub fn workflow_tax_for_repetition(baselines: &[(usize, u128)], repetition: usize, compared_p99: u128) -> f64 { baselines.iter().find(|(r, _)| *r == repetition).map_or(0.0, |(_, base)| workflow_tax_percent(*base, compared_p99)) }
pub fn workflow_tax_for_concurrency_repetition(baselines: &[((usize, usize), u128)], concurrency: usize, repetition: usize, compared_p99: u128) -> f64 {
    baselines.iter().find(|((c, r), _)| *c == concurrency && *r == repetition).map_or(0.0, |(_, base)| workflow_tax_percent(*base, compared_p99))
}
pub fn workflow_tax_for_phase_repetition(baselines: &[((String, usize), u128)], phase: &str, repetition: usize, compared_p99: u128) -> f64 {
    baselines.iter().find(|((p, r), _)| p == phase && *r == repetition).map_or(0.0, |(_, base)| workflow_tax_percent(*base, compared_p99))
}
pub fn write_once_atomic(path: &Path, records: &[RawRecord]) -> Result<(), OutputError> {
    if path.exists() { return Err(OutputError::AlreadyExists); }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let contents = records.iter().map(|record| format!("{}\n", record.to_json())).collect::<String>();
    fs::write(&temporary, contents).map_err(OutputError::Io)?;
    fs::rename(&temporary, path).map_err(OutputError::Io)
}
pub fn write_once(path: &Path, contents: &str) -> io::Result<()> {
    if path.exists() { return Err(io::Error::new(io::ErrorKind::AlreadyExists, "output already exists")); }
    fs::write(path, contents)
}
