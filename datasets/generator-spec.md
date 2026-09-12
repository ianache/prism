# Generator specification

Use SplitMix64 with the recorded 64-bit seed, unsigned wraparound, and the standard increment/multiply/xor sequence. Generate fixed fields in table order, then sensor records, then CRC-32C. The generator must record the seed, generator version, payload class, validity class, and SHA-256 digest. No platform RNG, locale, wall clock, floating point, or map iteration may affect bytes.

Mutation fixtures are produced from a valid frame by one named mutation only; the expected rejection precedence is the F1 order in telemetry-workflow.md.
