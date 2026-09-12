# Dataset release status

## Status

The deterministic dataset generator and oracle are implemented and verified through the 100,000-fixture scale.

The 1,000,000-fixture release is **not published**. It was intentionally not committed to Git because it requires one individual binary file per fixture, and the full run exceeded the practical execution window of the current environment.

## Verified evidence

| Count | Valid | Invalid | Edge complex | Generation | Verification | Result |
|---:|---:|---:|---:|---:|---:|---|
| 300 | 240 | 15 | 45 | smoke test | smoke test | PASS |
| 1,000 | 800 | 50 | 150 | 1.176 s | 2.367 s | PASS |
| 100,000 | 80,000 | 5,000 | 15,000 | 236.462 s | 450.845 s | PASS |

The complete dataset test suite passes with 30 tests. Temporary artifacts used for these runs were removed after verification.

## Full release command

After an environment with sufficient execution time and storage is available:

```text
python -m tools.dataset.generate --output <release-directory> --seed 0x505249534D5F5631 --count 1000000
python -m tools.dataset.verify --dataset <release-directory>
```

Expected workload: 800,000 valid, 50,000 invalid, and 150,000 edge_complex fixtures.
