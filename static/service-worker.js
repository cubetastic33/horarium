const CACHE_NAME = 'static-cache-v1';

const FILES_TO_CACHE = [
	'/',
	'/offline',
	'/install_pwa',
	'/styles/main.css',
	'/scripts/jquery-3.4.1.min.js',
	'/scripts/jquery-ui-1.12.1.custom/jquery-ui.min.js',
	'/scripts/timetable.js',
	'/images/favicon-32x32.png',
	'/images/favicon-16x16.png',
	'/images/favicon.ico',
	'/images/apple-touch-icon.png',
	'/images/safari-pinned-tab.svg',
	'/images/browserconfig.xml',
	'/fonts/Titillium_Web/TitilliumWeb-Regular.ttf',
	'/fonts/Source_Sans_Pro/SourceSansPro-Regular.ttf',
	'/fonts/Roboto/Roboto-Regular.ttf',
	'/fonts/Roboto/Roboto-Medium.ttf',
	'/fonts/Hind_Vadodara/HindVadodara-Medium.ttf',
];

self.addEventListener('install', (evt) => {
	console.log('[ServiceWorker] Install');
	evt.waitUntil(
		caches.open(CACHE_NAME).then((cache) => {
			console.log('[ServiceWorker] Pre-caching offline page');
			return cache.addAll(FILES_TO_CACHE);
		})
	);
	self.skipWaiting();
});

self.addEventListener('activate', (evt) => {
	console.log('[ServiceWorker] Activate');
	evt.waitUntil(
		caches.keys().then((keyList) => {
			return Promise.all(keyList.map((key) => {
				if (key !== CACHE_NAME) {
					console.log('[ServiceWorker] Removing old cache', key);
					return caches.delete(key);
				}
			}));
		})
	);
	self.clients.claim();
});

self.addEventListener('fetch', (evt) => {
	console.log('[ServiceWorker] Fetch', evt.request.url);
	if (!evt.request.url.includes('/timetables/')) {
		console.log('[Service Worker] Fetch (data)', evt.request.url);
		evt.respondWith(
			caches.open(CACHE_NAME).then(cache => {
				return fetch(evt.request)
					.then(response => {
						// If the response was good, clone it and store it in the cache.
						if (response.status === 200) {
							cache.put(evt.request.url, response.clone());
						}
						return response;
					}).catch(err => {
						// Network request failed, try to get it from the cache.
						return cache.match(evt.request);
					});
			}));
	}
	// If it's a navigation request and we're offline, show the offline page
	if (evt.request.mode === 'navigate' && !navigator.onLine) {
		evt.respondWith(
			caches.open(CACHE_NAME).then(cache => {
				return cache.match('offline');
			})
		);
	}
	evt.respondWith(
		caches.open(CACHE_NAME).then(cache => {
			return cache.match(evt.request)
				.then(response => {
					return response || fetch(evt.request);
				});
		})
	);
});
