use prism_bench::burst::{Calibration, Phase, PhaseSchedule};

#[test]
fn calibration_uses_median_then_80_percent_and_ten_times_burst() {
    let calibration = Calibration::from_throughputs(&[100.0, 120.0, 110.0, 90.0, 130.0]).unwrap();
    assert_eq!(calibration.median_frames_per_sec, 110.0);
    assert_eq!(calibration.baseline_frames_per_sec, 88.0);
    assert_eq!(calibration.burst_frames_per_sec, 880.0);
    assert!(calibration.baseline_interval_ns > calibration.burst_interval_ns);
}

#[test]
fn phase_schedule_is_deterministic_and_has_100k_offsets() {
    let calibration = Calibration::from_throughputs(&[100.0; 5]).unwrap();
    let first = PhaseSchedule::new(&calibration, Phase::Burst, 100_000).unwrap();
    let second = PhaseSchedule::new(&calibration, Phase::Burst, 100_000).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.frames(), 100_000);
    assert_eq!(first.arrival_offset_ns(0), 0);
    assert!(first.arrival_offset_ns(1) > first.arrival_offset_ns(0));
    assert_eq!(first.phase(), Phase::Burst);
}

#[test]
fn calibration_rejects_invalid_rates() {
    assert!(Calibration::from_throughputs(&[]).is_err());
    assert!(Calibration::from_throughputs(&[0.0; 5]).is_err());
    assert!(Calibration::from_throughputs(&[f64::NAN; 5]).is_err());
}
