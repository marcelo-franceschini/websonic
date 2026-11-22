import test from "node:test";
import assert from "node:assert/strict";
import { HistoricoEntradas, PoolBuffers } from "../buffers.js";

test("HistoricoEntradas cresce geometricamente, preserva dados e limpa sem realocar", () => {
  const hist = new HistoricoEntradas(16);
  assert.equal(hist.tamanho, 0);
  assert.equal(hist.capacidade, 16);

  for (let i = 0; i < 40; i++) {
    hist.registrar(i | 0x100);
  }
  assert.equal(hist.tamanho, 40);
  assert.ok(hist.capacidade >= 40);
  const fatia = hist.fatiaAtiva();
  assert.equal(fatia.length, 40);
  assert.equal(fatia[0], 0);
  assert.equal(fatia[39], 39);

  const capAntes = hist.capacidade;
  hist.limpar();
  assert.equal(hist.tamanho, 0);
  assert.equal(hist.capacidade, capAntes);
});

test("HistoricoEntradas.garantirCapacidade pre-reserva para replays longos", () => {
  const hist = new HistoricoEntradas(16);
  hist.registrar(0xAB);
  hist.garantirCapacidade(1000);
  assert.ok(hist.capacidade >= 1000);
  assert.equal(hist.tamanho, 1);
  assert.equal(hist.fatiaAtiva()[0], 0xAB);
});

test("PoolBuffers recicla buffers dentro do limite e respeita tamanho exato/minimo", () => {
  const pool = new PoolBuffers(Float32Array, 2, 128);
  assert.equal(pool.disponiveis, 2);

  const a = pool.adquirir(64);
  assert.equal(a.length, 128);
  // Pedir 256 nao descarta o buffer de 128 remanescente no pool.
  const b = pool.adquirir(256);
  assert.equal(b.length, 256);
  assert.equal(pool.disponiveis, 1);

  const c = pool.adquirir(128);
  assert.equal(c.length, 128);
  assert.equal(pool.disponiveis, 0);

  assert.equal(pool.devolverBuffer(a.buffer), true);
  assert.equal(pool.devolverBuffer(b.buffer), true);
  // Terceira devolucao acima do limite e descartada.
  assert.equal(pool.devolverBuffer(c.buffer), false);
  assert.equal(pool.disponiveis, 2);

  // Modo exato aloca novo quando tamanho difere.
  const exato = pool.adquirir(32, true);
  assert.equal(exato.length, 32);
  // Buffers originais permanecem preservados no pool apos falha de busca.
  assert.equal(pool.disponiveis, 2);
  assert.equal(pool.devolverBuffer(null), false);
});

test("HistoricoEntradas trata capacidade padrao e garantirCapacidade sem realocar quando suficiente", () => {
  const padrao = new HistoricoEntradas();
  assert.equal(padrao.capacidade, 36000);

  const vazio = new HistoricoEntradas(16);
  vazio.garantirCapacidade(8);
  assert.equal(vazio.capacidade, 16);
  vazio.garantirCapacidade(64);
  assert.equal(vazio.capacidade, 64);
  assert.equal(vazio.tamanho, 0);
});

test("PoolBuffers.adquirir usa swap-pop e preserva buffers menores sem descarte", () => {
  const pool = new PoolBuffers(Uint16Array, 3, 16);
  // Esvazia o pool padrao
  pool.adquirir();
  pool.adquirir();
  pool.adquirir();
  assert.equal(pool.disponiveis, 0);

  const grande = new Uint16Array(64);
  const pequeno = new Uint16Array(16);
  assert.equal(pool.devolverBuffer(grande.buffer), true);
  assert.equal(pool.devolverBuffer(pequeno.buffer), true);

  // Pedir 64 encontra o item no indice 0 e faz swap-pop preservando o de 16.
  const obtidoGrande = pool.adquirir(64, true);
  assert.equal(obtidoGrande.length, 64);
  assert.equal(pool.disponiveis, 1);

  const obtidoPequeno = pool.adquirir(16, true);
  assert.equal(obtidoPequeno.length, 16);
  assert.equal(pool.disponiveis, 0);
});

