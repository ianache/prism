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
    pub monotonic_at_ns: u128,
    pub process_cpu_ns: Option<u128>,
    pub system_cpu_ns: Option<u128>,
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
    pub window_started_ns: u128,
    pub window_finished_ns: u128,
    pub repetition_elapsed_ns: u128,
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

#[cfg(windows)]
#[repr(C)]
struct FileTime { low: u32, high: u32 }

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetProcessTimes(process: *mut std::ffi::c_void, creation: *mut FileTime, exit: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
    fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
}

#[cfg(windows)]
fn file_time_ns(value: FileTime) -> u128 { (((value.high as u128) << 32) | value.low as u128) * 100 }

fn process_cpu_ns() -> Option<u128> {
    #[cfg(windows)]
    unsafe {
        let mut creation = FileTime { low: 0, high: 0 };
        let mut exit = FileTime { low: 0, high: 0 };
        let mut kernel = FileTime { low: 0, high: 0 };
        let mut user = FileTime { low: 0, high: 0 };
        if GetProcessTimes((-1isize) as *mut _, &mut creation, &mut exit, &mut kernel, &mut user) != 0 {
            return Some(file_time_ns(kernel) + file_time_ns(user));
        }
    }
    None
}

fn system_cpu_ns() -> Option<u128> {
    #[cfg(windows)]
    unsafe {
        let mut idle = FileTime { low: 0, high: 0 };
        let mut kernel = FileTime { low: 0, high: 0 };
        let mut user = FileTime { low: 0, high: 0 };
        if GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0 {
            return Some(file_time_ns(kernel) + file_time_ns(user));
        }
    }
    None
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
    static MONOTONIC_ORIGIN: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let monotonic_at_ns = MONOTONIC_ORIGIN.get_or_init(Instant::now).elapsed().as_nanos();
    ResourceSnapshot { rss_bytes: rss_bytes(), sampled_at_ns, monotonic_at_ns, process_cpu_ns: process_cpu_ns(), system_cpu_ns: system_cpu_ns() }
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
        let repetition_started = Instant::now();
        let deadline = Instant::now() + std::time::Duration::from_secs(per_repetition);
        let mut window_index = 0;
        while Instant::now() < deadline || window_index == 0 {
            let before = snapshot();
            let run = run_level(level, dataset, &run_config)?;
            let after = snapshot();
            let window_started_ns = before.sampled_at_ns;
            let window_finished_ns = after.sampled_at_ns;
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
                window_started_ns,
                window_finished_ns,
                repetition_elapsed_ns: repetition_started.elapsed().as_nanos(),
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
