use std::collections::BTreeMap;
use std::sync::Barrier;
use std::time::Instant;

use prism_runtime::{b0, b1::{FilterId, ObservationOutcome, Observer, Pipeline}, Outcome};

use crate::dataset::Dataset;
use crate::percentiles::percentile_nearest_rank_thousandths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    B0,
    B1,
    B2,
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
    pub warmup_target: usize,
    pub warmup_frames: usize,
    pub convergence_window: usize,
    pub convergence_threshold_percent: u32,
    pub converged: bool,
    pub measured_frames: usize,
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
    pub typed_rejections: usize,
    pub execution_failures: usize,
    pub observability_variant: String,
    pub execution_id: String,
    pub filter_timings_ns: BTreeMap<String, u128>,
    pub filter_invocations: BTreeMap<String, usize>,
    pub filter_rejections: BTreeMap<String, usize>,
    pub filter_execution_failures: BTreeMap<String, usize>,
}

#[derive(Default)]
struct Collector {
    timings: BTreeMap<String, u128>,
    invocations: BTreeMap<String, usize>,
    rejections: BTreeMap<String, usize>,
    failures: BTreeMap<String, usize>,
}

struct WorkerResult {
    frames: Vec<(usize, Outcome, u128)>,
    collector: Collector,
}

fn filter_name(filter: FilterId) -> String { format!("F{}", filter as usize + 1) }

impl Observer for Collector {
    fn on_filter(&mut self, filter: FilterId, elapsed_ns: u128, outcome: ObservationOutcome) {
        let name = filter_name(filter);
        *self.timings.entry(name.clone()).or_default() += elapsed_ns;
        *self.invocations.entry(name.clone()).or_default() += 1;
        match outcome {
            ObservationOutcome::Rejected => *self.rejections.entry(name).or_default() += 1,
            ObservationOutcome::ExecutionFailure => *self.failures.entry(name).or_default() += 1,
            ObservationOutcome::Completed => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    InvalidConfig,
    WarmupDidNotConverge,
    CorrectnessMismatch { frame: usize },
    WorkerJoinFailure,
}

fn execute(level: Level, pipeline: Option<&Pipeline>, payload: &[u8]) -> Outcome {
    match level {
        Level::B0 => b0::process(payload),
        Level::B1 | Level::B2 => pipeline
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

fn execute_parallel(
    level: Level,
    dataset: &Dataset,
    measured_frames: usize,
    concurrency: usize,
) -> Result<(Vec<Outcome>, Vec<u128>, Collector, u128), RunError> {
    let barrier = Barrier::new(concurrency + 1);
    let start = Instant::now();
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(concurrency);
        for worker_id in 0..concurrency {
            let barrier = &barrier;
            handles.push(scope.spawn(move || {
                let pipeline = (level != Level::B0).then(|| Pipeline::new().expect("static B1 registry"));
                let mut collector = Collector::default();
                for name in ["F1", "F2", "F3", "F4", "F5", "F6"] {
                    collector.timings.insert(name.to_owned(), 0);
                    collector.invocations.insert(name.to_owned(), 0);
                    collector.rejections.insert(name.to_owned(), 0);
                    collector.failures.insert(name.to_owned(), 0);
                }
                let mut frames = Vec::new();
                barrier.wait();
                for index in (worker_id..measured_frames).step_by(concurrency) {
                    let payload = &dataset.fixtures[index % dataset.fixtures.len()];
                    let frame_start = Instant::now();
                    let outcome = if level == Level::B2 {
                        pipeline.as_ref().unwrap().process_observed(payload, &mut collector)
                    } else {
                        execute(level, pipeline.as_ref(), payload)
                    };
                    frames.push((index, outcome, frame_start.elapsed().as_nanos()));
                }
                WorkerResult { frames, collector }
            }));
        }
        barrier.wait();
        handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| RunError::WorkerJoinFailure))
            .collect::<Result<Vec<_>, _>>()
    });
    let elapsed_ns = start.elapsed().as_nanos();
    let mut frames = Vec::with_capacity(measured_frames);
    let mut collector = Collector::default();
    for result in results? {
        frames.extend(result.frames);
        for (key, value) in result.collector.timings {
            *collector.timings.entry(key).or_default() += value;
        }
        for (key, value) in result.collector.invocations {
            *collector.invocations.entry(key).or_default() += value;
        }
        for (key, value) in result.collector.rejections {
            *collector.rejections.entry(key).or_default() += value;
        }
        for (key, value) in result.collector.failures {
            *collector.failures.entry(key).or_default() += value;
        }
    }
    frames.sort_by_key(|(index, _, _)| *index);
    let outcomes = frames.iter().map(|(_, outcome, _)| outcome.clone()).collect();
    let samples = frames.iter().map(|(_, _, elapsed)| *elapsed).collect();
    Ok((outcomes, samples, collector, elapsed_ns))
}

