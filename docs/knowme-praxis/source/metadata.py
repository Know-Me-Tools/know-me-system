"""Static social metadata; release identities are supplied, never guessed."""
import html
from urllib.parse import urlsplit
ALT='KnowMe Praxis 1.2 architecture specification, showing the title, source coverage and chapter navigation.'
def validate_url(value, label):
 u=urlsplit(value)
 if u.scheme!='https' or not u.netloc or u.username or u.password or u.fragment:
  raise ValueError(label+' must be an absolute HTTPS URL without credentials or fragment')
 if label=='canonical' and '/ipfs/' in u.path:
  raise ValueError('Use the approved stable IPNS/alias identity, not a previous immutable CID')
 return value

def tags(doc, canonical=None, image=None):
 esc=lambda x:html.escape(str(x),quote=True)
 values=[('property','og:type','article'),('property','og:site_name','KnowMe'),('property','og:title',doc['h1']),('property','og:description',doc['description']),('property','og:locale','en_US'),('property','article:published_time',doc['iso']),('property','article:modified_time','2026-10-20T12:00:00Z'),('name','twitter:card','summary_large_image'),('name','twitter:title',doc['h1']),('name','twitter:description',doc['description'])]
 if canonical:
  validate_url(canonical,'canonical');values += [('property','og:url',canonical),('name','twitter:url',canonical)]
 if image:
  validate_url(image,'image');values += [('property','og:image',image),('property','og:image:secure_url',image),('property','og:image:type','image/png'),('property','og:image:width','2400'),('property','og:image:height','1260'),('property','og:image:alt',ALT),('name','twitter:image',image),('name','twitter:image:alt',ALT)]
 result='\n'.join(f'<meta {attr}="{esc(key)}" content="{esc(value)}">' for attr,key,value in values)
 if canonical:result+='\n<link rel="canonical" href="'+esc(canonical)+'">'
 # The renderer keeps prose attribution in meta name=author. No unverified profile URL.
 return result
