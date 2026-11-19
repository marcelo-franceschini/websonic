// Cola entre a pagina e o WebAssembly com WebGL2, AudioWorklet e Gamepad.
import init, { identificar, Jogo } from "./pkg/sonic1_web.js";
import { Relogio, PERIODO_QUADRO } from "./relogio.js";
import { criarRenderizadorGPU } from "./gpu.js";

const LIMITE_BYTES = 16 * 1024 * 1024;

// Elementos Principais do DOM
const entrada = document.getElementById("arquivo");
const dropzone = document.getElementById("dropzone");
const caixa = document.getElementById("resultado");
const btnAudio = document.getElementById("btn-audio");
const btnBipe = document.getElementById("btn-bipe");

const containerTela = document.getElementById("container-tela");
const camadaCrt = document.getElementById("camada-crt");
const barraControles = document.getElementById("barra-controles");
const seletorFps = document.getElementById("seletor-fps");
const seletorCrt = document.getElementById("seletor-crt");
const btnFullscreen = document.getElementById("btn-fullscreen");
const btnHud = document.getElementById("btn-hud");
const btnMudo = document.getElementById("btn-mudo");
const sliderVolume = document.getElementById("slider-volume");
const painelTeclas = document.getElementById("painel-teclas");

// Badges de Status do Cabecalho
const badgeAudio = document.getElementById("badge-audio");
const badgeGamepad = document.getElementById("badge-gamepad");

// HUD de Performance e Profiler
const hudPerformance = document.getElementById("hud-performance");
const elHudFps = document.getElementById("hud-fps");
const elHudHz = document.getElementById("hud-hz");
const elHudFrameTime = document.getElementById("hud-frame-time");
const elHudWasmTime = document.getElementById("hud-wasm-time");
const canvasHudGrafico = document.getElementById("hud-grafico");
const ctxHudGrafico = canvasHudGrafico ? canvasHudGrafico.getContext("2d") : null;

// Painel de Telemetria e Diagnostico
const btnToggleDiag = document.getElementById("btn-toggle-diag");
const painelDiag = document.getElementById("painel-diag");
const diagSeta = document.getElementById("diag-seta");
const elDiagEstado = document.getElementById("diag-estado");
const elDiagTaxa = document.getElementById("diag-taxa");
const elDiagPipeline = document.getElementById("diag-pipeline");
const elDiagFps = document.getElementById("diag-fps");
const elDiagTempoWasm = document.getElementById("diag-tempo-wasm");
const elDiagQuadro = document.getElementById("diag-quadro");
const elDiagAmostras = document.getElementById("diag-amostras");
const elDiagPico = document.getElementById("diag-pico");
const elBarraVolume = document.getElementById("barra-volume");
const elDiagPicoMax = document.getElementById("diag-pico-max");
const elDiagQuadrosSom = document.getElementById("diag-quadros-som");
const elDiagWorklet = document.getElementById("diag-worklet");

// Estado do Audio
let audioCtx = null;
let ganhoMasterNode = null;
let workletNode = null;
let volumeAtual = 1.0;
let mudoAtivo = false;

// Pool de buffers de audio reutilizados para transferencia ao worklet.
// 4096 floats cobrem um quadro estereo ate ~122 kHz; taxas maiores trocam o buffer por um maior.
const POOL_AUDIO_TAMANHO = 8;
const POOL_AUDIO_FLOATS = 4096;
const poolAudio = [];
for (let i = 0; i < POOL_AUDIO_TAMANHO; i++) {
  poolAudio.push(new Float32Array(POOL_AUDIO_FLOATS));
}
let vistaAudio = null;

// Metricas
let quadroNum = 0;
let picoHistorico = 0;
let quadrosComSom = 0;

// Configuracoes de Video
let fpsDesbloqueado = false;

// Referencia ao modulo Wasm instanciado
let wasmInstance = null;

// Profiler e Telemetria de Performance em Tempo Real
let hudVisivel = false;
const HISTORICO_TEMPOS_MAX = 80;
const historicoFrameTime = new Float32Array(HISTORICO_TEMPOS_MAX);
const historicoWasmTime = new Float32Array(HISTORICO_TEMPOS_MAX);
// Copia reutilizada para ordenar e achar o p95 sem alocar a cada atualizacao do painel.
const copiaWasmTime = new Float32Array(HISTORICO_TEMPOS_MAX);
let indiceHistorico = 0;
let totalHistorico = 0;

let ultimoTimestampRaf = 0;
let contadorApresentacoes = 0;
let acumuladorFpsTempo = 0;
let fpsApresentado = 60.0;
let simulacaoHz = 60.0;
let ultimoQuadroSim = 0;
let ultimoTempoHz = 0;
let ultimoWasmTime = 0;
let ultimoFrameTime = 0;

function alternarHud() {
  hudVisivel = !hudVisivel;
  if (hudPerformance) hudPerformance.hidden = !hudVisivel;
  if (btnHud) {
    if (hudVisivel) {
      btnHud.classList.add("btn-azul");
      btnHud.classList.remove("btn-cinza");
    } else {
      btnHud.classList.remove("btn-azul");
      btnHud.classList.add("btn-cinza");
    }
  }
}

