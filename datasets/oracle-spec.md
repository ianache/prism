# Oracle specification

The oracle runs the same logical F1-F6 contract as a language-neutral reference. It emits canonical JSONL. Success compares integer telemetry fields, ordered sensor records, classification, severity, and route. Rejection compares stage, stable code, and required context. JSON object member order is ignored; arrays are ordered. Unexpected faults are execution_failure and never count as typed rejections.

Raw bytes, expected JSONL, and manifest digests are immutable. Reports may reference them but may not replace them.
