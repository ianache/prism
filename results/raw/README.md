# Raw results

Store machine-readable JSONL records here. A record includes run_id, implementation, level, scenario, concurrency, payload_class, percentiles, throughput, correctness, memory metrics, metadata, commit, and dataset digest.

Existing raw output paths are immutable; the Rust harness refuses to overwrite them.

Raw JSONL is write-once evidence. Validate every line with `json.loads` before
publication, and run `scripts/audit-rust-s1.py` for the external 100K corpus.
The audit requires five repetitions for each of B0 and B1, matching dataset
identity and digest, complete protocol evidence, correctness, convergence, and
per-repetition workflow-tax arithmetic. The 100K binary fixtures remain
outside Git.