if (btnHud) {
  btnHud.addEventListener("click", alternarHud);
}

function desenharGraficoHud() {
  if (!ctxHudGrafico || !hudVisivel) return;
  const w = canvasHudGrafico.width;
  const h = canvasHudGrafico.height;
  ctxHudGrafico.clearRect(0, 0, w, h);

  // Linhas de referencia: 16.67ms (60 FPS) e 33.33ms (30 FPS)
  const y16 = Math.round(h - (16.67 * h / 40));
  const y33 = Math.round(h - (33.33 * h / 40));

  ctxHudGrafico.fillStyle = "rgba(0, 212, 255, 0.35)";
  ctxHudGrafico.fillRect(0, y16, w, 1);

  ctxHudGrafico.fillStyle = "rgba(239, 68, 68, 0.35)";
  ctxHudGrafico.fillRect(0, y33, w, 1);

  const n = Math.min(totalHistorico, HISTORICO_TEMPOS_MAX);
  const larguraBarra = w / HISTORICO_TEMPOS_MAX;

  for (let i = 0; i < n; i++) {
    const idx = (indiceHistorico - n + i + HISTORICO_TEMPOS_MAX) % HISTORICO_TEMPOS_MAX;
    const tempo = historicoFrameTime[idx];
    const altura = Math.min(h, Math.max(1, Math.round((tempo / 40) * h)));
    const x = i * larguraBarra;
    const y = h - altura;

    if (tempo <= 16.67) {
      ctxHudGrafico.fillStyle = "#10b981"; // Verde (< 16.67ms)
    } else if (tempo <= 33.33) {
      ctxHudGrafico.fillStyle = "#ffbe0b"; // Amarelo (16.67 a 33.33ms)
    } else {
      ctxHudGrafico.fillStyle = "#ef4444"; // Vermelho (> 33.33ms)
    }

    ctxHudGrafico.fillRect(x, y, Math.max(1, larguraBarra - 0.5), altura);
  }
}

function mostrar(valida, veredito, sha1, tamanho) {
  caixa.hidden = false;
  caixa.className = valida ? "ok" : "erro";
  document.getElementById("veredito").textContent = veredito;
  document.getElementById("sha1").textContent = sha1;
  document.getElementById("tamanho").textContent = String(tamanho);
}

// Inicializacao do WebAssembly
try {
  wasmInstance = await init();
  entrada.disabled = false;
} catch (e) {
  console.error("Falha ao inicializar WebAssembly:", e);
  const erro = document.getElementById("erro-jogo");
  erro.hidden = false;
  erro.textContent = "Falha ao carregar WebAssembly: " + e.message;
}

// Toggle da gaveta de telemetria
if (btnToggleDiag && painelDiag) {
  btnToggleDiag.addEventListener("click", () => {
    const oculto = painelDiag.hidden;
    painelDiag.hidden = !oculto;
    if (diagSeta) diagSeta.textContent = oculto ? "▲" : "▼";
  });
}

// Controle de Volume Mestre
function atualizarGanhoAudio() {
  if (!ganhoMasterNode || !audioCtx) return;
  const valor = mudoAtivo ? 0.0 : volumeAtual;
  ganhoMasterNode.gain.setValueAtTime(valor, audioCtx.currentTime);
}

if (sliderVolume) {
  sliderVolume.addEventListener("input", (e) => {
    volumeAtual = parseFloat(e.target.value);
    if (mudoAtivo && volumeAtual > 0) {
      mudoAtivo = false;
      if (btnMudo) btnMudo.textContent = "🔊";
    }
    atualizarGanhoAudio();
  });
}

if (btnMudo) {
  btnMudo.addEventListener("click", () => {
    mudoAtivo = !mudoAtivo;
    btnMudo.textContent = mudoAtivo ? "🔇" : "🔊";
    atualizarGanhoAudio();
    if (tela) tela.focus();
  });
}

// Filtro de Scanlines CRT
if (seletorCrt && camadaCrt) {
  seletorCrt.addEventListener("change", () => {
    if (seletorCrt.value === "ligado") {
      camadaCrt.style.display = "block";
      camadaCrt.classList.add("ativo");
    } else {
      camadaCrt.classList.remove("ativo");
      camadaCrt.style.display = "none";
    }
  });
}

function criarAudioContext() {
  if (!audioCtx) {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    audioCtx = new AudioContextClass();
    audioCtx.addEventListener("statechange", () => {
      if (audioCtx.state === "running") {
        if (workletNode) workletNode.port.postMessage({ tipo: "reset" });
      }
      atualizarEstadoAudioUI();
    });
  }
  return audioCtx;
}

// Teste de Bipe (Web Audio puro)
btnBipe.addEventListener("click", () => {
  try {
    const ctx = criarAudioContext();
    ctx.resume().then(() => {
      atualizarEstadoAudioUI();
      const osc = ctx.createOscillator();
      const ganho = ctx.createGain();
      osc.type = "sine";
      osc.frequency.setValueAtTime(440, ctx.currentTime);
      ganho.gain.setValueAtTime(0.25, ctx.currentTime);
      ganho.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.35);
      osc.connect(ganho);
      ganho.connect(ctx.destination);
      osc.start();
      osc.stop(ctx.currentTime + 0.35);
    });
  } catch (e) {
    alert("Erro ao tocar bipe de teste: " + e.message);
  }
});

