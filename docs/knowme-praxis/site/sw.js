// Praxis offline cache (htmx-doc-forge). Rules, each tested in tests/test_service_worker.py:
//  - only GET requests for files inside this worker's own scope are cached; navigations are network first (the
//    cache answers only offline), other files cache first;
//  - on a path gateway (/ipfs/<cid>/ or /ipns/<name>/), where every report shares one origin, the worker caches
//    nothing, intercepts nothing and unregisters itself;
//  - feedback-origin, other-origin, event-stream and credential- or token-bearing requests are never
//    intercepted, so they never touch the cache;
//  - activation deletes only this scope's own older caches, never another report's on the same origin;
//  - nothing here, or in the page, writes to web storage, IndexedDB or cookies.
const CFG = {"version": "b6da6181c765", "local": ["./", "index.html", "manifest.webmanifest", "icon-192.png", "icon-512.png", "icon-maskable-512.png", "apple-touch-icon.png", "assets/vendor/htmx.min.js", "assets/vendor/htmx-ext-sse.min.js", "assets/vendor/htmx-ext-json-enc.min.js", "assets/vendor/htmx-ext-response-targets.min.js", "assets/vendor/htmx-ext-class-tools.min.js", "assets/vendor/alpinejs-intersect.min.js", "assets/vendor/alpinejs.min.js"], "feedbackOrigins": []};
const SCOPE = self.registration.scope;
const PREFIX = 'forge:';
const CACHE = PREFIX + CFG.version + ':' + SCOPE;
const TOKEN_PARAMS = ['t', 'token', 'access_token', 'key', 'sig', 'signature', 'code'];
const TOKEN_FRAGMENT = /(^|[#&])(t|token|access_token|key|rft)=/i;
const SHARED_ORIGIN = /^\/ip[fn]s\//.test(new URL(SCOPE).pathname) || self.location.hostname === 'ipfs.prometheusags.ai';

function cacheable(req) {
  if (SHARED_ORIGIN) return false;
  if (req.method !== 'GET') return false;
  let url;
  try { url = new URL(req.url); } catch (e) { return false; }
  if (CFG.feedbackOrigins.indexOf(url.origin) !== -1) return false;
  if (url.origin !== self.location.origin) return false;
  if (url.href.indexOf(SCOPE) !== 0) return false;
  if (url.hash && TOKEN_FRAGMENT.test(url.hash)) return false;
  for (const p of TOKEN_PARAMS) if (url.searchParams.has(p)) return false;
  if (req.headers.get('authorization') || req.headers.get('x-feedback-token')) return false;
  if ((req.headers.get('accept') || '').indexOf('text/event-stream') !== -1) return false;
  if (/(^|\/)(report-feedback|agui|api)(\/|$)/.test(url.pathname.slice(new URL(SCOPE).pathname.length))) return false;
  return true;
}

async function store(c, req, res) {
  if (!cacheable(req) || !res || !res.ok) return;
  if (/no-store/i.test(res.headers.get('cache-control') || '')) return;
  await c.put(req, res);
}

self.addEventListener('install', e => {
  if (SHARED_ORIGIN) { e.waitUntil(Promise.resolve(self.skipWaiting())); return; }
  e.waitUntil(caches.open(CACHE).then(c => Promise.all(CFG.local.map(u => {
    const req = new Request(new URL(u, SCOPE).href);
    return fetch(req).then(r => store(c, req, r)).catch(() => {});
  }))).then(() => self.skipWaiting()));
});

self.addEventListener('activate', e => {
  if (SHARED_ORIGIN) { e.waitUntil(self.registration.unregister()); return; }
  e.waitUntil(caches.keys().then(ks => Promise.all(ks
    .filter(k => k.indexOf(PREFIX) === 0 && k.slice(k.indexOf(':', PREFIX.length) + 1) === SCOPE && k !== CACHE)
    .map(k => caches.delete(k)))).then(() => self.clients.claim()));
});

self.addEventListener('fetch', e => {
  const req = e.request;
  if (!cacheable(req)) return;
  e.respondWith(caches.open(CACHE).then(async c => {
    if (req.mode === 'navigate') {
      // Network first: a stale or planted copy of the page must never win over the server's.
      try { const r = await fetch(req); store(c, req, r.clone()); return r; }
      catch (err) { const hit = await c.match(req, { ignoreSearch: true }); if (hit) return hit; throw err; }
    }
    const hit = await c.match(req);
    const net = fetch(req).then(r => { store(c, req, r.clone()); return r; }).catch(() => hit);
    return hit || net;
  }));
});
