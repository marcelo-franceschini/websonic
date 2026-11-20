// Coordenador da pagina: WebGL2, AudioWorklet, Worker de simulacao, entrada e HUD.
import { criarRenderizadorGPU } from "./gpu.js";
import { criarControladorEntrada } from "./entrada.js";
import { criarHud } from "./hud.js";

const LIMITE_BYTES = 16 * 1024 * 1024;

// Elementos principais do DOM
const entradaArquivo = document.getElementById("arquivo");
const dropzone = document.getElementById("dropzone");
const caixaResultado = document.getElementById("resultado");
const elVeredito = document.getElementById("veredito");
const elSha1 = document.getElementById("sha1");
const elTamanho = document.getElementById("tamanho");
const elAviso = document.getElementById("aviso");
const elErroJogo = document.getElementById("erro-jogo");

const containerTela = document.getElementById("container-tela");
const camadaCrt = document.getElementById("camada-crt");
const barraControles = document.getElementById("barra-controles");
const seletorFps = document.getElementById("seletor-fps");
const seletorCrt = document.getElementById("seletor-crt");
const seletorCram = document.getElementById("seletor-cram");
const btnResetar = document.getElementById("btn-resetar");
const btnTrocarRom = document.getElementById("btn-trocar-rom");
const btnFullscreen = document.getElementById("btn-fullscreen");
const btnToque = document.getElementById("btn-toque");
const btnHud = document.getElementById("btn-hud");
const btnMudo = document.getElementById("btn-mudo");
const sliderVolume = document.getElementById("slider-volume");
const painelTeclas = document.getElementById("painel-teclas");
const controlesToque = document.getElementById("controles-toque");
const btnAudio = document.getElementById("btn-audio");
const btnBipe = document.getElementById("btn-bipe");

// Estado da sessao e video
let tela = document.getElementById("tela");
let compositor = null;
let ctx2d = null;
const imagem2d = new ImageData(320, 224);
let contextoPerdido = false;
let fpsDesbloqueado = false;

let sessaoAtiva = 0;
let jogando = false;
let romBytesAtual = null;
let animacao = 0;
let quadroAtual = null;
let quadroPendente = null;
let quadroPendenteNovo = false;
let quadroNumAtual = 0;
let ultimaMascaraJogo = -1;
let ultimaMascaraTitulo = -1;

// Estado do audio
let audioCtx = null;
let ganhoMasterNode = null;
let workletNode = null;
let volumeAtual = 1.0;
let mudoAtivo = false;
let ouvintesDesbloqueioRegistrados = false;

// Tabela de conversao de 9 bits para RGBA (Uint32, little-endian) para fallback 2D
const TABELA_COR_9BITS = new Uint32Array(512);
for (let c = 0; c < 512; c++) {
  const r = c & 7;
  const g = (c >> 3) & 7;
  const b = (c >> 6) & 7;
  const r8 = (r << 5) | (r << 2) | (r >> 1);
  const g8 = (g << 5) | (g << 2) | (g >> 1);
  const b8 = (b << 5) | (b << 2) | (b >> 1);
  TABELA_COR_9BITS[c] = 0xFF000000 | (b8 << 16) | (g8 << 8) | r8;
}

// Worker de simulacao e validacao
let worker = null;
let wasmPronto = false;
let proxIdIdent = 1;
const promessasIdent = new Map();

function mostrarErroJogo(texto) {
  if (!elErroJogo) return;
  elErroJogo.textContent = texto;
  elErroJogo.hidden = false;
}

function limparErroJogo() {
  if (!elErroJogo) return;
  elErroJogo.hidden = true;
  elErroJogo.textContent = "";
}

function mostrarResultadoRom(valida, veredito, sha1, tamanho) {
  if (!caixaResultado) return;
  caixaResultado.hidden = false;
  caixaResultado.className = valida ? "ok" : "erro";
  if (elVeredito) elVeredito.textContent = veredito;
  if (elSha1) elSha1.textContent = sha1;
  if (elTamanho) elTamanho.textContent = String(tamanho);
}

