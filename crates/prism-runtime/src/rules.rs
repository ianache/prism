use crate::{Classification, Route, SensorSet, Severity, TelemetryFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleEvaluation {
    pub rule_id: &'static str,
    pub classification: Classification,
    pub severity: Severity,
    pub route: Route,
}

fn sensor_value(sensors: &SensorSet, id: u8) -> Option<i32> {
    sensors
        .as_slice()
        .iter()
        .find(|sensor| sensor.id == id)
        .map(|sensor| sensor.value)
}

pub fn evaluate_rules(frame: &TelemetryFrame) -> RuleEvaluation {
    if frame.ignition == 0 {
        return RuleEvaluation {
            rule_id: "R001",
            classification: Classification::Parked,
            severity: Severity::Info,
            route: Route::Standard,
        };
    }
    if frame.battery_mv < 11_000 {
        return RuleEvaluation {
            rule_id: "R002",
            classification: Classification::LowBattery,
            severity: Severity::Warn,
            route: Route::Alert,
        };
    }
    if frame.speed_cm_per_s >= 2_778 {
        return RuleEvaluation {
            rule_id: "R003",
            classification: Classification::Moving,
            severity: Severity::Info,
            route: Route::Standard,
        };
    }
    if sensor_value(&frame.sensors, 1).is_some_and(|value| value >= 85_000) {
        return RuleEvaluation {
            rule_id: "R004",
            classification: Classification::Overheat,
            severity: Severity::Critical,
            route: Route::Quarantine,
        };
    }
    RuleEvaluation {
        rule_id: "DEFAULT",
        classification: Classification::Normal,
        severity: Severity::Info,
        route: Route::Standard,
    }
}
