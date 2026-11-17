import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { Relogio, PERIODO_QUADRO } from "../relogio.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Modelo de AudioWorkletProcessor para auditoria automatizada sem browser
class AudioWorkletAudit {
  constructor(sampleRate = 48000) {
    this.sampleRate = sampleRate;
    this.tam = 16384;
    this.bufL = new Float32Array(this.tam);
    this.bufR = new Float32Array(this.tam);
    this.leitura = 0;
    this.escrita = 0;
    this.disponiveis = 0;
    this.falhas = 0;
    this.descartadas = 0;
    this.iniciado = false;
    this.preenchimentoInicial = Math.ceil(sampleRate * 3 * 896040 / 53693175);
    this.preenchimentoRetomada = 128;
    this.tevePrimeiraPartida = false;
    this.ultimoL = 0;
    this.ultimoR = 0;
  }

  enviar(dados) {
    const pares = dados.length >> 1;
    for (let i = 0; i < pares; i++) {
      if (this.disponiveis < this.tam) {
        this.bufL[this.escrita] = dados[i * 2];
        this.bufR[this.escrita] = dados[i * 2 + 1];
        this.escrita = (this.escrita + 1) % this.tam;
        this.disponiveis++;
      } else {
        this.descartadas++;
      }
    }
  }

  consumirBloco() {
    const limiar = this.tevePrimeiraPartida ? this.preenchimentoRetomada : this.preenchimentoInicial;
    if (!this.iniciado && this.disponiveis >= limiar) {
      this.iniciado = true;
      this.tevePrimeiraPartida = true;
    }
    const n = 128;
    for (let i = 0; i < n; i++) {
      if (this.iniciado && this.disponiveis > 0) {
        this.ultimoL = this.bufL[this.leitura];
        this.ultimoR = this.bufR[this.leitura];
        this.leitura = (this.leitura + 1) % this.tam;
        this.disponiveis--;
      } else if (this.iniciado) {
        this.falhas++;
        this.iniciado = false;
        this.ultimoL *= 0.95;
        this.ultimoR *= 0.95;
      }
    }
  }
}

test("Auditoria de Pacing: relogio estabiliza V-Sync a 60Hz sob jitter sem quedas espurias", () => {
  const relogio = new Relogio();
  relogio.iniciar(0);

  let dist = {};
  let totalPassos = 0;
  let tempo = 0;
  let s = 54321;
  function rand() { s = (s * 1103515245 + 12345) & 0x7fffffff; return (s / 0x7fffffff); }

  const totalFrames = 600; // ~10 segundos a 60Hz
  for (let i = 0; i < totalFrames; i++) {
    // Simula jitter tipico de requestAnimationFrame (+-1.5ms)
    const delta = 16.666 + (rand() - 0.5) * 3.0;
    tempo += delta;
    const p = relogio.avancar(tempo);
    dist[p] = (dist[p] || 0) + 1;
    totalPassos += p;
  }

  // Com a janela de estabilizacao, pelo menos 98% dos quadros devem ter exatamente 1 passo
  const taxaExata = (dist[1] || 0) / totalFrames;
  assert.ok(taxaExata >= 0.98, `Taxa de quadros perfeitamente estaveis deve ser >= 98%: ${(taxaExata * 100).toFixed(1)}%`);
  assert.ok((dist[0] || 0) <= 6, `Quadros com 0 passos devem ser minimos sob jitter: ${dist[0] || 0}`);
  assert.ok((dist[2] || 0) <= 6, `Quadros duplos devem ser minimos sob jitter: ${dist[2] || 0}`);
});

test("Auditoria de Audio e Simulacao Web: fluxo contínuo sem underrun, sem clipping e sem estalos", async () => {
  const caminhoWasm = path.resolve(__dirname, "../../dist/pkg/sonic1_web_bg.wasm");
  const caminhoJs = path.resolve(__dirname, "../../dist/pkg/sonic1_web.js");
  const caminhoRom = path.resolve(__dirname, "../../sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen");

  if (!fs.existsSync(caminhoRom) || !fs.existsSync(caminhoWasm)) {
    return; // Pula em ambiente sem ROM
  }

  const { default: init, Jogo } = await import(caminhoJs);
  const wasmBuffer = fs.readFileSync(caminhoWasm);
  await init({ module_or_path: wasmBuffer });

  const rom = fs.readFileSync(caminhoRom);
  const taxaAudio = 48000;
  const jogo = Jogo.criar_com_audio(rom, taxaAudio);
  const worklet = new AudioWorkletAudit(taxaAudio);

  let tempoAudio = 0;
  let tempoVisual = 0;
  const relogio = new Relogio();
  relogio.iniciar(0);

  let maxPico = 0;
  let totalDiscontinuidades = 0;
  let ultimoL = 0;
  let ultimoR = 0;

  let s = 12345;
  function rand() { s = (s * 1103515245 + 12345) & 0x7fffffff; return (s / 0x7fffffff); }

  // Simula 750 quadros (~12.5 segundos, passando da abertura ate o gameplay ativo)
  for (let f = 0; f < 750; f++) {
    const deltaRaf = 16.666 + (rand() - 0.5) * 3.0;
    tempoVisual += deltaRaf;

    const passos = relogio.avancar(tempoVisual);

    for (let p = 0; p < passos; p++) {
      const modo = jogo.modo_jogo();

      // Pressiona Start na tela de titulo
      if (modo === 0x04) {
        jogo.botoes(0x80);
      } else if (modo === 0x0C) {
        // No gameplay, segura Direita (bit 0x08)
        jogo.botoes(0x08);
      } else {
        jogo.botoes(0);
      }

      jogo.simular_quadro();

      const audio = jogo.audio();
      if (audio.length > 0) {
        worklet.enviar(audio);

        for (let i = 0; i < audio.length; i += 2) {
          const l = audio[i];
          const r = audio[i + 1];
          const absL = Math.abs(l);
          const absR = Math.abs(r);
          if (absL > maxPico) maxPico = absL;
          if (absR > maxPico) maxPico = absR;

          // Detecta saltos descontinuos que indicam estalo/click
          const dL = Math.abs(l - ultimoL);
          const dR = Math.abs(r - ultimoR);
          if (dL > 0.6 || dR > 0.6) {
            totalDiscontinuidades++;
          }
          ultimoL = l;
          ultimoR = r;
        }
      }
    }

    // AudioWorklet consome amostras em tempo real a 48kHz
    while (tempoAudio + (128 / 48) <= tempoVisual) {
      worklet.consumirBloco();
      tempoAudio += (128 / 48);
      jogo.ocupacao_audio(worklet.disponiveis);
    }
  }

  assert.equal(worklet.falhas, 0, `Nao deve haver underrun de audio: falhas=${worklet.falhas}`);
  assert.equal(worklet.descartadas, 0, `Nao deve haver descarte de amostras por fila cheia: descartadas=${worklet.descartadas}`);
  assert.ok(worklet.disponiveis > 500 && worklet.disponiveis < 5000, `Buffer deve se manter estavel proximo de ~2400: obtido=${worklet.disponiveis}`);
  assert.ok(maxPico <= 1.0, `Audio nao deve clipar (maxPico <= 1.0): obtido=${maxPico}`);
  assert.equal(totalDiscontinuidades, 0, `Nao deve haver estalos no sinal de audio: ${totalDiscontinuidades}`);
  assert.equal(jogo.modo_jogo(), 0x0C, "Deve alcancar o modo de gameplay ativo (0x0C)");
});