// HUD e painel de diagnostico
const hud = criarHud({
  hudPerformance: document.getElementById("hud-performance"),
  btnHud,
  elHudFps: document.getElementById("hud-fps"),
  elHudHz: document.getElementById("hud-hz"),
  elHudFrameTime: document.getElementById("hud-frame-time"),
  elHudWasmTime: document.getElementById("hud-wasm-time"),
  canvasHudGrafico: document.getElementById("hud-grafico"),
  btnToggleDiag: document.getElementById("btn-toggle-diag"),
  painelDiag: document.getElementById("painel-diag"),
  diagSeta: document.getElementById("diag-seta"),
  elDiagEstado: document.getElementById("diag-estado"),
  elDiagTaxa: document.getElementById("diag-taxa"),
  elDiagPipeline: document.getElementById("diag-pipeline"),
  elDiagFps: document.getElementById("diag-fps"),
  elDiagTempoWasm: document.getElementById("diag-tempo-wasm"),
  elDiagQuadro: document.getElementById("diag-quadro"),
  elDiagAmostras: document.getElementById("diag-amostras"),
  elDiagPico: document.getElementById("diag-pico"),
  elBarraVolume: document.getElementById("barra-volume"),
  elDiagPicoMax: document.getElementById("diag-pico-max"),
  elDiagQuadrosSom: document.getElementById("diag-quadros-som"),
  elDiagWorklet: document.getElementById("diag-worklet"),
  badgeAudio: document.getElementById("badge-audio"),
  btnAudio,
  onToggleDiag(aberto) {
    if (worker) worker.postMessage({ tipo: "diag-visivel", visivel: aberto });
  },
});

// Controlador de entrada (teclado, gamepad, toque)
const controladorEntrada = criarControladorEntrada({
  obterTela: () => tela,
  badgeGamepad: document.getElementById("badge-gamepad"),
  controlesToque,
  btnToque,
  onAcaoEspecial(acao) {
    if (acao === "fullscreen") alternarTelaCheia();
    else if (acao === "hud") hud.alternarHud();
  },
  onInteracao: desbloquearAudio,
  estaJogando: () => jogando,
  onMudancaEntrada: sincronizarEntradaWorker,
});

function sincronizarEntradaWorker() {
  if (!worker || !jogando) return;
  const par = controladorEntrada.lerParMascaras();
  if (par.jogo !== ultimaMascaraJogo || par.titulo !== ultimaMascaraTitulo) {
    ultimaMascaraJogo = par.jogo;
    ultimaMascaraTitulo = par.titulo;
    worker.postMessage({
      tipo: "entrada",
      mascaraJogo: par.jogo,
      mascaraTitulo: par.titulo,
    });
  }
}

function devolverBufferVideo(arr) {
  if (!arr || !worker) return;
  worker.postMessage({ tipo: "devolver-video", buffer: arr.buffer }, [arr.buffer]);
}

