// Teste do modo de porta direta (MessageChannel) no SonicAudioProcessor.
const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const path = require("node:path");

function carregarProcessor() {
  let Processor;
  class WorkletBase {
    constructor() {
      this.port = {
        mensagens: [],
        onmessage: null,
        postMessage(msg) {
          this.mensagens.push(msg);
        },
      };
    }
  }
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../audio-worklet.js"), "utf8"), {
    AudioWorkletProcessor: WorkletBase,
    registerProcessor: (_nome, classe) => { Processor = classe; },
    sampleRate: 48000,
    Float32Array,
    Math,
  });
  return Processor;
}

test("porta direta do MessageChannel recebe audio, devolve buffer e emite status", () => {
  const Processor = carregarProcessor();
  const p = new Processor();

  const recebidasPorta = [];
  const portaDireta = {
    onmessage: null,
    postMessage(msg) {
      recebidasPorta.push(msg);
    },
    close() {},
  };

  p.port.onmessage({ data: { tipo: "porta", porta: portaDireta } });
  assert.equal(typeof portaDireta.onmessage, "function");

  // Envia bloco de audio pela porta direta (4000 pares > 24 * 128 = 3072 pares)
  const bloco = new Float32Array(8000).fill(0.25);
  portaDireta.onmessage({ data: bloco });

  // Buffer deve ter sido devolvido pela porta direta, nao pela porta principal
  assert.equal(p.port.mensagens.length, 0);
  assert.equal(recebidasPorta.length, 1);
  assert.equal(recebidasPorta[0], bloco.buffer);

  // Roda 24 blocos de 128 amostras para disparar mensagem de status
  const canais = [[new Float32Array(128), new Float32Array(128)]];
  for (let i = 0; i < 24; i++) {
    p.process([], canais);
  }
  assert.equal(canais[0][0][0], 0.25);

  const status = recebidasPorta.find((m) => m && m.tipo === "status");
  assert.ok(status, "status deve chegar pela porta direta");
  assert.equal(status.falhas, 0);
  assert.equal(p.port.mensagens.length, 0);

  // Reset pela porta direta zera fila
  portaDireta.onmessage({ data: { tipo: "reset" } });
  assert.equal(p.disponiveis, 0);
  assert.equal(p.iniciado, false);
});
