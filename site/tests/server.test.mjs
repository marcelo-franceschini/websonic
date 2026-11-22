import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  resolverRequisicaoEstatica,
  obterRaizPublica,
  criarServidorEstatico,
  MIME_TYPES,
} from "../../server.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SITE_ROOT = path.resolve(__dirname, "..");

test("resolverRequisicaoEstatica resolve raiz e arquivos estaticos validos com MIME correto", () => {
  const raiz = resolverRequisicaoEstatica(SITE_ROOT, "/");
  assert.equal(raiz.status, 200);
  assert.equal(path.basename(raiz.caminho), "index.html");
  assert.equal(raiz.contentType, MIME_TYPES[".html"]);
  assert.ok(raiz.tamanho > 0);

  const modulo = resolverRequisicaoEstatica(SITE_ROOT, "/app.js?h=deadbeef");
  assert.equal(modulo.status, 200);
  assert.equal(path.basename(modulo.caminho), "app.js");
  assert.equal(modulo.contentType, MIME_TYPES[".js"]);
});

test("resolverRequisicaoEstatica bloqueia URLs malformadas, bytes nulos e path traversal", () => {
  assert.equal(resolverRequisicaoEstatica(SITE_ROOT, "/%E0%A4%A").status, 400);
  assert.equal(resolverRequisicaoEstatica(SITE_ROOT, "/index.html%00.js").status, 400);
  assert.equal(resolverRequisicaoEstatica(SITE_ROOT, "/../package.json").status, 403);
  assert.equal(resolverRequisicaoEstatica(SITE_ROOT, "/%2e%2e/server.mjs").status, 403);
  assert.equal(resolverRequisicaoEstatica(SITE_ROOT, "/nao-existe.js").status, 404);
});

test("obterRaizPublica prioriza dist/index.html e faz fallback para site/", () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "websonic-root-"));
  try {
    assert.equal(obterRaizPublica(tmp), path.join(tmp, "site"));
    fs.mkdirSync(path.join(tmp, "dist"), { recursive: true });
    fs.writeFileSync(path.join(tmp, "dist", "index.html"), "<!doctype html>");
    assert.equal(obterRaizPublica(tmp), path.join(tmp, "dist"));
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
});

test("resolverRequisicaoEstatica resolve subdiretorio com index.html e extensao desconhecida", () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "websonic-static-"));
  try {
    fs.mkdirSync(path.join(tmp, "sub"), { recursive: true });
    fs.writeFileSync(path.join(tmp, "sub", "index.html"), "<h1>ok</h1>");
    fs.writeFileSync(path.join(tmp, "dados.bin"), "1234");

    const dir = resolverRequisicaoEstatica(tmp, "/sub");
    assert.equal(dir.status, 200);
    assert.equal(path.basename(dir.caminho), "index.html");

    const bin = resolverRequisicaoEstatica(tmp, "/dados.bin");
    assert.equal(bin.status, 200);
    assert.equal(bin.contentType, "application/octet-stream");
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
});

function requisitar(porta, metodo, caminho) {
  return new Promise((resolve, reject) => {
    const req = http.request({ host: "127.0.0.1", port: porta, method: metodo, path: caminho }, (res) => {
      const pedacos = [];
      res.on("data", (chunk) => pedacos.push(chunk));
      res.on("end", () => {
        resolve({
          status: res.statusCode,
          headers: res.headers,
          body: Buffer.concat(pedacos).toString("utf8"),
        });
      });
    });
    req.on("error", reject);
    req.end();
  });
}

test("criarServidorEstatico serve GET/HEAD, cabecalhos de seguranca e trata 405/404/403/400", async () => {
  const servidor = criarServidorEstatico(() => SITE_ROOT);
  await new Promise((resolve) => servidor.listen(0, "127.0.0.1", resolve));
  const { port } = servidor.address();

  try {
    const getOk = await requisitar(port, "GET", "/");
    assert.equal(getOk.status, 200);
    assert.equal(getOk.headers["x-content-type-options"], "nosniff");
    assert.equal(getOk.headers["referrer-policy"], "no-referrer");
    assert.equal(getOk.headers["cache-control"], "no-store");
    assert.match(getOk.body, /WebSonic/);

    const headOk = await requisitar(port, "HEAD", "/manifest.webmanifest");
    assert.equal(headOk.status, 200);
    assert.equal(headOk.body, "");
    assert.match(headOk.headers["content-type"], /application\/manifest\+json/);

    const post405 = await requisitar(port, "POST", "/");
    assert.equal(post405.status, 405);
    assert.equal(post405.headers.allow, "GET, HEAD");

    const get404 = await requisitar(port, "GET", "/nao-existe-mesmo.js");
    assert.equal(get404.status, 404);
    assert.equal(get404.body, "Not Found");

    const head404 = await requisitar(port, "HEAD", "/nao-existe-mesmo.js");
    assert.equal(head404.status, 404);
    assert.equal(head404.body, "");

    const get403 = await requisitar(port, "GET", "/%2e%2e/package.json");
    assert.equal(get403.status, 403);

    const get400 = await requisitar(port, "GET", "/%E0%A4%A");
    assert.equal(get400.status, 400);
  } finally {
    await new Promise((resolve) => servidor.close(resolve));
  }
});

