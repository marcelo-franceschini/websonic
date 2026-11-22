// Testes unitarios da logica pura de video (escala inteira e LUT de cor 9 bits -> RGBA).
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  gerarTabelaCor9Bits,
  calcularFatorEscalaInteira,
  criarApresentadorVideo,
} from "../video.js";
import { LARGURA, ALTURA } from "../gpu.js";

test("gerarTabelaCor9Bits converte preto, branco e canais primarios com replicacao de bits", () => {
  const lut = gerarTabelaCor9Bits();
  assert.equal(lut.length, 512);
  // Preto (0) -> RGBA (0, 0, 0, 255) = 0xFF000000
  assert.equal(lut[0], 0xFF000000);
  // Branco (0b111_111_111 = 511) -> RGBA (255, 255, 255, 255) = 0xFFFFFFFF
  assert.equal(lut[511], 0xFFFFFFFF);
  // Vermelho puro (r=7, g=0, b=0) -> 0xFF0000FF
  assert.equal(lut[7], 0xFF0000FF);
  // Verde puro (r=0, g=7, b=0 = 56) -> 0xFF00FF00
  assert.equal(lut[56], 0xFF00FF00);
  // Azul puro (r=0, g=0, b=7 = 448) -> 0xFFFF0000
  assert.equal(lut[448], 0xFFFF0000);
});

test("calcularFatorEscalaInteira preserva escala inteira minima 1 e aproveita tela cheia", () => {
  // Mobile estreito -> fator minimo 1
  assert.equal(
    calcularFatorEscalaInteira({ larguraJanela: 390, alturaJanela: 844, emTelaCheia: false }),
    1,
  );
  // Desktop 1440x900 -> dispW=960 (3x), dispH=648 (2x) -> fator 2
  assert.equal(
    calcularFatorEscalaInteira({ larguraJanela: 1440, alturaJanela: 900, emTelaCheia: false }),
    2,
  );
  // Tela cheia 1920x1080 -> min(6, 4.82) -> fator 4
  assert.equal(
    calcularFatorEscalaInteira({ larguraJanela: 1920, alturaJanela: 1080, emTelaCheia: true }),
    4,
  );
});

function criarWebgl2Minimo({ falharRender = false } = {}) {
  return {
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
    createShader: () => ({}),
    shaderSource() {},
    compileShader() {},
    getShaderParameter: () => true,
    getShaderInfoLog: () => "",
    deleteShader() {},
    createProgram: () => ({}),
    attachShader() {},
    linkProgram() {},
    detachShader() {},
    getProgramParameter: () => true,
    getProgramInfoLog: () => "",
    deleteProgram() {},
    useProgram() {},
    createVertexArray: () => ({}),
    bindVertexArray() {},
    deleteVertexArray() {},
    disable() {},
    createTexture: () => ({}),
    activeTexture() {},
    bindTexture() {},
    deleteTexture() {},
    texParameteri() {},
    texStorage2D() {},
    pixelStorei() {},
    getUniformLocation: () => 0,
    uniform1i() {},
    viewport() {},
    getExtension: () => null,
    getParameter: () => "StubGL",
    texSubImage2D() {
      if (falharRender) throw new Error("gpu-crash");
    },
    drawArrays() {},
    finish() {},
  };
}

function criarCanvasStub({ modoGl = "ok", comOffscreen = true, falharOffscreen = false } = {}) {
  const ouvintes = new Map();
  const puts2d = [];
  let focos = 0;

  const canvas = {
    width: 0,
    height: 0,
    tabIndex: -1,
    style: { width: "", height: "" },
    puts2d,
    get focos() {
      return focos;
    },
    focus() {
      focos++;
    },
    addEventListener(tipo, fn) {
      ouvintes.set(tipo, fn);
    },
    disparar(tipo, evt = {}) {
      const fn = ouvintes.get(tipo);
      if (fn) fn(evt);
    },
    cloneNode() {
      return criarCanvasStub({ modoGl, comOffscreen, falharOffscreen });
    },
    replaceWith() {},
    getContext(tipo) {
      if (tipo === "2d") {
        return {
          putImageData(img) {
            puts2d.push(img);
          },
        };
      }
      if (modoGl === "null") return null;
      if (modoGl === "throw") throw new Error("gl-init-err");
      return criarWebgl2Minimo({ falharRender: modoGl === "render-throw" });
    },
  };

  if (comOffscreen) {
    canvas.transferControlToOffscreen = () => {
      if (falharOffscreen) throw new Error("transfer-fail");
      return { offscreen: true };
    };
  }
  return canvas;
}

