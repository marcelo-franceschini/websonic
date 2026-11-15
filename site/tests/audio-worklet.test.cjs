const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const path = require("node:path");

let Processor;
class WorkletBase {
  constructor() {
    this.port = { postMessage() {}, onmessage: null };
  }
}
vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../audio-worklet.js"), "utf8"), {
  AudioWorkletProcessor: WorkletBase,
  registerProcessor: (_nome, classe) => { Processor = classe; },
  sampleRate: 48000,
  Float32Array,
  Math,
});

test("aguarda preenchimento, reproduz e reinicia após falta", () => {
  const p = new Processor();
  const canais = [[new Float32Array(128), new Float32Array(128)]];
  p.process([], canais);
  assert.equal(p.falhas, 0);
  assert.equal(p.iniciado, false);

  p.port.onmessage({ data: new Float32Array(5000).fill(0.5) });
  p.process([], canais);
  assert.equal(canais[0][0][0], 0.5);
  assert.equal(p.iniciado, true);
  while (p.iniciado) p.process([], canais);
  assert.equal(p.falhas, 1);
  assert.ok(canais[0][0][69] > 0);
  assert.ok(canais[0][0][70] < canais[0][0][69]);
  p.process([], canais);
  assert.equal(p.falhas, 1);

  p.port.onmessage({ data: { tipo: "reset" } });
  assert.equal(p.disponiveis, 0);
  assert.equal(p.falhas, 0);
});

test("contabiliza amostras descartadas quando a fila está cheia", () => {
  const p = new Processor();
  p.port.onmessage({ data: new Float32Array((p.tam + 17) * 2) });
  assert.equal(p.disponiveis, p.tam);
  assert.equal(p.descartadas, 17);
});
