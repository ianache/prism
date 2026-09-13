# Rust S5 Evidence Report

Raw path: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5.jsonl`
Raw SHA-256: `d192cf6d284990396dfcb6eb9b89b08d190b2b199129189c819819e984ba40fc`
Audit output: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-audit.txt`

Dataset: `prism.telemetry.p0.v1`  
Dataset manifest SHA-256: `269eb27cdeed982f81cb0ecb82e2c279014a19fa67bf998ccf529a681ea145c4`  
Protocol: v1.1, 100,000 frames/window, five repetitions, 900 seconds total  
Preflight commit: `465303b21016d8428f1c0575bd5605ea909a85a9`  
Toolchain: `cargo 1.98.1 (797e8a9bc 2026-08-05)`

## Audit

ok levels=b0,b1,b2 repetitions=5 frames=100000 windows=515 gate_failures=p99:b1:r2,p99:b1:r3,p99:b1:r4,p99:b2:r5

## Summary

Rows: 515
Complete windows per tuple: 28
Maximum p99 change: 124.015748%
Maximum RSS change: 0.669886%

## Gates

| Gate | Status | Detail |
|---|---|---|
| Correctness | PASS | all windows match |
| p99 stability | FAIL | threshold <= 10% |
| RSS growth | PASS | threshold < 10% |

## Limitations

The result is single-host engineering evidence and is not an automatic P0 qualification. Thermal drift, scheduler noise, frequency changes, and unavailable platform metrics remain threats to validity.
