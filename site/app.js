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
const btnMudo = document.getElementById("btn-mudo");
const sliderVolume = document.getElementById("slider-volume");
const painelTeclas = document.getElementById("painel-teclas");

// Badges de Status do Cabecalho
const badgeAudio = document.getElementById("badge-audio");
const badgeGamepad = document.getElementById("badge-gamepad");

// Painel de Telemetria e Diagnostico
const btnToggleDiag = document.getElementById("btn-toggle-diag");
const painelDiag = document.getElementById("painel-diag");
const diagSeta = document.getElementById("diag-seta");
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

// Estado do Audio
let audioCtx = null;
let ganhoMasterNode = null;
let workletNode = null;
let fallbackNode = null;
let volumeAtual = 1.0;
let mudoAtivo = false;

// Pool de buffers de audio reutilizados para transferencia ao worklet
const POOL_AUDIO_TAMANHO = 8;
const poolAudio = [];
for (let i = 0; i < POOL_AUDIO_TAMANHO; i++) {
  poolAudio.push(new Float32Array(4096));
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
      camadaCrt.classList.add("ativo");
    } else {
      camadaCrt.classList.remove("ativo");
    }
  });
}

// Teste de Bipe (Web Audio puro)
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
      osc.frequency.setValueAtTime(440, audioCtx.currentTime);
      ganho.gain.setValueAtTime(0.25, audioCtx.currentTime);
      ganho.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.35);
      osc.connect(ganho);
      ganho.connect(audioCtx.destination);
      osc.start();
      osc.stop(audioCtx.currentTime + 0.35);
    });
  } catch (e) {
    alert("Erro ao tocar bipe de teste: " + e.message);
  }
});

function atualizarEstadoAudioUI() {
  if (!audioCtx) {
    elDiagEstado.innerHTML = `<span style="color:var(--text-muted);">Não inicializado</span>`;
    if (badgeAudio) {
      badgeAudio.className = "pill";
      badgeAudio.textContent = "🔇 Áudio Desligado";
    }
    return;
  }
  const st = audioCtx.state;
  if (st === "running") {
    elDiagEstado.innerHTML = `<span style="color:var(--emerald-green); font-weight:bold;">🟢 Ativo (running)</span>`;
    btnAudio.textContent = "🔊 Áudio Ativo";
    btnAudio.style.background = "var(--emerald-green)";
    if (badgeAudio) {
      badgeAudio.className = "pill ativo";
      badgeAudio.textContent = "🔊 Áudio Ativo";
    }
  } else if (st === "suspended") {
    elDiagEstado.innerHTML = `<span style="color:var(--ring-gold); font-weight:bold;">🟡 Pausado pelo navegador (suspended)</span>`;
    btnAudio.textContent = "⚠️ Desbloquear Áudio";
    btnAudio.style.background = "var(--ring-gold)";
    if (badgeAudio) {
      badgeAudio.className = "pill aviso";
      badgeAudio.textContent = "🟡 Clique para Áudio";
    }
  } else {
    elDiagEstado.innerHTML = `<span style="color:var(--text-muted);">${st}</span>`;
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
  if (tela) tela.focus();
});

