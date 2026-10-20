"""Rendered accessibility checks. Reports measured results, not WCAG certification."""
from pathlib import Path
import argparse,json,sys,hashlib,struct
from playwright.sync_api import sync_playwright
p=argparse.ArgumentParser();p.add_argument('--forge-root',required=True,type=Path);p.add_argument('--axe-script',required=True,type=Path);a=p.parse_args()
sys.path.insert(0,str(a.forge_root/'scripts'))
from forge.pipeline import serve,launch
root=Path(__file__).resolve().parents[1];site=root/'dist/knowme-praxis';evidence=root/'checks-1.2';shots=evidence/'screenshots';shots.mkdir(parents=True,exist_ok=True)
MEASURE=r'''() => {
 const rgb=s=>(s.match(/[\d.]+/g)||[]).map(Number); const lum=c=>c.slice(0,3).map(v=>{v/=255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4}).reduce((s,v,i)=>s+v*[.2126,.7152,.0722][i],0);
 const ratio=(a,b)=>(Math.max(lum(a),lum(b))+.05)/(Math.min(lum(a),lum(b))+.05);
 const blend=(f,b)=>{let a=f.length>3?f[3]:1;return f.slice(0,3).map((v,i)=>v*a+b[i]*(1-a))};
 const bg=e=>{let chain=[];for(let p=e;p;p=p.parentElement)chain.unshift(p);return chain.reduce((b,p)=>blend(rgb(getComputedStyle(p).backgroundColor),b),[255,255,255])};
 const isVisible=e=>{if(!e.getClientRects().length||e.closest('[hidden],[inert]'))return false;for(let p=e;p;p=p.parentElement){let s=getComputedStyle(p);if(s.display==='none'||s.visibility==='hidden'||+s.opacity===0)return false}return true};
 let tested=0,failures=[],minimum=100; const walker=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT);
 while(walker.nextNode()) {let n=walker.currentNode,e=n.parentElement;if(!n.textContent.trim()||!e||e.closest('script,style,svg,[disabled],[aria-disabled="true"]')||!isVisible(e))continue;let range=document.createRange();range.selectNodeContents(n);if(![...range.getClientRects()].some(r=>r.width&&r.height))continue;let s=getComputedStyle(e),back=bg(e),fore=blend(rgb(s.color),back),value=ratio(fore,back),size=parseFloat(s.fontSize),large=size>=24||(size>=18.666&&+s.fontWeight>=700),need=large?3:4.5;tested++;minimum=Math.min(minimum,value);if(value+.001<need)failures.push({text:n.textContent.trim().slice(0,100),tag:e.tagName,cls:e.className,fg:s.color,bg:back,ratio:value,required:need}); }
 let scroll=[...document.querySelectorAll('*')].filter(e=>{if(!isVisible(e)||!e.clientWidth)return false;let s=getComputedStyle(e);return (/auto|scroll/.test(s.overflowX)&&e.scrollWidth>e.clientWidth+1)||(/auto|scroll/.test(s.overflowY)&&e.scrollHeight>e.clientHeight+1)});
 let scrollInfo=scroll.map((e,i)=>{e.dataset.auditScroll=String(i);return {id:i,tag:e.tagName,cls:e.className,tabindex:e.tabIndex,focusableDescendant:!!e.querySelector('a[href],button,input,[tabindex="0"]'),x:e.scrollWidth>e.clientWidth+1,y:e.scrollHeight>e.clientHeight+1}});
 let selectors=['pre.retained-text','.retained-group>summary','.toc h2','.toc a:not(.on) .tn','footer.foot','.audit-links a','.source-clause h4','.source-note'];
 let pairs=selectors.flatMap(selector=>{let e=document.querySelector(selector);if(!e)return [];let s=getComputedStyle(e),back=bg(e);return [{selector,fg:s.color,bg:back,ratio:ratio(blend(rgb(s.color),back),back)}]});
 return {text_nodes_tested:tested,text_contrast_failures:failures,minimum_text_ratio:minimum,pairs,scroll:scrollInfo,unfocusable:scrollInfo.filter(e=>e.tabindex<0&&!e.focusableDescendant),overflow:document.documentElement.scrollWidth>innerWidth,retained:document.querySelectorAll('.source-clause').length,chapters:document.querySelectorAll('section.sec').length};
}'''
report={'scope':'All 811 retained sections expanded on main document in four viewport/theme combinations; standalone record, home and coverage surfaces sampled with shared stylesheet. Automated and keyboard checks only; screen-reader user testing is not performed.','views':[],'supporting_pages':[],'axe_script_sha256':hashlib.sha256(a.axe_script.read_bytes()).hexdigest()}
def axe(page):
 page.add_script_tag(path=str(a.axe_script))
 result=page.evaluate("""async () => {const r=await axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21a','wcag21aa','wcag22aa']},rules:{'scrollable-region-focusable':{enabled:true}},resultTypes:['violations','incomplete']});return {version:r.testEngine.version,violations:r.violations,incomplete:r.incomplete,passes:r.passes.map(x=>x.id)}}""")
 return result
