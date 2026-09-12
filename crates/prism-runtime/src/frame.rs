use crate::{
    crc32c, Rejection, RejectionCode, RejectionContext, RejectionStage, SensorRecord, SensorSet,
    TelemetryFrame,
};

const MAGIC: [u8; 2] = *b"PR";
const VERSION: u8 = 1;
const HEADER_SIZE: usize = 41;
const CHECKSUM_SIZE: usize = 4;
const SENSOR_KINDS: [u8; 3] = [1, 2, 3];

pub fn payload_class(payload_length: usize) -> Option<u16> {
    match payload_length {
        105 | 249 | 501 => Some(payload_length as u16),
        _ => None,
    }
}

fn reject(code: RejectionCode, context: RejectionContext) -> Rejection {
    Rejection {
        code,
        stage: RejectionStage::F1Validate,
        context,
    }
}

fn read_u16(payload: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([payload[offset], payload[offset + 1]])
}

fn read_u32(payload: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap())
}

fn read_u64(payload: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap())
}

fn read_i32(payload: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap())
}

pub fn validate_ranges(frame: &TelemetryFrame) -> Result<(), Rejection> {
    let valid = frame.device_id >= 1
        && frame.device_id <= 999_999_999_999_999
        && frame.timestamp_unix_s <= 4_102_444_800
        && (-900_000_000..=900_000_000).contains(&frame.latitude_e7)
        && (-1_800_000_000..=1_800_000_000).contains(&frame.longitude_e7)
        && frame.speed_cm_per_s <= 50_000
        && frame.heading_cdeg <= 35_999
        && frame.ignition <= 1
        && frame.battery_mv <= 60_000
        && frame.protocol == 1;
    if !valid {
        return Err(reject(
            RejectionCode::RangeViolation,
            RejectionContext::None,
        ));
    }
    Ok(())
}

pub fn decode_frame(payload: &[u8]) -> Result<TelemetryFrame, Rejection> {
    if payload.len() < 105 {
        return Err(reject(
            RejectionCode::Truncated,
            RejectionContext::Offset {
                offset: payload.len() as u16,
            },
        ));
    }
    if payload[0..2] != MAGIC {
        return Err(reject(
            RejectionCode::BadMagic,
            RejectionContext::Offset { offset: 0 },
        ));
    }
    if payload[2] != VERSION {
        return Err(reject(
            RejectionCode::UnsupportedVersion,
            RejectionContext::Offset { offset: 2 },
        ));
    }

    let declared_length = read_u16(payload, 4) as usize;
    if declared_length != payload.len() || payload_class(payload.len()).is_none() {
        return Err(reject(
            RejectionCode::LengthMismatch,
            RejectionContext::Length {
                declared: declared_length as u16,
                actual: payload.len() as u16,
            },
        ));
    }
    if payload[6] != 1 {
        return Err(reject(
            RejectionCode::UnsupportedProtocol,
            RejectionContext::Offset { offset: 6 },
        ));
    }

    let sensor_count = payload[38] as usize;
    let area_len = read_u16(payload, 39) as usize;
    let expected_area_len = payload.len() - HEADER_SIZE - CHECKSUM_SIZE;
    if area_len != expected_area_len || area_len % 6 != 0 || sensor_count > area_len / 6 {
        return Err(reject(
            RejectionCode::RangeViolation,
            RejectionContext::Offset { offset: 39 },
        ));
    }

    let mut sensors = SensorSet::empty();
    let mut previous_id = None;
    for index in 0..sensor_count {
        let offset = HEADER_SIZE + index * 6;
        let sensor_id = payload[offset];
        let sensor_kind = payload[offset + 1];
        if previous_id.is_some_and(|previous| sensor_id <= previous)
            || !SENSOR_KINDS.contains(&sensor_kind)
        {
            return Err(reject(
                RejectionCode::RangeViolation,
                RejectionContext::Offset {
                    offset: offset as u16,
                },
            ));
        }
        sensors
            .push(SensorRecord {
                id: sensor_id,
                kind: sensor_kind,
                value: read_i32(payload, offset + 2),
            })
            .unwrap();
        previous_id = Some(sensor_id);
    }
    let padding_start = HEADER_SIZE + sensor_count * 6;
    if payload[padding_start..HEADER_SIZE + area_len]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(reject(
            RejectionCode::RangeViolation,
            RejectionContext::Offset {
                offset: padding_start as u16,
            },
        ));
    }

    let frame = TelemetryFrame {
        device_id: read_u64(payload, 7),
        timestamp_unix_s: read_u64(payload, 15),
        latitude_e7: read_i32(payload, 23),
        longitude_e7: read_i32(payload, 27),
        speed_cm_per_s: read_u16(payload, 31),
        heading_cdeg: read_u16(payload, 33),
        ignition: payload[35],
        battery_mv: read_u16(payload, 36),
        sensors,
        flags: payload[3],
        protocol: payload[6],
    };
    validate_ranges(&frame)?;

    let expected = read_u32(payload, payload.len() - CHECKSUM_SIZE);
    let actual = crc32c(&payload[..payload.len() - CHECKSUM_SIZE]);
    if expected != actual {
        return Err(reject(
            RejectionCode::ChecksumFailure,
            RejectionContext::Checksum { expected, actual },
        ));
    }
    Ok(frame)
}
