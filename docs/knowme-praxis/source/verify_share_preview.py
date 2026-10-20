"""Validate the compact share page and lossless specification package without publication."""
from pathlib import Path
import argparse,ast,json,hashlib,sys,re
from urllib.parse import urlsplit,unquote
from bs4 import BeautifulSoup
from playwright.sync_api import sync_playwright
p=argparse.ArgumentParser();p.add_argument('--forge-root',required=True,type=Path);p.add_argument('--axe-script',required=True,type=Path);a=p.parse_args()
root=Path(__file__).resolve().parents[1];base=root/'dist/knowme-praxis';site=root/'dist/knowme-praxis-share';evidence=root/'checks-preview-delivery';evidence.mkdir(exist_ok=True)
sys.path.insert(0,str(a.forge_root/'scripts'));from forge.pipeline import serve,launch
node=next(n for n in ast.parse((root/'source/verify_accessibility.py').read_text()).body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='MEASURE' for t in n.targets));MEASURE=ast.literal_eval(node.value)
manifest=json.loads((root/'preview-delivery-manifest.json').read_text());raw=(site/'index.html').read_bytes();soup=BeautifulSoup(raw,'html.parser');early=BeautifulSoup(raw[:8192],'html.parser');spec=BeautifulSoup((site/'specification.html').read_bytes(),'html.parser')
assert len(raw)<100000;assert (site/'specification.html').read_bytes()==(base/'index.html').read_bytes()
assert len(spec.select('section.sec'))==17 and len(spec.select('.source-clause'))==811
required=['og:title','og:type','og:description','og:site_name','og:url','og:image','og:image:secure_url','og:image:type','og:image:width','og:image:height','og:image:alt','twitter:card','twitter:title','twitter:description','twitter:url','twitter:image','twitter:image:alt']
offsets={}
for key in required:
 selector=f'meta[property="{key}"],meta[name="{key}"]';assert len(early.select(selector))==1,key;assert len(soup.select(selector))==1,key;offsets[key]=raw.index(key.encode())
assert early.select_one('link[rel="canonical"]')['href']==manifest['canonical'];assert early.select_one('meta[property="og:url"]')['content']==manifest['canonical'];assert early.select_one('meta[property="og:image"]')['content']==manifest['preview_image']['url'];assert not soup.select('meta[http-equiv="refresh"],script')
# Rewriting hrefs must never alter visible source text, code, headings or coverage rows.
for name in manifest['navigation_only_support_pages']:
 before=BeautifulSoup((base/name).read_text(),'html.parser');after=BeautifulSoup((site/name).read_text(),'html.parser')
 assert before.get_text()==after.get_text(),name
 beforelinks=[x.get('href') for x in before.select('[href]')];afterlinks=[x.get('href') for x in after.select('[href]')]
 expected=[re.sub(r'^(\.\./)?index\.html(?=#|$)',lambda m:(m[1] or '')+'specification.html',v) for v in beforelinks];assert afterlinks==expected,name
broken=[];checked=0;cache={}
for page in site.rglob('*.html'):
 s=BeautifulSoup(page.read_text(),'html.parser')
 for el in s.select('[href],[src]'):
  value=el.get('href',el.get('src',''));u=urlsplit(value)
  if u.scheme or u.netloc or not value:continue
  target=(page.parent/unquote(u.path)).resolve() if u.path else page
  if not target.exists():broken.append([str(page.relative_to(site)),value,'file missing']);continue
  if u.fragment and target.suffix=='.html':
   if target not in cache:cache[target]={e.get('id') for e in BeautifulSoup(target.read_text(),'html.parser').select('[id]')}
   if unquote(u.fragment) not in cache[target]:broken.append([str(page.relative_to(site)),value,'anchor missing'])
  checked+=1
