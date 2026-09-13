use std::time::Instant;

use prism_runtime::{b0, b1::Pipeline, Outcome};

use crate::dataset::Dataset;
use crate::percentiles::percentile_nearest_rank_thousandths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    B0,
    B1,
}

pub struct RunConfig {
    pub warmup: usize,
    pub samples: usize,
    pub warmup_frames: usize,
    pub convergence_window: usize,
    pub convergence_threshold_percent: u32,
    pub max_warmup_frames: usize,
    pub measured_frames: usize,
    pub repetitions: usize,
    pub concurrency: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepetitionMetrics {
    pub repetition: usize,
    pub frames: usize,
    pub elapsed_ns: u128,
    pub p50_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
    pub p99_9_ns: u128,
    pub max_ns: u128,
    pub frames_per_sec: f64,
    pub mb_per_sec: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawRun {
    pub level: Level,
    pub samples: usize,
    pub p50_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
    pub p99_9_ns: u128,
    pub max_ns: u128,
    pub frames_per_sec: f64,
    pub mb_per_sec: f64,
    pub repetitions: Vec<RepetitionMetrics>,
    pub correctness_total: usize,
    pub correctness_matches: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    InvalidConfig,
    WarmupDidNotConverge,
    CorrectnessMismatch { frame: usize },
}

fn execute(level: Level, pipeline: Option<&Pipeline>, payload: &[u8]) -> Outcome {
    match level {
        Level::B0 => b0::process(payload),
        Level::B1 => pipeline
            .expect("B1 pipeline must be initialized outside timing")
            .process(payload),
    }
}

fn percentile_set(samples: &mut [u128]) -> (u128, u128, u128, u128, u128) {
    let p50 = percentile_nearest_rank_thousandths(samples, 50_000);
    let p95 = percentile_nearest_rank_thousandths(samples, 95_000);
    let p99 = percentile_nearest_rank_thousandths(samples, 99_000);
    let p99_9 = percentile_nearest_rank_thousandths(samples, 99_900);
    let max = *samples.iter().max().unwrap();
    (p50, p95, p99, p99_9, max)
}

fn convergence_ok(previous: u128, current: u128, threshold: u32) -> bool {
    if previous == 0 {
        return current == 0;
    }
    previous.abs_diff(current) * 100 <= previous * threshold as u128
}

pub fn run_level(level: Level, dataset: &Dataset, config: &RunConfig) -> Result<RawRun, RunError> {
    if dataset.fixtures.is_empty()
        || config.concurrency != 1
        || config.repetitions == 0
        || config.measured_frames == 0
    {
        return Err(RunError::InvalidConfig);
    }
    let pipeline = (level == Level::B1).then(|| Pipeline::new().expect("static B1 registry"));
    let warmup_target = config.warmup_frames.max(config.warmup);
    let mut previous_p99 = None;
    let mut warmed = 0usize;
    let max_warmup = config.max_warmup_frames.max(warmup_target);
    let mut converged = warmup_target == 0;
    while !converged && warmed < max_warmup {
        let window_frames = config.convergence_window.min(max_warmup - warmed).max(1);
        let mut window_samples = Vec::with_capacity(window_frames);
        for offset in 0..window_frames {
            let payload = &dataset.fixtures[(warmed + offset) % dataset.fixtures.len()];
            let start = Instant::now();
            let _ = execute(level, pipeline.as_ref(), payload);
            window_samples.push(start.elapsed().as_nanos());
        }
        warmed += window_frames;
        let current_p99 = percentile_nearest_rank_thousandths(&mut window_samples, 99_000);
        if previous_p99.is_some_and(|previous| {
            convergence_ok(previous, current_p99, config.convergence_threshold_percent)
        }) && warmed >= warmup_target
        {
            converged = true;
        }
        previous_p99 = Some(current_p99);
    }
    if !converged {
        return Err(RunError::WarmupDidNotConverge);
    }

    let mut all_samples = Vec::with_capacity(config.repetitions * config.measured_frames);
    let mut repetitions = Vec::with_capacity(config.repetitions);
    let mut correctness_matches = 0usize;
    let mut correctness_total = 0usize;
    for repetition in 0..config.repetitions {
        let mut samples = Vec::with_capacity(config.measured_frames);
        let mut outcomes = Vec::with_capacity(config.measured_frames);
        let start = Instant::now();
        for index in 0..config.measured_frames {
            let payload = &dataset.fixtures[index % dataset.fixtures.len()];
            let frame_start = Instant::now();
            outcomes.push(execute(level, pipeline.as_ref(), payload));
            samples.push(frame_start.elapsed().as_nanos());
        }
        let elapsed_ns = start.elapsed().as_nanos();
        for (index, (outcome, expected)) in outcomes
            .iter()
            .zip(dataset.expected.iter().cycle())
            .take(config.measured_frames)
            .enumerate()
        {
            correctness_total += 1;
            if serialize(outcome) == *expected {
                correctness_matches += 1;
            } else {
                return Err(RunError::CorrectnessMismatch { frame: index });
            }
        }
        let all_for_metrics = samples.clone();
        let (p50, p95, p99, p99_9, max) = percentile_set(&mut samples);
        let bytes = (0..config.measured_frames)
            .map(|index| dataset.fixtures[index % dataset.fixtures.len()].len())
            .sum::<usize>();
        let seconds = elapsed_ns as f64 / 1_000_000_000.0;
        repetitions.push(RepetitionMetrics {
            repetition: repetition + 1,
            frames: config.measured_frames,
            elapsed_ns,
            p50_ns: p50,
            p95_ns: p95,
            p99_ns: p99,
            p99_9_ns: p99_9,
            max_ns: max,
            frames_per_sec: config.measured_frames as f64 / seconds.max(f64::MIN_POSITIVE),
            mb_per_sec: bytes as f64 / 1_000_000.0 / seconds.max(f64::MIN_POSITIVE),
        });
        all_samples.extend(all_for_metrics);
    }
    let elapsed_ns = repetitions.iter().map(|item| item.elapsed_ns).sum::<u128>();
    let bytes = config.repetitions
        * (0..config.measured_frames)
            .map(|index| dataset.fixtures[index % dataset.fixtures.len()].len())
            .sum::<usize>();
    let seconds = elapsed_ns as f64 / 1_000_000_000.0;
    let (p50, p95, p99, p99_9, max) = percentile_set(&mut all_samples);
    Ok(RawRun {
        level,
        samples: config.measured_frames,
        p50_ns: p50,
        p95_ns: p95,
        p99_ns: p99,
        p99_9_ns: p99_9,
        max_ns: max,
        frames_per_sec: (config.repetitions * config.measured_frames) as f64
            / seconds.max(f64::MIN_POSITIVE),
        mb_per_sec: bytes as f64 / 1_000_000.0 / seconds.max(f64::MIN_POSITIVE),
        repetitions,
        correctness_total,
        correctness_matches,
    })
}

fn serialize(outcome: &Outcome) -> String {
    prism_runtime::b0::serialize_outcome_json(outcome)
}
