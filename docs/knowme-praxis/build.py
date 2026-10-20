"""Rebuild the versioned document using the pinned htmx-doc-forge checkout."""
from pathlib import Path
import argparse,subprocess,sys,shutil,hashlib,json
p=argparse.ArgumentParser();p.add_argument('--forge-root',required=True,type=Path);args=p.parse_args()
root=Path(__file__).resolve().parent
forge_revision=subprocess.check_output(['git','-C',str(args.forge_root),'rev-parse','HEAD'],text=True).strip()
if forge_revision!='42336a3eee61261ab3b3cd896fa41dfd9bc23aa7':raise SystemExit('Use the documented forge commit 42336a3eee61261ab3b3cd896fa41dfd9bc23aa7')
# Reuse the exact audited icon assets; no browser or network is needed to assemble HTML.
icons=root/'dist/_icons';icons.mkdir(parents=True,exist_ok=True)
for name in ['icon-192.png','icon-512.png','icon-maskable-512.png','apple-touch-icon.png']:
 shutil.copy(root/'site'/name,icons/name)
social=root/'source/social.json'
if social.exists():shutil.copy(social,root/'dist/og.json')
subprocess.run([sys.executable,str(args.forge_root/'scripts/docforge.py'),'build',str(root),'--slug','knowme-praxis'],check=True)
out=root/'dist/knowme-praxis'
if social.exists():
 path=out/'index.html';html=path.read_text()
 extra='<meta name="twitter:image:alt" content="KnowMe Praxis 1.1 architecture specification, showing title, source coverage and chapter navigation.">\n<meta property="article:modified_time" content="2026-10-20T11:00:00Z">\n'
 html=html.replace('content="KnowMe Praxis, KnowMe"','content="KnowMe Praxis 1.1 architecture specification, showing title, source coverage and chapter navigation."')
 path.write_text(html.replace('</head>',extra+'</head>',1))
receipt_path=root/'candidate-manifest.json'
if not receipt_path.exists():receipt_path=root/'publication-receipt.json'
receipt=json.loads(receipt_path.read_text())
failures=[]
actual={str(p.relative_to(out)) for p in out.rglob('*') if p.is_file()}
expected={entry['name'] for entry in receipt['files']}
if actual!=expected:failures.extend(sorted(actual^expected))
for entry in receipt['files']:
 path=out/entry['name']
 if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest()!=entry['sha256']:failures.append(entry['name'])
if failures:raise SystemExit('Rebuild differs from the published artifact: '+', '.join(failures))
print(f"Verified exact rebuild of {len(receipt['files'])} files against {receipt_path.name}.")