function inicializarWorker() {
  if (typeof Worker === "undefined") {
    mostrarErroJogo("Este navegador não suporta Web Workers.");
    return;
  }
  try {
    worker = new Worker(new URL("./sim-worker.js", import.meta.url), { type: "module" });
  } catch (e) {
    mostrarErroJogo("Falha ao criar Worker de simulação: " + e.message);
    return;
  }

  worker.onerror = () => {
    wasmPronto = false;
    if (entradaArquivo) entradaArquivo.disabled = true;
    mostrarErroJogo("Falha ao carregar o módulo WebAssembly. Rode 'cargo xtask site' para gerar site/pkg.");
  };

  worker.onmessage = (e) => {
    const msg = e.data;
    if (!msg || !msg.tipo) return;

    switch (msg.tipo) {
      case "pronto": {
        wasmPronto = true;
        if (entradaArquivo) entradaArquivo.disabled = false;
        limparErroJogo();
        break;
      }

      case "erro-wasm": {
        wasmPronto = false;
        if (entradaArquivo) entradaArquivo.disabled = true;
        mostrarErroJogo("Falha ao carregar WebAssembly: " + msg.mensagem);
        break;
      }

      case "identificado": {
        const resolver = promessasIdent.get(msg.id);
        if (resolver) {
          promessasIdent.delete(msg.id);
          resolver(msg);
        }
        break;
      }

      case "iniciado": {
        if (msg.sessao !== sessaoAtiva) return;
        jogando = true;
        limparErroJogo();
        if (dropzone) dropzone.hidden = true;
        if (containerTela) containerTela.hidden = false;
        if (tela) tela.hidden = false;
        if (barraControles) barraControles.hidden = false;
        if (painelTeclas) painelTeclas.hidden = false;
        if (controlesToque) controlesToque.hidden = !controladorEntrada.toqueEstaAtivo();
        if (elAviso) elAviso.hidden = false;
        ajustarEscala();
        if (tela) tela.focus();
        hud.resetar();
        ultimaMascaraJogo = -1;
        ultimaMascaraTitulo = -1;
        sincronizarEntradaWorker();
        if (!document.hidden && !contextoPerdido && !animacao) {
          animacao = requestAnimationFrame(lacoApresentacao);
        }
        break;
      }

      case "erro-iniciar": {
        if (msg.sessao !== sessaoAtiva) return;
        jogando = false;
        if (dropzone) dropzone.hidden = false;
        mostrarErroJogo("Falha ao iniciar a simulação: " + msg.mensagem);
        break;
      }

      case "quadro": {
        const novo = new Uint16Array(msg.buffer);
        if (msg.sessao !== sessaoAtiva || !jogando) {
          devolverBufferVideo(novo);
          return;
        }
        if (quadroPendente) {
          devolverBufferVideo(quadroPendente);
        }
        quadroPendente = novo;
        quadroPendenteNovo = true;
        quadroNumAtual = msg.quadroNum;
        hud.registrarTempoWasm(msg.tempoWasm);
        if (msg.diag) {
          hud.atualizarDiag(msg.diag, audioCtx);
        }
        break;
      }

      case "worklet-status": {
        if (msg.sessao === sessaoAtiva) {
          hud.atualizarWorklet(msg.status);
        }
        break;
      }

      case "erro-nucleo": {
        if (msg.sessao !== sessaoAtiva) return;
        jogando = false;
        cancelAnimationFrame(animacao);
        animacao = 0;
        if (quadroPendente) {
          if (quadroAtual) devolverBufferVideo(quadroAtual);
          quadroAtual = quadroPendente;
          quadroPendente = null;
          quadroPendenteNovo = false;
          apresentar(quadroAtual);
        }
        mostrarErroJogo("O núcleo parou: " + msg.mensagem);
        break;
      }
    }
  };

  worker.postMessage({ tipo: "init" });
}

inicializarWorker();

// Audio
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

function criarAudioContext() {
  if (!audioCtx) {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    audioCtx = new AudioContextClass();
    audioCtx.addEventListener("statechange", () => {
      const ativo = audioCtx.state === "running";
      if (worker) {
        worker.postMessage({ tipo: "audio-ativo", ativo, resetar: ativo });
      }
      hud.atualizarEstadoAudio(audioCtx);
    });
  }
  return audioCtx;
}

function desbloquearAudio() {
  if (audioCtx && audioCtx.state === "suspended") {
    audioCtx.resume().then(() => hud.atualizarEstadoAudio(audioCtx)).catch(() => {});
  }
}

if (btnAudio) {
  btnAudio.addEventListener("click", () => {
    desbloquearAudio();
    if (tela) tela.focus();
  });
}

if (btnBipe) {
  btnBipe.addEventListener("click", () => {
    try {
      const ctx = criarAudioContext();
      ctx.resume().then(() => {
        hud.atualizarEstadoAudio(audioCtx);
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
      }).catch((e) => {
        mostrarErroJogo("Erro ao retomar áudio para bipe de teste: " + e.message);
      });
    } catch (e) {
      mostrarErroJogo("Erro ao tocar bipe de teste: " + e.message);
    }
  });
}

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
      await audioCtx.audioWorklet.addModule("audio-worklet.js");
      try {
        workletNode = new AudioWorkletNode(audioCtx, "sonic-audio-processor", {
          outputChannelCount: [2],
        });
      } catch {
        workletNode = new AudioWorkletNode(audioCtx, "sonic-audio-processor");
      }
      workletNode.onprocessorerror = (e) => {
        hud.definirErroWorklet("Erro no AudioWorkletProcessor!");
        console.warn("AudioWorklet erro:", e);
      };
      workletNode.connect(ganhoMasterNode);
      hud.definirPipeline("AudioWorklet (Worker direto)", "diag-ativo");
    } catch (eWorklet) {
      console.warn("AudioWorklet indisponivel:", eWorklet);
      hud.definirPipeline("AudioWorklet indisponivel", "diag-aviso");
      hud.definirErroWorklet("AudioWorklet indisponivel");
    }

    if (!ouvintesDesbloqueioRegistrados) {
      ouvintesDesbloqueioRegistrados = true;
      window.addEventListener("click", desbloquearAudio);
      window.addEventListener("keydown", desbloquearAudio);
      window.addEventListener("pointerdown", desbloquearAudio);
      window.addEventListener("touchstart", desbloquearAudio);
    }

    desbloquearAudio();
    hud.atualizarEstadoAudio(audioCtx);
  } catch (e) {
    hud.definirPipeline(`Falha ao iniciar audio: ${e.message}`, "diag-erro");
  }
}