function atualizarEstadoAudioUI() {
  if (!audioCtx) {
    if (elDiagEstado && elDiagEstado.textContent !== "Não inicializado") {
      elDiagEstado.textContent = "Não inicializado";
      elDiagEstado.className = "diag-mudo";
    }
    if (badgeAudio && badgeAudio.textContent !== "🔇 Áudio Desligado") {
      badgeAudio.className = "pill";
      badgeAudio.textContent = "🔇 Áudio Desligado";
    }
    return;
  }
  const st = audioCtx.state;
  if (st === "running") {
    if (elDiagEstado && elDiagEstado.textContent !== "🟢 Ativo (running)") {
      elDiagEstado.textContent = "🟢 Ativo (running)";
      elDiagEstado.className = "diag-ativo";
    }
    if (btnAudio && btnAudio.textContent !== "🔊 Áudio Ativo") {
      btnAudio.textContent = "🔊 Áudio Ativo";
      btnAudio.style.background = "var(--emerald-green)";
    }
    if (badgeAudio && badgeAudio.textContent !== "🔊 Áudio Ativo") {
      badgeAudio.className = "pill ativo";
      badgeAudio.textContent = "🔊 Áudio Ativo";
    }
  } else if (st === "suspended") {
    if (elDiagEstado && elDiagEstado.textContent !== "🟡 Pausado pelo navegador (suspended)") {
      elDiagEstado.textContent = "🟡 Pausado pelo navegador (suspended)";
      elDiagEstado.className = "diag-aviso";
    }
    if (btnAudio && btnAudio.textContent !== "⚠️ Desbloquear Áudio") {
      btnAudio.textContent = "⚠️ Desbloquear Áudio";
      btnAudio.style.background = "var(--ring-gold)";
    }
    if (badgeAudio && badgeAudio.textContent !== "🟡 Clique para Áudio") {
      badgeAudio.className = "pill aviso";
      badgeAudio.textContent = "🟡 Clique para Áudio";
    }
  } else {
    if (elDiagEstado && elDiagEstado.textContent !== st) {
      elDiagEstado.textContent = st;
      elDiagEstado.className = "diag-mudo";
    }
  }
  const taxaStr = audioCtx.sampleRate + " Hz";
  if (elDiagTaxa && elDiagTaxa.textContent !== taxaStr) {
    elDiagTaxa.textContent = taxaStr;
  }
}

function desbloquearAudio() {
  if (audioCtx && audioCtx.state === "suspended") {
    audioCtx.resume().then(atualizarEstadoAudioUI).catch(() => {});
  }
}

btnAudio.addEventListener("click", () => {
  desbloquearAudio();
  if (tela) tela.focus();
});

async function iniciarAudio() {
  if (audioCtx && workletNode) return;
  try {
    criarAudioContext();

    if (!ganhoMasterNode) {
      ganhoMasterNode = audioCtx.createGain();
      ganhoMasterNode.gain.setValueAtTime(mudoAtivo ? 0.0 : volumeAtual, audioCtx.currentTime);
      ganhoMasterNode.connect(audioCtx.destination);
    }

    try {
      await audioCtx.audioWorklet.addModule("audio-worklet.js?v=6");
      try {
        workletNode = new AudioWorkletNode(audioCtx, "sonic-audio-processor", {
          outputChannelCount: [2],
        });
      } catch {
        workletNode = new AudioWorkletNode(audioCtx, "sonic-audio-processor");
      }
      workletNode.port.onmessage = (e) => {
        if (e.data instanceof ArrayBuffer) {
          if (poolAudio.length < POOL_AUDIO_TAMANHO) {
            poolAudio.push(new Float32Array(e.data));
          }
          return;
        }
        if (e.data && e.data.tipo === "status") {
          if (painelDiag && !painelDiag.hidden) {
            elDiagWorklet.textContent = `Fila: ${e.data.disponiveis} amostras | Falhas: ${e.data.falhas} | Descartadas: ${e.data.descartadas}`;
          }
          if (jogo) jogo.ocupacao_audio(e.data.disponiveis);
        }
      };
      workletNode.onprocessorerror = (e) => {
        elDiagWorklet.textContent = "Erro no AudioWorkletProcessor!";
        console.warn("AudioWorklet erro:", e);
      };
      workletNode.connect(ganhoMasterNode);
      if (elDiagPipeline) {
        elDiagPipeline.textContent = "AudioWorklet";
        elDiagPipeline.className = "diag-ativo";
      }
    } catch (eWorklet) {
      console.warn("AudioWorklet indisponivel:", eWorklet);
      if (elDiagPipeline) {
        elDiagPipeline.textContent = "AudioWorklet indisponivel";
        elDiagPipeline.className = "diag-aviso";
      }
      if (elDiagWorklet) {
        elDiagWorklet.textContent = "AudioWorklet indisponivel";
      }
    }

    window.addEventListener("click", desbloquearAudio);
    window.addEventListener("keydown", desbloquearAudio);
    window.addEventListener("pointerdown", desbloquearAudio);
    window.addEventListener("touchstart", desbloquearAudio);
    tela.addEventListener("click", desbloquearAudio);

    desbloquearAudio();
    atualizarEstadoAudioUI();
  } catch (e) {
    if (elDiagPipeline) {
      elDiagPipeline.textContent = `Falha ao iniciar audio: ${e.message}`;
      elDiagPipeline.className = "diag-erro";
    }
  }
}

