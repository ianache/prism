use std::collections::BTreeMap;

use prism_bench::cli::parse_args;
use prism_bench::dataset::Dataset;
use prism_bench::metadata::Metadata;
use prism_bench::output::{
    workflow_tax_for_concurrency_repetition, write_once_atomic, RawRecord,
};
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
    let mut runs: Vec<(String, usize, RawRun)> = Vec::new();
    for concurrency in &config.concurrencies {
      let run_config = RunConfig {
        warmup: config.warmup,
        samples: config.samples,
        warmup_frames: config.warmup_frames,
        convergence_window: config.convergence_window,
        convergence_threshold_percent: config.convergence_threshold_percent,
        max_warmup_frames: config.max_warmup_frames,
        measured_frames: config.measured_frames,
        repetitions: config.repetitions,
        concurrency: *concurrency,
      };
      for level_name in config.levels.split(',') {
        let level = match level_name {
            "b0" => Level::B0,
            "b1" => Level::B1,
            "b2" => Level::B2,
            _ => unreachable!(),
        };
        match run_level(level, &dataset, &run_config) {
            Ok(run) => runs.push((level_name.to_owned(), *concurrency, run)),
            Err(error) => {
                eprintln!("benchmark invalid: {error:?}");
                std::process::exit(2);
            }
        }
      }
    }
    let b0_p99_by_key = runs
        .iter()
        .filter(|(level, _, _)| level == "b0")
        .flat_map(|(_, concurrency, run)| {
            run.repetitions.iter().map(move |repetition| {
                ((*concurrency, repetition.repetition), repetition.p99_ns)
            })
        })
        .collect::<Vec<_>>();
    let b1_p99_by_key = runs
        .iter()
        .filter(|(level, _, _)| level == "b1")
        .flat_map(|(_, concurrency, run)| {
            run.repetitions.iter().map(move |repetition| {
                ((*concurrency, repetition.repetition), repetition.p99_ns)
            })
        })
        .collect::<Vec<_>>();
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
    metadata.insert("commit".to_owned(), host.commit);
    metadata.insert("command".to_owned(), host.command.clone());
    metadata.insert("run_id".to_owned(), host.run_id);
    let mut records = Vec::new();
    for (level, concurrency, run) in runs {
        for repetition in &run.repetitions {
            records.push(RawRecord {
                run_id: format!("{}-{}-c{}-r{}", config.scenario, level, concurrency, repetition.repetition),
                implementation: "rust".into(),
                level: level.clone(),
                scenario: config.scenario.clone(),
                concurrency,
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
                timestamp_utc: host.timestamp_utc.clone(),
                dataset_id: dataset.dataset_id.clone(),
                fixture_count: dataset.fixture_count,
                payload_class_105: dataset.payload_counts.class_105,
                payload_class_249: dataset.payload_counts.class_249,
                payload_class_501: dataset.payload_counts.class_501,
                valid_count: dataset.validity_counts.valid,
                invalid_count: dataset.validity_counts.invalid,
                edge_complex_count: dataset.validity_counts.edge_complex,
                warmup_target: run.warmup_target,
                warmup_frames: run.warmup_frames,
                convergence_window: run.convergence_window,
                convergence_threshold_percent: run.convergence_threshold_percent,
                converged: run.converged,
                measured_frames: run.measured_frames,
                typed_rejections: run.typed_rejections,
                execution_failures: run.execution_failures,
                command: host.command.clone(),
                dataset_digest: dataset.manifest_digest.clone(),
                workflow_tax_percent: if level == "b0" {
                    0.0
                } else {
                    workflow_tax_for_concurrency_repetition(
                        &b0_p99_by_key, concurrency, repetition.repetition, repetition.p99_ns,
                    )
                },
                metadata: metadata.clone(),
                protocol_version: "1.1".into(),
                observability_variant: run.observability_variant.clone(),
                execution_id: run.execution_id.clone(),
                filter_timings_ns: run.filter_timings_ns.clone(),
                filter_invocations: run.filter_invocations.clone(),
                filter_rejections: run.filter_rejections.clone(),
                filter_execution_failures: run.filter_execution_failures.clone(),
                observability_tax_percent: if level == "b2" {
                    workflow_tax_for_concurrency_repetition(
                        &b1_p99_by_key, concurrency, repetition.repetition, repetition.p99_ns,
                    )
                } else { 0.0 },
            });
        }
    }
    if let Err(error) = write_once_atomic(std::path::Path::new(&config.output), &records) {
        eprintln!("output failed: {error:?}");
        std::process::exit(2);
    }
}
