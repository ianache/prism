use crate::frame::decode_frame;
use crate::rules::evaluate_rules;
use crate::{
    Classification, ExecutionFailure, NormalizedTelemetry, Outcome, Rejection, Route, Severity,
};

pub fn process(payload: &[u8]) -> Outcome {
    let frame = match decode_frame(payload) {
        Ok(frame) => frame,
        Err(rejection) => return Outcome::Rejected(rejection),
    };
    let rules = evaluate_rules(&frame);
    Outcome::Normalized(NormalizedTelemetry {
        frame,
        classification: rules.classification,
        severity: rules.severity,
        route: rules.route,
    })
}

fn classification_name(value: Classification) -> &'static str {
    match value {
        Classification::Normal => "NORMAL",
        Classification::Parked => "PARKED",
        Classification::LowBattery => "LOW_BATTERY",
        Classification::Moving => "MOVING",
        Classification::Overheat => "OVERHEAT",
    }
}

fn severity_name(value: Severity) -> &'static str {
    match value {
        Severity::Info => "INFO",
        Severity::Warn => "WARN",
        Severity::Critical => "CRITICAL",
    }
}

fn route_name(value: Route) -> &'static str {
    match value {
        Route::Standard => "STANDARD",
        Route::Alert => "ALERT",
        Route::Quarantine => "QUARANTINE",
    }
}

fn serialize_normalized(value: &NormalizedTelemetry) -> String {
    let frame = &value.frame;
    let sensors = frame
        .sensors
        .as_slice()
        .iter()
        .map(|sensor| {
            format!(
                "{{\"id\":{},\"kind\":{},\"value\":{}}}",
                sensor.id, sensor.kind, sensor.value
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"classification\":\"{}\",\"device_id\":{},\"heading_cdeg\":{},\"ignition\":{},\"kind\":\"normalized_telemetry\",\"latitude_e7\":{},\"longitude_e7\":{},\"route\":\"{}\",\"schema_version\":\"1.0\",\"sensors\":[{}],\"severity\":\"{}\",\"speed_cm_per_s\":{},\"timestamp_unix_s\":{}}}",
        classification_name(value.classification),
        frame.device_id,
        frame.heading_cdeg,
        frame.ignition,
        frame.latitude_e7,
        frame.longitude_e7,
        route_name(value.route),
        sensors,
        severity_name(value.severity),
        frame.speed_cm_per_s,
        frame.timestamp_unix_s,
    )
}

fn serialize_rejection(value: &Rejection) -> String {
    let context = match value.context {
        crate::RejectionContext::None => "{}".to_string(),
        crate::RejectionContext::Length { declared, actual } => {
            format!(
                "{{\"actual_length\":{},\"declared_length\":{}}}",
                actual, declared
            )
        }
        crate::RejectionContext::Checksum { expected, actual } => {
            format!(
                "{{\"actual_crc32c\":{},\"expected_crc32c\":{}}}",
                actual, expected
            )
        }
        crate::RejectionContext::Offset { offset } => format!("{{\"offset\":{}}}", offset),
    };
    format!(
        "{{\"code\":\"{:?}\",\"context\":{},\"kind\":\"rejection\",\"schema_version\":\"1.0\",\"stage\":\"F1_VALIDATE\"}}",
        value.code, context
    )
}

pub fn serialize_outcome_json(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Normalized(value) => serialize_normalized(value),
        Outcome::Rejected(value) => serialize_rejection(value),
        Outcome::ExecutionFailure(ExecutionFailure { error_type }) => format!(
            "{{\"error_type\":\"{}\",\"kind\":\"execution_failure\",\"schema_version\":\"1.0\"}}",
            error_type
        ),
    }
}
