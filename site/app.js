// Cola mínima entre a página e o WebAssembly. A lógica fica em Rust (crate sonic1-rom).
import init, { identificar, Jogo } from "./pkg/sonic1_web.js";

const LIMITE_BYTES = 16 * 1024 * 1024;

const entrada = document.getElementById("arquivo");
const caixa = document.getElementById("resultado");
const btnAudio = document.getElementById("btn-audio");
const btnBipe = document.getElementById("btn-bipe");

// Elementos de Diagnóstico
const elDiagEstado = document.getElementById("diag-estado");
const elDiagTaxa = document.getElementById("diag-taxa");
const elDiagPipeline = document.getElementById("diag-pipeline");
const elDiagQuadro = document.getElementById("diag-quadro");
const elDiagAmostras = document.getElementById("diag-amostras");
const elDiagPico = document.getElementById("diag-pico");
const elBarraVolume = document.getElementById("barra-volume");
const elDiagPicoMax = document.getElementById("diag-pico-max");
const elDiagQuadrosSom = document.getElementById("diag-quadros-som");
const elDiagWorklet = document.getElementById("diag-worklet");

let audioCtx = null;
let workletNode = null;
let fallbackNode = null;

let quadroNum = 0;
let picoHistorico = 0;
let quadrosComSom = 0;

function mostrar(valida, veredito, sha1, tamanho) {
  caixa.hidden = false;
  caixa.className = valida ? "ok" : "erro";
  document.getElementById("veredito").textContent = veredito;
  document.getElementById("sha1").textContent = sha1;
  document.getElementById("tamanho").textContent = String(tamanho);
}

await init();
entrada.disabled = false;

// 1. Teste de Bipe (Web Audio puro)
btnBipe.addEventListener("click", () => {
  try {
    if (!audioCtx) {
      const AudioContextClass = window.AudioContext || window.webkitAudioContext;
      audioCtx = new AudioContextClass();
    }
    audioCtx.resume().then(() => {
      atualizarEstadoAudioUI();
      const osc = audioCtx.createOscillator();
      const ganho = audioCtx.createGain();
      osc.type = "sine";
      osc.frequency.setValueAtTime(440, audioCtx.currentTime); // La 440 Hz
      ganho.gain.setValueAtTime(0.25, audioCtx.currentTime);
      ganho.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.4);
      osc.connect(ganho);
      ganho.connect(audioCtx.destination);
      osc.start();
      osc.stop(audioCtx.currentTime + 0.4);
    });
  } catch (e) {
    alert("Erro ao tocar bipe de teste: " + e.message);
  }
});

function atualizarEstadoAudioUI() {
  if (!audioCtx) {
    elDiagEstado.innerHTML = `<span style="color:#666;">Não inicializado</span>`;
    return;
  }
  const st = audioCtx.state;
  if (st === "running") {
    elDiagEstado.innerHTML = `<span style="color:#1a7f37; font-weight:bold;">🟢 Ativo (running)</span>`;
    btnAudio.textContent = "🔊 Áudio Ativo";
    btnAudio.style.background = "#1a7f37";
  } else if (st === "suspended") {
    elDiagEstado.innerHTML = `<span style="color:#b3261e; font-weight:bold;">🟡 Pausado pelo navegador (suspended)</span>`;
    btnAudio.textContent = "⚠️ Clique para Desbloquear Áudio";
    btnAudio.style.background = "#b3261e";
  } else {
    elDiagEstado.innerHTML = `<span style="color:#666;">${st}</span>`;
  }
  elDiagTaxa.textContent = audioCtx.sampleRate + " Hz";
}

function desbloquearAudio() {
  if (audioCtx && audioCtx.state === "suspended") {
    audioCtx.resume().then(atualizarEstadoAudioUI).catch(() => {});
  }
}

btnAudio.addEventListener("click", () => {
  desbloquearAudio();
});

async function iniciarAudio() {
  if (audioCtx && (workletNode || fallbackNode)) return;
  try {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    if (!audioCtx) {
      try {
        audioCtx = new AudioContextClass({ sampleRate: 48000 });
      } catch {
        audioCtx = new AudioContextClass();
      }
    }

    try {
      await audioCtx.audioWorklet.addModule("audio-worklet.js?v=4");
      try {
        workletNode = new AudioWorkletNode(audioCtx, "sonic-audio-processor", {
          outputChannelCount: [2],
        });
      } catch {
        workletNode = new AudioWorkletNode(audioCtx, "sonic-audio-processor");
      }
      workletNode.port.onmessage = (e) => {
        if (e.data && e.data.tipo === "status") {
          elDiagWorklet.textContent = `Fila: ${e.data.disponiveis} amostras | Falhas: ${e.data.falhas}`;
        }
      };
      workletNode.onprocessorerror = (e) => {
        elDiagWorklet.textContent = "Erro no AudioWorkletProcessor!";
        console.warn("AudioWorklet erro:", e);
      };
      workletNode.connect(audioCtx.destination);
      elDiagPipeline.innerHTML = `<span style="color:#1a7f37; font-weight:bold;">AudioWorklet (Zero latência)</span>`;
    } catch (eWorklet) {
      console.warn("AudioWorklet indisponível, usando ScriptProcessor:", eWorklet);
      const spNode = audioCtx.createScriptProcessor(2048, 0, 2);
      const tam = 16384;
      const bL = new Float32Array(tam);
      const bR = new Float32Array(tam);
      let rIdx = 0;
      let wIdx = 0;
      let disp = 0;

      spNode.onaudioprocess = (ev) => {
        const outL = ev.outputBuffer.getChannelData(0);
        const outR = ev.outputBuffer.getChannelData(1);
        const len = outL.length;
        for (let i = 0; i < len; i++) {
          if (disp > 0) {
            outL[i] = bL[rIdx];
            outR[i] = bR[rIdx];
            rIdx = (rIdx + 1) % tam;
            disp--;
          } else {
            outL[i] = 0;
            outR[i] = 0;
          }
        }
      };
      spNode.connect(audioCtx.destination);
      fallbackNode = {
        enviar: (dados) => {
          const pares = dados.length >> 1;
          for (let i = 0; i < pares; i++) {
            if (disp < tam) {
              bL[wIdx] = dados[i * 2];
              bR[wIdx] = dados[i * 2 + 1];
              wIdx = (wIdx + 1) % tam;
              disp++;
            }
          }
        },
      };
      elDiagPipeline.innerHTML = `<span style="color:#0969da; font-weight:bold;">ScriptProcessor (Fallback universal)</span>`;
      elDiagWorklet.textContent = "Usando ScriptProcessor fallback";
    }

    window.addEventListener("click", desbloquearAudio);
    window.addEventListener("keydown", desbloquearAudio);
    window.addEventListener("pointerdown", desbloquearAudio);
    window.addEventListener("touchstart", desbloquearAudio);
    tela.addEventListener("click", desbloquearAudio);

    desbloquearAudio();
    atualizarEstadoAudioUI();
  } catch (e) {
    elDiagPipeline.innerHTML = `<span style="color:#b3261e;">Falha ao iniciar áudio: ${e.message}</span>`;
  }
}