// Video (WebGL2 + fallback 2D)
function registrarCliqueTela(canvas) {
  canvas.addEventListener("click", () => {
    canvas.focus();
    desbloquearAudio();
  });
}

function usarCpu(motivo) {
  const nova = tela.cloneNode(false);
  tela.replaceWith(nova);
  tela = nova;
  tela.tabIndex = 0;
  registrarCliqueTela(tela);
  compositor = null;
  ctx2d = tela.getContext("2d");
  contextoPerdido = false;
  mostrarErroJogo(`Vídeo por software: ${motivo}`);
}

registrarCliqueTela(tela);

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
  if (worker) worker.postMessage({ tipo: "pausar" });
  if (audioCtx && audioCtx.state === "running") {
    audioCtx.suspend().then(() => {
      if (!contextoPerdido && !document.hidden) desbloquearAudio();
    }).catch(() => {});
  }
  mostrarErroJogo("Contexto gráfico perdido. Aguardando recuperação.");
});

tela.addEventListener("webglcontextrestored", () => {
  try {
    compositor = criarRenderizadorGPU(tela);
    if (!compositor) throw new Error("WebGL2 indisponível");
    contextoPerdido = false;
    limparErroJogo();
  } catch (erro) {
    usarCpu(`falha ao restaurar WebGL2 (${erro.message}).`);
  }
  if (jogando && !document.hidden) {
    if (worker) worker.postMessage({ tipo: "retomar" });
    desbloquearAudio();
    if (!animacao) animacao = requestAnimationFrame(lacoApresentacao);
  }
});

