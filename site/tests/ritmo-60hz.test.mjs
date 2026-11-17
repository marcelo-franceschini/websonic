// O relogio do site contra vsync com jitter nao cumulativo de +-1 ms, por 10 minutos.
// Requisito: logica a 60 Hz exatos (docs/requisitos.md, Plataforma e restricoes).
import { test } from "node:test";
import assert from "node:assert/strict";
import { Relogio } from "../relogio.js";

function simular(hz, minutos = 10) {
  const r = new Relogio();
  r.iniciar(0);
  let semente = 7, zero = 0, dois = 0;
  const aleatorio = () => { semente = (semente * 1103515245 + 12345) % 2147483648; return semente / 2147483648; };
  for (let k = 1; k <= hz * 60 * minutos; k++) {
    const passos = r.avancar(k * 1000 / hz + (aleatorio() * 2 - 1));
    if (passos === 0) zero++;
    if (passos >= 2) dois++;
  }
  return { zero, dois };
}

test("monitor de 60,000 Hz: nenhum quadro repetido nem pulado em 10 minutos", () => {
  const { zero, dois } = simular(60);
  assert.equal(zero, 0, `${zero} ticks sem passo`);
  assert.equal(dois, 0, `${dois} ticks com 2 ou mais passos`);
});

test("monitor de 59,94 Hz: no maximo 4 ticks com 2 passos por minuto (batimento inevitavel)", () => {
  const { zero, dois } = simular(59.94);
  assert.equal(zero, 0, `${zero} ticks sem passo`);
  assert.ok(dois <= 40, `${dois} ticks com 2 ou mais passos em 10 minutos`);
});
