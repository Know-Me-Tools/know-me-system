"""Prepare 1.2 locally. Release metadata requires parent-supplied public identities."""
from pathlib import Path
import argparse,subprocess,sys,shutil,hashlib,json,os,runpy
p=argparse.ArgumentParser();p.add_argument('--forge-root',required=True,type=Path);p.add_argument('--draft',action='store_true');args=p.parse_args()
root=Path(__file__).resolve().parent
canonical=os.environ.get('PUBLIC_CANONICAL_URL');image=os.environ.get('PUBLIC_OG_IMAGE_URL')
if not args.draft and (not canonical or not image):raise SystemExit('Release build requires PUBLIC_CANONICAL_URL and PUBLIC_OG_IMAGE_URL supplied after approved identity/image publication. Use --draft for unpublished accessibility QA.')
forge_revision=subprocess.check_output(['git','-C',str(args.forge_root),'rev-parse','HEAD'],text=True).strip()
if forge_revision!='42336a3eee61261ab3b3cd896fa41dfd9bc23aa7':raise SystemExit('Use forge commit 42336a3eee61261ab3b3cd896fa41dfd9bc23aa7')
icons=root/'dist/_icons';icons.mkdir(parents=True,exist_ok=True)
for name in ['icon-192.png','icon-512.png','icon-maskable-512.png','apple-touch-icon.png']:shutil.copy(root/'site'/name,icons/name)
# Do not inherit the old edition's social image or canonical identity.
(root/'dist/og.json').write_text('{}\n')
subprocess.run([sys.executable,str(args.forge_root/'scripts/docforge.py'),'build',str(root),'--slug','knowme-praxis'],check=True)
sys.path.insert(0,str(args.forge_root/'scripts'));doc=runpy.run_path(str(root/'source/knowme_praxis.py'))['DOC']
sys.path.insert(0,str(root/'source'));from metadata import tags
out=root/'dist/knowme-praxis';path=out/'index.html';text=path.read_text();text=text.replace('</head>',tags(doc,canonical,image)+'\n</head>',1);path.write_text(text)
asset=out/'assets/accessibility.js';asset.parent.mkdir(exist_ok=True);shutil.copy(root/'source/accessibility.js',asset)
for path in out.rglob('*.html'):
 if path.parent.name=='fragments':continue
 relative='../assets/accessibility.js' if path.parent.name=='records' else 'assets/accessibility.js'
 text=path.read_text();text=text.replace('</body>',f'<script defer src="{relative}"></script></body>',1);path.write_text(text)
files=[{'name':str(f.relative_to(out)),'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted(out.rglob('*')) if f.is_file()]
(root/'candidate-manifest-1.2.json').write_text(json.dumps({'edition':'1.2','status':'unpublished draft; metadata identities pending' if args.draft else 'prepared release candidate; not uploaded','canonical':canonical,'image':image,'files':files},indent=2)+'\n')
print(f'Prepared {len(files)} files; no upload, credentials, key, DNS or server changes.')
