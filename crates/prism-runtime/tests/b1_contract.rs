use prism_runtime::b0;
use prism_runtime::b1::{b1, FilterId, ObservationOutcome, Observer, Pipeline};
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
fn registry_exposes_exact_ordered_contracts() {
    let pipeline = Pipeline::new().unwrap();
    let descriptors = pipeline.descriptors();
    assert_eq!(descriptors.len(), 6);
    assert_eq!(descriptors[0].id, FilterId::F1);
    assert_eq!(descriptors[5].id, FilterId::F6);
    assert_eq!(descriptors[0].input, "raw_bytes");
    assert_eq!(descriptors[5].output, "outcome");
}

#[test]
fn b1_rejects_early_and_matches_b0() {
    let pipeline = Pipeline::new().unwrap();
    let mut invalid = payload();
    invalid[0] = 0;
    assert!(matches!(pipeline.process(&invalid), Outcome::Rejected(_)));
    assert_eq!(pipeline.process(&payload()), b0::process(&payload()));
    assert_eq!(b1(&pipeline, &payload()), b0::process(&payload()));
}

#[derive(Default)]
struct RecordingObserver {
    events: Vec<(FilterId, u128, ObservationOutcome)>,
}

impl Observer for RecordingObserver {
    fn on_filter(&mut self, filter: FilterId, elapsed_ns: u128, outcome: ObservationOutcome) {
        self.events.push((filter, elapsed_ns, outcome));
    }
}

#[test]
fn observed_pipeline_preserves_output_and_records_reached_filters() {
    let pipeline = Pipeline::new().unwrap();
    let mut observer = RecordingObserver::default();
    let observed = pipeline.process_observed(&payload(), &mut observer);

    assert_eq!(observed, pipeline.process(&payload()));
    assert_eq!(
        observer.events.iter().map(|(filter, _, _)| *filter).collect::<Vec<_>>(),
        vec![FilterId::F1, FilterId::F2, FilterId::F3, FilterId::F4, FilterId::F5, FilterId::F6]
    );
    assert!(observer.events.iter().all(|(_, _, outcome)| {
        matches!(outcome, ObservationOutcome::Completed)
    }));
}

#[test]
fn observed_pipeline_stops_after_f1_rejection() {
    let pipeline = Pipeline::new().unwrap();
    let mut invalid = payload();
    invalid[0] = 0;
    let mut observer = RecordingObserver::default();

    assert!(matches!(
        pipeline.process_observed(&invalid, &mut observer),
        Outcome::Rejected(_)
    ));
    assert_eq!(observer.events.len(), 1);
    assert_eq!(observer.events[0].0, FilterId::F1);
    assert_eq!(observer.events[0].2, ObservationOutcome::Rejected);
}
