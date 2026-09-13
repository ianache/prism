# Benchmark results

Results are generated from immutable raw records. Hand-edited benchmark values are forbidden. Every run must include correctness counts, dataset digest, protocol version, and complete metadata.

Rust S1 output is evidence only and is not a P0 qualification.

The raw S1 contract is self-describing: each record carries the dataset ID,
fixture and workload counts, UTC timestamp, exact invocation command, protocol
parameters, convergence result, timing percentiles, correctness and failure
counts, workflow tax, and host metadata (`N/D` when unavailable).
