import argparse, json, math
from pathlib import Path

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--raw', type=Path, required=True)
    parser.add_argument('--dataset', type=Path, required=True)
    args = parser.parse_args()
    rows = [json.loads(x) for x in args.raw.read_text(encoding='utf-8').splitlines() if x]
    if len(rows) != 15 or {r.get('level') for r in rows} != {'b0', 'b1', 'b2'}:
        raise SystemExit('expected 15 S2 records for b0,b1,b2')
    if sorted((r.get('level'), r.get('repetition')) for r in rows) != sorted((l, i) for l in ('b0','b1','b2') for i in range(1,6)):
        raise SystemExit('invalid S2 repetitions')
    manifest = json.loads((args.dataset / 'manifest.json').read_text(encoding='utf-8'))
    digest = (args.dataset / 'manifest.sha256').read_text(encoding='ascii').strip()
    base = {r['repetition']: r for r in rows if r['level'] == 'b0'}
    b1 = {r['repetition']: r for r in rows if r['level'] == 'b1'}
    for r in rows:
        if r.get('scenario') != 'S2' or r.get('protocol_version') != '1.1' or r.get('measured_frames') != 100000:
            raise SystemExit('invalid S2 protocol fields')
        if r.get('dataset_digest') != digest or r.get('dataset_id') != manifest['dataset_id']:
            raise SystemExit('dataset identity mismatch')
        if r.get('correctness_total') != r.get('correctness_matches'):
            raise SystemExit('correctness mismatch')
        for key in ('p50_ns','p95_ns','p99_ns','p99_9_ns','max_ns','frames_per_sec','mb_per_sec'):
            if not isinstance(r.get(key), (int,float)) or not math.isfinite(r[key]) or r[key] < 0:
                raise SystemExit(f'invalid metric: {key}')
        if r['level'] == 'b2' and set(r.get('filter_invocations', {})) != {'F1','F2','F3','F4','F5','F6'}:
            raise SystemExit('incomplete B2 filter evidence')
        if r['level'] == 'b1':
            expected = 0.0 if base[r['repetition']]['p99_ns'] == 0 else (r['p99_ns']-base[r['repetition']]['p99_ns'])/base[r['repetition']]['p99_ns']*100
            if not math.isclose(r['workflow_tax_percent'], expected, rel_tol=1e-9, abs_tol=1e-9): raise SystemExit('workflow tax mismatch')
        if r['level'] == 'b2':
            expected = 0.0 if b1[r['repetition']]['p99_ns'] == 0 else (r['p99_ns']-b1[r['repetition']]['p99_ns'])/b1[r['repetition']]['p99_ns']*100
            if not math.isclose(r['observability_tax_percent'], expected, rel_tol=1e-9, abs_tol=1e-9): raise SystemExit('observability tax mismatch')
    print(f'ok rows={len(rows)} levels=b0,b1,b2 frames=100000')

if __name__ == '__main__':
    main()
