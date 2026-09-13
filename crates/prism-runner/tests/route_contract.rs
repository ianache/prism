use prism_runner::{parse_args, CliError, Route};

#[test]
fn accepts_routes_and_default_prefix() {
    let args = parse_args(["prism-run", "--route", "b2"]).unwrap();
    assert_eq!(args.route, Route::B2);
    assert_eq!(args.request_id_prefix, "req");
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
    assert_eq!(parse_args(["prism-run", "--help"]), Err(CliError::Help));
}
