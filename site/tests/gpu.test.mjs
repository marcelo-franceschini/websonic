// Testes unitarios do renderizador WebGL2 (site/gpu.js) com contexto deterministico.
import { test } from "node:test";
import assert from "node:assert/strict";
import { criarRenderizadorGPU, LARGURA, ALTURA } from "../gpu.js";

function criarWebgl2Stub({
  falharVs = false,
  falharFs = false,
  falharLink = false,
  comDebugExt = true,
} = {}) {
  const chamadas = [];
  const shadersDeletados = [];
  const programasDeletados = [];
  const texturasDeletadas = [];
  const vaosDeletados = [];
  let seq = 1;

  const gl = {
    VERTEX_SHADER: 1,
    FRAGMENT_SHADER: 2,
    COMPILE_STATUS: 3,
    LINK_STATUS: 4,
    BLEND: 5,
    DITHER: 6,
    TEXTURE0: 7,
    TEXTURE_2D: 8,
    TEXTURE_MIN_FILTER: 9,
    TEXTURE_MAG_FILTER: 10,
    NEAREST: 11,
    R16UI: 12,
    UNPACK_ALIGNMENT: 13,
    RENDERER: 14,
    RED_INTEGER: 15,
    UNSIGNED_SHORT: 16,
    TRIANGLES: 17,
    RGBA: 18,
    UNSIGNED_BYTE: 19,

    createShader(tipo) {
      return { id: seq++, tipo };
    },
    shaderSource() {},
    compileShader() {},
    getShaderParameter(s) {
      if (s.tipo === gl.VERTEX_SHADER && falharVs) return false;
      if (s.tipo === gl.FRAGMENT_SHADER && falharFs) return false;
      return true;
    },
    getShaderInfoLog(s) {
      return s.tipo === gl.VERTEX_SHADER ? "erro-vs" : "erro-fs";
    },
    deleteShader(s) {
      shadersDeletados.push(s);
    },
    createProgram() {
      return { id: seq++ };
    },
    attachShader() {},
    linkProgram() {},
    detachShader() {},
    getProgramParameter() {
      return !falharLink;
    },
    getProgramInfoLog() {
      return "erro-link";
    },
    deleteProgram(p) {
      programasDeletados.push(p);
    },
    useProgram(p) {
      chamadas.push(["useProgram", p]);
    },
    createVertexArray() {
      return { vao: 1 };
    },
    bindVertexArray(v) {
      chamadas.push(["bindVertexArray", v]);
    },
    deleteVertexArray(v) {
      vaosDeletados.push(v);
    },
    disable() {},
    createTexture() {
      return { tex: 1 };
    },
    activeTexture() {},
    bindTexture(_t, tex) {
      chamadas.push(["bindTexture", tex]);
    },
    deleteTexture(tex) {
      texturasDeletadas.push(tex);
    },
    texParameteri() {},
    texStorage2D() {},
    pixelStorei() {},
    getUniformLocation() {
      return 42;
    },
    uniform1i() {},
    viewport() {},
    getExtension(nome) {
      if (nome === "WEBGL_debug_renderer_info" && comDebugExt) {
        return { UNMASKED_RENDERER_WEBGL: 999 };
      }
      return null;
    },
    getParameter(param) {
      return param === 999 ? "GPU Unmasked" : "WebGL Renderer";
    },
    texSubImage2D(...args) {
      chamadas.push(["texSubImage2D", args]);
    },
    drawArrays(...args) {
      chamadas.push(["drawArrays", args]);
    },
    readPixels(_x, _y, _w, _h, _fmt, _type, dst) {
      dst[0] = 0xAA;
      dst[1] = 0xBB;
      chamadas.push(["readPixels"]);
    },
    finish() {
      chamadas.push(["finish"]);
    },
  };

  return {
    gl,
    chamadas,
    shadersDeletados,
    programasDeletados,
    texturasDeletadas,
    vaosDeletados,
  };
}

test("criarRenderizadorGPU retorna null quando webgl2 nao esta disponivel", () => {
  const canvas = { getContext: () => null };
  assert.equal(criarRenderizadorGPU(canvas), null);
});

test("criarRenderizadorGPU trata falhas de compilacao de shaders e link sem vazar recursos", () => {
  const vsFail = criarWebgl2Stub({ falharVs: true });
  assert.throws(() => criarRenderizadorGPU({ getContext: () => vsFail.gl }), /erro-vs/);
  assert.equal(vsFail.shadersDeletados.length, 1);

  const fsFail = criarWebgl2Stub({ falharFs: true });
  assert.throws(() => criarRenderizadorGPU({ getContext: () => fsFail.gl }), /erro-fs/);
  // Deve deletar o fragment shader falho E o vertex shader ja compilado.
  assert.equal(fsFail.shadersDeletados.length, 2);

  const linkFail = criarWebgl2Stub({ falharLink: true });
  assert.throws(() => criarRenderizadorGPU({ getContext: () => linkFail.gl }), /erro-link/);
  assert.equal(linkFail.shadersDeletados.length, 2);
  assert.equal(linkFail.programasDeletados.length, 1);
});

test("criarRenderizadorGPU renderiza, le pixels, sincroniza e destroi de forma idempotente", () => {
  const stub = criarWebgl2Stub({ comDebugExt: true });
  const canvas = { width: 0, height: 0, getContext: () => stub.gl };
  const gpu = criarRenderizadorGPU(canvas);

  assert.equal(canvas.width, LARGURA);
  assert.equal(canvas.height, ALTURA);
  assert.equal(gpu.renderer, "GPU Unmasked");
  assert.equal(stub.shadersDeletados.length, 2);

  const quadro = new Uint16Array(LARGURA * ALTURA);
  gpu.renderizar(quadro);
  assert.ok(stub.chamadas.some((c) => c[0] === "texSubImage2D"));
  assert.ok(stub.chamadas.some((c) => c[0] === "drawArrays"));

  const pixels = gpu.lerPixels();
  assert.equal(pixels.length, LARGURA * ALTURA * 4);
  assert.equal(pixels[0], 0xAA);
  assert.equal(pixels[1], 0xBB);

  gpu.sincronizar();
  assert.ok(stub.chamadas.some((c) => c[0] === "finish"));

  // Destruir libera recursos uma unica vez e torna renderizar no-op.
  const qtdAntes = stub.chamadas.length;
  gpu.destruir();
  gpu.destruir();
  assert.equal(stub.texturasDeletadas.length, 1);
  assert.equal(stub.vaosDeletados.length, 1);
  assert.equal(stub.programasDeletados.length, 1);

  const aposDestruir = stub.chamadas.length;
  assert.ok(aposDestruir > qtdAntes);
  gpu.renderizar(quadro);
  assert.equal(stub.chamadas.length, aposDestruir);

  // Sem extensao WEBGL_debug_renderer_info usa gl.RENDERER.
  const stubSemExt = criarWebgl2Stub({ comDebugExt: false });
  const gpuSemExt = criarRenderizadorGPU({ width: 0, height: 0, getContext: () => stubSemExt.gl });
  assert.equal(gpuSemExt.renderer, "WebGL Renderer");
});
