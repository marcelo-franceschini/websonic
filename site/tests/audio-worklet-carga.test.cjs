// Carga do SonicAudioProcessor na thread de audio: mensagens por segundo e faltas com entrada estavel.
const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

test("worklet envia no maximo 16 mensagens por segundo e nao falha com entrada estavel", () => {
  const ctx = { sampleRate: 48000, mensagens: 0 };
  // Conta so as mensagens de status. Depois do item W2, o worklet tambem devolve buffers de audio.
  ctx.AudioWorkletProcessor = class { constructor() { this.port = { postMessage: (m) => { if (m && m.tipo === "status") ctx.mensagens++; } }; } };
  ctx.registerProcessor = (_nome, Classe) => { ctx.Classe = Classe; };
  vm.createContext(ctx);
  vm.runInContext(fs.readFileSync(path.join(__dirname, "../audio-worklet.js"), "utf8"), ctx);
  const p = new ctx.Classe();
  const L = new Float32Array(128), R = new Float32Array(128);
  const periodo = 1000 * 896040 / 53693175;
  let tAudio = 0, tQuadro = 0, fracao = 0;
  const segundos = 10;
  while (tAudio < segundos * 1000) {
    while (tQuadro <= tAudio) {
      fracao += 48000 * periodo / 1000;
      const pares = Math.floor(fracao); fracao -= pares;
      p.port.onmessage({ data: new Float32Array(pares * 2).fill(0.1) });
      tQuadro += periodo;
    }
    p.process([], [[L, R]]);
    tAudio += 128 * 1000 / 48000;
  }
  assert.ok(ctx.mensagens <= 16 * segundos, `${ctx.mensagens} mensagens em ${segundos} s`);
  assert.equal(p.falhas, 0);
});
