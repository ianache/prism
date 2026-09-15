use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
fn real_process_serves_health_readiness_and_configured_b2_over_http() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_prism-run"))
        .args([
            "--route",
            "b2",
            "--protocol",
            "http",
            "--listen",
            "127.0.0.1:0",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let mut stderr = BufReader::new(stderr);
    let mut ready = String::new();
    stderr.read_line(&mut ready).unwrap();
    let address = ready.strip_prefix("READY ").unwrap().trim().to_owned();

    let mut health = TcpStream::connect(&address).unwrap();
    health.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    health
        .write_all(b"GET /healthz HTTP/1.1\r\nContent-Length: 0\r\n\r\n")
        .unwrap();
    let mut health_response = String::new();
    health.read_to_string(&mut health_response).unwrap();
    assert!(health_response.starts_with("HTTP/1.1 200 OK"));
    assert!(health_response.contains("{\"status\":\"ok\"}"));

    let body = br#"{"request_id":"one","payload_hex":"00"}"#;
    let mut process = TcpStream::connect(&address).unwrap();
    process.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    write!(
        process,
        "POST /v1/process HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    )
    .unwrap();
    process.write_all(body).unwrap();
    let mut process_response = String::new();
    process.read_to_string(&mut process_response).unwrap();
    assert!(process_response.starts_with("HTTP/1.1 200 OK"));
    assert!(process_response.contains("\"route\":\"b2\""));

    child.kill().unwrap();
    let _ = child.wait();
}