with serve(site) as base,sync_playwright() as pw:
 browser=launch(pw)
 for width,scheme,name in [(1440,'light','desktop-light'),(1440,'dark','desktop-dark'),(390,'light','mobile-light'),(390,'dark','mobile-dark')]:
  print('Checking '+name,flush=True)
  ctx=browser.new_context(viewport={'width':width,'height':900 if width>400 else 844},color_scheme=scheme,reduced_motion='reduce');page=ctx.new_page();errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
  page.goto(base+'/index.html',wait_until='networkidle');page.screenshot(path=str(shots/(name+'.png')))
  page.evaluate("document.querySelectorAll('details').forEach(d=>d.open=true)");page.wait_for_timeout(300)
  view=page.evaluate(MEASURE);view.update(name=name,width=width,scheme=scheme)
  view['axe']=axe(page);view['keyboard']=[]
  for region in view['scroll']:
   if region['tabindex']<0:continue
   el=page.locator('[data-audit-scroll="'+str(region['id'])+'"]');el.scroll_into_view_if_needed()
   el.evaluate("e=>{let b=document.createElement('button');b.id='audit-tab-start';b.textContent='Audit start';e.before(b);b.focus()}")
   page.keyboard.press('Tab');focused=el.evaluate('e=>document.activeElement===e');style=el.evaluate("e=>{let c=getComputedStyle(e);return {outlineStyle:c.outlineStyle,width:parseFloat(c.outlineWidth),offset:parseFloat(c.outlineOffset),visible:e.matches(':focus-visible')}}")
   before=el.evaluate('e=>e.scrollLeft');page.keyboard.press('ArrowRight');page.wait_for_timeout(100);after=el.evaluate('e=>e.scrollLeft')
   page.locator('#audit-tab-start').evaluate('e=>e.remove()')
   view['keyboard'].append({'region':region['id'],'tab_reached':focused,'focus_style':style,'horizontal_key_scroll':after>before if region['x'] else None})
  view['summaries']=page.evaluate("""() => [...document.querySelectorAll('.retained-group>summary')].map(e=>{e.focus();let s=getComputedStyle(e),r=e.getBoundingClientRect(),p=e.parentElement.getBoundingClientRect();return {outline:parseFloat(s.outlineWidth),offset:parseFloat(s.outlineOffset),contained:r.left>=p.left-1&&r.right<=p.right+1&&r.top>=p.top-1,focusable:e.tabIndex>=0}})""")
  page.locator('pre.retained-text').first.scroll_into_view_if_needed();page.screenshot(path=str(shots/('retained-'+name+'.png')))
  view['page_errors']=errors;report['views'].append(view);(evidence/'accessibility.json').write_text(json.dumps(report,indent=2));ctx.close()
 for scheme in ['light','dark']:
  for path in ['records/S00.html','records/S07.html','home.html','coverage.html']:
   print('Checking '+scheme+' '+path,flush=True)
   pg=browser.new_page(viewport={'width':390,'height':844},color_scheme=scheme,reduced_motion='reduce');pg.goto(base+'/'+path,wait_until='networkidle');pg.wait_for_timeout(100);item=pg.evaluate(MEASURE);item.update(path=path,scheme=scheme);item['axe']=axe(pg);report['supporting_pages'].append(item);pg.close();(evidence/'accessibility.json').write_text(json.dumps(report,indent=2))
 ctx=browser.new_context(viewport={'width':1200,'height':630},device_scale_factor=2,color_scheme='light');pg=ctx.new_page();pg.goto(base+'/index.html',wait_until='networkidle');pg.screenshot(path=str(shots/'og-1.2.png'));ctx.close();browser.close()
views=report['views']+report['supporting_pages']
report['pass']=all(not v['text_contrast_failures'] and not v['unfocusable'] and not v['overflow'] and not v['axe']['violations'] for v in views) and all(v['retained']==811 and v['chapters']==17 and not v['page_errors'] and all(k['tab_reached'] and k['focus_style']['visible'] and k['focus_style']['width']>=2 and k['horizontal_key_scroll'] is not False for k in v['keyboard']) and all(s['contained'] and s['focusable'] and s['outline']>=2 and s['offset']<0 for s in v['summaries']) for v in report['views'])
report['automated_incomplete_items']=sum(len(v['axe']['incomplete']) for v in views)
(evidence/'accessibility.json').write_text(json.dumps(report,indent=2));print(json.dumps({'pass':report['pass'],'views':[{'name':v.get('name',v.get('path')),'scheme':v['scheme'],'text_nodes':v['text_nodes_tested'],'contrast_failures':len(v['text_contrast_failures']),'axe_violations':[(x['id'],len(x['nodes'])) for x in v['axe']['violations']],'unfocusable':len(v['unfocusable'])} for v in views],'incomplete_items':report['automated_incomplete_items']},indent=2))
