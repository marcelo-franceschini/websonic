// Tempo de parede de cada quadro do Wasm real do site (dist/pkg), na partida-ghz1 com a ROM.
// Pula sem a ROM ou sem o dist. Rode `cargo xtask site` antes, para medir o codigo atual.
import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const raiz = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const caminhoJs = path.join(raiz, "dist/pkg/sonic1_web.js");
const caminhoWasm = path.join(raiz, "dist/pkg/sonic1_web_bg.wasm");
const caminhoRom = path.join(raiz, "sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen");

test("quadro do Wasm cabe no orcamento de 16,67 ms na partida-ghz1", async (t) => {
  if (!fs.existsSync(caminhoRom) || !fs.existsSync(caminhoWasm)) return t.skip("sem ROM ou sem dist");
  const { default: init, Jogo } = await import(caminhoJs);
  await init({ module_or_path: fs.readFileSync(caminhoWasm) });
  const jogo = Jogo.criar_com_audio(fs.readFileSync(caminhoRom), 48000);
  // Mesmo modo de video que o site usa hoje. Depois do item V1a esta funcao nao existe mais.
  if (typeof jogo.ativar_gpu_sem_referencia === "function") jogo.ativar_gpu_sem_referencia();
  const entradas = fs.readFileSync(path.join(raiz, "corpus/partida-ghz1.s1in"), "utf8")
    .split("\n").map((s) => s.trim()).filter((s) => s && !s.startsWith("#")).map((s) => parseInt(s, 16));
  const tempos = new Float64Array(entradas.length);
  for (let i = 0; i < entradas.length; i++) {
    const inicio = performance.now();
    jogo.ocupacao_audio(2400);
    jogo.botoes(entradas[i]);
    jogo.simular_quadro();
    tempos[i] = performance.now() - inicio;
  }
  const ordem = Float64Array.from(tempos).sort();
  const media = tempos.reduce((a, b) => a + b, 0) / tempos.length;
  const p95 = ordem[Math.round((ordem.length - 1) * 0.95)];
  const acima = tempos.filter((x) => x > 16.67).length / tempos.length;
  console.log(`wasm partida-ghz1: media=${media.toFixed(2)} ms p95=${p95.toFixed(2)} ms acima_16_67=${(100 * acima).toFixed(1)}%`);
  assert.ok(media <= 8.0, `media ${media.toFixed(2)} ms (limite 8,0)`);
  assert.ok(p95 <= 13.0, `p95 ${p95.toFixed(2)} ms (limite 13,0)`);
  assert.ok(acima <= 0.02, `${(100 * acima).toFixed(1)}% dos quadros acima de 16,67 ms (limite 2%)`);
  jogo.free();
});