test("criarApresentadorVideo gerencia Worker zero-copy, perda/restauracao, fallback 2D, escala e tela cheia", () => {
  const origWindow = globalThis.window;
  const origDocument = globalThis.document;
  const origImageData = globalThis.ImageData;

  let reqFullscreen = 0;
  let webkitFullscreen = 0;
  let exitFullscreen = 0;

  globalThis.window = {
    innerWidth: 1440,
    innerHeight: 900,
    addEventListener() {},
  };
  globalThis.document = {
    fullscreenElement: null,
    addEventListener() {},
    exitFullscreen: async () => {
      exitFullscreen++;
    },
  };
  globalThis.ImageData = class FakeImageData {
    constructor(w, h) {
      this.width = w;
      this.height = h;
      this.data = new Uint8ClampedArray(w * h * 4);
    }
  };

  try {
    const erros = [];
    let limpezas = 0;
    let perdas = 0;
    let restauracoes = 0;
    let interacoes = 0;
    const badgeGpu = { textContent: "" };
    const containerTela = {
      requestFullscreen: async () => {
        reqFullscreen++;
      },
    };

    const canvas = criarCanvasStub({ modoGl: "ok", comOffscreen: true });
    const video = criarApresentadorVideo({
      canvasInicial: canvas,
      containerTela,
      badgeGpu,
      onErro: (m) => erros.push(m),
      onLimparErro: () => limpezas++,
      onContextoPerdido: () => perdas++,
      onContextoRestaurado: () => restauracoes++,
      onInteracao: () => interacoes++,
    });

    // Clique na tela foca e notifica interacao.
    canvas.disparar("click");
    assert.equal(canvas.focos, 1);
    assert.equal(interacoes, 1);

    // Anexar no Worker via OffscreenCanvas.
    const msgsWorker = [];
    const anexado = video.tentarAnexarNoWorker({
      postMessage: (m) => msgsWorker.push(m),
    });
    assert.equal(anexado, true);
    assert.equal(msgsWorker[0].tipo, "anexar-canvas");

    video.confirmarAnexoWorker(true);
    assert.equal(video.estaNoWorker(), true);
    assert.equal(badgeGpu.textContent, "WebGL2 · Zero-Copy");

    // No Worker, apresentar na thread principal e no-op.
    video.apresentar(new Uint16Array(LARGURA * ALTURA));

    // Perda e restauracao de contexto no Worker.
    video.notificarPerdaNoWorker();
    assert.equal(video.estaPerdido(), true);
    assert.equal(perdas, 1);

    video.notificarRestauracaoNoWorker(true);
    assert.equal(video.estaPerdido(), false);
    assert.equal(limpezas, 1);
    assert.equal(restauracoes, 1);

    // Falha de restauracao no Worker cai para thread principal (WebGL2).
    video.notificarRestauracaoNoWorker(false);
    assert.equal(video.estaNoWorker(), false);
    assert.equal(badgeGpu.textContent, "WebGL2 · Wasm");

    // Apresenta quadro em WebGL2 na thread principal.
    const quadro = new Uint16Array(LARGURA * ALTURA);
    quadro[0] = 511; // branco
    video.apresentar(quadro);

    // Eventos webglcontextlost e webglcontextrestored na thread principal.
    const telaAtual = video.obterTela();
    let prevenido = false;
    telaAtual.disparar("webglcontextlost", {
      preventDefault() {
        prevenido = true;
      },
    });
    assert.equal(prevenido, true);
    assert.equal(video.estaPerdido(), true);

    telaAtual.disparar("webglcontextrestored");
    assert.equal(video.estaPerdido(), false);

    // Se restauracao WebGL2 falhar (contexto null), faz fallback para 2D CPU.
    const origGetContext = telaAtual.getContext;
    telaAtual.getContext = (t) => (t === "webgl2" ? null : origGetContext.call(telaAtual, t));
    telaAtual.disparar("webglcontextrestored");
    assert.equal(badgeGpu.textContent, "Wasm · 2D");
    assert.ok(erros.some((e) => /restaurar WebGL2/.test(e)));

    // Escala inteira e tela cheia (entrar, webkit fallback, sair).
    video.ajustarEscala();
    assert.equal(video.obterTela().style.width, "640px");
    assert.equal(video.obterTela().style.height, "448px");

    video.alternarTelaCheia();
    assert.equal(reqFullscreen, 1);

    delete containerTela.requestFullscreen;
    containerTela.webkitRequestFullscreen = () => {
      webkitFullscreen++;
    };
    video.alternarTelaCheia();
    assert.equal(webkitFullscreen, 1);

    globalThis.document.fullscreenElement = containerTela;
    video.alternarTelaCheia();
    assert.equal(exitFullscreen, 1);

    // Fallback para CPU 2D quando transferControlToOffscreen falha e WebGL2 retorna null.
    const canvasSemGl = criarCanvasStub({ modoGl: "null", comOffscreen: true, falharOffscreen: true });
    const videoCpu = criarApresentadorVideo({
      canvasInicial: canvasSemGl,
      badgeGpu,
      onErro: (m) => erros.push(m),
    });
    assert.equal(videoCpu.tentarAnexarNoWorker({ postMessage() {} }), false);
    assert.equal(badgeGpu.textContent, "Wasm · 2D");
    videoCpu.apresentar(quadro);
    assert.equal(videoCpu.obterTela().puts2d.length, 1);
    const imgData = videoCpu.obterTela().puts2d[0];
    assert.equal(imgData.data[0], 255);
    assert.equal(imgData.data[1], 255);
    assert.equal(imgData.data[2], 255);
    assert.equal(imgData.data[3], 255);

    // Fallback para CPU 2D quando renderizar WebGL2 lanca erro em runtime.
    const canvasCrash = criarCanvasStub({ modoGl: "render-throw", comOffscreen: false });
    const videoCrash = criarApresentadorVideo({
      canvasInicial: canvasCrash,
      badgeGpu,
      onErro: (m) => erros.push(m),
    });
    videoCrash.tentarAnexarNoWorker(null);
    videoCrash.apresentar(quadro);
    assert.equal(badgeGpu.textContent, "Wasm · 2D");
    assert.ok(erros.some((e) => /gpu-crash/.test(e)));

    // Fallback quando criarRenderizadorGPU lanca durante inicializacao.
    const canvasThrow = criarCanvasStub({ modoGl: "throw", comOffscreen: false });
    const videoThrow = criarApresentadorVideo({
      canvasInicial: canvasThrow,
      badgeGpu,
      onErro: (m) => erros.push(m),
    });
    videoThrow.confirmarAnexoWorker(false);
    assert.ok(erros.some((e) => /gl-init-err/.test(e)));
  } finally {
    globalThis.window = origWindow;
    globalThis.document = origDocument;
    globalThis.ImageData = origImageData;
  }
});

