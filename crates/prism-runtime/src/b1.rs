use crate::frame::decode_frame;
use crate::rules::evaluate_rules;
use crate::{NormalizedTelemetry, Outcome};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterId {
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationOutcome {
    Completed,
    Rejected,
    ExecutionFailure,
}

pub trait Observer {
    fn on_filter(&mut self, filter: FilterId, elapsed_ns: u128, outcome: ObservationOutcome);
}

struct NoopObserver;

impl Observer for NoopObserver {
    fn on_filter(&mut self, _filter: FilterId, _elapsed_ns: u128, _outcome: ObservationOutcome) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterDescriptor {
    pub id: FilterId,
    pub input: &'static str,
    pub output: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryError;

const DESCRIPTORS: [FilterDescriptor; 6] = [
    FilterDescriptor {
        id: FilterId::F1,
        input: "raw_bytes",
        output: "validated_frame",
    },
    FilterDescriptor {
        id: FilterId::F2,
        input: "validated_frame",
        output: "classified_frame",
    },
    FilterDescriptor {
        id: FilterId::F3,
        input: "classified_frame",
        output: "severity_frame",
    },
    FilterDescriptor {
        id: FilterId::F4,
        input: "severity_frame",
        output: "routed_frame",
    },
    FilterDescriptor {
        id: FilterId::F5,
        input: "routed_frame",
        output: "normalized_telemetry",
    },
    FilterDescriptor {
        id: FilterId::F6,
        input: "normalized_telemetry",
        output: "outcome",
    },
];

pub struct Pipeline {
    descriptors: &'static [FilterDescriptor; 6],
}

impl Pipeline {
    pub fn new() -> Result<Self, RegistryError> {
        let valid = DESCRIPTORS
            .iter()
            .enumerate()
            .all(|(index, descriptor)| descriptor.id as usize == index)
            && DESCRIPTORS[0].input == "raw_bytes"
            && DESCRIPTORS[5].output == "outcome";
        if valid {
            Ok(Self {
                descriptors: &DESCRIPTORS,
            })
        } else {
            Err(RegistryError)
        }
    }

    pub fn descriptors(&self) -> &'static [FilterDescriptor; 6] {
        self.descriptors
    }

    pub fn process(&self, payload: &[u8]) -> Outcome {
        let mut observer = NoopObserver;
        self.process_observed(payload, &mut observer)
    }

    pub fn process_observed<O: Observer>(&self, payload: &[u8], observer: &mut O) -> Outcome {
        let started = Instant::now();
        let frame = match decode_frame(payload) {
            Ok(frame) => frame,
            Err(rejection) => {
                observer.on_filter(
                    FilterId::F1,
                    started.elapsed().as_nanos(),
                    ObservationOutcome::Rejected,
                );
                return Outcome::Rejected(rejection);
            }
        };

        observer.on_filter(
            FilterId::F1,
            started.elapsed().as_nanos(),
            ObservationOutcome::Completed,
        );
        let started = Instant::now();
        let frame = frame;
        observer.on_filter(
            FilterId::F2,
            started.elapsed().as_nanos(),
            ObservationOutcome::Completed,
        );
        let started = Instant::now();
        let frame = frame;
        observer.on_filter(
            FilterId::F3,
            started.elapsed().as_nanos(),
            ObservationOutcome::Completed,
        );
        let started = Instant::now();
        let evaluation = evaluate_rules(&frame);
        observer.on_filter(
            FilterId::F4,
            started.elapsed().as_nanos(),
            ObservationOutcome::Completed,
        );
        let started = Instant::now();
        let classification = evaluation.classification;
        observer.on_filter(
            FilterId::F5,
            started.elapsed().as_nanos(),
            ObservationOutcome::Completed,
        );
        let started = Instant::now();
        let route = evaluation.route;
        observer.on_filter(
            FilterId::F6,
            started.elapsed().as_nanos(),
            ObservationOutcome::Completed,
        );
        Outcome::Normalized(NormalizedTelemetry {
            frame,
            classification,
            severity: evaluation.severity,
            route,
        })
    }
}

pub fn b1(pipeline: &Pipeline, payload: &[u8]) -> Outcome {
    pipeline.process(payload)
}
