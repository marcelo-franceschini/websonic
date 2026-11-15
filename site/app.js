// Cola mínima entre a página e o WebAssembly. A lógica fica em Rust (crate sonic1-rom).
import init, { identificar, Jogo } from "./pkg/sonic1_web.js";
import { Relogio, PERIODO_QUADRO } from "./relogio.js";
import { criarRenderizadorGPU } from "./gpu.js";

const LIMITE_BYTES = 16 * 1024 * 1024;

const entrada = document.getElementById("arquivo");
const caixa = document.getElementById("resultado");
const btnAudio = document.getElementById("btn-audio");
const btnBipe = document.getElementById("btn-bipe");

const containerTela = document.getElementById("container-tela");
const barraControles = document.getElementById("barra-controles");
const seletorProporcao = document.getElementById("seletor-proporcao");
const seletorFps = document.getElementById("seletor-fps");
const btnFullscreen = document.getElementById("btn-fullscreen");
const painelTeclas = document.getElementById("painel-teclas");

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

let modoWidescreen = false;
let fpsDesbloqueado = false;

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
          elDiagWorklet.textContent = `Fila: ${e.data.disponiveis} amostras | Falhas: ${e.data.falhas} | Descartadas: ${e.data.descartadas}`;
          if (jogo) jogo.ocupacao_audio(e.data.disponiveis);
        }
      };
      workletNode.onprocessorerror = (e) => {
        elDiagWorklet.textContent = "Erro no AudioWorkletProcessor!";
        console.warn("AudioWorklet erro:", e);
      };
      workletNode.connect(audioCtx.destination);
      elDiagPipeline.innerHTML = `<span style="color:#1a7f37; font-weight:bold;">AudioWorklet</span>`;
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
        reset: () => { rIdx = wIdx = disp = 0; },
        ocupacao: () => disp,
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
let jogo = null;
let tela = document.getElementById("tela");
let compositor = null;
let ctx = null;
const imagem = new ImageData(320, 224);
let contextoPerdido = false;

function usarCpu(motivo) {
  // Um canvas que já teve contexto WebGL não pode receber um contexto 2D.
  const nova = tela.cloneNode(false);
  tela.replaceWith(nova);
  tela = nova;
  tela.addEventListener("click", desbloquearAudio);
  compositor = null;
  ctx = tela.getContext("2d");
  contextoPerdido = false;
  if (jogo) jogo.desativar_gpu();
  const erro = document.getElementById("erro-jogo");
  erro.hidden = false;
  erro.textContent = `Vídeo por software: ${motivo}`;
}

try {
  compositor = criarRenderizadorGPU(tela);
  if (!compositor) usarCpu("WebGL2 indisponível.");
} catch (erro) {
  usarCpu(`falha ao iniciar WebGL2 (${erro.message}).`);
}

tela.addEventListener("webglcontextlost", (e) => {
  e.preventDefault();
  contextoPerdido = true;
  cancelAnimationFrame(animacao);
  animacao = 0;
  if (audioCtx && audioCtx.state === "running") {
    audioCtx.suspend().then(() => {
      if (!contextoPerdido && !document.hidden) desbloquearAudio();
    }).catch(() => {});
  }
  document.getElementById("erro-jogo").textContent = "Contexto gráfico perdido. Aguardando recuperação.";
  document.getElementById("erro-jogo").hidden = false;
});

tela.addEventListener("webglcontextrestored", () => {
  try {
    compositor = criarRenderizadorGPU(tela);
    if (!compositor) throw new Error("WebGL2 indisponível");
    contextoPerdido = false;
    document.getElementById("erro-jogo").hidden = true;
  } catch (erro) {
    usarCpu(`falha ao restaurar WebGL2 (${erro.message}).`);
  }
  if (jogo && !document.hidden && !animacao) {
    relogio.iniciar(performance.now());
    desbloquearAudio();
    animacao = requestAnimationFrame((t) => laco(t, geracao));
  }
});

