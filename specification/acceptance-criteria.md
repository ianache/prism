# Telemetry Acceptance Criteria

**Status:** Provisional P0/P1 gates frozen for the spike; comparable decisions use the v1.1 100,000-frame baseline.

Every B2 and later comparable decision uses 100,000 measured frames per
repetition, five repetitions, and the frozen warm-up/convergence protocol.
Existing 10,000-frame S1 records are historical evidence and are not
reinterpreted as 100,000-frame decisions.

All gates use the matched warmed runs and timer defined in benchmark-protocol.md. Correctness is a hard prerequisite: any oracle mismatch disqualifies performance qualification.

| Profile | Gate | Threshold | Evidence |
| --- | --- | --- | --- |
| P0 | p50 | <= 250 microseconds | primary-timer p50 |
| P0 | p95 | <= 500 microseconds | primary-timer p95 |
| P0 | p99 | <= 1 millisecond | primary-timer p99 |
| P0 | correctness | 100 percent | oracle comparison over every fixture |
| P0 | memory | bounded; <10 percent RSS growth in S5 | RSS/heap/allocation series |
| P0 | stability | <=10 percent p99 increase first-to-last S5 interval | S5 p99 trend |
| P0 | workflow tax | <=20 percent at matched p99 | B0/B1 tax formula |
| P1 | p50/p95/p99 | <=10/25/50 milliseconds | B3 transport runs |
| P1 | reliability | no loss under declared semantics | loss/recovery records |
| P1 | backpressure | bounded queue and explicit backpressure | S3/S4 evidence |

A candidate passes P0 only when every P0 row passes in every measured repetition after correctness passes. P1 is future transport evaluation; it does not authorize a broker or runtime implementation. Thresholds are provisional and can change only through a versioned review decision.
