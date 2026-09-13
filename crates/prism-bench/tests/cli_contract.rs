use prism_bench::cli::{parse_args, CliError};
use prism_bench::metadata::Metadata;

fn valid_args() -> [&'static str; 27] {
    [
        "bench",
        "--dataset",
        "smoke",
        "--levels",
        "b0,b1",
        "--scenario",
        "S1",
        "--warmup",
        "10000",
        "--samples",
        "10000",
        "--repetitions",
        "5",
        "--output",
        "out.jsonl",
        "--cpu-model",
        "cpu",
        "--cores",
        "8",
        "--ram-bytes",
        "100",
        "--os",
        "windows",
        "--governor",
        "test",
        "--affinity",
        "none",
    ]
}

#[test]
fn valid_s1_config_contains_protocol_parameters() {
    let config = parse_args(valid_args()).unwrap();
    assert_eq!(config.warmup_frames, 10_000);
    assert_eq!(config.convergence_window, 1_000);
    assert_eq!(config.convergence_threshold_percent, 5);
    assert_eq!(config.repetitions, 5);
    assert_eq!(config.measured_frames, 10_000);
    assert_eq!(config.concurrency, 1);
    assert_eq!(config.concurrencies, vec![1]);
    let metadata = Metadata::collect(&config);
    assert_eq!(metadata.cpu_model, "cpu");
    assert_eq!(metadata.runtime, "rust");
    assert_eq!(metadata.container_limits, "N/D");
    assert_ne!(metadata.command, "N/D");
    assert!(!metadata.timestamp_utc.is_empty());
}

#[test]
fn cli_rejects_invalid_scenario_and_level() {
    let mut args = valid_args().to_vec();
    args[6] = "S5";
    assert_eq!(parse_args(args), Err(CliError::InvalidScenario));
    let mut args = valid_args().to_vec();
    args[4] = "b0,b3";
    assert_eq!(parse_args(args), Err(CliError::InvalidLevel));
}

#[test]
fn cli_accepts_b2_level_with_the_100k_decision_target() {
    let mut args = valid_args().to_vec();
    args[4] = "b1,b2";
    args[10] = "100000";
    let config = parse_args(args).unwrap();
    assert_eq!(config.levels, "b1,b2");
    assert_eq!(config.measured_frames, 100_000);
}

#[test]
fn cli_rejects_b2_without_b1_baseline() {
    let mut args = valid_args().to_vec();
    args[4] = "b2";
    args[10] = "100000";
    assert_eq!(parse_args(args), Err(CliError::MissingValue));
}

#[test]
fn cli_accepts_s2_with_b0_b1_b2_and_100k() {
    let mut args = valid_args().to_vec();
    args[4] = "b0,b1,b2";
    args[6] = "S2";
    args[10] = "100000";
    let config = parse_args(args).unwrap();
    assert_eq!(config.scenario, "S2");
    assert_eq!(config.measured_frames, 100_000);
}

#[test]
fn cli_accepts_s3_with_all_concurrencies() {
    let mut args = valid_args().to_vec();
    args[4] = "b0,b1,b2";
    args[6] = "S3";
    args[10] = "100000";
    args.extend(["--concurrency", "1,2,4,8,16,32,64"]);
    let config = parse_args(args).unwrap();
    assert_eq!(config.scenario, "S3");
    assert_eq!(config.concurrencies, vec![1, 2, 4, 8, 16, 32, 64]);
}

#[test]
fn cli_accepts_s4_with_b0_b1_b2_and_single_concurrency() {
    let mut args = valid_args().to_vec();
    args[4] = "b0,b1,b2";
    args[6] = "S4";
    args[10] = "100000";
    let config = parse_args(args).unwrap();
    assert_eq!(config.scenario, "S4");
    assert_eq!(config.concurrency, 1);
}

#[test]
fn cli_rejects_s4_without_all_levels_or_with_parallel_concurrency() {
    let mut missing_level = valid_args().to_vec();
    missing_level[4] = "b0,b1";
    missing_level[6] = "S4";
    missing_level[10] = "100000";
    assert_eq!(parse_args(missing_level), Err(CliError::MissingValue));

    let mut parallel = valid_args().to_vec();
    parallel[4] = "b0,b1,b2";
    parallel[6] = "S4";
    parallel[10] = "100000";
    parallel.extend(["--concurrency", "2"]);
    assert_eq!(parse_args(parallel), Err(CliError::MissingValue));
}

#[test]
fn cli_rejects_s3_without_all_levels() {
    let mut args = valid_args().to_vec();
    args[6] = "S3";
    args[10] = "100000";
    args.extend(["--concurrency", "1,2,4,8,16,32,64"]);
    assert_eq!(parse_args(args), Err(CliError::MissingValue));
}

#[test]
fn cli_rejects_s3_duplicate_or_invalid_concurrency() {
    for value in ["1,2,2,4,8,16,32,64", "1,3,4,8,16,32,64"] {
        let mut args = valid_args().to_vec();
        args[4] = "b0,b1,b2";
        args[6] = "S3";
        args[10] = "100000";
        args.extend(["--concurrency", value]);
        assert_eq!(parse_args(args), Err(CliError::MissingValue));
    }
}
