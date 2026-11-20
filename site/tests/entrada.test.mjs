// Testes unitarios da logica pura de entrada (teclado, gamepad, toque e cancelamento de opostas).
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  calcularMascaraTeclado,
  calcularMascaraGamepad,
  calcularMascaraToque,
  combinarMascaras,
  cancelarOpostas,
  ehModoTituloOuDemo,
} from "../entrada.js";

test("ehModoTituloOuDemo identifica Sega, Titulo e Demo", () => {
  assert.equal(ehModoTituloOuDemo(0x00), true);
  assert.equal(ehModoTituloOuDemo(0x04), true);
  assert.equal(ehModoTituloOuDemo(0x08), true);
  assert.equal(ehModoTituloOuDemo(0x0C), false);
  assert.equal(ehModoTituloOuDemo(0x10), false);
});

test("teclado mapeia Space para Start no titulo e para pulo (C) na fase", () => {
  const teclas = new Set(["Space"]);
  assert.equal(calcularMascaraTeclado(teclas, 0x04), 0x80);
  assert.equal(calcularMascaraTeclado(teclas, 0x00), 0x80);
  assert.equal(calcularMascaraTeclado(teclas, 0x0C), 0x20);
});

test("cancelamento de direcoes opostas funciona em teclado e na combinacao", () => {
  assert.equal(calcularMascaraTeclado(new Set(["ArrowLeft", "ArrowRight"]), 0x0C), 0x00);
  assert.equal(calcularMascaraTeclado(new Set(["KeyW", "KeyS", "KeyZ"]), 0x0C), 0x40);
  assert.equal(cancelarOpostas(0x0F), 0x00);
  // Esquerda no teclado + Direita no toque devem se cancelar na combinacao final.
  assert.equal(combinarMascaras(0x04, 0x00, 0x08 | 0x20), 0x20);
});

test("gamepad le D-pad, analogico e botoes de acao (e mapeia acao para Start no titulo)", () => {
  const botoes = Array.from({ length: 17 }, () => ({ pressed: false, value: 0 }));
  botoes[0] = { pressed: true, value: 1 }; // A -> B (0x10)
  const gp = [{ connected: true, buttons: botoes, axes: [0.8, 0] }];

  assert.equal(calcularMascaraGamepad(gp, 0x0C), 0x08 | 0x10);
  assert.equal(calcularMascaraGamepad(gp, 0x04), 0x08 | 0x10 | 0x80);
});

test("toque mapeia direcoes, botoes e Start no titulo", () => {
  const toques = new Set(["direita", "a"]);
  assert.equal(calcularMascaraToque(toques, 0x0C), 0x08 | 0x40);
  assert.equal(calcularMascaraToque(toques, 0x04), 0x08 | 0x40 | 0x80);
});
