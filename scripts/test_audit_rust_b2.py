import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name('audit-rust-b2.py')


def row(level, repetition, p99=100):
    return {'run_id': f'{level}-{repetition}', 'level': level, 'repetition': repetition,
            'protocol_version': '1.1', 'observability_variant': 'local_metrics',
            'execution_id': 'exec', 'measured_frames': 100000, 'correctness_total': 10,
            'correctness_matches': 10, 'p99_ns': p99,
            'filter_timings_ns': {f'F{i}': 1 for i in range(1, 7)},
            'filter_invocations': {f'F{i}': 10 for i in range(1, 7)},
            'filter_rejections': {}, 'filter_execution_failures': {},
            'observability_tax_percent': 0.0}


class AuditB2Tests(unittest.TestCase):
    def run_audit(self, mutate=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline = [row('b1', i, 100) for i in range(1, 6)]
            raw = [row('b2', i, 110) for i in range(1, 6)]
            for item in raw:
                item['observability_tax_percent'] = 10.0
            if mutate:
                mutate(raw[0])
            base_path, raw_path = root / 'b1.jsonl', root / 'b2.jsonl'
            base_path.write_text(''.join(json.dumps(x) + '\n' for x in baseline), encoding='utf-8')
            raw_path.write_text(''.join(json.dumps(x) + '\n' for x in raw), encoding='utf-8')
            return subprocess.run([sys.executable, str(SCRIPT), '--baseline', str(base_path), '--raw', str(raw_path)], capture_output=True, text=True)

    def test_valid_records_are_accepted(self):
        self.assertEqual(self.run_audit().returncode, 0)

    def test_missing_field_is_rejected(self):
        self.assertNotEqual(self.run_audit(lambda x: x.pop('execution_id')).returncode, 0)

    def test_wrong_tax_is_rejected(self):
        self.assertNotEqual(self.run_audit(lambda x: x.update(observability_tax_percent=0.0)).returncode, 0)


if __name__ == '__main__':
    unittest.main()
