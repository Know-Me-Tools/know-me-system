"""Prepare a compact share entry point from the exact 1.2 package. No network or credentials."""
from pathlib import Path
import argparse,hashlib,json,re,shutil,sys,html
from bs4 import BeautifulSoup
p=argparse.ArgumentParser();p.add_argument('--base-dir',type=Path);args=p.parse_args()
root=Path(__file__).resolve().parent;base=args.base_dir or root/'dist/knowme-praxis';out=root/'dist/knowme-praxis-share'
manifest=json.loads((root/'release-transfer-1.2.json').read_text());receipt=json.loads((root/'publication-receipt-1.2.json').read_text())
for f in manifest['files']:
 assert hashlib.sha256((base/f['name']).read_bytes()).hexdigest()==f['sha256'],f['name']
if out.exists():shutil.rmtree(out)
shutil.copytree(base,out)
original=(base/'index.html').read_bytes();(out/'specification.html').write_bytes(original)
soup=BeautifulSoup(original,'html.parser');chapters=soup.select('section.sec');assert len(chapters)==17
sys.path.insert(0,str(root/'source'));from metadata import tags
doc={'h1':'KnowMe Praxis','description':soup.select_one('meta[property="og:description"]')['content'],'iso':'2026-10-20T10:00:00Z'}
page=(root/'source/share-index.html').read_text().replace('<!--PREVIEW_METADATA-->',tags(doc,receipt['stable_url'],receipt['image']['url'])).replace('<!--PREVIEW_IMAGE-->',html.escape(receipt['image']['url'],quote=True))
links=[]
for i,c in enumerate(chapters):
 title=c.select_one('h2').get_text(' ',strip=True);title=re.sub(r'^\d{2}\s+','',title)
 links.append(f'<li><a href="specification.html#{html.escape(c["id"],quote=True)}"><span class="num" aria-hidden="true">{i:02d}</span><span>{html.escape(title)}</span></a></li>')
page=page.replace('<!--CHAPTER_LINKS-->','\n'.join(links));(out/'index.html').write_text(page)
# Navigation-only changes in support pages: point existing specification links to its new filename.
changed=[]
for path in sorted(out.rglob('*.html')):
 if path.name in ['index.html','specification.html']:continue
 text=path.read_text();revised=re.sub(r'(\bhref=[\"\'])(\.\./)?index\.html(?=[#\"\'])',lambda m:m[1]+(m[2] or '')+'specification.html',text)
 if revised!=text:path.write_text(revised);changed.append(str(path.relative_to(out)))
assert (out/'specification.html').read_bytes()==original
files=[{'name':str(f.relative_to(out)),'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted(out.rglob('*')) if f.is_file()]
report={'status':'unpublished preview-delivery candidate; no IPFS upload or IPNS update performed','base_edition':'1.2','canonical':receipt['stable_url'],'preview_image':receipt['image'],'landing_bytes':len(page.encode()),'specification_sha256':hashlib.sha256(original).hexdigest(),'specification_byte_identical_to_published_index':True,'navigation_only_support_pages':changed,'files':files}
(root/'preview-delivery-manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['files','preview_image','navigation_only_support_pages']},indent=2));print('Support pages with corrected links:',len(changed))
