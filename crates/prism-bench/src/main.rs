use std::collections::BTreeMap;

use prism_bench::cli::parse_args;
use prism_bench::burst::{Calibration, Phase, PhaseSchedule};
use prism_bench::dataset::Dataset;
use prism_bench::metadata::Metadata;
use prism_bench::output::{
    workflow_tax_for_concurrency_repetition, workflow_tax_for_phase_repetition,
    write_once_atomic, RawRecord,
};
use prism_bench::runner::{run_level, run_phase, Level, PhaseRun, RawRun, RunConfig};
use prism_bench::sustained::{run_sustained, SustainedConfig, SustainedWindow};

fn run_s5(config: &prism_bench::cli::Config, dataset: &Dataset) {
    let per_level_duration = config.duration_seconds / 3;
    let sustained_config = SustainedConfig { duration_seconds: per_level_duration, window_frames: config.measured_frames, repetitions: config.repetitions };
    let host = Metadata::collect(config);
    let mut metadata = BTreeMap::new();
    metadata.insert("cpu_model".into(), host.cpu_model.clone()); metadata.insert("physical_cores".into(), host.physical_cores.clone());
    metadata.insert("logical_cores".into(), host.logical_cores.clone()); metadata.insert("ram_bytes".into(), host.ram_bytes.clone());
    metadata.insert("os".into(), host.os.clone()); metadata.insert("kernel".into(), host.kernel.clone());
    metadata.insert("runtime".into(), host.runtime.clone()); metadata.insert("compiler".into(), host.compiler.clone());
    metadata.insert("governor".into(), host.governor.clone()); metadata.insert("container_limits".into(), host.container_limits.clone());
    metadata.insert("affinity".into(), host.affinity.clone()); metadata.insert("commit".into(), host.commit.clone());
    let mut runs = Vec::new();
    for (name, level) in [("b0", Level::B0), ("b1", Level::B1), ("b2", Level::B2)] {
        let run = run_sustained(level, dataset, &sustained_config).unwrap_or_else(|error| { eprintln!("sustained run invalid: {error:?}"); std::process::exit(2) });
        runs.push((name, run));
    }
    let p99 = runs.iter().flat_map(|(level, run)| run.repetitions.iter().map(move |window| ((*level, window.repetition, window.window_index), window.p99_ns))).collect::<BTreeMap<_, _>>();
    let mut records = Vec::new();
    for (level, run) in runs {
        for window in run.repetitions {
            let base = if level == "b1" { p99.get(&("b0", window.repetition, window.window_index)).copied() } else if level == "b2" { p99.get(&("b1", window.repetition, window.window_index)).copied() } else { None };
            records.push(s5_record(config, dataset, &host, &metadata, level, &window, base));
        }
    }
    if let Err(error) = write_once_atomic(std::path::Path::new(&config.output), &records) { eprintln!("output failed: {error:?}"); std::process::exit(2); }
}

fn s5_record(config: &prism_bench::cli::Config, dataset: &Dataset, host: &Metadata, metadata: &BTreeMap<String, String>, level: &str, window: &SustainedWindow, base_p99: Option<u128>) -> RawRecord {
    let tax = base_p99.map_or(0.0, |base| prism_bench::output::workflow_tax_percent(base, window.p99_ns));
    RawRecord {
        run_id: format!("S5-{}-r{}-w{}", level, window.repetition, window.window_index), implementation: "rust".into(), level: level.into(), scenario: "S5".into(), concurrency: 1, repetition: window.repetition,
        p50_ns: window.p50_ns, p95_ns: window.p95_ns, p99_ns: window.p99_ns, p99_9_ns: window.p99_9_ns, max_ns: window.max_ns, frames_per_sec: window.frames_per_sec, mb_per_sec: window.mb_per_sec,
        correctness_total: window.correctness_total, correctness_matches: window.correctness_matches, timestamp_utc: host.timestamp_utc.clone(), dataset_id: dataset.dataset_id.clone(), fixture_count: dataset.fixture_count,
        payload_class_105: dataset.payload_counts.class_105, payload_class_249: dataset.payload_counts.class_249, payload_class_501: dataset.payload_counts.class_501, valid_count: dataset.validity_counts.valid, invalid_count: dataset.validity_counts.invalid, edge_complex_count: dataset.validity_counts.edge_complex,
        warmup_target: config.warmup, warmup_frames: config.warmup, convergence_window: config.convergence_window, convergence_threshold_percent: config.convergence_threshold_percent, converged: true, measured_frames: window.measured_frames,
        typed_rejections: window.typed_rejections, execution_failures: window.execution_failures, command: host.command.clone(), dataset_digest: dataset.manifest_digest.clone(), workflow_tax_percent: if level == "b1" { tax } else { 0.0 }, metadata: metadata.clone(), protocol_version: "1.1".into(), observability_variant: window.observability_variant.clone(), execution_id: format!("S5-{}-r{}-w{}", level, window.repetition, window.window_index),
        filter_timings_ns: window.filter_timings_ns.clone(), filter_invocations: window.filter_invocations.clone(), filter_rejections: window.filter_rejections.clone(), filter_execution_failures: window.filter_execution_failures.clone(), observability_tax_percent: if level == "b2" { tax } else { 0.0 },
        phase: format!("window-{}", window.window_index), offered_frames_per_sec: window.frames_per_sec, processed_frames_per_sec: window.frames_per_sec, late_frames: 0, on_time_frames: window.measured_frames, lateness_p50_ns: 0, lateness_p95_ns: 0, lateness_p99_ns: 0, calibration_median_frames_per_sec: 0.0, baseline_frames_per_sec: 0.0, burst_frames_per_sec: 0.0,
        window_index: window.window_index, duration_seconds: window.duration_seconds, rss_before_bytes: window.resource_before.rss_bytes, rss_after_bytes: window.resource_after.rss_bytes, incomplete_tail_frames: window.incomplete_tail_frames,
    }
}

