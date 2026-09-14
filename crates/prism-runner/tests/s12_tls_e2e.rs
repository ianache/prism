use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

use rcgen::generate_simple_self_signed;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{
    ClientConfig, ClientConnection, DigitallySignedStruct, Error, SignatureScheme, StreamOwned,
};

type SecureClient = StreamOwned<ClientConnection, TcpStream>;
static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn launch_secure(
    extra: &[String],
) -> (
    std::process::Child,
    BufReader<std::process::ChildStderr>,
    PathBuf,
    String,
) {
    let dir = std::env::temp_dir().join(format!(
        "prism-s12-matrix-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let generated = generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    fs::write(dir.join("cert.pem"), generated.cert.pem()).unwrap();
    fs::write(dir.join("key.pem"), generated.key_pair.serialize_pem()).unwrap();
    fs::write(dir.join("token.txt"), "secret\n").unwrap();
    let mut args = vec!["--route", "b0", "--listen", "127.0.0.1:0", "--tls-cert"]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    args.push(dir.join("cert.pem").to_string_lossy().into_owned());
    args.extend([
        "--tls-key".into(),
        dir.join("key.pem").to_string_lossy().into_owned(),
    ]);
    args.extend([
        "--auth-token-file".into(),
        dir.join("token.txt").to_string_lossy().into_owned(),
    ]);
    args.extend(extra.iter().cloned());
    let mut child = Command::new(env!("CARGO_BIN_EXE_prism-run"))
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let mut stderr = BufReader::new(stderr);
    let mut ready = String::new();
    loop {
        ready.clear();
        stderr.read_line(&mut ready).unwrap();
        if ready.starts_with("READY ") {
            break;
        }
    }
    let address = ready.strip_prefix("READY ").unwrap().trim().to_owned();
    (child, stderr, dir, address)
}

fn secure_client(address: &str) -> SecureClient {
    let socket = TcpStream::connect(address).unwrap();
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TestOnlyVerifier))
        .with_no_client_auth();
    let connection =
        ClientConnection::new(Arc::new(config), ServerName::try_from("localhost").unwrap())
            .unwrap();
    let mut client = StreamOwned::new(connection, socket);
    client.conn.complete_io(&mut client.sock).unwrap();
    client
}

fn send_and_read(client: &mut SecureClient, line: &[u8]) -> String {
    try_send_and_read(client, line).unwrap()
}

fn try_send_and_read(client: &mut SecureClient, line: &[u8]) -> std::io::Result<String> {
    client.write_all(line)?;
    client.write_all(b"\n")?;
    client.flush()?;
    try_read(client)
}

fn try_read(client: &mut SecureClient) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let count = client.read(&mut byte)?;
        if count == 0 {
            break;
        }
        bytes.push(byte[0]);
        if byte[0] == b'\n' {
            break;
        }
    }
    Ok(String::from_utf8(bytes).unwrap())
}

fn read_response(client: &mut SecureClient) -> String {
    try_read(client).unwrap()
}

#[derive(Debug)]
struct TestOnlyVerifier;

impl ServerCertVerifier for TestOnlyVerifier {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}

#[test]
fn real_process_completes_tls_then_exchanges_jsonl() {
    let _guard = TEST_LOCK.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("prism-s12-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let generated = generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    fs::write(dir.join("cert.pem"), generated.cert.pem()).unwrap();
    fs::write(dir.join("key.pem"), generated.key_pair.serialize_pem()).unwrap();
    fs::write(dir.join("token.txt"), "secret\n").unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_prism-run"))
        .args(["--route", "b0", "--listen", "127.0.0.1:0", "--tls-cert"])
        .arg(dir.join("cert.pem"))
        .args(["--tls-key"])
        .arg(dir.join("key.pem"))
        .args(["--auth-token-file"])
        .arg(dir.join("token.txt"))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let mut ready_line = String::new();
    BufReader::new(stderr).read_line(&mut ready_line).unwrap();
    let address = ready_line.strip_prefix("READY ").unwrap().trim();
    let socket = TcpStream::connect(address).unwrap();
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TestOnlyVerifier))
        .with_no_client_auth();
    let connection =
        ClientConnection::new(Arc::new(config), ServerName::try_from("localhost").unwrap())
            .unwrap();
    let mut tls = StreamOwned::new(connection, socket);
    tls.conn.complete_io(&mut tls.sock).unwrap();
    tls.write_all(br#"{"request_id":"bad","payload_hex":"00"}"#)
        .unwrap();
    tls.write_all(b"\n").unwrap();
    tls.flush().unwrap();
    let mut reader = BufReader::new(&mut tls);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();
    assert!(response.contains("AUTHENTICATION_FAILED"));
    reader
        .get_mut()
        .write_all(br#"{"request_id":"good","payload_hex":"00","auth_token":"secret"}"#)
        .unwrap();
    reader.get_mut().write_all(b"\n").unwrap();
    reader.get_mut().flush().unwrap();
    response.clear();
    reader.read_line(&mut response).unwrap();
    assert!(response.contains("\"ok\":true"));
    assert!(!response.contains("secret"));
    child.kill().unwrap();
    let _ = child.wait();
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn tls_capacity_rejects_and_recovers_after_client_eof() {
    let _guard = TEST_LOCK.lock().unwrap();
    let (mut child, _stderr, dir, address) = launch_secure(&["--workers".into(), "1".into()]);
    let busy = secure_client(&address);
    let mut rejected = secure_client(&address);
    let rejected_response = read_response(&mut rejected);
    assert!(rejected_response.contains("CAPACITY_EXCEEDED"));
    drop(rejected);
    drop(busy);
    let mut recovered = secure_client(&address);
    let recovered_response = send_and_read(
        &mut recovered,
        br#"{"request_id":"recovered","payload_hex":"00","auth_token":"secret"}"#,
    );
    assert!(recovered_response.contains("\"ok\":true"));
    drop(recovered);
    child.kill().unwrap();
    let _ = child.wait();
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn tls_lifecycle_drains_and_rejects_new_secure_connections() {
    let _guard = TEST_LOCK.lock().unwrap();
    let sentinel_dir = std::env::temp_dir().join(format!(
        "prism-s12-sentinel-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&sentinel_dir).unwrap();
    let sentinel = sentinel_dir.join("shutdown.flag");
    let args = vec![
        "--shutdown-file".into(),
        sentinel.to_string_lossy().into_owned(),
        "--drain-timeout-ms".into(),
        "5000".into(),
    ];
    let (mut child, mut stderr, dir, address) = launch_secure(&args);
    let busy = secure_client(&address);
    fs::write(&sentinel, b"").unwrap();
    let mut state = String::new();
    stderr.read_line(&mut state).unwrap();
    assert_eq!(state.trim(), "DRAINING");
    let mut rejected = secure_client(&address);
    let rejected_result = try_read(&mut rejected);
    assert!(rejected_result
        .as_ref()
        .map(|response| response.contains("SERVER_DRAINING"))
        .unwrap_or(true));
    drop(rejected);
    drop(busy);
    state.clear();
    stderr.read_line(&mut state).unwrap();
    assert_eq!(state.trim(), "STOPPED");
    assert_eq!(child.wait().unwrap().code(), Some(0));
    let _ = fs::remove_dir_all(dir);
    let _ = fs::remove_dir_all(sentinel_dir);
}
