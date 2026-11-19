// Guarda do Wasm real do site (dist/pkg): hashes agregados da imagem de 9 bits e do áudio de saída,
// quadro a quadro, nas três entradas do corpus, no formato da guarda_rom nativa (FNV-1a de 64 bits).
// A imagem bate com a nativa. O áudio difere do nativo porque seno e cosseno da tabela sinc vêm de
// outra libm no Wasm (docs/relatorio-performance-2.md, seção 3.1): por isso os valores são próprios.
// Pula sem a ROM ou sem o dist. Rode `cargo xtask site` antes.
import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const raiz = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const caminhoJs = path.join(raiz, "dist/pkg/sonic1_web.js");
const caminhoWasm = path.join(raiz, "dist/pkg/sonic1_web_bg.wasm");
const caminhoRom = path.join(raiz, "sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen");

// FNV-1a de 64 bits em duas metades de 32 bits. Os produtos ficam abaixo de 2^42: exatos em double.
function fnvByte(h, b) {
  const lo = (h[0] ^ b) >>> 0;
  const p = lo * 0x1b3;
  h[1] = (h[1] * 0x1b3 + Math.floor(p / 4294967296) + ((lo << 8) >>> 0)) % 4294967296;
  h[0] = p % 4294967296;
}
const hex = (h) => h[1].toString(16).padStart(8, "0") + h[0].toString(16).padStart(8, "0");

function lerS1in(nome) {
  return fs.readFileSync(path.join(raiz, "corpus", nome), "utf8").split("\n").map((s) => s.trim())
    .filter((s) => s && !s.startsWith("#")).map((s) => parseInt(s, 16));
}

test("Wasm do site: imagem e áudio iguais aos valores medidos nas três entradas", async (t) => {
  if (!fs.existsSync(caminhoRom) || !fs.existsSync(caminhoWasm)) return t.skip("sem ROM ou sem dist");
  const { default: init, Jogo } = await import(caminhoJs);
  const wasm = await init({ module_or_path: fs.readFileSync(caminhoWasm) });
  const rom = fs.readFileSync(caminhoRom);
  // Valores medidos em 18/11/2026 no HEAD 2786b6e. A imagem é igual à da guarda_rom nativa.
  const casos = [
    ["boot-demos", new Array(7200).fill(0), "c42c937482ce693b", "fc9b2dfbec403b2f"],
    ["partida-ghz1", lerS1in("partida-ghz1.s1in"), "60e258d3c5e554fc", "4b3cd0e30c6c5284"],
    ["lz-agua", lerS1in("lz-agua.s1in"), "40ba0d3e3b00edc7", "0e5bf97f60c5e428"],
  ];
  for (const [nome, entradas, imagemEsperada, audioEsperado] of casos) {
    const jogo = Jogo.criar_com_audio(rom, 48000);
    const img = [0x84222325, 0xcbf29ce4], aud = [0x84222325, 0xcbf29ce4];
    for (const b of entradas) {
      jogo.ocupacao_audio(2400);
      jogo.botoes(b);
      jogo.simular_quadro();
      const u = new Uint32Array(wasm.memory.buffer, jogo.audio_ptr(), jogo.audio_len());
      for (let i = 0; i < u.length; i++) {
        const x = u[i];
        fnvByte(aud, x & 255); fnvByte(aud, (x >>> 8) & 255); fnvByte(aud, (x >>> 16) & 255); fnvByte(aud, x >>> 24);
      }
      // hash_da_imagem do quadro (u64 em hexadecimal) dobrado em little-endian, como a guarda nativa.
      const q = jogo.hash_da_imagem();
      for (const w of [parseInt(q.slice(8), 16), parseInt(q.slice(0, 8), 16)]) {
        for (let k = 0; k < 4; k++) fnvByte(img, (w >>> (8 * k)) & 255);
      }
    }
    jogo.free();
    assert.equal(hex(img), imagemEsperada, `${nome}: imagem mudou`);
    assert.equal(hex(aud), audioEsperado, `${nome}: áudio mudou`);
  }
});