// Processamento de Arquivo ROM
async function carregarArquivoRom(arquivo) {
  if (!arquivo) return;
  if (arquivo.size > LIMITE_BYTES) {
    mostrar(false, "Recusada: arquivo grande demais para ser a REV01.", "(nao calculado)", arquivo.size);
    return;
  }
  const bytes = new Uint8Array(await arquivo.arrayBuffer());
  const r = identificar(bytes);
  mostrar(r.valida, r.valida ? "ROM REV01 valida." : "Recusada: " + r.motivo + ".", r.sha1, r.tamanho);
  const valida = r.valida;
  r.free();
  if (valida) {
    if (dropzone) {
      dropzone.style.display = "none";
    }
    iniciar(bytes);
  }
}

entrada.addEventListener("change", () => {
  carregarArquivoRom(entrada.files[0]);
});

// Suporte a Arrastar e Soltar (Drag and Drop)
if (dropzone) {
  dropzone.addEventListener("dragover", (e) => {
    e.preventDefault();
    dropzone.classList.add("arrastando");
  });

  dropzone.addEventListener("dragleave", () => {
    dropzone.classList.remove("arrastando");
  });

  dropzone.addEventListener("drop", (e) => {
    e.preventDefault();
    dropzone.classList.remove("arrastando");
    const arquivos = e.dataTransfer?.files;
    if (arquivos && arquivos.length > 0) {
      carregarArquivoRom(arquivos[0]);
    }
  });
}

// Jogo e Renderizacao
let jogo = null;
let tela = document.getElementById("tela");
let compositor = null;
let ctx = null;
const imagem = new ImageData(320, 224);
let contextoPerdido = false;

function usarCpu(motivo) {
  const nova = tela.cloneNode(false);
  tela.replaceWith(nova);
  tela = nova;
  tela.tabIndex = 0;
  tela.addEventListener("click", () => {
    desbloquearAudio();
    tela.focus();
  });
  compositor = null;
  ctx = tela.getContext("2d");
  contextoPerdido = false;
  const erro = document.getElementById("erro-jogo");
  erro.hidden = false;
  erro.textContent = `Video por software: ${motivo}`;
}

try {
  compositor = criarRenderizadorGPU(tela);
  if (!compositor) usarCpu("WebGL2 indisponivel.");
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
  document.getElementById("erro-jogo").textContent = "Contexto grafico perdido. Aguardando recuperacao.";
  document.getElementById("erro-jogo").hidden = false;
});

tela.addEventListener("webglcontextrestored", () => {
  try {
    compositor = criarRenderizadorGPU(tela);
    if (!compositor) throw new Error("WebGL2 indisponivel");
    contextoPerdido = false;
    document.getElementById("erro-jogo").hidden = true;
  } catch (erro) {
    usarCpu(`falha ao restaurar WebGL2 (${erro.message}).`);
  }
  if (jogo && !document.hidden && !animacao) {
    relogio.iniciar(performance.now());
    desbloquearAudio();
    if (funcaoRafAtual) animacao = requestAnimationFrame(funcaoRafAtual);
  }
});

// Tabela de conversao de 9 bits para RGBA (Uint32, little-endian)
const TABELA_COR_9BITS = new Uint32Array(512);
for (let c = 0; c < 512; c++) {
  const r = (c & 7), g = ((c >> 3) & 7), b = ((c >> 6) & 7);
  const r8 = (r << 5) | (r << 2) | (r >> 1);
  const g8 = (g << 5) | (g << 2) | (g >> 1);
  const b8 = (b << 5) | (b << 2) | (b >> 1);
  TABELA_COR_9BITS[c] = 0xFF000000 | (b8 << 16) | (g8 << 8) | r8;
}

// View da imagem de 9 bits na memoria Wasm. Recriada so quando a memoria cresce.
let vistaImagem = null;
function imagemAtual() {
  const mem = wasmInstance.memory.buffer;
  if (!vistaImagem || vistaImagem.buffer !== mem) {
    vistaImagem = new Uint16Array(mem, jogo.imagem_ptr(), jogo.imagem_len());
  }
  return vistaImagem;
}

if (seletorFps) {
  seletorFps.addEventListener("change", () => {
    fpsDesbloqueado = seletorFps.value === "desbloqueado";
    if (tela) tela.focus();
  });
}

function alternarTelaCheia() {
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
}

