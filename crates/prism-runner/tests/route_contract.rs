use prism_runner::{parse_args, CliError, Route};

#[test]
fn accepts_routes_and_default_prefix() {
    let args = parse_args(["prism-run", "--route", "b2"]).unwrap();
    assert_eq!(args.route, Route::B2);
    assert_eq!(args.request_id_prefix, "req");
    assert_eq!(args.listen, None);
    assert_eq!((args.workers, args.connection_queue), (1, 0));
}

#[test]
fn accepts_bounded_worker_settings() {
    let args = parse_args([
        "prism-run",
        "--route",
        "b1",
        "--listen",
        "127.0.0.1:0",
        "--workers",
        "2",
        "--connection-queue",
        "3",
    ])
    .unwrap();
    assert_eq!((args.workers, args.connection_queue), (2, 3));
    assert_eq!(
        parse_args(["prism-run", "--route", "b1", "--workers", "0"]),
        Err(CliError::InvalidValue("--workers".into()))
    );
    assert_eq!(
        parse_args(["prism-run", "--route", "b1", "--connection-queue", "bad"]),
        Err(CliError::InvalidValue("--connection-queue".into()))
    );
}

#[test]
fn rejects_missing_and_unknown_routes() {
    assert_eq!(parse_args(["prism-run"]), Err(CliError::MissingRoute));
    assert_eq!(
        parse_args(["prism-run", "--route", "b9"]),
        Err(CliError::UnknownRoute("b9".into()))
    );
}

#[test]
fn accepts_custom_prefix_and_help() {
    let args = parse_args(["prism-run", "--route", "b0", "--request-id-prefix", "demo"]).unwrap();
    assert_eq!(args.request_id_prefix, "demo");
    assert_eq!(args.listen, None);
    assert_eq!(parse_args(["prism-run", "--help"]), Err(CliError::Help));
}

#[test]
fn accepts_optional_listen_address() {
    let args = parse_args(["prism-run", "--route", "b1", "--listen", "127.0.0.1:0"]).unwrap();
    assert_eq!(args.listen.as_deref(), Some("127.0.0.1:0"));
}
