pub mod cli;
pub mod dispatch;
pub mod output;
pub mod protocol;
pub mod transport;

pub use cli::{parse_args, usage, Args, CliError, Route};

pub fn process_line(route: Route, prefix: &str, sequence: usize, line: &str) -> String {
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
    match dispatch::run(route, &envelope.payload) {
        Ok((outcome, stats)) => output::success(&request_id, route.as_str(), &outcome, &stats),
        Err(message) => output::error(&request_id, route.as_str(), "RUNTIME_FAILURE", &message),
    }
}
