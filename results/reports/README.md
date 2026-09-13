# Reports

Reports are generated from results/raw only and include gate evaluation, threats to validity, and matched B0-B3 tax calculations. B2 reports must identify the `local_metrics` variant and use 100,000 measured frames per repetition.

The Rust S3 package is external evidence at `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s3.jsonl`. It contains 105 records across B0/B1/B2 and concurrencies 1, 2, 4, 8, 16, 32, and 64. S3 results describe concurrency behavior only; they do not qualify P0.

The Rust S4 package is external evidence at `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s4.jsonl`. It contains 45 records for baseline, burst, and recovery phases. S4 measures offered-rate pressure and processing lateness without a queue; it does not qualify P0.
