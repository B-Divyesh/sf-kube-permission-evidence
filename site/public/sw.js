const CACHE = 'kpe-field-guide-v2';
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
  if (event.request.method !== 'GET' || new URL(event.request.url).origin !== self.location.origin) return;
  event.respondWith(fetch(event.request).then(async (response) => {
    if (response.ok) await (await caches.open(CACHE)).put(event.request, response.clone());
    return response;
  }).catch(async () => (await caches.match(event.request)) || caches.match('/')));
});
