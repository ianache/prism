use std::time::{SystemTime, UNIX_EPOCH, Instant};

use crate::dataset::Dataset;
use crate::runner::{run_level, Level, RunConfig, RunError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SustainedConfig {
    pub duration_seconds: u64,
    pub window_frames: usize,
    pub repetitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSnapshot {
    pub rss_bytes: Option<u64>,
    pub sampled_at_ns: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SustainedWindow {
    pub repetition: usize,
    pub window_index: usize,
    pub measured_frames: usize,
    pub duration_seconds: f64,
    pub p50_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
    pub p99_9_ns: u128,
    pub max_ns: u128,
    pub frames_per_sec: f64,
    pub mb_per_sec: f64,
    pub correctness_total: usize,
    pub correctness_matches: usize,
    pub typed_rejections: usize,
    pub execution_failures: usize,
    pub resource_before: ResourceSnapshot,
    pub resource_after: ResourceSnapshot,
    pub incomplete_tail_frames: usize,
    pub execution_id: String,
    pub observability_variant: String,
    pub filter_timings_ns: std::collections::BTreeMap<String, u128>,
    pub filter_invocations: std::collections::BTreeMap<String, usize>,
    pub filter_rejections: std::collections::BTreeMap<String, usize>,
    pub filter_execution_failures: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SustainedRun {
    pub level: Level,
    pub duration_seconds: u64,
    pub repetitions: Vec<SustainedWindow>,
}

#[cfg(windows)]
#[repr(C)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[cfg(windows)]
#[link(name = "psapi")]
extern "system" {
    fn GetProcessMemoryInfo(process: *mut std::ffi::c_void, counters: *mut ProcessMemoryCounters, size: u32) -> i32;
}

fn rss_bytes() -> Option<u64> {
    #[cfg(windows)]
    unsafe {
        let mut counters = std::mem::zeroed::<ProcessMemoryCounters>();
        counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
        let process = (-1isize) as *mut std::ffi::c_void;
        if GetProcessMemoryInfo(process, &mut counters, counters.cb) != 0 {
            return Some(counters.working_set_size as u64);
        }
    }
    None
}

fn snapshot() -> ResourceSnapshot {
    let sampled_at_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    ResourceSnapshot { rss_bytes: rss_bytes(), sampled_at_ns }
}

pub fn run_sustained(
    level: Level,
    dataset: &Dataset,
    config: &SustainedConfig,
) -> Result<SustainedRun, RunError> {
    if dataset.fixtures.is_empty()
        || config.duration_seconds == 0
        || config.window_frames != 100_000
        || config.repetitions != 5
    {
        return Err(RunError::InvalidConfig);
    }
    let per_repetition = config.duration_seconds / config.repetitions as u64;
    let run_config = RunConfig {
        warmup: 0,
        samples: config.window_frames,
        warmup_frames: 0,
        convergence_window: 1,
        convergence_threshold_percent: 5,
        max_warmup_frames: 0,
        measured_frames: config.window_frames,
        repetitions: 1,
        concurrency: 1,
    };
    let mut windows = Vec::new();
    for repetition in 1..=config.repetitions {
        let deadline = Instant::now() + std::time::Duration::from_secs(per_repetition);
        let mut window_index = 0;
        while Instant::now() < deadline || window_index == 0 {
            let before = snapshot();
            let run = run_level(level, dataset, &run_config)?;
            let after = snapshot();
            let metrics = &run.repetitions[0];
            windows.push(SustainedWindow {
                repetition,
                window_index,
                measured_frames: metrics.frames,
                duration_seconds: metrics.elapsed_ns as f64 / 1_000_000_000.0,
                p50_ns: metrics.p50_ns,
                p95_ns: metrics.p95_ns,
                p99_ns: metrics.p99_ns,
                p99_9_ns: metrics.p99_9_ns,
                max_ns: metrics.max_ns,
                frames_per_sec: metrics.frames_per_sec,
                mb_per_sec: metrics.mb_per_sec,
                correctness_total: run.correctness_total,
                correctness_matches: run.correctness_matches,
                typed_rejections: run.typed_rejections,
                execution_failures: run.execution_failures,
                resource_before: before,
                resource_after: after,
                incomplete_tail_frames: 0,
                execution_id: run.execution_id,
                observability_variant: run.observability_variant,
                filter_timings_ns: run.filter_timings_ns,
                filter_invocations: run.filter_invocations,
                filter_rejections: run.filter_rejections,
                filter_execution_failures: run.filter_execution_failures,
            });
            window_index += 1;
        }
    }
    Ok(SustainedRun { level, duration_seconds: config.duration_seconds, repetitions: windows })
}
