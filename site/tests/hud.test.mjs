// Testes unitarios das metricas puras do HUD (media, p95 e classificacao de frametime).
import { test } from "node:test";
import assert from "node:assert/strict";
import { calcularMediaEP95, classificarFrametime, criarHud } from "../hud.js";

test("calcularMediaEP95 retorna zero para janela vazia", () => {
  const res = calcularMediaEP95(new Float32Array(10), 0);
  assert.deepEqual(res, { media: 0, p95: 0 });
});

test("calcularMediaEP95 calcula media e p95 sem mutar o vetor de entrada", () => {
  const amostras = new Float32Array([10, 2, 4, 8, 6, 12, 14, 16, 18, 20]);
  const copiaOriginal = amostras.slice();
  const scratch = new Float32Array(10);

  const { media, p95 } = calcularMediaEP95(amostras, 10, scratch);
  assert.equal(media, 11);
  assert.equal(p95, 20);
  assert.deepEqual(amostras, copiaOriginal);
});

test("classificarFrametime separa verde (60 FPS), amarelo (30 FPS) e vermelho", () => {
  assert.equal(classificarFrametime(8.3), "#10b981");
  assert.equal(classificarFrametime(16.67), "#10b981");
  assert.equal(classificarFrametime(20.0), "#ffbe0b");
  assert.equal(classificarFrametime(33.33), "#ffbe0b");
  assert.equal(classificarFrametime(40.0), "#ef4444");
});

function criarElStub(inicial = {}) {
  const classes = new Set();
  const attrs = new Map();
  const ouvintes = new Map();
  return {
    textContent: "",
    className: "",
    hidden: false,
    style: { transform: "" },
    ...inicial,
    classList: {
      toggle(c, forca) {
        const ativo = forca !== undefined ? Boolean(forca) : !classes.has(c);
        if (ativo) classes.add(c);
        else classes.delete(c);
        return ativo;
      },
      add(...cs) {
        cs.forEach((c) => classes.add(c));
      },
      remove(...cs) {
        cs.forEach((c) => classes.delete(c));
      },
      contains(c) {
        return classes.has(c);
      },
    },
    setAttribute(k, v) {
      attrs.set(k, String(v));
    },
    getAttribute(k) {
      return attrs.get(k);
    },
    addEventListener(tipo, fn) {
      ouvintes.set(tipo, fn);
    },
    disparar(tipo) {
      const fn = ouvintes.get(tipo);
      if (fn) fn();
    },
  };
}

