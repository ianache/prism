# Raw results

Store machine-readable JSONL records here. A record includes run_id, implementation, level, scenario, concurrency, payload_class, percentiles, throughput, correctness, memory metrics, metadata, commit, and dataset digest.

Existing raw output paths are immutable; the Rust harness refuses to overwrite them.

Raw JSONL is write-once evidence. Validate every line with `json.loads` before
publication, and run `scripts/audit-rust-s1.py` for the external 100K corpus.
The audit requires five repetitions for each of B0 and B1, matching dataset
identity and digest, complete protocol evidence, correctness, convergence, and
per-repetition workflow-tax arithmetic. The 100K binary fixtures remain
outside Git.

S3 external evidence: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s3.jsonl`.
SHA-256: `7BC5D384ADD138D69C83AF49684715A71BED501CBFDB60BA23BCFA99459156F4`.
The package has 105 JSONL records, five repetitions for each B0/B1/B2 and
each concurrency in `1,2,4,8,16,32,64`; it remains outside Git and is audited
with `scripts/audit-rust-s3.py`.

S4 external evidence: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s4.jsonl`.
SHA-256: `9E93A04938B826980DCC2F1A1DCB3D87F942169333EC32DCE3A79621B1582EC6`.
The package has 45 JSONL records, five repetitions for each B0/B1/B2 phase in
`baseline`, `burst`, and `recovery`; it remains outside Git and is audited
with `scripts/audit-rust-s4.py`.
