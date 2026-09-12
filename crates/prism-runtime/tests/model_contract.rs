use prism_runtime::{crc32c, SensorRecord, SensorSet};

#[test]
fn sensor_set_rejects_more_than_76_records() {
    let mut sensors = SensorSet::empty();
    for id in 1..=76 {
        sensors
            .push(SensorRecord {
                id,
                kind: 1,
                value: 0,
            })
            .unwrap();
    }
    assert!(sensors
        .push(SensorRecord {
            id: 77,
            kind: 1,
            value: 0
        })
        .is_err());
}

#[test]
fn crc32c_matches_ascii_vector() {
    assert_eq!(crc32c(b"123456789"), 0xe306_9283);
}
