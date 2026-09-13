use std::fs;
use std::io::{BufReader, Cursor};
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;

#[derive(Clone)]
pub struct SecurityConfig {
    pub tls: Option<Arc<ServerConfig>>,
    pub auth_token: Option<String>,
}

pub fn load(
    cert_path: Option<&str>,
    key_path: Option<&str>,
    token_path: Option<&str>,
) -> Result<SecurityConfig, String> {
    if cert_path.is_some() != key_path.is_some() {
        return Err("--tls-cert and --tls-key must be provided together".into());
    }
    let tls = match (cert_path, key_path) {
        (Some(cert_path), Some(key_path)) => {
            Some(Arc::new(load_server_config(cert_path, key_path)?))
        }
        _ => None,
    };
    let auth_token = token_path.map(read_token).transpose()?;
    Ok(SecurityConfig { tls, auth_token })
}

fn load_server_config(cert_path: &str, key_path: &str) -> Result<ServerConfig, String> {
    let cert_bytes = fs::read(cert_path).map_err(|e| format!("TLS certificate failed: {e}"))?;
    let key_bytes = fs::read(key_path).map_err(|e| format!("TLS private key failed: {e}"))?;
    let mut cert_reader = BufReader::new(Cursor::new(cert_bytes));
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("TLS certificate parse failed: {e}"))?;
    if certs.is_empty() {
        return Err("TLS certificate file contains no certificates".into());
    }
    let mut key_reader = BufReader::new(Cursor::new(key_bytes));
    let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(&mut key_reader)
        .map_err(|e| format!("TLS private key parse failed: {e}"))?
        .ok_or_else(|| "TLS private key file contains no private key".to_owned())?;
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| format!("TLS configuration failed: {e}"))
}

fn read_token(path: &str) -> Result<String, String> {
    let token = fs::read_to_string(path)
        .map_err(|e| format!("authentication token file failed: {e}"))?
        .trim_end_matches(['\r', '\n'])
        .to_owned();
    if token.is_empty() {
        return Err("authentication token file is empty".into());
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_partial_tls_configuration_before_file_access() {
        match load(Some("missing.pem"), None, None) {
            Err(error) => assert!(error.contains("provided together")),
            Ok(_) => panic!("partial TLS configuration was accepted"),
        }
    }

    #[test]
    fn rejects_empty_authentication_token() {
        let path = std::env::temp_dir().join(format!("prism-empty-token-{}", std::process::id()));
        fs::write(&path, "\r\n").unwrap();
        assert!(load(None, None, path.to_str()).is_err());
        let _ = fs::remove_file(path);
    }
}