function aplicarProporcao(widescreen) {
  modoWidescreen = Boolean(widescreen);
  if (seletorProporcao && seletorProporcao.value !== (modoWidescreen ? "16:9" : "4:3")) {
    seletorProporcao.value = modoWidescreen ? "16:9" : "4:3";
  }
  const larguraBase = modoWidescreen ? 426 : 320;
  const alturaBase = 224;
  tela.width = larguraBase;
  tela.height = alturaBase;
  if (compositor) {
    compositor.redimensionar(larguraBase, alturaBase);
  }
  if (jogo) {
    if (modoWidescreen) {
      if (typeof jogo.ativar_gpu_widescreen === "function") {
        jogo.ativar_gpu_widescreen();
      } else if (typeof jogo.ativar_gpu_com_dimensoes === "function") {
        jogo.ativar_gpu_com_dimensoes(426, 224);
      }
    } else {
      jogo.ativar_gpu_sem_referencia();
    }
  }
  ajustarEscala();
  apresentar();
}

if (seletorProporcao) {
  seletorProporcao.addEventListener("change", () => {
    aplicarProporcao(seletorProporcao.value === "16:9");
  });
}

if (seletorFps) {
  seletorFps.addEventListener("change", () => {
    fpsDesbloqueado = seletorFps.value === "desbloqueado";
  });
}

if (btnFullscreen) {
  btnFullscreen.addEventListener("click", () => {
    const alvo = containerTela || tela;
    if (!document.fullscreenElement) {
      if (alvo.requestFullscreen) {
        alvo.requestFullscreen().catch(() => {});
      } else if (alvo.webkitRequestFullscreen) {
        alvo.webkitRequestFullscreen();
      }
    } else if (document.exitFullscreen) {
      document.exitFullscreen().catch(() => {});
    }
  });
}

function apresentar() {
  if (contextoPerdido || !jogo) return;
  const larguraBase = modoWidescreen ? 426 : 320;
  const alturaBase = 224;
  if (compositor) {
    try {
      compositor.renderizar(
        jogo.camadas_gpu(),
        jogo.sprites_gpu(),
        jogo.fundos_gpu(),
        jogo.seletores_gpu(),
        jogo.paletas_gpu(),
        larguraBase,
        alturaBase
      );
    } catch (erro) {
      usarCpu(`falha de WebGL2 (${erro.message}).`);
    }
  } else {
    ctx.putImageData(imagem, 0, 0);
  }
}

let apertadas = 0;
const relogio = new Relogio();
let geracao = 0;
let animacao = 0;

