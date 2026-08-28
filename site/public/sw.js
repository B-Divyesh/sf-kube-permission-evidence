const CACHE = 'kpe-field-guide-v3';
const SHELL = ['/', '/privacy/', '/terms/', '/specimen-map.webp', '/favicon.svg'];
self.addEventListener('install', (event) => event.waitUntil(
  caches.open(CACHE).then((cache) => cache.addAll(SHELL)).then(() => self.skipWaiting()),
));
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
  event.respondWith(fetch(event.request).then(async (response) => {
    if (response.ok) await (await caches.open(CACHE)).put(event.request, response.clone());
    return response;
  }).catch(async () => (await caches.match(event.request)) || caches.match('/')));
});
