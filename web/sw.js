"use strict";

const PREFIX = `memorama:${self.registration.scope}:`;
const CACHE = `${PREFIX}__BUILD_ID__`;
const ASSETS = [
  "./", "./index.html", "./style.css", "./app.js", "./manifest.webmanifest",
  "./gl.js", "./highscores.js", "./memorama_arcade.wasm",
  "./icon-180.png", "./icon-192.png", "./icon-512.png",
];

self.addEventListener("install", (event) => {
  event.waitUntil((async () => {
    const cache = await caches.open(CACHE);
    await cache.addAll(ASSETS.map((path) => new Request(
      new URL(path, self.registration.scope), { cache: "reload" },
    )));
  })());
});

self.addEventListener("activate", (event) => {
  event.waitUntil((async () => {
    const names = await caches.keys();
    await Promise.all(names.filter((name) => name.startsWith(PREFIX) && name !== CACHE)
      .map((name) => caches.delete(name)));
    await self.clients.claim();
  })());
});

self.addEventListener("message", (event) => {
  if (event.data && event.data.type === "SKIP_WAITING") self.skipWaiting();
});

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (event.request.method !== "GET" || !url.href.startsWith(self.registration.scope)) return;
  event.respondWith((async () => {
    const cache = await caches.open(CACHE);
    const cached = await cache.match(event.request, {
      ignoreSearch: event.request.mode === "navigate",
    });
    if (cached) return cached;
    return fetch(event.request);
  })());
});