if (btnFullscreen) {
  btnFullscreen.addEventListener("click", () => {
    alternarTelaCheia();
    if (tela) tela.focus();
  });
}

// Apresentacao Grafica com Modo Zero-Copy Otimizado
function apresentar() {
  if (contextoPerdido || !jogo) return;
  const pts = imagemAtual();
  if (compositor) {
    try {
      compositor.renderizar(pts);
    } catch (erro) {
      usarCpu(`falha de WebGL2 (${erro.message}).`);
    }
  } else {
    const d32 = new Uint32Array(imagem.data.buffer);
    for (let i = 0; i < 71680; i++) {
      d32[i] = TABELA_COR_9BITS[pts[i] & 0x1FF];
    }
    ctx.putImageData(imagem, 0, 0);
  }
}

const relogio = new Relogio();
let geracao = 0;
let animacao = 0;
let funcaoRafAtual = null;
let temGamepadConectado = false;

// Mapeamento de Teclas Fisicas e Virtuais (Colecao Deterministica com Set)
const teclasSeguradas = new Set();

const MAPA_TECLAS_CODE = {
  // Direcionais
  ArrowUp: 0x01, KeyW: 0x01, Numpad8: 0x01,
  ArrowDown: 0x02, KeyS: 0x02, Numpad2: 0x02,
  ArrowLeft: 0x04, KeyA: 0x04, Numpad4: 0x04,
  ArrowRight: 0x08, KeyD: 0x08, Numpad6: 0x08,

  // Botoes A, B, C (No Sonic 1, todos sao pulo)
  KeyX: 0x10, KeyK: 0x10, // B
  KeyC: 0x20, KeyL: 0x20, // C
  KeyZ: 0x40, KeyJ: 0x40, // A
  Space: 0x20, // Padrao C / Pulo

  // Start / Pausa
  Enter: 0x80, NumpadEnter: 0x80, KeyP: 0x80,
};

const MAPA_TECLAS_KEY = {
  ArrowUp: 0x01, w: 0x01, W: 0x01,
  ArrowDown: 0x02, s: 0x02, S: 0x02,
  ArrowLeft: 0x04, a: 0x04, A: 0x04,
  ArrowRight: 0x08, d: 0x08, D: 0x08,
  x: 0x10, X: 0x10, k: 0x10, K: 0x10,
  c: 0x20, C: 0x20, l: 0x20, L: 0x20,
  z: 0x40, Z: 0x40, j: 0x40, J: 0x40,
  " ": 0x20, Spacebar: 0x20,
  Enter: 0x80, p: 0x80, P: 0x80,
};

function calcularMascaraTeclado(modo = 4) {
  let mascara = 0;
  // Abertura / Titulo / Demo: 0x00 (Sega), 0x04 (Titulo), 0x08 (Demo)
  const telaTituloOuDemo = modo === 0x00 || modo === 0x04 || modo === 0x08;

  for (const chave of teclasSeguradas) {
    let b = MAPA_TECLAS_CODE[chave] || MAPA_TECLAS_KEY[chave] || 0;
    if (chave === "Space" || chave === " " || chave === "Spacebar") {
      // No titulo ou demo, barra de espaco inicia o jogo como Start.
      // Em qualquer outro lugar (fases, telas de continue, etc), e pulo.
      b = telaTituloOuDemo ? 0x80 : 0x20;
    }
    mascara |= b;
  }

  // Previne direcoes opostas simultaneas (invalido no controle real do Mega Drive)
  if ((mascara & 0x0C) === 0x0C) mascara &= ~0x0C; // Esquerda + Direita cancelam
  if ((mascara & 0x03) === 0x03) mascara &= ~0x03; // Cima + Baixo cancelam

  return mascara;
}

// Cache de elementos de teclas virtuais por data-code
const cacheKeycaps = new Map();
document.querySelectorAll(".tecla-cap[data-code]").forEach((cap) => {
  const code = cap.getAttribute("data-code");
  if (!code) return;
  let lista = cacheKeycaps.get(code);
  if (!lista) {
    lista = [];
    cacheKeycaps.set(code, lista);
  }
  lista.push(cap);
});

function iluminarKeycap(code, ativa) {
  if (!code) return;
  const caps = cacheKeycaps.get(code);
  if (!caps) return;
  for (let i = 0; i < caps.length; i++) {
    if (ativa) caps[i].classList.add("ativa");
    else caps[i].classList.remove("ativa");
  }
}

window.addEventListener("keydown", (e) => {
  if (e.repeat) return;
  if (e.code === "KeyF" && !e.ctrlKey && !e.metaKey && !e.altKey) {
    alternarTelaCheia();
    e.preventDefault();
    return;
  }
  if (e.code === "F3" && !e.ctrlKey && !e.metaKey && !e.altKey) {
    alternarHud();
    e.preventDefault();
    return;
  }

  const b = MAPA_TECLAS_CODE[e.code] || MAPA_TECLAS_KEY[e.key];
  if (b) {
    teclasSeguradas.add(e.code || e.key);
    iluminarKeycap(e.code, true);
    desbloquearAudio();

    // Remove foco de botoes DOM para evitar ativacao acidental com Enter/Espaco
    if (document.activeElement && document.activeElement !== document.body && document.activeElement !== tela && document.activeElement.tagName !== "INPUT") {
      document.activeElement.blur();
    }

    e.preventDefault();
  }
});

