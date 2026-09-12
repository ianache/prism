# Deterministic telemetry dataset

The dataset is a language-neutral immutable input. Seed is hexadecimal 0x505249534D5F5631. Generation uses SplitMix64, emits fixtures in lexicographic fixture_id order, and writes raw payloads as exact bytes in fixtures/*.bin. The manifest and expected results are UTF-8 JSONL with LF endings.

The three payload classes are 105, 249, and 501 bytes. For any N, category quotas use largest remainder over valid=80%, edge_complex=15%, and invalid=5%, with ties resolved in that order. Payload classes rotate 105, 249, 501 within each category. Fixture IDs are `p0-{validity_class}-{index:07d}-{payload_class:03d}`, so generation can stream directly in lexicographic order without retaining the full manifest in memory.

Invalid coverage includes truncation, bad magic/version, length mismatch, range violation, unsupported protocol, and checksum failure. Edge coverage includes threshold equality, minimum/maximum legal values, absent optional sensors, and maximum sensor area.

The oracle compares logical integer fields, ordered classification/severity/route, and stable rejection code/context. It ignores object layout, pointers, JSON object order, and incidental metadata. Unexpected faults are separate execution failures.

Use docs/dataset-generation.md for the reproducible generate, oracle, and verify commands. The committed manifest is the contract; generated release artifacts are verified before use and are not benchmark outputs.