function apresentar(pts) {
  if (contextoPerdido || !pts) return;
  if (compositor) {
    try {
      compositor.renderizar(pts);
    } catch (erro) {
      usarCpu(`falha de WebGL2 (${erro.message}).`);
    }
  } else if (ctx2d) {
    const d32 = new Uint32Array(imagem2d.data.buffer);
    for (let i = 0; i < 71680; i++) {
      d32[i] = TABELA_COR_9BITS[pts[i] & 0x1FF];
    }
    ctx2d.putImageData(imagem2d, 0, 0);
  }
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

// Controles da barra superior
if (seletorFps) {
  seletorFps.addEventListener("change", () => {
    fpsDesbloqueado = seletorFps.value === "desbloqueado";
    if (tela) tela.focus();
  });
}

if (seletorCrt && camadaCrt) {
  seletorCrt.addEventListener("change", () => {
    camadaCrt.classList.toggle("ativo", seletorCrt.value === "ligado");
    if (tela) tela.focus();
  });
}

if (seletorCram) {
  seletorCram.addEventListener("change", () => {
    if (worker) {
      worker.postMessage({ tipo: "pontos-cram", valor: seletorCram.value === "hardware" });
    }
    if (tela) tela.focus();
  });
}

if (btnFullscreen) {
  btnFullscreen.addEventListener("click", () => {
    alternarTelaCheia();
    if (tela) tela.focus();
  });
}

if (btnResetar) {
  btnResetar.addEventListener("click", () => {
    if (romBytesAtual) {
      iniciarSessao(romBytesAtual);
    }
  });
}

if (btnTrocarRom) {
  btnTrocarRom.addEventListener("click", () => {
    encerrarSessaoAtual();
    if (dropzone) dropzone.hidden = false;
    if (containerTela) containerTela.hidden = true;
    if (barraControles) barraControles.hidden = true;
    if (painelTeclas) painelTeclas.hidden = true;
    if (controlesToque) controlesToque.hidden = true;
    if (entradaArquivo) entradaArquivo.value = "";
    if (elAviso) elAviso.hidden = true;
    limparErroJogo();
  });
}

// Laco de apresentacao na thread principal (apenas GPU, HUD e polling de gamepad)
function lacoApresentacao(agora) {
  animacao = 0;
  if (!jogando || document.hidden || contextoPerdido) return;

  const inicioQuadro = performance.now();

  if (controladorEntrada.temGamepad()) {
    sincronizarEntradaWorker();
  }

  let apresentouNovo = false;
  if (quadroPendenteNovo && quadroPendente) {
    if (quadroAtual) {
      devolverBufferVideo(quadroAtual);
    }
    quadroAtual = quadroPendente;
    quadroPendente = null;
    quadroPendenteNovo = false;
    apresentar(quadroAtual);
    apresentouNovo = true;
  } else if (fpsDesbloqueado && quadroAtual) {
    apresentar(quadroAtual);
  }

  const frameTimeMs = performance.now() - inicioQuadro;
  if (apresentouNovo || fpsDesbloqueado) {
    hud.registrarApresentacao(agora, frameTimeMs, quadroNumAtual);
  }

  animacao = requestAnimationFrame(lacoApresentacao);
}

// Carga de ROM e ciclo de sessao
function identificarNoWorker(bytes) {
  return new Promise((resolve) => {
    const id = proxIdIdent++;
    promessasIdent.set(id, resolve);
    // Copia leve para manter `bytes` disponivel caso o usuario inicie o jogo em seguida
    const copia = bytes.slice();
    worker.postMessage({ tipo: "identificar", id, bytes: copia }, [copia.buffer]);
  });
}

async function carregarArquivoRom(arquivo) {
  if (!arquivo || !wasmPronto) return;
  if (arquivo.size > LIMITE_BYTES) {
    mostrarResultadoRom(false, "Recusada: arquivo grande demais para ser a REV01.", "(não calculado)", arquivo.size);
    return;
  }
  mostrarResultadoRom(true, "Lendo e validando ROM...", "calculando...", arquivo.size);
  const bytes = new Uint8Array(await arquivo.arrayBuffer());
  const r = await identificarNoWorker(bytes);
  mostrarResultadoRom(r.valida, r.valida ? "ROM REV01 válida." : "Recusada: " + r.motivo + ".", r.sha1, r.tamanho);
  if (r.valida) {
    romBytesAtual = bytes;
    iniciarSessao(bytes);
  }
}

if (entradaArquivo) {
  entradaArquivo.addEventListener("change", () => {
    carregarArquivoRom(entradaArquivo.files[0]);
  });
}

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

function encerrarSessaoAtual() {
  sessaoAtiva++;
  jogando = false;
  cancelAnimationFrame(animacao);
  animacao = 0;
  if (worker) worker.postMessage({ tipo: "parar" });
  if (quadroPendente) {
    devolverBufferVideo(quadroPendente);
    quadroPendente = null;
    quadroPendenteNovo = false;
  }
  if (quadroAtual) {
    devolverBufferVideo(quadroAtual);
    quadroAtual = null;
  }
}

async function iniciarSessao(bytes) {
  encerrarSessaoAtual();
  const sessao = sessaoAtiva;

  await iniciarAudio();
  if (sessao !== sessaoAtiva) return;

  let portaWorkerAudio = null;
  const transferiveis = [];
  if (workletNode && typeof MessageChannel !== "undefined") {
    const canal = new MessageChannel();
    workletNode.port.postMessage({ tipo: "porta", porta: canal.port1 }, [canal.port1]);
    portaWorkerAudio = canal.port2;
    transferiveis.push(canal.port2);
  }

  const copiaRom = bytes.slice();
  transferiveis.push(copiaRom.buffer);

  worker.postMessage(
    {
      tipo: "iniciar",
      sessao,
      bytes: copiaRom,
      taxaBase: audioCtx ? audioCtx.sampleRate : 48000,
      pontosCram: Boolean(seletorCram && seletorCram.value === "hardware"),
      audioAtivo: Boolean(audioCtx && audioCtx.state === "running"),
      diagVisivel: hud.estaDiagAberto(),
      portaAudio: portaWorkerAudio,
    },
    transferiveis,
  );
}

document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    cancelAnimationFrame(animacao);
    animacao = 0;
    if (worker && jogando) worker.postMessage({ tipo: "pausar" });
    if (audioCtx && audioCtx.state === "running") {
      audioCtx.suspend().then(() => {
        if (!document.hidden && !contextoPerdido) desbloquearAudio();
      }).catch(() => {});
    }
  } else if (jogando && !contextoPerdido) {
    desbloquearAudio();
    if (worker) worker.postMessage({ tipo: "retomar" });
    if (!animacao) animacao = requestAnimationFrame(lacoApresentacao);
  }
});