window.addEventListener("keyup", (e) => {
  const b = MAPA_TECLAS_CODE[e.code] || MAPA_TECLAS_KEY[e.key];
  if (b) {
    teclasSeguradas.delete(e.code);
    teclasSeguradas.delete(e.key);
    iluminarKeycap(e.code, false);
    e.preventDefault();
  }
});

window.addEventListener("blur", () => {
  teclasSeguradas.clear();
  document.querySelectorAll(".tecla-cap.ativa").forEach((cap) => cap.classList.remove("ativa"));
});

// Suporte a Gamepad USB e Bluetooth
function atualizarBadgeGamepad(gp) {
  if (!badgeGamepad) return;
  if (gp && gp.connected) {
    const nome = gp.id.length > 22 ? gp.id.slice(0, 20) + "..." : gp.id;
    badgeGamepad.className = "pill ativo";
    badgeGamepad.textContent = `🎮 ${nome}`;
  } else {
    badgeGamepad.className = "pill";
    badgeGamepad.textContent = "🎮 Gamepad";
  }
}

window.addEventListener("gamepadconnected", (e) => {
  temGamepadConectado = true;
  atualizarBadgeGamepad(e.gamepad);
});

window.addEventListener("gamepaddisconnected", () => {
  const gps = navigator.getGamepads ? navigator.getGamepads() : [];
  temGamepadConectado = false;
  let primeiro = null;
  for (let i = 0; i < gps.length; i++) {
    if (gps[i]?.connected) {
      temGamepadConectado = true;
      primeiro = gps[i];
      break;
    }
  }
  atualizarBadgeGamepad(primeiro);
});

function botoesDoControle(modo = 4) {
  if (!temGamepadConectado || !navigator.getGamepads) return 0;
  const gamepads = navigator.getGamepads();
  if (!gamepads) return 0;

  const telaTituloOuDemo = modo === 0x00 || modo === 0x04 || modo === 0x08;

  let mascara = 0;
  for (let idx = 0; idx < gamepads.length; idx++) {
    const controle = gamepads[idx];
    if (!controle || !controle.connected) continue;

    const botao = (i) => Boolean(controle.buttons[i]?.pressed || (controle.buttons[i]?.value > 0.45));
    const ax = (i) => controle.axes[i] || 0;

    // D-Pad por botoes padrao
    if (botao(12) || ax(1) < -0.38) mascara |= 0x01; // Cima
    if (botao(13) || ax(1) > 0.38) mascara |= 0x02;  // Baixo
    if (botao(14) || ax(0) < -0.38) mascara |= 0x04; // Esquerda
    if (botao(15) || ax(0) > 0.38) mascara |= 0x08; // Direita

    // Suporte a eixos adicionais (DirectInput D-pad axes 4/5)
    if (controle.axes.length >= 6) {
      if (ax(5) < -0.38) mascara |= 0x01;
      if (ax(5) > 0.38) mascara |= 0x02;
      if (ax(4) < -0.38) mascara |= 0x04;
      if (ax(4) > 0.38) mascara |= 0x08;
    }

    // Acoes: Botoes 0 (A), 1 (B), 2 (X), 3 (Y), 4 (L1), 5 (R1) pulam no Sonic
    if (botao(0)) mascara |= 0x10; // B
    if (botao(1)) mascara |= 0x20; // C
    if (botao(2)) mascara |= 0x40; // A
    if (botao(3)) mascara |= 0x20; // C
    if (botao(4) || botao(5) || botao(6) || botao(7)) mascara |= 0x10; // Bumpers e Triggers

    // Start / Pausa: Botoes 9 (Start), 8 (Select), 16 (Home)
    if (botao(9) || botao(8) || botao(16)) mascara |= 0x80;

    // Se estiver estritamente na tela de titulo ou demo, botoes de acao tambem iniciam o jogo
    if (telaTituloOuDemo && (botao(0) || botao(1) || botao(2) || botao(3))) {
      mascara |= 0x80;
    }

    if (mascara !== 0) break;
  }

  // Previne direcoes opostas simultaneas
  if ((mascara & 0x0C) === 0x0C) mascara &= ~0x0C;
  if ((mascara & 0x03) === 0x03) mascara &= ~0x03;

  return mascara;
}

function ajustarEscala() {
  const larguraBase = 320;
  const alturaBase = 224;
  const emTelaCheia = Boolean(document.fullscreenElement);
  let f;
  if (emTelaCheia) {
    f = Math.max(1, Math.floor(Math.min(window.innerWidth / larguraBase, window.innerHeight / alturaBase)));
  } else {
    const dispW = Math.min(window.innerWidth - 32, 960);
    f = Math.max(1, Math.floor(Math.min(dispW / larguraBase, (window.innerHeight * 0.72) / alturaBase)));
  }
  tela.style.width = (larguraBase * f) + "px";
  tela.style.height = (alturaBase * f) + "px";
}

