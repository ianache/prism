use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::Arc;

use rcgen::generate_simple_self_signed;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{
    ClientConfig, ClientConnection, DigitallySignedStruct, Error, SignatureScheme, StreamOwned,
};

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
