import { test } from "node:test";
import assert from "node:assert/strict";
import { Relogio, PERIODO_QUADRO, interpolarLinear, interpolarPosicao } from "../relogio.js";

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

test("interpolação temporal de movimento gera alpha suave para 120Hz, 144Hz e 240Hz", () => {
  // Monitor de 120Hz: avança a cada ~8.33ms (aproximadamente meio quadro NTSC de 16.69ms).
  const relogio120 = new Relogio();
  relogio120.iniciar(0);
  const passo1 = relogio120.avancar(1000 / 120);
  assert.equal(passo1, 0, "No primeiro quadro de 120Hz a simulação ainda não dá passo completo");
  assert.ok(relogio120.alpha >= 0.49 && relogio120.alpha <= 0.51, `Alpha esperado proximo de 0.5, obtido: ${relogio120.alpha}`);

  // Monitor de 240Hz: passos intermediarios a ~4.17ms.
  const relogio240 = new Relogio();
  relogio240.iniciar(0);
  const alphas = [];
  for (let i = 1; i <= 4; i++) {
    relogio240.avancar(i * 1000 / 240);
    alphas.push(relogio240.alpha);
    assert.ok(relogio240.alpha >= 0 && relogio240.alpha <= 1, `Alpha deve estar entre 0 e 1: ${relogio240.alpha}`);
  }
  assert.ok(alphas[0] > 0.20 && alphas[0] < 0.30, `Primeiro quarto ~0.25: ${alphas[0]}`);
  assert.ok(alphas[1] > 0.45 && alphas[1] < 0.55, `Segundo quarto ~0.50: ${alphas[1]}`);
  assert.ok(alphas[2] > 0.70 && alphas[2] < 0.80, `Terceiro quarto ~0.75: ${alphas[2]}`);

  // Monitor de 144Hz: confere que todos os alphas permanecem estritamente normalizados no intervalo [0, 1].
  const relogio144 = new Relogio();
  relogio144.iniciar(0);
  for (let i = 1; i <= 144; i++) {
    const res = relogio144.passosEAlpha(i * 1000 / 144);
    assert.ok(res.alpha >= 0 && res.alpha <= 1);
    assert.equal(res.alpha, relogio144.obterAlpha());
    assert.equal(res.alpha, relogio144.fase);
  }
});

test("funções de interpolação interpolam escalares e vetores de posição", () => {
  const relogio = new Relogio();
  relogio.iniciar(0);
  // Simula avanço de meio quadro.
  relogio.avancar(PERIODO_QUADRO * 0.5);
  const posInterpolada = relogio.interpolar(100, 200);
  assert.ok(Math.abs(posInterpolada - 150) < 0.001, `Valor esperado 150, obtido: ${posInterpolada}`);

  const linear = interpolarLinear(50, 150, 0.25);
  assert.equal(linear, 75);

  const vetor = interpolarPosicao([10, 20], [30, 60], 0.5);
  assert.deepEqual(vetor, [20, 40]);
});
