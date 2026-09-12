#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SensorRecord {
    pub id: u8,
    pub kind: u8,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorSet {
    len: u8,
    records: [SensorRecord; 76],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorSetError {
    CapacityExceeded,
}

impl SensorSet {
    pub fn empty() -> Self {
        Self {
            len: 0,
            records: [SensorRecord::default(); 76],
        }
    }

    pub fn push(&mut self, record: SensorRecord) -> Result<(), SensorSetError> {
        let index = usize::from(self.len);
        if index >= self.records.len() {
            return Err(SensorSetError::CapacityExceeded);
        }
        self.records[index] = record;
        self.len += 1;
        Ok(())
    }

    pub fn as_slice(&self) -> &[SensorRecord] {
        &self.records[..usize::from(self.len)]
    }

    pub fn len(&self) -> usize {
        usize::from(self.len)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryFrame {
    pub device_id: u64,
    pub timestamp_unix_s: u64,
    pub latitude_e7: i32,
    pub longitude_e7: i32,
    pub speed_cm_per_s: u16,
    pub heading_cdeg: u16,
    pub ignition: u8,
    pub battery_mv: u16,
    pub sensors: SensorSet,
    pub flags: u8,
    pub protocol: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Normal,
    Parked,
    LowBattery,
    Moving,
    Overheat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warn,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Standard,
    Alert,
    Quarantine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizedTelemetry {
    pub frame: TelemetryFrame,
    pub classification: Classification,
    pub severity: Severity,
    pub route: Route,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionCode {
    Truncated,
    BadMagic,
    UnsupportedVersion,
    LengthMismatch,
    RangeViolation,
    UnsupportedProtocol,
    ChecksumFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionStage {
    F1Validate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionContext {
    None,
    ActualLength {
        actual: u16,
    },
    Magic {
        actual: [u8; 2],
    },
    Version {
        actual: u8,
    },
    Protocol {
        protocol: u8,
    },
    Length {
        declared: u16,
        actual: u16,
    },
    Checksum {
        expected: u32,
        actual: u32,
    },
    Offset {
        offset: u16,
    },
    Field {
        name: &'static str,
        value: i64,
    },
    Area {
        sensor_area_len: u16,
        sensor_count: u8,
    },
    Sensor {
        sensor_id: u8,
        sensor_kind: u8,
    },
    Padding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rejection {
    pub code: RejectionCode,
    pub stage: RejectionStage,
    pub context: RejectionContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionFailure {
    pub error_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Normalized(NormalizedTelemetry),
    Rejected(Rejection),
    ExecutionFailure(ExecutionFailure),
}