entrada.addEventListener("change", async () => {
  const arquivo = entrada.files[0];
  if (!arquivo) return;
  if (arquivo.size > LIMITE_BYTES) {
    mostrar(false, "Recusada: arquivo grande demais para ser a REV01.", "(não calculado)", arquivo.size);
    return;
  }
  const bytes = new Uint8Array(await arquivo.arrayBuffer());
  const r = identificar(bytes);
  mostrar(r.valida, r.valida ? "ROM REV01 válida." : "Recusada: " + r.motivo + ".", r.sha1, r.tamanho);
  const valida = r.valida;
  r.free();
  if (valida) iniciar(bytes);
});

// ---- Jogo ----
const tela = document.getElementById("tela");
const ctx = tela.getContext("2d");
const imagem = ctx.createImageData(320, 224);
let jogo = null;
let apertadas = 0;
let ultimo = 0;
let acumulado = 0;
const PASSO = 1000 / 60;

const TECLAS = {
  ArrowUp: 0x01, ArrowDown: 0x02, ArrowLeft: 0x04, ArrowRight: 0x08,
  x: 0x10, c: 0x20, z: 0x40, Enter: 0x80,
};
window.addEventListener("keydown", (e) => {
  const b = TECLAS[e.key];
  if (b) {
    apertadas |= b;
    desbloquearAudio();
    e.preventDefault();
  }
});
window.addEventListener("keyup", (e) => {
  const b = TECLAS[e.key];
  if (b) {
    apertadas &= ~b;
    e.preventDefault();
  }
});

function ajustarEscala() {
  const f = Math.max(1, Math.floor(Math.min((window.innerWidth - 32) / 320, (window.innerHeight * 0.7) / 224)));
  tela.style.width = 320 * f + "px";
  tela.style.height = 224 * f + "px";
}
window.addEventListener("resize", ajustarEscala);

function laco(agora) {
  if (!jogo) return;
  acumulado += Math.min(agora - ultimo, 100);
  ultimo = agora;
  if (acumulado >= PASSO) {
    acumulado -= PASSO;
    quadroNum++;

    jogo.botoes(apertadas);
    imagem.data.set(jogo.quadro());
    ctx.putImageData(imagem, 0, 0);

    const audio = jogo.audio();
    let picoQuadro = 0;
    if (audio.length > 0) {
      for (let i = 0; i < audio.length; i++) {
        const abs = Math.abs(audio[i]);
        if (abs > picoQuadro) picoQuadro = abs;
      }
      if (picoQuadro > picoHistorico) picoHistorico = picoQuadro;
      if (picoQuadro > 0.002) quadrosComSom++;

      if (workletNode) {
        workletNode.port.postMessage(audio.buffer, [audio.buffer]);
      } else if (fallbackNode) {
        fallbackNode.enviar(audio);
      }
    }

    // Atualiza Painel de Diagnóstico a cada 6 quadros (~10 vezes por segundo)
    if (quadroNum % 6 === 0) {
      const seg = (quadroNum / 60).toFixed(1);
      elDiagQuadro.textContent = `${quadroNum} (${seg}s de jogo)`;
      elDiagAmostras.textContent = String(audio.length);
      elDiagPico.textContent = picoQuadro.toFixed(4);
      elBarraVolume.style.width = Math.min(100, Math.round(picoQuadro * 100)) + "%";
      elDiagPicoMax.textContent = picoHistorico.toFixed(4);
      elDiagQuadrosSom.textContent = `${quadrosComSom} quadros`;
      atualizarEstadoAudioUI();
    }

    const erro = jogo.erro();
    if (erro) {
      const p = document.getElementById("erro-jogo");
      p.hidden = false;
      p.textContent = "O núcleo parou: " + erro;
    }
  }
  requestAnimationFrame(laco);
}

async function iniciar(bytes) {
  try {
    jogo = Jogo.criar(bytes);
  } catch (e) {
    jogo = null;
    return;
  }
  tela.hidden = false;
  document.getElementById("aviso").hidden = false;
  ajustarEscala();
  await iniciarAudio();
  ultimo = performance.now();
  requestAnimationFrame(laco);
}
