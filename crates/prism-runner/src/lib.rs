pub mod cli;
pub mod dispatch;
pub mod http;
pub mod lifecycle;
pub mod output;
pub mod protocol;
pub mod security;
pub mod transport;

pub use cli::{parse_args, usage, Args, CliError, Protocol, Route};

pub fn process_line(route: Route, prefix: &str, sequence: usize, line: &str) -> String {
    process_line_with_auth(route, prefix, sequence, line, None)
}

pub fn process_line_with_auth(
    route: Route,
    prefix: &str,
    sequence: usize,
    line: &str,
    expected_token: Option<&str>,
) -> String {
    let fallback_id = format!("{}-{}", prefix, sequence);
    let envelope = match protocol::parse_line(line) {
        Ok(envelope) => envelope,
        Err(error) => {
            return output::error(&fallback_id, route.as_str(), error.code, &error.message)
        }
    };
    let request_id = if envelope.request_id.is_empty() {
        fallback_id
    } else {
        format!("{}-{}", prefix, envelope.request_id)
    };
    if let Some(expected) = expected_token {
        if envelope.auth_token.as_deref() != Some(expected) {
            return output::error(
                &request_id,
                route.as_str(),
                "AUTHENTICATION_FAILED",
                "authentication failed",
            );
        }
    }
    match dispatch::run(route, &envelope.payload) {
        Ok((outcome, stats)) => output::success(&request_id, route.as_str(), &outcome, &stats),
        Err(message) => output::error(&request_id, route.as_str(), "RUNTIME_FAILURE", &message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authentication_failure_is_in_band_and_does_not_execute_runtime() {
        let response = process_line_with_auth(
            Route::B0,
            "req",
            1,
            r#"{"request_id":"one","payload_hex":"00"}"#,
            Some("secret"),
        );
        assert!(response.contains("AUTHENTICATION_FAILED"));
        assert!(response.contains("req-one"));
        assert!(!response.contains("secret"));
    }
}
