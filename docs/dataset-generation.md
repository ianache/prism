# Dataset generation workflow

The generator and oracle use only the Python standard library and run from the repository root.

Smoke dataset:

    python -m tools.dataset.generate --output <temporary-directory>/dataset --seed 0x505249534D5F5631 --count 300
    python -m tools.dataset.verify --dataset <temporary-directory>/dataset

Run the oracle independently over fixture bytes:

    python -m tools.dataset.oracle --input <temporary-directory>/dataset/fixtures --output <temporary-directory>/dataset/oracle-results.jsonl

The generator refuses a non-empty output directory unless --replace is supplied. Generation writes to a sibling temporary directory and promotes only after all files are complete. The canonical repository dataset is never modified by smoke tests.

The full release command uses --count 1000000. Its raw binaries are release artifacts and must not be committed unless repository size policy explicitly approves them.
