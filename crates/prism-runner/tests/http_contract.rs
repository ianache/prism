use prism_runner::http::{parse_request, response_bytes, HttpError, Request};

#[test]
fn parses_process_request_with_bounded_headers_and_body() {
    let raw = b"POST /v1/process HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 39\r\nAuthorization: Bearer secret\r\n\r\n{\"request_id\":\"one\",\"payload_hex\":\"00\"}";
    let request = parse_request(raw)
    .unwrap();
    assert_eq!(request, Request {
        method: "POST".into(),
        path: "/v1/process".into(),
        content_type: Some("application/json".into()),
        authorization: Some("Bearer secret".into()),
        body: br#"{"request_id":"one","payload_hex":"00"}"#.to_vec(),
    });
}

#[test]
fn parses_health_and_readiness_get_requests_without_a_body() {
    for path in ["/healthz", "/readyz"] {
        let request = parse_request(
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n")
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, path);
        assert!(request.body.is_empty());
    }
}

#[test]
fn rejects_unsupported_methods_paths_content_types_and_chunked_bodies() {
    for (raw, status) in [
        (b"PUT /v1/process HTTP/1.1\r\nContent-Length: 0\r\n\r\n".as_slice(), 405),
        (b"POST /unknown HTTP/1.1\r\nContent-Length: 0\r\n\r\n".as_slice(), 404),
        (b"POST /v1/process HTTP/1.1\r\nContent-Type: text/plain\r\nContent-Length: 0\r\n\r\n".as_slice(), 415),
        (b"POST /v1/process HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n".as_slice(), 400),
    ] {
        let error = parse_request(raw).unwrap_err();
        assert_eq!(error.status(), status);
    }
}

#[test]
fn rejects_malformed_content_length_and_oversized_body() {
    let malformed = parse_request(b"POST /v1/process HTTP/1.1\r\nContent-Length: nope\r\n\r\n");
    assert_eq!(malformed.unwrap_err().status(), 400);

    let oversized = format!(
        "POST /v1/process HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        64 * 1024 + 1
    );
    assert_eq!(parse_request(oversized.as_bytes()).unwrap_err().status(), 413);
}

#[test]
fn extracts_only_bearer_authorization_and_writes_stable_json_response() {
    let request = parse_request(
        b"POST /v1/process HTTP/1.1\r\nAuthorization: Bearer secret\r\nContent-Type: application/json\r\nContent-Length: 0\r\n\r\n",
    )
    .unwrap();
    assert_eq!(request.bearer_token(), Some("secret"));

    let response = response_bytes(200, br#"{"ok":true}"#);
    let text = String::from_utf8(response).unwrap();
    assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(text.contains("Content-Type: application/json\r\n"));
    assert!(text.contains("Content-Length: 11\r\n"));
    assert!(text.ends_with("\r\n\r\n{\"ok\":true}"));
    assert!(!text.contains("secret"));
}

#[test]
fn http_error_messages_are_not_credentials() {
    let error = HttpError::Unauthorized;
    assert_eq!(error.status(), 401);
    assert!(!format!("{error:?}").contains("token"));
}
