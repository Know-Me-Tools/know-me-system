"""Headless edition checks; requires Playwright and the pinned forge checkout."""
import argparse, json, struct, sys
from pathlib import Path
from playwright.sync_api import sync_playwright

p=argparse.ArgumentParser();p.add_argument('--forge-root',required=True,type=Path);a=p.parse_args()
sys.path.insert(0,str(a.forge_root/'scripts'))
from forge.pipeline import serve, launch
r=Path(__file__).resolve().parents[1];site=r/'site';report={'views':[]}
with serve(site) as base,sync_playwright() as pw:
 browser=launch(pw)
 for name,width,scheme in [('desktop',1440,'light'),('desktop-dark',1440,'dark'),('mobile',390,'light'),('mobile-dark',390,'dark')]:
  context=browser.new_context(viewport={'width':width,'height':900 if width>400 else 844},color_scheme=scheme)
  page=context.new_page();errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
  page.goto(base+'/index.html',wait_until='networkidle')
  page.evaluate("document.querySelectorAll('details').forEach(x=>x.open=true)")
  result=page.evaluate("""() => ({chapters:document.querySelectorAll('section.sec').length,retained:document.querySelectorAll('.source-clause').length,overflow:document.documentElement.scrollWidth>innerWidth,webmcp:document.querySelector('#workspace').innerText.includes('WEB-14'),integration:Object.fromEntries(Object.entries({workspace:'WEB-14',api:'API-07',plugins:'PL-09',runtimes:'Adapter placement',surfaces:'UI-12',security:'SEC-07',acceptance:'G17',implementation:'Phased implementation',decisions:'Browser integration release decisions',provenance:'Primary-source byte verification'}).map(([id,text])=>[id,document.getElementById(id).innerText.includes(text)])),canonical:document.head.querySelector('link[rel="canonical"]')?.href||null,metadata:[...document.head.querySelectorAll('meta[property],meta[name^="twitter:"]')].map(x=>({key:x.getAttribute('property')||x.name,value:x.content}))})""")
  result.update(name=name,page_errors=errors);report['views'].append(result)
  if name in ['desktop','mobile']:
   page.locator('#workspace h3').filter(has_text='First-class WebMCP').scroll_into_view_if_needed()
   page.screenshot(path=str(site/f'screenshots/revision-1.1-webmcp-{name}.png'))
  context.close()
 browser.close()
png=(site/'screenshots/revision-1.1-og.png').read_bytes()
report['image']={'signature_valid':png[:8]==b'\x89PNG\r\n\x1a\n','dimensions':struct.unpack('>II',png[16:24]),'public_response':'not verified: authenticated IPFS upload blocked'}
required=['og:type','og:site_name','og:title','og:description','og:image','og:image:secure_url','og:image:type','og:image:width','og:image:height','og:image:alt','og:locale','article:published_time','article:modified_time','article:author','twitter:card','twitter:title','twitter:description','twitter:image','twitter:image:alt']
expected_image=json.loads((r/'source/social.json').read_text())['knowme-praxis']['image']
for v in report['views']:
 tags={t['key']:t['value'] for t in v['metadata']}
 v['metadata_ok']=(all(sum(t['key']==key and bool(t['value']) for t in v['metadata'])==1 for key in required)
  and tags.get('og:image')==tags.get('twitter:image')==tags.get('og:image:secure_url')==expected_image
  and tags.get('og:image:width')=='2400' and tags.get('og:image:height')=='1260'
  and tags.get('og:image:type')=='image/png' and tags.get('twitter:card')=='summary_large_image'
  and tags.get('og:type')=='article' and not v['canonical'] and 'og:url' not in tags and 'twitter:url' not in tags)
report['ok']=(report['image']['signature_valid'] and report['image']['dimensions']==(2400,1260)
 and all(v['chapters']==17 and v['retained']==811 and v['webmcp'] and all(v['integration'].values()) and v['metadata_ok'] and not v['overflow'] and not v['page_errors'] for v in report['views']))
target=site/'verification/revision-1.1.json';target.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2));raise SystemExit(0 if report['ok'] else 1)
