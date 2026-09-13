# Decision matrix

The decision matrix records evidence for P0 runtime, P0 data representation, filter invocation, observability, P1 transport, and execution-profile compatibility. It is generated from raw results and never hand-edited. B2 decisions use the v1.1 100,000-frame baseline and must retain correctness and per-repetition observability tax evidence.

S3 adds a Rust-only concurrency dimension with 105 externally audited records and matched B0/B1/B2 p99 taxes for every concurrency and repetition. This is engineering evidence, not a P0 qualification or a cross-language decision.

S4 adds Rust burst evidence with 45 externally audited records across baseline, burst, and recovery. The selected synchronous design reports offered-rate pressure and lateness; it does not evaluate queue depth or backpressure.
