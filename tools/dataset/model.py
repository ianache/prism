from dataclasses import dataclass


@dataclass(frozen=True)
class SensorRecord:
    id: int
    kind: int
    value: int


@dataclass(frozen=True)
class TelemetryFrame:
    device_id: int
    timestamp_unix_s: int
    latitude_e7: int
    longitude_e7: int
    speed_cm_per_s: int
    heading_cdeg: int
    ignition: int
    battery_mv: int
    sensors: tuple[SensorRecord, ...]
    flags: int = 0
    protocol: int = 1


@dataclass(frozen=True)
class Rejection:
    code: str
    stage: str
    context: dict[str, int | str]

    def as_dict(self) -> dict[str, object]:
        return {
            "kind": "rejection",
            "schema_version": "1.0",
            "code": self.code,
            "stage": self.stage,
            "context": self.context,
        }
