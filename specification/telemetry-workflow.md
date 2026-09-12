# P0 Telemetry Workflow Specification

**Status:** Frozen v1 specification; runtime implementation gate remains closed pending review.

## Scope and canonical path

This is a specification-only artifact. The timed path is resident BinaryFrame -> F1 Validate -> F2 Decode -> F3 Normalize -> F4 Evaluate Rules -> F5 Classify -> F6 Route -> NormalizedTelemetry or Rejection. Correctness against the independent oracle is a hard prerequisite for performance qualification.

## Canonical binary frame

Offsets are zero-based, widths are bytes, and all multi-byte values are little-endian. Version 1 has a 41-byte fixed prefix, a sensor area of 0..456 bytes in 6-byte records, and a 4-byte checksum. payload_length is the total frame length 45 + sensor_area_len. Payload classes are exactly 105, 249, and 501 bytes (sensor_area_len 60, 204, and 456).

| Offset | Width | Field | Encoding | Unit | Allowed range | Required | Checksum coverage |
| ---: | ---: | --- | --- | --- | --- | --- | --- |
| 0 | 2 | magic | ASCII PR (50 52) | n/a | exact | yes | included |
| 2 | 1 | version | uint8 | n/a | 1 | yes | included |
| 3 | 1 | flags | uint8 | bit flags; reserved bits zero | 0..255 | yes | included |
| 4 | 2 | payload_length | uint16 LE | bytes | 45..65535 | yes | included |
| 6 | 1 | protocol | uint8 | protocol id | 1 | yes | included |
| 7 | 8 | device_id | uint64 LE | numeric id | 1..999999999999999 | yes | included |
| 15 | 8 | timestamp | uint64 LE | Unix seconds UTC | 0..4102444800 | yes | included |
| 23 | 4 | latitude | int32 LE | degrees x 10^7 | -900000000..900000000 | yes | included |
| 27 | 4 | longitude | int32 LE | degrees x 10^7 | -1800000000..1800000000 | yes | included |
| 31 | 2 | speed | uint16 LE | centimetres/second | 0..50000 | yes | included |
| 33 | 2 | heading | uint16 LE | centidegrees | 0..35999 | yes | included |
| 35 | 1 | ignition | uint8 | boolean | 0..1 | yes | included |
| 36 | 2 | battery | uint16 LE | millivolts | 0..60000 | yes | included |
| 38 | 1 | sensor_count | uint8 | records | 0..76 | yes | included |
| 39 | 2 | sensor_area_len | uint16 LE | bytes | 0..456, multiple of 6 | yes | included |
| 41 | variable | sensor_area | records: id uint8, kind uint8, value int32 LE | kind-specific fixed point | ids strictly increasing | optional | included |
| 41 + sensor_area_len | 4 | checksum | CRC-32C uint32 LE | n/a | exact computed value | yes | excluded |

CRC-32C uses reflected polynomial 0x82F63B78, initial value 0xFFFFFFFF, final XOR 0xFFFFFFFF, and covers offsets 0..40+sensor_area_len. Sensor kind 1 is temperature in milli-degrees Celsius, kind 2 is fuel in milli-percent, and kind 3 is integer IO units. Unknown kinds, duplicate/out-of-order ids, and a count/length mismatch are RANGE_VIOLATION.

Coordinates are signed integers in degrees x 10^7; normalization is exact integer division with no rounding. Speed is centimetres/second and heading is centidegrees. Canonical comparisons use wire integers and never floating point.

## Data contracts

The schemas are in contracts/data/. They use JSON Schema Draft 2020-12, require schema_version 1.0, and reject undeclared properties. Contract validation happens at control-plane/load time, never per invocation.

Valid output:

{"kind":"normalized_telemetry","schema_version":"1.0","device_id":123456789012345,"timestamp_unix_s":1700000000,"latitude_e7":-120000000,"longitude_e7":-770000000,"speed_cm_per_s":1250,"heading_cdeg":9000,"ignition":1,"battery_mv":12400,"sensors":[],"classification":"MOVING","severity":"INFO","route":"STANDARD"}

Invalid output:

{"kind":"rejection","schema_version":"1.0","code":"CHECKSUM_FAILURE","stage":"F1_VALIDATE","context":{"expected_crc32c":123,"actual_crc32c":456}}

## Filter semantics

| Filter | Input -> output | Errors/capabilities |
| --- | --- | --- |
| F1 Validate | bytes -> ValidatedFrame or Rejection | stops on typed rejection; no I/O |
| F2 Decode | ValidatedFrame -> DecodedTelemetry | deterministic, no I/O |
| F3 Normalize | DecodedTelemetry -> NormalizedTelemetryInput | integer unit normalization; no floating point |
| F4 Evaluate Rules | NormalizedTelemetryInput -> RuleEvaluation | ordered rule table; integer comparisons only |
| F5 Classify | RuleEvaluation -> Classification | deterministic first-match label/severity |
| F6 Route | Classification -> route enum | no network, broker, DB, HTTP, LLM, or human interaction |

F1 validation order is truncation, magic, version, length, protocol, ranges, then checksum. Stable codes are TRUNCATED, BAD_MAGIC, UNSUPPORTED_VERSION, LENGTH_MISMATCH, RANGE_VIOLATION, UNSUPPORTED_PROTOCOL, and CHECKSUM_FAILURE. Unexpected faults are execution failures, not typed rejections.

## Workflow, QoS, and versioning

The workflow schema requires identity prism.telemetry.p0, version 1.0, exactly ordered nodes F1..F6, and profile P0. All contracts use Draft 2020-12 and local $ref resolution. Exact-major compatibility is allowed; unsupported versions or unresolved references are rejected before timing.

P0 is in-process, bounded-memory, minimal-allocation, and no-network. The QoS contract records p50/p95/p99 targets and the provisional B1 p99 workflow-tax limit of 20 percent.

## Cross-artifact links

- Frame/data: this document and contracts/data/.
- Filters/workflow/rules: this document, contracts/filter/, contracts/workflow/, and rule-table.json.
- Dataset/oracle: datasets/README.md.
- Timing/scenarios/metrics: benchmark-protocol.md.
- Gate evidence: acceptance-criteria.md.
