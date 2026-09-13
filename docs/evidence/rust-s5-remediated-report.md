# Rust S5 Evidence Report

Raw path: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-remediated.jsonl`
Raw SHA-256: `0dbda7ccab17142529b51e99fac7291ce394b93c8d4125bd783d1a87a66f5418`
Audit output: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-remediated-audit.txt`

## Audit

ok levels=b0,b1,b2 repetitions=5 frames=100000 windows=615 gate_failures=p99:b0:r1,p99:b0:r3,p99:b0:r4,p99:b1:r1,p99:b1:r3,p99:b1:r4,p99:b2:r5

## Summary

Rows: 615
Complete windows per tuple: 27
Maximum p99 change: 127.956989%
Maximum RSS change: 0.878967%

## Gates

| Gate | Status | Detail |
|---|---|---|
| Correctness | PASS | all windows match |
| p99 stability | FAIL | threshold <= 10% |
| RSS growth | PASS | threshold < 10% |

## Diagnostics

Timestamp diagnostics: PRESENT
Window boundary ordering: PASS

| Level | Rep | Windows | First p99 (ns) | Last p99 (ns) | First-to-last change |
|---|---:|---:|---:|---:|---:|
| b0 | 1 | 46 | 7700 | 8800 | 14.285714% |
| b0 | 2 | 53 | 8400 | 7900 | -5.952381% |
| b0 | 3 | 47 | 8300 | 10600 | 27.710843% |
| b0 | 4 | 27 | 16900 | 20000 | 18.343195% |
| b0 | 5 | 45 | 29100 | 9700 | -66.666667% |
| b1 | 1 | 41 | 9300 | 21200 | 127.956989% |
| b1 | 2 | 41 | 20800 | 8600 | -58.653846% |
| b1 | 3 | 41 | 7900 | 9900 | 25.316456% |
| b1 | 4 | 41 | 10300 | 21400 | 107.766990% |
| b1 | 5 | 39 | 15400 | 9400 | -38.961039% |
| b2 | 1 | 37 | 36900 | 19300 | -47.696477% |
| b2 | 2 | 38 | 24500 | 13500 | -44.897959% |
| b2 | 3 | 40 | 14700 | 13000 | -11.564626% |
| b2 | 4 | 41 | 17100 | 14300 | -16.374269% |
| b2 | 5 | 38 | 13700 | 19300 | 40.875912% |

## Hypotheses

| Hypothesis | Evidence | Conclusion |
|---|---|---|
| Scheduler noise / frequency drift | p99 changes persist across matched windows while RSS remains below 10% | UNSOLVED; requires controlled host telemetry |
| Resource growth | RSS gate and bounded growth | NOT SUPPORTED as primary cause |
| Orchestration artifact | Unequal tails are reported without dropping common windows | POSSIBLE CONTRIBUTOR; not proven |

## Limitations

The result is single-host engineering evidence and is not an automatic P0 qualification. Thermal drift, scheduler noise, frequency changes, and unavailable platform metrics remain threats to validity.