async function iniciarAudio() {
  if (audioCtx && (workletNode || fallbackNode)) return;
  try {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    if (!audioCtx) {
      // Usa a taxa nativa do dispositivo para evitar distorcoes de resampling
      audioCtx = new AudioContextClass();
    }

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
          elDiagWorklet.textContent = `Fila: ${e.data.disponiveis} amostras | Falhas: ${e.data.falhas} | Descartadas: ${e.data.descartadas}`;
          if (jogo) jogo.ocupacao_audio(e.data.disponiveis);
        }
      };
      workletNode.onprocessorerror = (e) => {
        elDiagWorklet.textContent = "Erro no AudioWorkletProcessor!";
        console.warn("AudioWorklet erro:", e);
      };
      workletNode.connect(ganhoMasterNode);
      elDiagPipeline.innerHTML = `<span style="color:var(--emerald-green); font-weight:bold;">AudioWorklet</span>`;
    } catch (eWorklet) {
      console.warn("AudioWorklet indisponivel, usando ScriptProcessor:", eWorklet);
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
      spNode.connect(ganhoMasterNode);
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
      elDiagPipeline.innerHTML = `<span style="color:var(--sonic-blue); font-weight:bold;">ScriptProcessor (Fallback universal)</span>`;
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
    elDiagPipeline.innerHTML = `<span style="color:var(--danger-red);">Falha ao iniciar audio: ${e.message}</span>`;
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
    animacao = requestAnimationFrame((t) => laco(t, geracao));
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

function calcularMascaraTeclado() {
  let mascara = 0;
  const modo = jogo?.modo_jogo?.() ?? 4;
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

function iluminarKeycap(code, ativa) {
  if (!code) return;
  const caps = document.querySelectorAll(`.tecla-cap[data-code="${code}"]`);
  caps.forEach((cap) => {
    if (ativa) cap.classList.add("ativa");
    else cap.classList.remove("ativa");
  });
}

window.addEventListener("keydown", (e) => {
  if (e.code === "KeyF" && !e.ctrlKey && !e.metaKey && !e.altKey) {
    alternarTelaCheia();
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
  atualizarBadgeGamepad(e.gamepad);
});

window.addEventListener("gamepaddisconnected", () => {
  atualizarBadgeGamepad(null);
});

function botoesDoControle() {
  if (!navigator.getGamepads) return 0;
  const gamepads = navigator.getGamepads();
  if (!gamepads) return 0;

  const modo = jogo?.modo_jogo?.() ?? 4;
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

function laco(agora, sessao) {
  animacao = 0;
  if (!jogo || sessao !== geracao || document.hidden) return;

  const passos = relogio.avancar(agora);
  let atualizou = false;

  for (let passo = 0; passo < passos; passo++) {
    quadroNum++;

    const estadoControles = calcularMascaraTeclado() | botoesDoControle();
    jogo.botoes(estadoControles);

    if (fallbackNode) jogo.ocupacao_audio(fallbackNode.ocupacao());
    jogo.simular_quadro();
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
      if (diagAberto) {
        for (let i = 0; i < lenAudio; i++) {
          const abs = Math.abs(fatiaAudio[i]);
          if (abs > picoQuadro) picoQuadro = abs;
        }
        if (picoQuadro > picoHistorico) picoHistorico = picoQuadro;
        if (picoQuadro > 0.002) quadrosComSom++;
      } else if (quadrosComSom < 1000 && (fatiaAudio[0] !== 0 || fatiaAudio[lenAudio - 1] !== 0)) {
        quadrosComSom++;
      }

      const buf = poolAudio.pop() || new Float32Array(4096);
      buf.set(fatiaAudio);
      const sub = buf.subarray(0, lenAudio);

      if (workletNode) {
        workletNode.port.postMessage(sub, [buf.buffer]);
      } else if (fallbackNode) {
        fallbackNode.enviar(sub);
        if (poolAudio.length < POOL_AUDIO_TAMANHO) {
          poolAudio.push(buf);
        }
      }
    }

    // Atualiza Painel de Diagnostico a cada 6 quadros (~10 vezes por segundo)
    if (quadroNum % 6 === 0 && painelDiag && !painelDiag.hidden) {
      const seg = (quadroNum * PERIODO_QUADRO / 1000).toFixed(1);
      elDiagQuadro.textContent = `${quadroNum} (${seg}s de jogo)`;
      elDiagAmostras.textContent = String(lenAudio);
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
      p.textContent = "O nucleo parou: " + erro;
      if (atualizou) apresentar();
      return;
    }
  }

  if (fpsDesbloqueado) {
    // Apresenta na taxa nativa do monitor
    apresentar();
  } else if (atualizou) {
    // Apresenta estritamente no ritmo de 60Hz
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
