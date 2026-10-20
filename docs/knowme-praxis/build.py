"""Rebuild the approved document using a local htmx-doc-forge checkout."""
from pathlib import Path
import argparse,subprocess,sys,shutil,hashlib,json
p=argparse.ArgumentParser();p.add_argument('--forge-root',required=True,type=Path);args=p.parse_args()
root=Path(__file__).resolve().parent
# Reuse the exact audited icon assets; no browser or network is needed to assemble HTML.
icons=root/'dist/_icons';icons.mkdir(parents=True,exist_ok=True)
for name in ['icon-192.png','icon-512.png','icon-maskable-512.png','apple-touch-icon.png']:
 shutil.copy(root/'site'/name,icons/name)
subprocess.run([sys.executable,str(args.forge_root/'scripts/docforge.py'),'build',str(root),'--slug','knowme-praxis'],check=True)
receipt=json.loads((root/'publication-receipt.json').read_text());out=root/'dist/knowme-praxis'
failures=[]
for entry in receipt['files']:
 path=out/entry['name']
 if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest()!=entry['sha256']:failures.append(entry['name'])
if failures:raise SystemExit('Rebuild differs from the published artifact: '+', '.join(failures))
print(f"Verified exact rebuild of {len(receipt['files'])} published files.")
