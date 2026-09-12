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
        "{{\"battery_mv\":{},\"classification\":\"{}\",\"device_id\":{},\"heading_cdeg\":{},\"ignition\":{},\"kind\":\"normalized_telemetry\",\"latitude_e7\":{},\"longitude_e7\":{},\"route\":\"{}\",\"schema_version\":\"1.0\",\"sensors\":[{}],\"severity\":\"{}\",\"speed_cm_per_s\":{},\"timestamp_unix_s\":{}}}",
        frame.battery_mv,
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
    let code = match value.code {
        crate::RejectionCode::Truncated => "TRUNCATED",
        crate::RejectionCode::BadMagic => "BAD_MAGIC",
        crate::RejectionCode::UnsupportedVersion => "UNSUPPORTED_VERSION",
        crate::RejectionCode::LengthMismatch => "LENGTH_MISMATCH",
        crate::RejectionCode::RangeViolation => "RANGE_VIOLATION",
        crate::RejectionCode::UnsupportedProtocol => "UNSUPPORTED_PROTOCOL",
        crate::RejectionCode::ChecksumFailure => "CHECKSUM_FAILURE",
    };
    let context = match value.context {
        crate::RejectionContext::None => "{}".to_string(),
        crate::RejectionContext::ActualLength { actual } => {
            format!("{{\"actual_length\":{}}}", actual)
        }
        crate::RejectionContext::Magic { actual } => format!(
            "{{\"actual_magic\":\"{:02x}{:02x}\"}}",
            actual[0], actual[1]
        ),
        crate::RejectionContext::Version { actual } => format!("{{\"actual_version\":{}}}", actual),
        crate::RejectionContext::Protocol { protocol } => format!("{{\"protocol\":{}}}", protocol),
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
        crate::RejectionContext::Field { name, value } => format!("{{\"{}\":{}}}", name, value),
        crate::RejectionContext::Area {
            sensor_area_len,
            sensor_count,
        } => format!(
            "{{\"sensor_area_len\":{},\"sensor_count\":{}}}",
            sensor_area_len, sensor_count
        ),
        crate::RejectionContext::Sensor {
            sensor_id,
            sensor_kind,
        } => format!(
            "{{\"sensor_id\":{},\"sensor_kind\":{}}}",
            sensor_id, sensor_kind
        ),
        crate::RejectionContext::Padding => "{\"padding\":\"nonzero\"}".to_string(),
    };
    format!(
        "{{\"code\":\"{}\",\"context\":{},\"kind\":\"rejection\",\"schema_version\":\"1.0\",\"stage\":\"F1_VALIDATE\"}}",
        code, context
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
