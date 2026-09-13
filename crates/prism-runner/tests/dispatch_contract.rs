use prism_runner::{dispatch, Route};
use prism_runtime::Outcome;

fn payload() -> Vec<u8> {
    let mut bytes = vec![0u8; 105];
    bytes[0..2].copy_from_slice(b"PR");
    bytes[2] = 1;
    bytes[4..6].copy_from_slice(&105u16.to_le_bytes());
    bytes[6] = 1;
    bytes[7..15].copy_from_slice(&1u64.to_le_bytes());
    bytes[15..23].copy_from_slice(&1_700_000_000u64.to_le_bytes());
    bytes[35] = 1;
    bytes[36..38].copy_from_slice(&12_000u16.to_le_bytes());
    bytes[39..41].copy_from_slice(&60u16.to_le_bytes());
    let checksum = prism_runtime::crc32c(&bytes[..101]);
    bytes[101..105].copy_from_slice(&checksum.to_le_bytes());
    bytes
}

#[test]
fn routes_preserve_outcome_and_b2_observes() {
    let p = payload();
    let b0 = dispatch::run(Route::B0, &p).unwrap();
    let b1 = dispatch::run(Route::B1, &p).unwrap();
    let b2 = dispatch::run(Route::B2, &p).unwrap();
    assert!(matches!(b0.0, Outcome::Normalized(_)));
    assert_eq!(b0.0, b1.0);
    assert_eq!(b1.0, b2.0);
    assert_eq!(b2.1.events, 6);
}
