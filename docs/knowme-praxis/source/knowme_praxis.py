import json
from pathlib import Path
from forge.kit import section
HERE=Path(__file__).parent
DOC={
 'slug':'knowme-praxis','title':'KnowMe Praxis','short':'Praxis',
 'description':'The Sovereign Agentic Workspace: complete architecture, functional requirements, native Flutter, Rust, portable plugins, private dots and LiveKit realtime.',
 'h1':'KnowMe Praxis','eyebrow':'KNOWME · ARCHITECTURE & FUNCTIONAL SPECIFICATION · 1.0',
 'lede':'The Sovereign Agentic Workspace. Intent, authority and durable work—connected across native mobile, desktop and the web.',
 'bar':'PRAXIS · TECHNICAL SPECIFICATION','iso':'2026-10-20T10:00:00Z',
 'meta':['Status <b>Proposed integration</b>','Sources <b>45</b>','Coverage <b>811 sections</b>','Date <b>20 October 2026</b>'],
 'footer':'KnowMe Praxis · Technical specification · 1.0 · 20 October 2026',
 'author':'AI-generated with Codex, OpenAI','prepared_by':'Prepared for KnowMe',
 'agui':False,'report_privacy':'public','print_breaks':True,
 'documents':[{'path':p,'privacy':'public','src':'site/'+p} for p in json.loads((HERE/'bundle-manifest.json').read_text())]
}
def body(figure):
 content=json.loads((HERE/'body.json').read_text())
 return '<link rel="stylesheet" href="praxis.css">'+''.join(section(c['id'],c['num'],c['title'],c['html']) for c in content)
