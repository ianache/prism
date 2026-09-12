fn main() {
    let config = match prism_bench::cli::parse_args(std::env::args()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let dataset = match prism_bench::dataset::Dataset::load(std::path::Path::new(&config.dataset)) {
        Ok(dataset) => dataset,
        Err(_) => {
            eprintln!("unable to load dataset");
            std::process::exit(2);
        }
    };
    let run_config = prism_bench::runner::RunConfig {
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
    let mut lines = Vec::new();
    for level in config.levels.split(',') {
        let level = match level {
            "b0" => prism_bench::runner::Level::B0,
            "b1" => prism_bench::runner::Level::B1,
            _ => {
                eprintln!("unknown level");
                std::process::exit(2);
            }
        };
        let run = match prism_bench::runner::run_level(level, &dataset, &run_config) {
            Ok(run) => run,
            Err(error) => {
                eprintln!("benchmark invalid: {error:?}");
                std::process::exit(2);
            }
        };
        let name = match level {
            prism_bench::runner::Level::B0 => "b0",
            prism_bench::runner::Level::B1 => "b1",
        };
        lines.push(format!("{{\"implementation\":\"rust\",\"level\":\"{}\",\"scenario\":\"{}\",\"samples\":{},\"p50_ns\":{},\"p95_ns\":{},\"p99_ns\":{},\"p99_9_ns\":{},\"max_ns\":{},\"frames_per_sec\":{},\"mb_per_sec\":{},\"correctness_total\":{},\"correctness_matches\":{},\"dataset_digest\":\"{}\",\"cpu_model\":\"{}\",\"cores\":\"{}\",\"ram_bytes\":\"{}\",\"os\":\"{}\",\"governor\":\"{}\",\"affinity\":\"{}\"}}\n", name, config.scenario, run.samples, run.p50_ns, run.p95_ns, run.p99_ns, run.p99_9_ns, run.max_ns, run.frames_per_sec, run.mb_per_sec, run.correctness_total, run.correctness_matches, dataset.manifest_digest, config.metadata[0], config.metadata[1], config.metadata[2], config.metadata[3], config.metadata[4], config.metadata[5]));
    }
    if let Err(error) =
        prism_bench::output::write_once(std::path::Path::new(&config.output), &lines.concat())
    {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