test("criarHud alterna HUD/diagnostico, desenha grafico e atualiza metricas e audio", () => {
  const chamadasCanvas = [];
  const ctx2d = {
    fillStyle: "",
    clearRect(...a) {
      chamadasCanvas.push(["clearRect", a]);
    },
    fillRect(...a) {
      chamadasCanvas.push(["fillRect", a, this.fillStyle]);
    },
  };

  let diagToggle = null;
  const el = {
    canvasHudGrafico: { width: 160, height: 40, getContext: () => ctx2d },
    hudPerformance: criarElStub({ hidden: true }),
    btnHud: criarElStub(),
    btnToggleDiag: criarElStub(),
    painelDiag: criarElStub({ hidden: true }),
    diagSeta: criarElStub(),
    elHudFps: criarElStub(),
    elHudHz: criarElStub(),
    elHudFrameTime: criarElStub(),
    elHudWasmTime: criarElStub(),
    elDiagQuadro: criarElStub(),
    elDiagAmostras: criarElStub(),
    elDiagPico: criarElStub(),
    elBarraVolume: criarElStub(),
    elDiagPicoMax: criarElStub(),
    elDiagQuadrosSom: criarElStub(),
    elDiagFps: criarElStub(),
    elDiagTempoWasm: criarElStub(),
    elDiagWorklet: criarElStub(),
    elDiagPipeline: criarElStub(),
    elDiagEstado: criarElStub(),
    elDiagTaxa: criarElStub(),
    btnAudio: criarElStub(),
    badgeAudio: criarElStub(),
    onToggleDiag: (aberto) => {
      diagToggle = aberto;
    },
  };

  const hud = criarHud(el);
  assert.equal(hud.estaDiagAberto(), false);

  // Atualiza sem painel aberto e sem HUD visivel (no-op seguro).
  hud.atualizarDiag({ quadroNum: 1, lenAudio: 0, picoQuadro: 0, picoHistorico: 0, quadrosComSom: 0 }, null);
  hud.atualizarWorklet({ disponiveis: 0, falhas: 0, descartadas: 0 });
  assert.equal(el.elDiagQuadro.textContent, "");

  // Alternar HUD via clique no botao.
  el.btnHud.disparar("click");
  assert.equal(el.hudPerformance.hidden, false);
  assert.equal(el.btnHud.getAttribute("aria-pressed"), "true");
  assert.equal(el.btnHud.classList.contains("btn-azul"), true);

  // Registrar tempos e apresentacoes ao longo de >500ms para disparar FPS e Hz.
  hud.registrarTempoWasm(2.5);
  hud.registrarTempoWasm(3.5);
  hud.registrarApresentacao(1000, 16.0, 60);
  hud.registrarApresentacao(1500, 20.0, 90);
  hud.registrarApresentacao(2000, 38.0, 120);

  assert.match(el.elHudFps.textContent, /FPS$/);
  assert.match(el.elHudHz.textContent, /Hz sim$/);
  assert.equal(el.elHudFrameTime.textContent, "38.0 ms");
  assert.equal(el.elHudWasmTime.textContent, "3.5 ms");
  assert.ok(chamadasCanvas.some((c) => c[0] === "clearRect"));
  assert.ok(chamadasCanvas.some((c) => c[0] === "fillRect" && c[2] === "#ef4444"));

  // Abrir painel de diagnostico.
  el.btnToggleDiag.disparar("click");
  assert.equal(hud.estaDiagAberto(), true);
  assert.equal(diagToggle, true);
  assert.equal(el.diagSeta.textContent, "▲");
  assert.equal(el.btnToggleDiag.getAttribute("aria-expanded"), "true");

  // Atualizar diagnostico, worklet, pipeline e erro.
  hud.definirPipeline("AudioWorklet", "diag-ativo");
  assert.equal(el.elDiagPipeline.textContent, "AudioWorklet");
  assert.equal(el.elDiagPipeline.className, "diag-ativo");

  hud.definirErroWorklet("erro-teste");
  assert.equal(el.elDiagWorklet.textContent, "erro-teste");

  hud.atualizarWorklet({ disponiveis: 512, falhas: 1, descartadas: 2 });
  assert.equal(el.elDiagWorklet.textContent, "Fila: 512 amostras | Falhas: 1 | Descartadas: 2");

  const diag = {
    quadroNum: 120,
    lenAudio: 1600,
    picoQuadro: 0.5,
    picoHistorico: 0.95,
    quadrosComSom: 80,
  };
  hud.atualizarDiag(diag, null);
  assert.equal(el.elDiagQuadro.textContent, "120 (2.0s de jogo)");
  assert.equal(el.elDiagAmostras.textContent, "1600");
  assert.equal(el.elDiagPico.textContent, "0.5000");
  assert.equal(el.elBarraVolume.style.transform, "scaleX(0.500)");
  assert.equal(el.elDiagPicoMax.textContent, "0.9500");
  assert.equal(el.elDiagQuadrosSom.textContent, "80 quadros");
  assert.match(el.elDiagFps.textContent, /FPS/);
  assert.match(el.elDiagTempoWasm.textContent, /p95:/);
  assert.equal(el.elDiagEstado.textContent, "Não inicializado");
  assert.equal(el.badgeAudio.textContent, "Áudio inativo");

  // Estados de audio: running, suspended, closed.
  hud.atualizarEstadoAudio({ state: "running", sampleRate: 48000 });
  assert.equal(el.elDiagEstado.textContent, "Ativo (running)");
  assert.equal(el.btnAudio.textContent, "Áudio ativo");
  assert.equal(el.badgeAudio.textContent, "Áudio ativo");
  assert.equal(el.elDiagTaxa.textContent, "48000 Hz");

  hud.atualizarEstadoAudio({ state: "suspended", sampleRate: 44100 });
  assert.equal(el.elDiagEstado.textContent, "Suspenso pelo navegador");
  assert.equal(el.btnAudio.textContent, "Desbloquear áudio");
  assert.equal(el.badgeAudio.textContent, "Clique para ativar áudio");
  assert.equal(el.elDiagTaxa.textContent, "44100 Hz");

  hud.atualizarEstadoAudio({ state: "closed", sampleRate: 44100 });
  assert.equal(el.elDiagEstado.textContent, "closed");
  assert.equal(el.elDiagEstado.className, "diag-mudo");

  // Fechar diagnostico e resetar contadores.
  el.btnToggleDiag.disparar("click");
  assert.equal(hud.estaDiagAberto(), false);
  assert.equal(el.diagSeta.textContent, "▼");

  hud.resetar();
  hud.alternarHud();
  assert.equal(el.hudPerformance.hidden, true);
});

