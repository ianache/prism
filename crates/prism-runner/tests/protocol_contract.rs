use prism_runner::protocol::parse_line;

#[test]
fn preserves_request_id_and_decodes_bytes() {
    let e = parse_line(r#"{"request_id":"r-1","payload_hex":"5052"}"#).unwrap();
    assert_eq!(e.request_id, "r-1");
    assert_eq!(e.payload, [0x50, 0x52]);
}

#[test]
fn input_errors_are_stable() {
    for (line, code) in [
        ("", "BLANK_LINE"),
        ("{}", "MISSING_FIELD"),
        (r#"{"request_id":"x","payload_hex":"gg"}"#, "INVALID_HEX"),
    ] {
        assert_eq!(parse_line(line).unwrap_err().code, code);
    }
}
