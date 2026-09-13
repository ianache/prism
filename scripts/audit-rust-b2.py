import argparse
import json
import math
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--raw', type=Path, required=True)
    args = parser.parse_args()
    rows = [json.loads(line) for line in args.raw.read_text(encoding='utf-8').splitlines() if line]
    if len(rows) != 5 or {row.get('level') for row in rows} != {'b2'}:
        raise SystemExit('expected five B2 records')
    if [row.get('repetition') for row in rows] != [1, 2, 3, 4, 5]:
        raise SystemExit('expected repetitions 1..5')
    required = {'protocol_version', 'observability_variant', 'execution_id', 'filter_timings_ns', 'filter_invocations', 'filter_rejections', 'filter_execution_failures', 'observability_tax_percent'}
    for row in rows:
        missing = required - row.keys()
        if missing:
            raise SystemExit(f'missing B2 fields: {sorted(missing)}')
        if row['protocol_version'] != '1.1' or row['observability_variant'] != 'local_metrics' or row['measured_frames'] != 100000:
            raise SystemExit('invalid B2 protocol evidence')
        if row['correctness_total'] != row['correctness_matches']:
            raise SystemExit('correctness mismatch')
        if set(row['filter_invocations']) != {'F1', 'F2', 'F3', 'F4', 'F5', 'F6'}:
            raise SystemExit('incomplete filter invocation keys')
        if any(not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 for value in row['filter_timings_ns'].values()):
            raise SystemExit('invalid filter timing')
        if not math.isfinite(row['observability_tax_percent']):
            raise SystemExit('invalid observability tax')
    print(f'ok rows={len(rows)} frames=100000')


if __name__ == '__main__':
    main()
