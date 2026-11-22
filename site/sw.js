// Service Worker offline-first do WebSonic.
// O marcador de versao e substituido por `cargo xtask site` pelo hash de conteudo do build.
const CACHE_NAME = "__WEBSONIC_CACHE_VERSION__";

const CORE_ASSETS = [
  "./",
  "./index.html",
  "./style.css",
  "./app.js",
  "./audio.js",
  "./video.js",
  "./preferencias.js",
  "./gpu.js",
  "./modal-controles.js",
  "./entrada.js",
  "./hud.js",
  "./saves.js",
  "./save-ui.js",
  "./buffers.js",
  "./pwa.js",
  "./relogio.js",
  "./sim-worker.js",
  "./audio-worklet.js",
  "./manifest.webmanifest",
  "./favicon.svg",
  "./icon-192.png",
  "./icon-512.png",
  "./icon-maskable-512.png",
  "./apple-touch-icon.png",
];

// Em producao (dist/), estes ativos sao reescritos com hash por `cargo xtask site`.
// Em desenvolvimento sem build Wasm local, o cache individual ignora 404 sem falhar a instalacao.
const OPTIONAL_WASM_ASSETS = [
  "./pkg/sonic1_web.js",
  "./pkg/sonic1_web_bg.wasm",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then(async (cache) => {
        await cache.addAll(CORE_ASSETS);
        await Promise.allSettled(OPTIONAL_WASM_ASSETS.map((url) => cache.add(url)));
      })
      .catch(() => {})
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((chaves) =>
        Promise.all(
          chaves
            .filter((chave) => chave !== CACHE_NAME)
            .map((chave) => caches.delete(chave)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;

  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return;

  // Navegacao HTML: network-first com fallback para cache offline.
  if (req.mode === "navigate") {
    event.respondWith(
      fetch(req)
        .then((res) => {
          if (res && res.ok && res.type === "basic") {
            const copia = res.clone();
            caches.open(CACHE_NAME).then((cache) => cache.put(req, copia)).catch(() => {});
          }
          return res;
        })
        .catch(() =>
          caches.match(req).then((hit) => hit || caches.match("./index.html") || caches.match("./")),
        ),
    );
    return;
  }

  // Recursos estaticos locais (JS, CSS, Wasm, icones): cache-first com preenchimento sob demanda.
  event.respondWith(
    caches.match(req).then((cached) => {
      if (cached) return cached;
      return fetch(req).then((res) => {
        if (res && res.ok && res.type === "basic") {
          const copia = res.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(req, copia)).catch(() => {});
        }
        return res;
      });
    }),
  );
});
