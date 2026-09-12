use prism_runtime::frame::{decode_frame, payload_class};
use prism_runtime::{RejectionCode, RejectionStage};

fn valid_payload() -> Vec<u8> {
    let mut payload = vec![0u8; 105];
    payload[0..2].copy_from_slice(b"PR");
    payload[2] = 1;
    payload[4..6].copy_from_slice(&105u16.to_le_bytes());
    payload[6] = 1;
    payload[7..15].copy_from_slice(&1u64.to_le_bytes());
    payload[15..23].copy_from_slice(&1_700_000_000u64.to_le_bytes());
    payload[31..33].copy_from_slice(&100u16.to_le_bytes());
    payload[33..35].copy_from_slice(&90u16.to_le_bytes());
    payload[36..38].copy_from_slice(&12_000u16.to_le_bytes());
    payload[38] = 1;
    payload[39..41].copy_from_slice(&60u16.to_le_bytes());
    payload[41] = 1;
    payload[42] = 1;
    payload[43..47].copy_from_slice(&42i32.to_le_bytes());
    let checksum = prism_runtime::crc32c(&payload[..101]);
    payload[101..105].copy_from_slice(&checksum.to_le_bytes());
    payload
}

fn rejection(payload: &[u8]) -> RejectionCode {
    decode_frame(payload).unwrap_err().code
}

#[test]
fn payload_classes_are_exact() {
    assert_eq!(payload_class(105), Some(105));
    assert_eq!(payload_class(249), Some(249));
    assert_eq!(payload_class(501), Some(501));
    assert_eq!(payload_class(104), None);
}

#[test]
fn valid_frame_decodes_with_sensor_data() {
    let frame = decode_frame(&valid_payload()).unwrap();
    assert_eq!(frame.device_id, 1);
    assert_eq!(frame.sensors.as_slice()[0].value, 42);
    assert_eq!(frame.sensors.len(), 1);
}

#[test]
fn f1_rejection_precedence_is_stable() {
    let mut payload = valid_payload();
    payload[0] = 0;
    payload[2] = 2;
    assert_eq!(rejection(&payload), RejectionCode::BadMagic);

    let mut payload = valid_payload();
    payload[2] = 2;
    assert_eq!(rejection(&payload), RejectionCode::UnsupportedVersion);

    let mut payload = valid_payload();
    payload[4..6].copy_from_slice(&104u16.to_le_bytes());
    payload[6] = 2;
    assert_eq!(rejection(&payload), RejectionCode::LengthMismatch);

    let mut payload = valid_payload();
    payload[6] = 2;
    assert_eq!(rejection(&payload), RejectionCode::UnsupportedProtocol);

    let mut payload = valid_payload();
    payload[35] = 2;
    payload[104] ^= 0xff;
    assert_eq!(rejection(&payload), RejectionCode::RangeViolation);

    let mut payload = valid_payload();
    payload[104] ^= 0xff;
    assert_eq!(rejection(&payload), RejectionCode::ChecksumFailure);
}

#[test]
fn truncated_input_is_rejected_at_f1() {
    let payload = valid_payload();
    let rejection = decode_frame(&payload[..104]).unwrap_err();
    assert_eq!(rejection.code, RejectionCode::Truncated);
    assert_eq!(rejection.stage, RejectionStage::F1Validate);
}
