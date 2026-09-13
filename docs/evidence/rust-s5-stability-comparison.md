# S5 Stability Remediation Comparison

Baseline SHA-256: `d192cf6d284990396dfcb6eb9b89b08d190b2b199129189c819819e984ba40fc`
Remediated SHA-256: `0dbda7ccab17142529b51e99fac7291ce394b93c8d4125bd783d1a87a66f5418`

| Level | Rep | Common windows | Baseline only | Remediated only | Last p99 delta |
|---|---:|---:|---:|---:|---:|
| b0 | 1 | 36 | 0 | 10 | -43.712574850299404 |
| b0 | 2 | 36 | 0 | 17 | -36.56716417910448 |
| b0 | 3 | 36 | 0 | 11 | -50.349650349650354 |
| b0 | 4 | 27 | 10 | 0 | 19.047619047619047 |
| b0 | 5 | 37 | 0 | 8 | -52.976190476190474 |
| b1 | 1 | 34 | 0 | 7 | -15.24390243902439 |
| b1 | 2 | 35 | 0 | 6 | -56.63716814159292 |
| b1 | 3 | 34 | 0 | 7 | -30.412371134020617 |
| b1 | 4 | 41 | 0 | 0 | 29.6969696969697 |
| b1 | 5 | 35 | 0 | 4 | -26.111111111111114 |
| b2 | 1 | 28 | 0 | 9 | -44.776119402985074 |
| b2 | 2 | 38 | 0 | 0 | -43.75 |
| b2 | 3 | 29 | 0 | 11 | -48.63813229571984 |
| b2 | 4 | 29 | 0 | 12 | -47.368421052631575 |
| b2 | 5 | 30 | 0 | 8 | -70.12302284710017 |

The comparison preserves raw p99 values and does not remove outliers or change the provisional gate.

This is engineering evidence and not an automatic P0 qualification.
