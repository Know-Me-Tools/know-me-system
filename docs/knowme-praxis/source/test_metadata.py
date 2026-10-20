"""Offline metadata contract tests. Reserved example URLs are fixtures only."""
import unittest
from html.parser import HTMLParser
from metadata import tags,validate_url
class Head(HTMLParser):
 def __init__(self,text):super().__init__();self.meta=[];self.links=[];self.feed(text)
 def handle_starttag(self,tag,attrs):
  a=dict(attrs)
  if tag=='meta':self.meta.append((a.get('property') or a.get('name'),a.get('content')))
  if tag=='link':self.links.append(a)
class MetadataTests(unittest.TestCase):
 def setUp(self):self.doc={'h1':'KnowMe Praxis','description':'Technical specification','iso':'2026-10-20T10:00:00Z'}
 def test_release_identity_and_image_are_consistent(self):
  canonical='https://example.org/praxis/';image='https://example.org/preview.png'
  h=Head(tags(self.doc,canonical,image));d=dict(h.meta)
  self.assertEqual(len(d),len(h.meta))
  for k in ['og:title','og:type','og:image','og:url','og:description','og:site_name','og:locale','og:image:secure_url','og:image:type','og:image:width','og:image:height','og:image:alt','twitter:card','twitter:title','twitter:description','twitter:image','twitter:image:alt','twitter:url','article:published_time','article:modified_time']:self.assertTrue(d.get(k),k)
  self.assertEqual(d['og:url'],canonical);self.assertEqual(d['twitter:url'],canonical);self.assertEqual(h.links,[{'rel':'canonical','href':canonical}])
  self.assertEqual(d['og:image'],d['og:image:secure_url']);self.assertEqual(d['og:image'],d['twitter:image']);self.assertEqual(d['og:image:width'],'2400');self.assertEqual(d['og:image:height'],'1260');self.assertNotIn('article:author',d)
 def test_draft_does_not_invent_identity_or_reuse_old_image(self):
  h=Head(tags(self.doc));d=dict(h.meta);self.assertNotIn('og:url',d);self.assertNotIn('og:image',d);self.assertFalse(h.links)
 def test_stale_cid_and_unsafe_urls_are_rejected(self):
  for value in ['https://example.org/ipfs/previous/index.html','http://example.org/praxis/','https://user:password@example.org/praxis/','https://example.org/praxis/#fragment']:
   with self.assertRaises(ValueError):validate_url(value,'canonical')
 def test_values_are_escaped(self):
  doc=dict(self.doc,h1='A "quoted" title <tag>');h=Head(tags(doc));self.assertEqual(dict(h.meta)['og:title'],doc['h1'])
if __name__=='__main__':unittest.main()
