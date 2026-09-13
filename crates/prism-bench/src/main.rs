use std::collections::BTreeMap;

use prism_bench::cli::parse_args;
use prism_bench::dataset::Dataset;
use prism_bench::metadata::Metadata;
use prism_bench::output::{workflow_tax_percent, write_once_atomic, RawRecord};
use prism_bench::runner::{run_level, Level, RawRun, RunConfig};

fn main() {
    let config = match parse_args(std::env::args()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let dataset = match Dataset::load(std::path::Path::new(&config.dataset)) {
        Ok(dataset) => dataset,
        Err(error) => {
            eprintln!("unable to load dataset: {error:?}");
            std::process::exit(2);
        }
    };
    let run_config = RunConfig {
        warmup: config.warmup,
        samples: config.samples,
        warmup_frames: config.warmup_frames,
        convergence_window: config.convergence_window,
        convergence_threshold_percent: config.convergence_threshold_percent,
        max_warmup_frames: config.max_warmup_frames,
        measured_frames: config.measured_frames,
        repetitions: config.repetitions,
        concurrency: config.concurrency,
    };
    let mut runs: Vec<(String, RawRun)> = Vec::new();
    for level_name in config.levels.split(',') {
        let level = match level_name {
            "b0" => Level::B0,
            "b1" => Level::B1,
            _ => unreachable!(),
        };
        match run_level(level, &dataset, &run_config) {
            Ok(run) => runs.push((level_name.to_owned(), run)),
            Err(error) => {
                eprintln!("benchmark invalid: {error:?}");
                std::process::exit(2);
            }
        }
    }
    let b0_p99 = runs
        .iter()
        .find(|(level, _)| level == "b0")
        .map(|(_, run)| run.p99_ns)
        .unwrap_or(0);
    let host = Metadata::collect(&config);
    let mut metadata = BTreeMap::new();
    metadata.insert("cpu_model".to_owned(), host.cpu_model);
    metadata.insert("physical_cores".to_owned(), host.physical_cores);
    metadata.insert("logical_cores".to_owned(), host.logical_cores);
    metadata.insert("ram_bytes".to_owned(), host.ram_bytes);
    metadata.insert("os".to_owned(), host.os);
    metadata.insert("kernel".to_owned(), host.kernel);
    metadata.insert("runtime".to_owned(), host.runtime);
    metadata.insert("compiler".to_owned(), host.compiler);
    metadata.insert("governor".to_owned(), host.governor);
    metadata.insert("container_limits".to_owned(), host.container_limits);
    metadata.insert("affinity".to_owned(), host.affinity);
    let mut records = Vec::new();
    for (level, run) in runs {
        for repetition in &run.repetitions {
            records.push(RawRecord {
                run_id: format!("{}-{}-{}", config.scenario, level, repetition.repetition),
                implementation: "rust".into(),
                level: level.clone(),
                scenario: config.scenario.clone(),
                concurrency: config.concurrency,
                repetition: repetition.repetition,
                p50_ns: repetition.p50_ns,
                p95_ns: repetition.p95_ns,
                p99_ns: repetition.p99_ns,
                p99_9_ns: repetition.p99_9_ns,
                max_ns: repetition.max_ns,
                frames_per_sec: repetition.frames_per_sec,
                mb_per_sec: repetition.mb_per_sec,
                correctness_total: run.correctness_total,
                correctness_matches: run.correctness_matches,
                dataset_digest: dataset.manifest_digest.clone(),
                workflow_tax_percent: if level == "b0" {
                    0.0
                } else {
                    workflow_tax_percent(b0_p99, repetition.p99_ns)
                },
                metadata: metadata.clone(),
            });
        }
    }
    if let Err(error) = write_once_atomic(std::path::Path::new(&config.output), &records) {
        eprintln!("output failed: {error:?}");
        std::process::exit(2);
    }
}
