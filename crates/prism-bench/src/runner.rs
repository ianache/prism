use std::time::Instant;

use prism_runtime::{b0, b1::Pipeline, Outcome};

use crate::dataset::Dataset;
use crate::percentiles::percentile_nearest_rank;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    B0,
    B1,
}

pub struct RunConfig {
    pub warmup: usize,
    pub samples: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRun {
    pub level: Level,
    pub samples: usize,
    pub p50_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
    pub correctness_total: usize,
    pub correctness_matches: usize,
}

fn execute(level: Level, pipeline: Option<&Pipeline>, payload: &[u8]) -> Outcome {
    match level {
        Level::B0 => b0::process(payload),
        Level::B1 => pipeline
            .expect("B1 pipeline must be initialized outside timing")
            .process(payload),
    }
}

pub fn run_level(level: Level, dataset: &Dataset, config: &RunConfig) -> RawRun {
    let pipeline = (level == Level::B1).then(|| Pipeline::new().expect("static B1 registry"));
    for _ in 0..config.warmup {
        for payload in &dataset.fixtures {
            let _ = execute(level, pipeline.as_ref(), payload);
        }
    }
    let mut samples = Vec::with_capacity(config.samples);
    let mut matches = 0;
    for _ in 0..config.samples {
        let start = Instant::now();
        let outcomes = dataset
            .fixtures
            .iter()
            .map(|payload| execute(level, pipeline.as_ref(), payload))
            .collect::<Vec<_>>();
        samples.push(start.elapsed().as_nanos());
        for (outcome, expected) in outcomes.iter().zip(&dataset.expected) {
            if serialize(outcome) == *expected {
                matches += 1;
            }
        }
    }
    let total = dataset.fixtures.len() * config.samples;
    RawRun {
        level,
        samples: config.samples,
        p50_ns: percentile_nearest_rank(&mut samples, 50),
        p95_ns: percentile_nearest_rank(&mut samples, 95),
        p99_ns: percentile_nearest_rank(&mut samples, 99),
        correctness_total: total,
        correctness_matches: matches,
    }
}

fn serialize(outcome: &Outcome) -> String {
    prism_runtime::b0::serialize_outcome_json(outcome)
}
