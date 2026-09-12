# Generator specification

Use SplitMix64 with the recorded 64-bit seed, unsigned wraparound, and the standard increment/multiply/xor sequence. Generate fixed fields in table order, then sensor records, then CRC-32C. The generator must record the seed, generator version, payload class, validity class, and SHA-256 digest. No platform RNG, locale, wall clock, floating point, or map iteration may affect bytes.

For a requested count N, allocate valid=80%, edge_complex=15%, and invalid=5% with the largest-remainder method. Ties are resolved in the order valid, edge_complex, invalid. Within each category, payload classes rotate 105, 249, 501, starting at 105. Fixture IDs use `p0-{validity_class}-{index:07d}-{payload_class:03d}` and all output is sorted lexicographically by fixture ID. Generation streams fixture bytes, expected JSONL, and manifest entries without retaining the full fixture list in memory.

Mutation fixtures are produced from a valid frame by one named mutation only; the expected rejection precedence is the F1 order in telemetry-workflow.md.
