# S6 P99 Root-Cause Comparison

The comparison preserves raw p99 values and does not remove outliers.

Dataset digest: `269eb27cdeed982f81cb0ecb82e2c279014a19fa67bf998ccf529a681ea145c4`

| Package | Raw SHA-256 | Rows |
|---|---|---:|
| baseline | `d192cf6d284990396dfcb6eb9b89b08d190b2b199129189c819819e984ba40fc` | 515 |
| remediated | `0dbda7ccab17142529b51e99fac7291ce394b93c8d4125bd783d1a87a66f5418` | 615 |
| s6 | `e7c8be5561f773d474b7dc0938e57c88868792226ffb1254d4c80798c3e08f7e` | 549 |

Common windows across all packages: 413

| Level | Rep | Common windows | Baseline p99 | Remediated p99 | S6 p99 | S6 process CPU/wall |
|---|---:|---:|---:|---:|---:|---:|
| b0 | 1 | 36 | 16700 | 9400 | 9900 | 97.269030% |
| b0 | 2 | 36 | 13400 | 8500 | 9200 | 100.521154% |
| b0 | 3 | 36 | 14300 | 7100 | 6900 | 100.470707% |
| b0 | 4 | 27 | 16800 | 20000 | 5700 | 98.593063% |
| b0 | 5 | 37 | 16800 | 7900 | 13400 | 95.979694% |
| b1 | 1 | 34 | 16400 | 13900 | 8500 | 85.181625% |
| b1 | 2 | 35 | 22600 | 9800 | 9000 | 100.651837% |
| b1 | 3 | 28 | 19000 | 15800 | 10800 | 100.551362% |
| b1 | 4 | 41 | 16500 | 21400 | 9800 | 97.644312% |
| b1 | 5 | 35 | 18000 | 13300 | 8500 | 100.184174% |
| b2 | 1 | 10 | 26200 | 12800 | 122000 | 55.315643% |
| b2 | 2 | 9 | 7400 | 14100 | 117200 | 52.931499% |
| b2 | 3 | 10 | 23700 | 18700 | 51700 | 86.556086% |
| b2 | 4 | 17 | 23000 | 13800 | 41800 | 91.871509% |
| b2 | 5 | 22 | 28900 | 13600 | 34700 | 97.569203% |

## Conclusion

S6 provides CPU/wall diagnostics and confirms the p99 failure persists. RSS remains bounded, so resource growth is not supported as the primary cause. CPU telemetry alone does not isolate scheduler noise from frequency/thermal drift; the root cause remains unresolved.

S6 is engineering evidence only and does not qualify P0.
