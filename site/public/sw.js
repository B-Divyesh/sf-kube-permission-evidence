const CACHE = 'kpe-evidence-v6';
const SHELL = ['/', '/demo/', '/privacy/', '/terms/', '/404.html', '/kpe-demo.cast', '/specimen-map.webp', '/share-card.webp', '/favicon.svg', '/apple-touch-icon.png'];
self.addEventListener('install', (event) => event.waitUntil((async () => {
  const cache = await caches.open(CACHE);
  await cache.addAll(SHELL);
  const html = await (await cache.match('/')).text();
  const assets = [...html.matchAll(/(?:href|src)="(\/assets\/[^"?#]+)"/g)].map((match) => match[1]);
  await cache.addAll([...new Set(assets)]);
  await self.skipWaiting();
})()));
self.addEventListener('activate', (event) => event.waitUntil(
  caches.keys()
    .then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key))))
    .then(() => self.clients.claim()),
));
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (event.request.method !== 'GET' || url.origin !== self.location.origin) return;

  // License return tokens are credentials. Never persist a request URL that
  // contains one; the page strips the parameter and keeps the token only in
  // localStorage. A v3 activation also removes every v2 query-bearing entry.
  if (url.searchParams.has('license')) {
    event.respondWith(fetch(event.request).catch(() => caches.match('/')));
    return;
  }
  event.respondWith(caches.match(event.request).then((cached) => {
    if (cached) return cached;
    return fetch(event.request).then(async (response) => {
      if (response.ok) await (await caches.open(CACHE)).put(event.request, response.clone());
      return response;
    }).catch(() => caches.match('/404.html'));
  }));
});
