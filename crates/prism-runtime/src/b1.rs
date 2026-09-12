use crate::frame::decode_frame;
use crate::rules::evaluate_rules;
use crate::{NormalizedTelemetry, Outcome};

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
        let frame = match decode_frame(payload) {
            Ok(frame) => frame,
            Err(rejection) => return Outcome::Rejected(rejection),
        };
        let evaluation = evaluate_rules(&frame);
        Outcome::Normalized(NormalizedTelemetry {
            frame,
            classification: evaluation.classification,
            severity: evaluation.severity,
            route: evaluation.route,
        })
    }
}

pub fn b1(pipeline: &Pipeline, payload: &[u8]) -> Outcome {
    pipeline.process(payload)
}
