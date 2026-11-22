// Servidor estatico local para o preview do WebSonic na porta 3000.
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PORT = Number(process.env.PORT || 3000);
const HOST = "0.0.0.0";

export const MIME_TYPES = Object.freeze({
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".cjs": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".ico": "image/x-icon",
  ".json": "application/json; charset=utf-8",
  ".webmanifest": "application/manifest+json; charset=utf-8",
  ".txt": "text/plain; charset=utf-8",
  ".xml": "application/xml; charset=utf-8",
});

export function obterRaizPublica(baseDir = __dirname) {
  const distDir = path.join(baseDir, "dist");
  if (fs.existsSync(path.join(distDir, "index.html"))) {
    return distDir;
  }
  return path.join(baseDir, "site");
}

/**
 * Resolve de forma segura um caminho HTTP dentro de `root`, bloqueando URLs malformadas,
 * bytes nulos e path traversal.
 */
export function resolverRequisicaoEstatica(root, rawUrl = "/") {
  let cleanPath;
  try {
    cleanPath = decodeURIComponent(String(rawUrl).split("?")[0]);
  } catch {
    return { status: 400, erro: "Bad Request" };
  }

  if (cleanPath.includes("\0")) {
    return { status: 400, erro: "Bad Request" };
  }

  const rootResolvido = path.resolve(root);
  const relPath = cleanPath === "/" ? "index.html" : cleanPath.replace(/^\/+/, "");
  const fullPath = path.resolve(rootResolvido, relPath);
  const relativo = path.relative(rootResolvido, fullPath);

  if (relativo.startsWith("..") || path.isAbsolute(relativo)) {
    return { status: 403, erro: "Forbidden" };
  }

  let target = fullPath;
  if (fs.existsSync(target) && fs.statSync(target).isDirectory()) {
    target = path.join(target, "index.html");
  }

  if (!fs.existsSync(target) || !fs.statSync(target).isFile()) {
    return { status: 404, erro: "Not Found" };
  }

  const ext = path.extname(target).toLowerCase();
  const stat = fs.statSync(target);
  return {
    status: 200,
    caminho: target,
    tamanho: stat.size,
    contentType: MIME_TYPES[ext] || "application/octet-stream",
  };
}

export function criarServidorEstatico(obterRoot = () => obterRaizPublica()) {
  return http.createServer((req, res) => {
    const metodo = req.method || "GET";
    if (metodo !== "GET" && metodo !== "HEAD") {
      res.writeHead(405, {
        "Content-Type": "text/plain; charset=utf-8",
        Allow: "GET, HEAD",
        "X-Content-Type-Options": "nosniff",
      });
      res.end("Method Not Allowed");
      return;
    }

    const resolvido = resolverRequisicaoEstatica(obterRoot(), req.url || "/");
    if (resolvido.status !== 200) {
      res.writeHead(resolvido.status, {
        "Content-Type": "text/plain; charset=utf-8",
        "X-Content-Type-Options": "nosniff",
      });
      res.end(metodo === "HEAD" ? "" : resolvido.erro);
      return;
    }

    res.writeHead(200, {
      "Content-Type": resolvido.contentType,
      "Content-Length": String(resolvido.tamanho),
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff",
      "Referrer-Policy": "no-referrer",
    });

    if (metodo === "HEAD") {
      res.end();
      return;
    }

    const stream = fs.createReadStream(resolvido.caminho);
    stream.on("error", () => {
      if (!res.headersSent) {
        res.writeHead(500, { "Content-Type": "text/plain; charset=utf-8" });
      }
      res.end();
    });
    stream.pipe(res);
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === __filename) {
  const server = criarServidorEstatico();
  server.listen(PORT, HOST, () => {
    console.log(`WebSonic dev server rodando em http://${HOST}:${PORT} (${obterRaizPublica()})`);
  });
}
