use crate::dispatch::ObservationStats;
use prism_runtime::b0::serialize_outcome_json;
use prism_runtime::Outcome;

pub fn escape_json(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            c if c.is_control() => format!("\\u{:04x}", c as u32).chars().collect(),
            c => vec![c],
        })
        .collect()
}

pub fn success(
    request_id: &str,
    route: &str,
    outcome: &Outcome,
    stats: &ObservationStats,
) -> String {
    let observation = if route == "b2" {
        format!(",\"observations\":{{\"events\":{},\"completed\":{},\"rejected\":{},\"execution_failures\":{}}}", stats.events, stats.completed, stats.rejected, stats.execution_failures)
    } else {
        String::new()
    };
    format!(
        "{{\"request_id\":\"{}\",\"ok\":true,\"route\":\"{}\",\"outcome\":{}{}}}",
        escape_json(request_id),
        route,
        serialize_outcome_json(outcome),
        observation
    )
}

pub fn error(request_id: &str, route: &str, code: &str, message: &str) -> String {
    format!("{{\"request_id\":\"{}\",\"ok\":false,\"route\":\"{}\",\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}}}}", escape_json(request_id), route, escape_json(code), escape_json(message))
}
