use prism_runtime::b0;
use prism_runtime::frame::decode_frame;
use prism_runtime::rules::evaluate_rules;
use prism_runtime::{Classification, Outcome, Route, Severity};

fn payload(ignition: u8, battery_mv: u16, speed: u16, sensor_value: Option<i32>) -> Vec<u8> {
    let mut bytes = vec![0u8; 105];
    bytes[0..2].copy_from_slice(b"PR");
    bytes[2] = 1;
    bytes[4..6].copy_from_slice(&105u16.to_le_bytes());
    bytes[6] = 1;
    bytes[7..15].copy_from_slice(&1u64.to_le_bytes());
    bytes[15..23].copy_from_slice(&1_700_000_000u64.to_le_bytes());
    bytes[31..33].copy_from_slice(&speed.to_le_bytes());
    bytes[33..35].copy_from_slice(&90u16.to_le_bytes());
    bytes[35] = ignition;
    bytes[36..38].copy_from_slice(&battery_mv.to_le_bytes());
    bytes[39..41].copy_from_slice(&60u16.to_le_bytes());
    if let Some(value) = sensor_value {
        bytes[38] = 1;
        bytes[41] = 1;
        bytes[42] = 1;
        bytes[43..47].copy_from_slice(&value.to_le_bytes());
    }
    let checksum = prism_runtime::crc32c(&bytes[..101]);
    bytes[101..105].copy_from_slice(&checksum.to_le_bytes());
    bytes
}

#[test]
fn rules_follow_frozen_first_match_order() {
    let parked = decode_frame(&payload(0, 10_000, 3_000, Some(90_000))).unwrap();
    let result = evaluate_rules(&parked);
    assert_eq!(result.classification, Classification::Parked);
    assert_eq!(result.severity, Severity::Info);
    assert_eq!(result.route, Route::Standard);
    assert_eq!(result.rule_id, "R001");

    let low_battery = decode_frame(&payload(1, 10_999, 0, None)).unwrap();
    assert_eq!(
        evaluate_rules(&low_battery).classification,
        Classification::LowBattery
    );

    let moving = decode_frame(&payload(1, 12_000, 2_778, None)).unwrap();
    assert_eq!(
        evaluate_rules(&moving).classification,
        Classification::Moving
    );

    let overheat = decode_frame(&payload(1, 12_000, 0, Some(85_000))).unwrap();
    let result = evaluate_rules(&overheat);
    assert_eq!(result.classification, Classification::Overheat);
    assert_eq!(result.severity, Severity::Critical);
    assert_eq!(result.route, Route::Quarantine);
}

#[test]
fn missing_sensor_uses_default_classification() {
    let frame = decode_frame(&payload(1, 12_000, 0, None)).unwrap();
    let result = evaluate_rules(&frame);
    assert_eq!(result.classification, Classification::Normal);
    assert_eq!(result.rule_id, "DEFAULT");
}

#[test]
fn b0_returns_normalized_and_typed_rejections() {
    match b0::process(&payload(1, 12_000, 2_778, None)) {
        Outcome::Normalized(value) => assert_eq!(value.classification, Classification::Moving),
        other => panic!("unexpected outcome: {other:?}"),
    }
    let mut invalid = payload(1, 12_000, 0, None);
    invalid[0] = 0;
    assert!(matches!(b0::process(&invalid), Outcome::Rejected(_)));
}

#[test]
fn normalized_outcome_has_canonical_json_shape() {
    let output = b0::serialize_outcome_json(&b0::process(&payload(1, 12_000, 0, None)));
    assert!(output.starts_with("{\"classification\":\"NORMAL\""));
    assert!(output.contains("\"kind\":\"normalized_telemetry\""));
    assert!(!output.contains('\n'));
}
