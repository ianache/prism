import json, subprocess, sys, tempfile, unittest
from pathlib import Path
SCRIPT = Path(__file__).with_name('audit-rust-s2.py')
def row(level, repetition, p99=100):
    return {'level':level,'repetition':repetition,'scenario':'S2','protocol_version':'1.1','measured_frames':100000,'dataset_id':'d','dataset_digest':'a'*64,'correctness_total':1,'correctness_matches':1,'p50_ns':1,'p95_ns':2,'p99_ns':p99,'p99_9_ns':3,'max_ns':4,'frames_per_sec':1.0,'mb_per_sec':1.0,'workflow_tax_percent':0.0,'observability_tax_percent':0.0,'filter_invocations':{f'F{i}':1 for i in range(1,7)}}
class S2AuditTests(unittest.TestCase):
    def run_audit(self, mutate=None):
        with tempfile.TemporaryDirectory(dir=r'D:\02-PERSONAL\TOOLS\prism-datasets') as d:
            root=Path(d)/'dataset'; root.mkdir(); (root/'manifest.json').write_text(json.dumps({'dataset_id':'d'})); (root/'manifest.sha256').write_text('a'*64)
            rows=[row(l,i,100) for l in ('b0','b1','b2') for i in range(1,6)]
            for r in rows:
                if r['level']=='b2': r['observability_tax_percent']=0.0
            if mutate: mutate(rows[0])
            raw=root/'raw.jsonl'; raw.write_text(''.join(json.dumps(r)+'\n' for r in rows))
            return subprocess.run([sys.executable,str(SCRIPT),'--dataset',str(root),'--raw',str(raw)],capture_output=True)
    def test_valid(self): self.assertEqual(self.run_audit().returncode,0)
    def test_wrong_frame_rejected(self): self.assertNotEqual(self.run_audit(lambda r:r.update(measured_frames=10)).returncode,0)
if __name__=='__main__': unittest.main()
