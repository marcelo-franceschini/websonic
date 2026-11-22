import { test } from "node:test";
import assert from "node:assert/strict";
import { Relogio, PERIODO_QUADRO } from "../relogio.js";

test("simulação mantém o ritmo NTSC em diferentes monitores", () => {
  const esperado = Math.floor(10000 / PERIODO_QUADRO);
  for (const hz of [60, 120, 144, 240]) {
    const relogio = new Relogio();
    relogio.iniciar(0);
    let quadros = 0;
    for (let i = 1; i <= hz * 10; i++) quadros += relogio.avancar(i * 1000 / hz);
    assert.equal(quadros, esperado, `${hz} Hz`);
  }
});

test("atrasos são recuperados sem saltar tempo e pausas não avançam o jogo", () => {
  const relogio = new Relogio();
  relogio.iniciar(0);
  assert.equal(relogio.avancar(PERIODO_QUADRO * 10), 4);
  assert.equal(relogio.avancar(PERIODO_QUADRO * 10), 4);
  assert.equal(relogio.avancar(PERIODO_QUADRO * 10), 2);
  relogio.retomar(100000);
  assert.equal(relogio.avancar(100000), 0);
});

test("janela de 0,8 quadro: a 240 Hz, 3 ticks sem passo e o 4o com 1 passo", () => {
  // Com 0,75 no lugar de 0,8, tres ticks de 240 Hz somam 0,75 quadro e disparam um passo cedo.
  const r = new Relogio();
  r.iniciar(0);
  const passos = [1, 2, 3, 4].map((i) => r.avancar(i * 1000 / 240));
  assert.deepEqual(passos, [0, 0, 0, 1]);
});

test("rAF com pausas e vigia: 60 passos por segundo, sem contar tempo duas vezes", () => {
  const r = new Relogio();
  r.iniciar(0);
  let total = 0, naPausa = 0, ultimoRaf = 0;
  // 10 s de rAF a 165 Hz, com uma pausa de 100 ms a cada 500 ms. O vigia (site/app.js) roda a
  // cada 8,33 ms e le o performance.now() 12 ms depois da grade do rAF: o carimbo do rAF
  // seguinte fica no passado. Sem o tempo monotonico, o relogio conta 608 passos.
  const pausado = (t) => t % 500 > 400;
  for (let t = 0; t <= 10000; t += 1000 / 120) {
    const raf = Math.floor(t / (1000 / 165)) * (1000 / 165);
    if (raf > ultimoRaf && !pausado(raf)) {
      total += r.avancar(raf);
      ultimoRaf = raf;
    }
    if (t - ultimoRaf > 2 * PERIODO_QUADRO) {
      const n = r.avancar(t + 12);
      total += n;
      naPausa += n;
    }
  }
  assert.ok(Math.abs(total - 600) <= 1, `${total} passos em 10 s`);
  assert.ok(naPausa >= 60, `${naPausa} passos durante as pausas`);
});

test("trava longo e limitado a 10 quadros de acumulado para evitar espiral da morte", () => {
  const r = new Relogio();
  r.iniciar(0);
  // Salto de 25 quadros deve ser limitado a 10 quadros no acumulador (4 + 4 + 2 = 10).
  assert.equal(r.avancar(PERIODO_QUADRO * 25), 4);
  assert.equal(r.avancar(PERIODO_QUADRO * 25), 4);
  assert.equal(r.avancar(PERIODO_QUADRO * 25), 2);
  assert.equal(r.avancar(PERIODO_QUADRO * 25), 0);
});


