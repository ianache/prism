# Rust S5 Evidence Report

Raw path: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s6-root-cause.jsonl`
Raw SHA-256: `e7c8be5561f773d474b7dc0938e57c88868792226ffb1254d4c80798c3e08f7e`
Audit output: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s6-root-cause-audit.txt`

## Audit

ok levels=b0,b1,b2 repetitions=5 frames=100000 windows=549 gate_failures=p99:b0:r1,p99:b0:r3,p99:b0:r4,p99:b1:r2,p99:b1:r5,p99:b2:r1

## Summary

Rows: 549
Complete windows per tuple: 9
Maximum p99 change: 480.952381%
Maximum RSS change: 0.882947%

## Gates

| Gate | Status | Detail |
|---|---|---|
| Correctness | PASS | all windows match |
| p99 stability | FAIL | threshold <= 10% |
| RSS growth | PASS | threshold < 10% |

## Diagnostics

Timestamp diagnostics: PRESENT
Window boundary ordering: PASS
CPU telemetry: PRESENT
Process CPU/wall ratio range: 39.045347%–101.707787%

| Level | Rep | Windows | First p99 (ns) | Last p99 (ns) | First-to-last change |
|---|---:|---:|---:|---:|---:|
| b0 | 1 | 51 | 10800 | 20600 | 90.740741% |
| b0 | 2 | 58 | 12400 | 6600 | -46.774194% |
| b0 | 3 | 50 | 5800 | 6400 | 10.344828% |
| b0 | 4 | 66 | 5700 | 6800 | 19.298246% |
| b0 | 5 | 46 | 6300 | 6300 | 0.000000% |
| b1 | 1 | 45 | 9800 | 9000 | -8.163265% |
| b1 | 2 | 53 | 8700 | 10400 | 19.540230% |
| b1 | 3 | 28 | 10500 | 10800 | 2.857143% |
| b1 | 4 | 46 | 10300 | 7200 | -30.097087% |
| b1 | 5 | 38 | 7100 | 12400 | 74.647887% |
| b2 | 1 | 10 | 21000 | 122000 | 480.952381% |
| b2 | 2 | 9 | 118500 | 117200 | -1.097046% |
| b2 | 3 | 10 | 78000 | 51700 | -33.717949% |
| b2 | 4 | 17 | 74600 | 41800 | -43.967828% |
| b2 | 5 | 22 | 38900 | 34700 | -10.796915% |

## Hypotheses

| Hypothesis | Evidence | Conclusion |
|---|---|---|
| Scheduler noise / frequency drift | p99 changes persist across matched windows while RSS remains below 10% | UNSOLVED; requires controlled host telemetry |
| Resource growth | RSS gate and bounded growth | NOT SUPPORTED as primary cause |
| Orchestration artifact | Unequal tails are reported without dropping common windows | POSSIBLE CONTRIBUTOR; not proven |

## Limitations

The result is single-host engineering evidence and is not an automatic P0 qualification. Thermal drift, scheduler noise, frequency changes, and unavailable platform metrics remain threats to validity.