assert not broken,broken[:20]
report={'status':'unpublished candidate; actual iMessage rendering not tested','landing_bytes':len(raw),'metadata_offsets_bytes':offsets,'all_required_metadata_in_first_8192_bytes':True,'canonical':manifest['canonical'],'preview_image_url':manifest['preview_image']['url'],'local_links_checked':checked,'broken_links':broken,'specification_byte_identical':True,'specification_sha256':hashlib.sha256((site/'specification.html').read_bytes()).hexdigest(),'chapters':17,'source_mappings':811,'support_pages_with_navigation_only_changes':len(manifest['navigation_only_support_pages']),'views':[],'axe_sha256':hashlib.sha256(a.axe_script.read_bytes()).hexdigest()}
with serve(site) as url,sync_playwright() as pw:
 browser=launch(pw)
 for width,theme in [(1440,'light'),(1440,'dark'),(390,'light'),(390,'dark')]:
  context=browser.new_context(viewport={'width':width,'height':900 if width>400 else 844},color_scheme=theme,reduced_motion='reduce')
  # Use bytes freshly verified against the public URL; no browser network dependency.
  context.route(manifest['preview_image']['url'],lambda route:route.fulfill(path=str(root/'site/screenshots/revision-1.2-og.png'),content_type='image/png'))
  pg=context.new_page();errors=[];pg.on('pageerror',lambda e:errors.append(str(e)));response=pg.goto(url+'/index.html',wait_until='networkidle');assert response.status==200 and response.request.redirected_from is None
  assert pg.locator('.preview img').evaluate('e=>e.complete && e.naturalWidth===2400 && e.naturalHeight===1260')
  measure=pg.evaluate(MEASURE);assert not measure['text_contrast_failures'],measure['text_contrast_failures'];assert not measure['overflow'];assert not measure['unfocusable']
  pg.add_script_tag(path=str(a.axe_script));axe=pg.evaluate("""async()=>{let r=await axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21a','wcag21aa','wcag22aa']}});return {version:r.testEngine.version,violations:r.violations,incomplete:r.incomplete}}""");assert not axe['violations'],axe['violations']
  pg.screenshot(path=str(evidence/f'{width}-{theme}.png'))
  pg.evaluate('document.activeElement.blur()');pg.keyboard.press('Tab');assert pg.locator('.skip').evaluate('e=>document.activeElement===e');pg.keyboard.press('Enter');pg.keyboard.press('Tab');assert pg.locator('.primary').evaluate('e=>document.activeElement===e')
  focus=pg.locator('.primary').evaluate('e=>{let s=getComputedStyle(e);return {visible:e.matches(":focus-visible"),outline:s.outlineStyle,width:parseFloat(s.outlineWidth)}}');assert focus['visible'] and focus['width']>=3
  pg.keyboard.press('Enter');pg.wait_for_url('**/specification.html');assert pg.locator('section.sec').count()==17;assert pg.locator('.source-clause').count()==811
  assert not errors;report['views'].append({'width':width,'theme':theme,'status':200,'redirects':False,'contrast_failures':len(measure['text_contrast_failures']),'minimum_measured_text_ratio':measure['minimum_text_ratio'],'horizontal_overflow':False,'keyboard_skip_and_primary_link_passed':True,'focus':focus,'image_loaded':True,'page_errors':errors,'axe':axe});context.close()
 ctx=browser.new_context(java_script_enabled=False,viewport={'width':390,'height':844});ctx.route(manifest['preview_image']['url'],lambda route:route.fulfill(path=str(root/'site/screenshots/revision-1.2-og.png'),content_type='image/png'));pg=ctx.new_page();pg.goto(url+'/index.html');pg.locator('.primary').click();pg.wait_for_url('**/specification.html');assert pg.locator('section.sec').count()==17;ctx.close();browser.close()
report['javascript_disabled_navigation_passed']=True;report['pass']=True;report['limitations']='Headless browser and computed/axe checks only. No actual iMessage preview, native-device or screen-reader user testing. Axe incomplete results retained. Public image fetched separately; browser render intercepted to those exact verified local bytes. No candidate publication occurred.'
(evidence/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['views','metadata_offsets_bytes']},indent=2))