window.addEventListener("resize", ajustarEscala);
document.addEventListener("fullscreenchange", ajustarEscala);
document.addEventListener("webkitfullscreenchange", ajustarEscala);

// Foco no Canvas no clique
tela.addEventListener("click", () => {
  tela.focus();
  desbloquearAudio();
});

// Instante (performance.now) do ultimo rAF. O vigia usa para saber se o navegador parou de chamar o rAF.
let ultimoRaf = 0;

function laco(agora, sessao) {
  animacao = 0;
  if (!jogo || sessao !== geracao || document.hidden) return;
  ultimoRaf = performance.now();

  const inicioQuadro = performance.now();
  if (ultimoTimestampRaf > 0) {
    const deltaRaf = agora - ultimoTimestampRaf;
    contadorApresentacoes++;
    acumuladorFpsTempo += deltaRaf;
    if (acumuladorFpsTempo >= 400) {
      fpsApresentado = (contadorApresentacoes * 1000) / acumuladorFpsTempo;
      contadorApresentacoes = 0;
      acumuladorFpsTempo = 0;
    }
  }
  ultimoTimestampRaf = agora;

  const atualizou = simularPassos(agora);
  // O nucleo parou: simularPassos ja apresentou o ultimo quadro e o laco nao continua.
  if (atualizou === null) return;

  if (fpsDesbloqueado) {
    // Apresenta na taxa nativa do monitor
    apresentar();
  } else if (atualizou) {
    // Apresenta estritamente no ritmo de 60Hz
    apresentar();
  }

  const fimQuadro = performance.now();
  ultimoFrameTime = fimQuadro - inicioQuadro;
  historicoFrameTime[indiceHistorico] = ultimoFrameTime;
  indiceHistorico = (indiceHistorico + 1) % HISTORICO_TEMPOS_MAX;
  totalHistorico = Math.min(totalHistorico + 1, HISTORICO_TEMPOS_MAX);

  if (agora - ultimoTempoHz >= 500) {
    const deltaQuadros = quadroNum - ultimoQuadroSim;
    const deltaTempo = agora - ultimoTempoHz;
    simulacaoHz = deltaTempo > 0 ? (deltaQuadros * 1000) / deltaTempo : 60.0;
    ultimoQuadroSim = quadroNum;
    ultimoTempoHz = agora;
  }

  if (hudVisivel) {
    if (elHudFps) elHudFps.textContent = `${fpsApresentado.toFixed(1)} FPS`;
    if (elHudHz) elHudHz.textContent = `${simulacaoHz.toFixed(1)} Hz sim`;
    if (elHudFrameTime) elHudFrameTime.textContent = `${ultimoFrameTime.toFixed(1)} ms`;
    if (elHudWasmTime) elHudWasmTime.textContent = `${ultimoWasmTime.toFixed(1)} ms`;
    desenharGraficoHud();
  }

  if (funcaoRafAtual) animacao = requestAnimationFrame(funcaoRafAtual);
}