const TECLAS = {
  ArrowUp: 0x01, ArrowDown: 0x02, ArrowLeft: 0x04, ArrowRight: 0x08,
  x: 0x10, X: 0x10,
  c: 0x20, C: 0x20,
  z: 0x40, Z: 0x40,
  Enter: 0x80, " ": 0x80, Spacebar: 0x80,
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

window.addEventListener("blur", () => { apertadas = 0; });

// Gamepad padrão: direcional ou analógico esquerdo, A/B/X e Start.
function botoesDoControle() {
  if (!navigator.getGamepads) return 0;
  const controle = Array.from(navigator.getGamepads()).find((p) => p && p.connected);
  if (!controle) return 0;
  const botao = (i) => controle.buttons[i]?.pressed;
  const x = controle.axes[0] || 0;
  const y = controle.axes[1] || 0;
  let mascara = 0;
  if (botao(12) || y < -0.5) mascara |= 0x01;
  if (botao(13) || y > 0.5) mascara |= 0x02;
  if (botao(14) || x < -0.5) mascara |= 0x04;
  if (botao(15) || x > 0.5) mascara |= 0x08;
  if (botao(0)) mascara |= 0x10;
  if (botao(1)) mascara |= 0x20;
  if (botao(2)) mascara |= 0x40;
  if (botao(9)) mascara |= 0x80;
  return mascara;
}

function ajustarEscala() {
  const larguraBase = modoWidescreen ? 426 : 320;
  const alturaBase = 224;
  const emTelaCheia = Boolean(document.fullscreenElement);
  let f;
  if (emTelaCheia) {
    f = Math.max(1, Math.floor(Math.min(window.innerWidth / larguraBase, window.innerHeight / alturaBase)));
  } else {
    f = Math.max(1, Math.floor(Math.min((window.innerWidth - 32) / larguraBase, (window.innerHeight * 0.7) / alturaBase)));
  }
  tela.style.width = (larguraBase * f) + "px";
  tela.style.height = (alturaBase * f) + "px";
}

window.addEventListener("resize", ajustarEscala);
document.addEventListener("fullscreenchange", ajustarEscala);
document.addEventListener("webkitfullscreenchange", ajustarEscala);

function laco(agora, sessao) {
  animacao = 0;
  if (!jogo || sessao !== geracao || document.hidden) return;
  // Recupera atrasos sem descartar tempo, mantendo a página responsiva.
  const passos = relogio.avancar(agora);
  let atualizou = false;
  for (let passo = 0; passo < passos; passo++) {
    quadroNum++;

    jogo.botoes(apertadas | botoesDoControle());
    if (fallbackNode) jogo.ocupacao_audio(fallbackNode.ocupacao());
    if (compositor) jogo.simular_quadro();
    else imagem.data.set(jogo.quadro());
    atualizou = true;

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
      const seg = (quadroNum * PERIODO_QUADRO / 1000).toFixed(1);
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
      if (atualizou) apresentar();
      return;
    }
  }

  if (fpsDesbloqueado) {
    // Apresenta a cada quadro de animação da taxa nativa do monitor
    apresentar();
  } else if (atualizou) {
    // Apresenta estritamente no ritmo fixo NTSC de 60Hz
    apresentar();
  }

  animacao = requestAnimationFrame((t) => laco(t, sessao));
}

async function iniciar(bytes) {
  const sessao = ++geracao;
  cancelAnimationFrame(animacao);
  animacao = 0;
  await iniciarAudio();
  if (sessao !== geracao) return;
  if (workletNode) workletNode.port.postMessage({ tipo: "reset" });
  if (fallbackNode) fallbackNode.reset();
  try {
    const anterior = jogo;
    jogo = Jogo.criar_com_audio(bytes, audioCtx ? audioCtx.sampleRate : 48000);
    if (compositor) {
      if (modoWidescreen) {
        if (typeof jogo.ativar_gpu_widescreen === "function") {
          jogo.ativar_gpu_widescreen();
        } else if (typeof jogo.ativar_gpu_com_dimensoes === "function") {
          jogo.ativar_gpu_com_dimensoes(426, 224);
        }
      } else {
        jogo.ativar_gpu_sem_referencia();
      }
    }
    if (anterior) anterior.free();
  } catch (e) {
    jogo = null;
    return;
  }
  tela.hidden = false;
  if (barraControles) barraControles.hidden = false;
  if (painelTeclas) painelTeclas.hidden = false;
  document.getElementById("aviso").hidden = false;
  ajustarEscala();
  quadroNum = 0;
  relogio.iniciar(performance.now());
  if (!document.hidden) animacao = requestAnimationFrame((t) => laco(t, sessao));
}

document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    cancelAnimationFrame(animacao);
    animacao = 0;
    if (audioCtx && audioCtx.state === "running") {
      audioCtx.suspend().then(() => {
        if (!document.hidden && !contextoPerdido) desbloquearAudio();
      }).catch(() => {});
    }
  } else if (jogo && !animacao && !contextoPerdido) {
    desbloquearAudio();
    relogio.retomar(performance.now());
    animacao = requestAnimationFrame((t) => laco(t, geracao));
  }
});