pub fn run_level(level: Level, dataset: &Dataset, config: &RunConfig) -> Result<RawRun, RunError> {
    if dataset.fixtures.is_empty()
        || config.concurrency == 0
        || config.repetitions == 0
        || config.measured_frames == 0
    {
        return Err(RunError::InvalidConfig);
    }
    let pipeline = (level != Level::B0).then(|| Pipeline::new().expect("static B1 registry"));
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
    let mut typed_rejections = 0usize;
    let mut execution_failures = 0usize;
    let mut collector = Collector::default();
    for name in ["F1", "F2", "F3", "F4", "F5", "F6"] {
        collector.timings.insert(name.to_owned(), 0);
        collector.invocations.insert(name.to_owned(), 0);
        collector.rejections.insert(name.to_owned(), 0);
        collector.failures.insert(name.to_owned(), 0);
    }
    for repetition in 0..config.repetitions {
        let (outcomes, mut samples, elapsed_ns) = if config.concurrency == 1 {
            let mut samples = Vec::with_capacity(config.measured_frames);
            let mut outcomes = Vec::with_capacity(config.measured_frames);
            let start = Instant::now();
            for index in 0..config.measured_frames {
                let payload = &dataset.fixtures[index % dataset.fixtures.len()];
                let frame_start = Instant::now();
                outcomes.push(if level == Level::B2 {
                    pipeline.as_ref().unwrap().process_observed(payload, &mut collector)
                } else {
                    execute(level, pipeline.as_ref(), payload)
                });
                samples.push(frame_start.elapsed().as_nanos());
            }
            (outcomes, samples, start.elapsed().as_nanos())
        } else {
            let (outcomes, samples, worker_collector, elapsed_ns) =
                execute_parallel(level, dataset, config.measured_frames, config.concurrency)?;
            for (key, value) in worker_collector.timings {
                *collector.timings.entry(key).or_default() += value;
            }
            for (key, value) in worker_collector.invocations {
                *collector.invocations.entry(key).or_default() += value;
            }
            for (key, value) in worker_collector.rejections {
                *collector.rejections.entry(key).or_default() += value;
            }
            for (key, value) in worker_collector.failures {
                *collector.failures.entry(key).or_default() += value;
            }
            (outcomes, samples, elapsed_ns)
        };
        for (index, (outcome, expected)) in outcomes
            .iter()
            .zip(dataset.expected.iter().cycle())
            .take(config.measured_frames)
            .enumerate()
        {
            correctness_total += 1;
            match outcome {
                Outcome::Rejected(_) => typed_rejections += 1,
                Outcome::ExecutionFailure(_) => execution_failures += 1,
                Outcome::Normalized(_) => {}
            }
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
        warmup_target,
        warmup_frames: warmed,
        convergence_window: config.convergence_window,
        convergence_threshold_percent: config.convergence_threshold_percent,
        converged,
        measured_frames: config.measured_frames,
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
        typed_rejections,
        execution_failures,
        observability_variant: if level == Level::B2 { "local_metrics".into() } else { "none".into() },
        execution_id: format!("S1-{:?}", level),
        filter_timings_ns: collector.timings,
        filter_invocations: collector.invocations,
        filter_rejections: collector.rejections,
        filter_execution_failures: collector.failures,
    })
}

fn serialize(outcome: &Outcome) -> String {
    prism_runtime::b0::serialize_outcome_json(outcome)
}