// Roda os passos que o relogio pede: entrada, Wasm, audio e painel. Devolve se algum passo rodou,
// ou null se o nucleo parou (nesse caso ja apresentou o ultimo quadro).
function simularPassos(agora) {
  const passos = relogio.avancar(agora);
  let atualizou = false;

  for (let passo = 0; passo < passos; passo++) {
    quadroNum++;

    const modo = jogo?.modo_jogo?.() ?? 4;
    const estadoControles = calcularMascaraTeclado(modo) | botoesDoControle(modo);
    jogo.botoes(estadoControles);

    const inicioWasm = performance.now();
    jogo.simular_quadro();
    ultimoWasmTime = performance.now() - inicioWasm;
    historicoWasmTime[indiceHistorico] = ultimoWasmTime;

    atualizou = true;

    const ptrAudio = jogo.audio_ptr();
    const lenAudio = jogo.audio_len();
    let picoQuadro = 0;
    if (lenAudio > 0) {
      const diagAberto = painelDiag && !painelDiag.hidden;
      const mem = wasmInstance.memory.buffer;
      if (!vistaAudio || vistaAudio.buffer !== mem) {
        vistaAudio = new Float32Array(mem);
      }
      const fatiaAudio = vistaAudio.subarray(ptrAudio >> 2, (ptrAudio >> 2) + lenAudio);

      // Calcula o pico somente no passo que atualiza o painel
      const atualizaDiag = diagAberto && (quadroNum % 6 === 0);
      if (atualizaDiag) {
        for (let i = 0; i < lenAudio; i++) {
          const abs = Math.abs(fatiaAudio[i]);
          if (abs > picoQuadro) picoQuadro = abs;
        }
        if (picoQuadro > picoHistorico) picoHistorico = picoQuadro;
      }

      if (quadrosComSom < 1000 && (fatiaAudio[0] !== 0 || fatiaAudio[lenAudio - 1] !== 0)) {
        quadrosComSom++;
      }

      // Um buffer menor que o quadro (taxa acima de ~122 kHz) e descartado e trocado por um
      // do tamanho certo; sem isso, buf.set lancaria RangeError.
      let buf = poolAudio.pop();
      if (!buf || buf.length < lenAudio) buf = new Float32Array(Math.max(POOL_AUDIO_FLOATS, lenAudio));
      buf.set(fatiaAudio);
      const sub = buf.subarray(0, lenAudio);

      if (audioCtx && audioCtx.state === "running" && workletNode) {
        workletNode.port.postMessage(sub, [buf.buffer]);
      } else if (poolAudio.length < POOL_AUDIO_TAMANHO) {
        poolAudio.push(buf);
      }
    }

    // Atualiza Painel de Diagnostico a cada 6 quadros (~10 vezes por segundo)
    if (quadroNum % 6 === 0 && painelDiag && !painelDiag.hidden) {
      const seg = (quadroNum * PERIODO_QUADRO / 1000).toFixed(1);
      const strQuadro = `${quadroNum} (${seg}s de jogo)`;
      if (elDiagQuadro.textContent !== strQuadro) elDiagQuadro.textContent = strQuadro;

      const strAmostras = String(lenAudio);
      if (elDiagAmostras.textContent !== strAmostras) elDiagAmostras.textContent = strAmostras;

      const strPico = picoQuadro.toFixed(4);
      if (elDiagPico.textContent !== strPico) elDiagPico.textContent = strPico;

      const scaleStr = `scaleX(${Math.min(1, picoQuadro).toFixed(3)})`;
      if (elBarraVolume.style.transform !== scaleStr) elBarraVolume.style.transform = scaleStr;

      const strPicoMax = picoHistorico.toFixed(4);
      if (elDiagPicoMax.textContent !== strPicoMax) elDiagPicoMax.textContent = strPicoMax;

      const strQuadrosSom = `${quadrosComSom} quadros`;
      if (elDiagQuadrosSom.textContent !== strQuadrosSom) elDiagQuadrosSom.textContent = strQuadrosSom;

      if (elDiagFps) {
        const strFps = `${fpsApresentado.toFixed(1)} FPS (simulação: ${simulacaoHz.toFixed(1)} Hz)`;
        if (elDiagFps.textContent !== strFps) elDiagFps.textContent = strFps;
      }

      if (elDiagTempoWasm) {
        let soma = 0;
        const n = Math.min(totalHistorico, HISTORICO_TEMPOS_MAX);
        for (let i = 0; i < n; i++) {
          soma += historicoWasmTime[i];
          copiaWasmTime[i] = historicoWasmTime[i];
        }
        // TypedArray ordena numericamente e no lugar.
        const ordenados = copiaWasmTime.subarray(0, n).sort();
        const media = n > 0 ? soma / n : 0;
        const p95 = n > 0 ? ordenados[Math.floor(n * 0.95)] : 0;
        const strTempo = `${media.toFixed(2)} ms (p95: ${p95.toFixed(2)} ms)`;
        if (elDiagTempoWasm.textContent !== strTempo) elDiagTempoWasm.textContent = strTempo;
      }

      atualizarEstadoAudioUI();
    }

    if (jogo.tem_erro()) {
      const erro = jogo.erro();
      const p = document.getElementById("erro-jogo");
      p.hidden = false;
      p.textContent = "O nucleo parou: " + erro;
      if (atualizou) apresentar();
      return null;
    }
  }
  return atualizou;
}

// Vigia: o navegador as vezes para de chamar o rAF por 50 a 240 ms, mesmo com o jogo parado
// (pausa do compositor, docs/relatorio-performance-2.md, secao 3.1). O audio nasce na simulacao
// e faltaria. Se o rAF atrasar mais de 2 quadros, simula pelo temporizador. A imagem segue no rAF.
setInterval(() => {
  if (!jogo || !animacao || document.hidden || contextoPerdido) return;
  if (performance.now() - ultimoRaf < 2 * PERIODO_QUADRO) return;
  simularPassos(performance.now());
}, PERIODO_QUADRO / 2);

async function iniciar(bytes) {
  const sessao = ++geracao;
  funcaoRafAtual = (t) => laco(t, sessao);
  cancelAnimationFrame(animacao);
  animacao = 0;
  await iniciarAudio();
  if (sessao !== geracao) return;

  if (workletNode) workletNode.port.postMessage({ tipo: "reset" });

  try {
    const anterior = jogo;
    const taxaBase = audioCtx ? audioCtx.sampleRate : 48000;
    // O console roda a 53693175 / 896040 = 59,92 Hz; o site roda a 60 Hz exatos. A taxa ajustada
    // faz o reamostrador entregar 800 pares por quadro a 48 kHz (docs/estado.md, decisao 1).
    const taxaAjustada = taxaBase * (53693175 / 896040) / 60;
    jogo = Jogo.criar_com_audio(bytes, taxaAjustada);
    vistaImagem = null;
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
  tela.focus();
  quadroNum = 0;
  relogio.iniciar(performance.now());
  if (!document.hidden && funcaoRafAtual) animacao = requestAnimationFrame(funcaoRafAtual);
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
    if (funcaoRafAtual) animacao = requestAnimationFrame(funcaoRafAtual);
  }
});
