use prism_runner::http::{parse_request, response_for_request};
use prism_runner::Route;

fn request(raw: &[u8]) -> prism_runner::http::Request {
    parse_request(raw).unwrap()
}

#[test]
fn health_and_readiness_do_not_execute_telemetry() {
    let health = request(b"GET /healthz HTTP/1.1\r\nContent-Length: 0\r\n\r\n");
    let (status, body) = response_for_request(&health, Route::B2, "http", 1, None, true);
    assert_eq!(status, 200);
    assert_eq!(String::from_utf8(body).unwrap(), "{\"status\":\"ok\"}");

    let ready = request(b"GET /readyz HTTP/1.1\r\nContent-Length: 0\r\n\r\n");
    let (status, body) = response_for_request(&ready, Route::B2, "http", 1, None, false);
    assert_eq!(status, 503);
    assert_eq!(String::from_utf8(body).unwrap(), "{\"ready\":false}");
}

#[test]
fn process_uses_configured_route_and_external_bearer_token() {
    let process = request(
        b"POST /v1/process HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 39\r\nAuthorization: Bearer secret\r\n\r\n{\"request_id\":\"one\",\"payload_hex\":\"00\"}",
    );
    let (status, body) = response_for_request(&process, Route::B2, "http", 1, Some("secret"), true);
    assert_eq!(status, 200);
    let body = String::from_utf8(body).unwrap();
    assert!(body.contains("\"route\":\"b2\""));
    assert!(!body.contains("secret"));
}

#[test]
fn invalid_bearer_is_rejected_without_running_the_domain() {
    let process = request(
        b"POST /v1/process HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 39\r\nAuthorization: Bearer wrong\r\n\r\n{\"request_id\":\"one\",\"payload_hex\":\"00\"}",
    );
    let (status, body) = response_for_request(&process, Route::B2, "http", 1, Some("secret"), true);
    assert_eq!(status, 401);
    let body = String::from_utf8(body).unwrap();
    assert!(body.contains("AUTHENTICATION_FAILED"));
    assert!(!body.contains("wrong"));
}

#[test]
fn process_is_unavailable_while_the_server_is_draining() {
    let process = request(
        b"POST /v1/process HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 39\r\n\r\n{\"request_id\":\"one\",\"payload_hex\":\"00\"}",
    );
    let (status, body) = response_for_request(&process, Route::B2, "http", 1, None, false);
    assert_eq!(status, 503);
    assert_eq!(String::from_utf8(body).unwrap(), "{\"error\":\"server is draining\"}");
}

#[test]
fn malformed_process_envelope_maps_to_http_bad_request() {
    let process = request(
        b"POST /v1/process HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 8\r\n\r\nnot-json",
    );
    let (status, body) = response_for_request(&process, Route::B2, "http", 1, None, true);
    assert_eq!(status, 400);
    assert!(String::from_utf8(body).unwrap().contains("MALFORMED_JSON"));
}