fn run_s4(config: &prism_bench::cli::Config, dataset: &Dataset) {
    let run_config = RunConfig {
        warmup: config.warmup, samples: config.samples, warmup_frames: config.warmup_frames,
        convergence_window: config.convergence_window, convergence_threshold_percent: config.convergence_threshold_percent,
        max_warmup_frames: config.max_warmup_frames, measured_frames: config.measured_frames,
        repetitions: config.repetitions, concurrency: config.concurrency,
    };
    let calibration_run = run_level(Level::B0, dataset, &run_config).unwrap_or_else(|error| {
        eprintln!("calibration invalid: {error:?}"); std::process::exit(2)
    });
    let calibration = Calibration::from_throughputs(
        &calibration_run.repetitions.iter().map(|item| item.frames_per_sec).collect::<Vec<_>>(),
    ).unwrap_or_else(|error| { eprintln!("calibration invalid: {error:?}"); std::process::exit(2) });
    let phases = [Phase::Baseline, Phase::Burst, Phase::Recovery];
    let mut phase_runs: Vec<(String, Phase, usize, PhaseRun)> = Vec::new();
    for phase in phases {
        let schedule = PhaseSchedule::new(&calibration, phase, config.measured_frames).unwrap_or_else(|error| {
            eprintln!("schedule invalid: {error:?}"); std::process::exit(2)
        });
        for (level_name, level) in [("b0", Level::B0), ("b1", Level::B1), ("b2", Level::B2)] {
            for repetition in 1..=config.repetitions {
                let run = run_phase(level, dataset, &run_config, &schedule).unwrap_or_else(|error| {
                    eprintln!("phase invalid: {error:?}"); std::process::exit(2)
                });
                phase_runs.push((level_name.to_owned(), phase, repetition, run));
            }
        }
    }
    let host = Metadata::collect(config);
    let mut metadata = BTreeMap::new();
    metadata.insert("cpu_model".to_owned(), host.cpu_model); metadata.insert("physical_cores".to_owned(), host.physical_cores);
    metadata.insert("logical_cores".to_owned(), host.logical_cores); metadata.insert("ram_bytes".to_owned(), host.ram_bytes);
    metadata.insert("os".to_owned(), host.os); metadata.insert("kernel".to_owned(), host.kernel);
    metadata.insert("runtime".to_owned(), host.runtime); metadata.insert("compiler".to_owned(), host.compiler);
    metadata.insert("governor".to_owned(), host.governor); metadata.insert("container_limits".to_owned(), host.container_limits);
    metadata.insert("affinity".to_owned(), host.affinity); metadata.insert("commit".to_owned(), host.commit);
    metadata.insert("command".to_owned(), host.command.clone()); metadata.insert("run_id".to_owned(), host.run_id);
    let b0 = phase_runs.iter().filter(|(level, _, _, _)| level == "b0").map(|(_, phase, repetition, run)| {
        ((phase.as_str().to_owned(), *repetition), run.p99_ns)
    }).collect::<Vec<_>>();
    let b1 = phase_runs.iter().filter(|(level, _, _, _)| level == "b1").map(|(_, phase, repetition, run)| {
        ((phase.as_str().to_owned(), *repetition), run.p99_ns)
    }).collect::<Vec<_>>();
    let mut records = Vec::new();
    for (level, phase, repetition, run) in &phase_runs {
        let workflow_tax = if level == "b0" { 0.0 } else {
            workflow_tax_for_phase_repetition(&b0, phase.as_str(), *repetition, run.p99_ns)
        };
        let observability_tax = if level == "b2" {
            workflow_tax_for_phase_repetition(&b1, phase.as_str(), *repetition, run.p99_ns)
        } else { 0.0 };
        records.push(RawRecord {
            run_id: format!("S4-{}-{}-r{}", level, phase.as_str(), repetition), implementation: "rust".into(),
            level: level.clone(), scenario: "S4".into(), concurrency: 1, repetition: *repetition,
            p50_ns: run.p50_ns, p95_ns: run.p95_ns, p99_ns: run.p99_ns, p99_9_ns: run.p99_9_ns, max_ns: run.max_ns,
            frames_per_sec: run.frames_per_sec, mb_per_sec: run.mb_per_sec, correctness_total: run.correctness_total,
            correctness_matches: run.correctness_matches, timestamp_utc: host.timestamp_utc.clone(), dataset_id: dataset.dataset_id.clone(),
            fixture_count: dataset.fixture_count, payload_class_105: dataset.payload_counts.class_105, payload_class_249: dataset.payload_counts.class_249,
            payload_class_501: dataset.payload_counts.class_501, valid_count: dataset.validity_counts.valid, invalid_count: dataset.validity_counts.invalid,
            edge_complex_count: dataset.validity_counts.edge_complex, warmup_target: run_config.warmup, warmup_frames: run_config.warmup,
            convergence_window: run_config.convergence_window, convergence_threshold_percent: run_config.convergence_threshold_percent,
            converged: true, measured_frames: run.frames, typed_rejections: run.typed_rejections, execution_failures: run.execution_failures,
            command: host.command.clone(), dataset_digest: dataset.manifest_digest.clone(), workflow_tax_percent: workflow_tax,
            metadata: metadata.clone(), protocol_version: "1.1".into(), observability_variant: run.observability_variant.clone(),
            execution_id: format!("S4-{}-{}", level, phase.as_str()), filter_timings_ns: run.filter_timings_ns.clone(),
            filter_invocations: run.filter_invocations.clone(), filter_rejections: run.filter_rejections.clone(),
            filter_execution_failures: run.filter_execution_failures.clone(), observability_tax_percent: observability_tax,
            phase: phase.as_str().into(), offered_frames_per_sec: run.offered_frames_per_sec, processed_frames_per_sec: run.frames_per_sec,
            late_frames: run.late_frames, on_time_frames: run.on_time_frames, lateness_p50_ns: run.lateness_p50_ns,
            lateness_p95_ns: run.lateness_p95_ns, lateness_p99_ns: run.lateness_p99_ns,
            calibration_median_frames_per_sec: calibration.median_frames_per_sec, baseline_frames_per_sec: calibration.baseline_frames_per_sec,
            burst_frames_per_sec: calibration.burst_frames_per_sec,
            window_index: 0, duration_seconds: 0.0, rss_before_bytes: None, rss_after_bytes: None,
            incomplete_tail_frames: 0,
        });
    }
    if let Err(error) = write_once_atomic(std::path::Path::new(&config.output), &records) {
        eprintln!("output failed: {error:?}"); std::process::exit(2);
    }
}

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
    if config.scenario == "S4" { run_s4(&config, &dataset); return; }
    if config.scenario == "S5" { run_s5(&config, &dataset); return; }
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
                phase: "none".into(),
                offered_frames_per_sec: repetition.frames_per_sec,
                processed_frames_per_sec: repetition.frames_per_sec,
                late_frames: 0,
                on_time_frames: repetition.frames,
                lateness_p50_ns: 0,
                lateness_p95_ns: 0,
                lateness_p99_ns: 0,
                calibration_median_frames_per_sec: 0.0,
                baseline_frames_per_sec: 0.0,
                burst_frames_per_sec: 0.0,
                window_index: 0, duration_seconds: 0.0, rss_before_bytes: None, rss_after_bytes: None,
                incomplete_tail_frames: 0,
            });
        }
    }
    if let Err(error) = write_once_atomic(std::path::Path::new(&config.output), &records) {
        eprintln!("output failed: {error:?}");
        std::process::exit(2);
    }
}
