// Orquestrador da pagina: conecta Worker de simulacao, video, audio, entrada, Save States, HUD e PWA.
import { criarControladorAudio } from "./audio.js";
import { criarApresentadorVideo } from "./video.js";
import { carregarPreferenciasVisuais, salvarPreferenciasVisuais } from "./preferencias.js";
import { criarModalControles } from "./modal-controles.js";
import { criarControladorEntrada } from "./entrada.js";
import { criarHud } from "./hud.js";
import { criarControladorSavesUi } from "./save-ui.js";
import { inicializarPwa } from "./pwa.js";

const LIMITE_BYTES = 16 * 1024 * 1024;

// Referencias do DOM
const deckInicial = document.getElementById("deck-inicial");
const entradaArquivo = document.getElementById("arquivo");
const dropzone = document.getElementById("dropzone");
const caixaResultado = document.getElementById("resultado");
const elVeredito = document.getElementById("veredito");
const elSha1 = document.getElementById("sha1");
const elTamanho = document.getElementById("tamanho");
const elAviso = document.getElementById("aviso");
const elErroJogo = document.getElementById("erro-jogo");
const btnNavDiag = document.getElementById("btn-nav-diag");
const btnToggleDiag = document.getElementById("btn-toggle-diag");
const painelDiag = document.getElementById("painel-diag");

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
const painelTeclas = document.getElementById("painel-teclas");
const controlesToque = document.getElementById("controles-toque");

const btnPausar = document.getElementById("btn-pausar");
const btnPasso = document.getElementById("btn-passo");
const btnRapido = document.getElementById("btn-rapido");

// Estado da sessao
let worker = null;
let wasmPronto = false;
let sessaoAtiva = 0;
let jogando = false;
let pausadoUsuario = false;
let fastForwardToggle = false;
let fastForwardHold = false;
let fpsDesbloqueado = false;
let romBytesAtual = null;
let romSha1Atual = "";
let animacao = 0;
let quadroAtual = null;
let quadroPendente = null;
let quadroPendenteNovo = false;
let quadroGpuNovo = false;
let quadroNumAtual = 0;
let ultimaMascaraJogo = -1;
let ultimaMascaraTitulo = -1;
let proxIdReq = 1;
let idLeituraRom = 0;
const promessasIdent = new Map();
const promessasSave = new Map();

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

// 1. PWA
inicializarPwa({
  btnInstalar: document.getElementById("btn-instalar"),
  badgeOffline: document.getElementById("badge-offline"),
  modalIos: document.getElementById("modal-ios-pwa"),
  btnFecharModalIos: document.getElementById("btn-fechar-ios-pwa"),
});

// 2. HUD e painel de diagnostico
const hud = criarHud({
  hudPerformance: document.getElementById("hud-performance"),
  btnHud,
  elHudFps: document.getElementById("hud-fps"),
  elHudHz: document.getElementById("hud-hz"),
  elHudFrameTime: document.getElementById("hud-frame-time"),
  elHudWasmTime: document.getElementById("hud-wasm-time"),
  canvasHudGrafico: document.getElementById("hud-grafico"),
  btnToggleDiag,
  painelDiag,
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
  btnAudio: document.getElementById("btn-audio"),
  onToggleDiag(aberto) {
    if (worker) worker.postMessage({ tipo: "diag-visivel", visivel: aberto });
  },
});

if (btnNavDiag && btnToggleDiag && painelDiag) {
  btnNavDiag.addEventListener("click", () => {
    if (painelDiag.hidden) {
      btnToggleDiag.click();
    }
    const reduzido = Boolean(
      window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    );
    btnToggleDiag.scrollIntoView({ behavior: reduzido ? "auto" : "smooth", block: "nearest" });
    btnToggleDiag.focus({ preventScroll: true });
  });
}

// 3. Video (OffscreenCanvas zero-copy + fallback WebGL2/2D + tela cheia)
let controladorAudio = null;

const apresentadorVideo = criarApresentadorVideo({
  canvasInicial: document.getElementById("tela"),
  containerTela,
  badgeGpu: document.getElementById("badge-gpu"),
  onErro: mostrarErroJogo,
  onLimparErro: limparErroJogo,
  onInteracao: () => controladorAudio && controladorAudio.desbloquear(),
  onContextoPerdido() {
    cancelAnimationFrame(animacao);
    animacao = 0;
    if (worker) worker.postMessage({ tipo: "pausar" });
    if (controladorAudio) controladorAudio.suspenderSeAtivo();
  },
  onContextoRestaurado() {
    if (jogando && !document.hidden) {
      if (worker) worker.postMessage({ tipo: "retomar" });
      if (!pausadoUsuario && controladorAudio) controladorAudio.desbloquear();
      if (!animacao) animacao = requestAnimationFrame(lacoApresentacao);
    }
  },
});

// 4. Audio (AudioContext + AudioWorklet + volume + bipe de teste)
controladorAudio = criarControladorAudio({
  btnMudo: document.getElementById("btn-mudo"),
  sliderVolume: document.getElementById("slider-volume"),
  btnAudio: document.getElementById("btn-audio"),
  btnBipe: document.getElementById("btn-bipe"),
  hud,
  obterTela: () => apresentadorVideo.obterTela(),
  obterWorker: () => worker,
  estaPausadoUsuario: () => pausadoUsuario,
  podeRetomarAudio: () => !document.hidden && !apresentadorVideo.estaPerdido() && !pausadoUsuario,
  onErro: mostrarErroJogo,
});

// 5. Entrada e Modal de Remapeamento
const controladorEntrada = criarControladorEntrada({
  obterTela: () => apresentadorVideo.obterTela(),
  badgeGamepad: document.getElementById("badge-gamepad"),
  controlesToque,
  btnToque,
  onAcaoEspecial(acao) {
    if (acao === "fullscreen") apresentadorVideo.alternarTelaCheia();
    else if (acao === "hud") hud.alternarHud();
    else if (acao === "salvar-rapido") controladorSavesUi.salvarNoSlotAtual();
    else if (acao === "carregar-rapido") controladorSavesUi.carregarDoSlotAtual();
    else if (acao === "fast-forward-inicio") {
      fastForwardHold = true;
      atualizarVelocidadeWorker();
    } else if (acao === "fast-forward-fim" && fastForwardHold) {
      fastForwardHold = false;
      atualizarVelocidadeWorker();
    }
  },
  onInteracao: () => controladorAudio.desbloquear(),
  estaJogando: () => jogando,
  onMudancaEntrada: sincronizarEntradaWorker,
});

criarModalControles({
  modal: document.getElementById("modal-controles"),
  listaRemap: document.getElementById("lista-remap"),
  botoesAbrir: [
    document.getElementById("btn-controles-topo"),
    document.getElementById("btn-controles-spec"),
    document.getElementById("btn-controles"),
  ],
  btnFechar: document.getElementById("btn-fechar-controles"),
  btnConcluir: document.getElementById("btn-concluir-controles"),
  btnRestaurar: document.getElementById("btn-restaurar-controles"),
  controladorEntrada,
  onFechar() {
    const tela = apresentadorVideo.obterTela();
    if (jogando && tela) tela.focus();
  },
});

function focarTela() {
  const tela = apresentadorVideo.obterTela();
  if (tela) tela.focus();
}

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

// 6. Controles de reproducao (Pausa, +1 Quadro, Fast-Forward 2x)
function atualizarUiPausa() {
  if (btnPausar) {
    const rotulo = btnPausar.querySelector(".btn-label");
    if (rotulo) {
      rotulo.textContent = pausadoUsuario ? "Retomar" : "Pausar";
    } else {
      btnPausar.textContent = pausadoUsuario ? "Retomar" : "Pausar";
    }
    btnPausar.classList.toggle("pausado", pausadoUsuario);
    btnPausar.classList.toggle("btn-azul", pausadoUsuario);
    btnPausar.classList.toggle("btn-cinza", !pausadoUsuario);
    btnPausar.setAttribute("aria-pressed", String(pausadoUsuario));
  }
  if (btnPasso) {
    btnPasso.disabled = !jogando || !pausadoUsuario;
  }
}

function definirPausaUsuario(novoEstado) {
  pausadoUsuario = Boolean(novoEstado);
  atualizarUiPausa();
  if (worker && jogando) {
    worker.postMessage({ tipo: "pausar-usuario", pausado: pausadoUsuario });
  }
  if (pausadoUsuario) {
    controladorAudio.suspenderSeAtivo();
  } else if (jogando && !document.hidden && !apresentadorVideo.estaPerdido()) {
    controladorAudio.desbloquear();
  }
}

function atualizarVelocidadeWorker() {
  const mult = (fastForwardToggle || fastForwardHold) ? 2 : 1;
  if (btnRapido) {
    btnRapido.classList.toggle("btn-azul", mult === 2);
    btnRapido.classList.toggle("btn-cinza", mult !== 2);
    btnRapido.setAttribute("aria-pressed", String(mult === 2));
  }
  if (worker && jogando) {
    worker.postMessage({ tipo: "velocidade", multiplicador: mult });
  }
}

if (btnPausar) {
  btnPausar.addEventListener("click", () => {
    if (!jogando) return;
    definirPausaUsuario(!pausadoUsuario);
    focarTela();
  });
}

if (btnPasso) {
  btnPasso.addEventListener("click", () => {
    if (!jogando || !worker) return;
    if (!pausadoUsuario) definirPausaUsuario(true);
    worker.postMessage({ tipo: "passo-unico" });
    focarTela();
  });
}

if (btnRapido) {
  btnRapido.addEventListener("click", () => {
    fastForwardToggle = !fastForwardToggle;
    atualizarVelocidadeWorker();
    focarTela();
  });
}

// 7. Save States (IndexedDB + .s1sav)
function solicitarSaveWorker() {
  return new Promise((resolve, reject) => {
    if (!worker || !jogando) {
      reject(new Error("Nenhuma partida em andamento."));
      return;
    }
    const id = proxIdReq++;
    promessasSave.set(id, { resolve, reject });
    worker.postMessage({ tipo: "exportar-save", id });
  });
}

const controladorSavesUi = criarControladorSavesUi({
  seletorSlot: document.getElementById("seletor-slot"),
  btnSalvar: document.getElementById("btn-salvar"),
  btnCarregar: document.getElementById("btn-carregar"),
  btnExportar: document.getElementById("btn-exportar-save"),
  btnImportar: document.getElementById("btn-importar-save"),
  inputImportar: document.getElementById("input-importar-save"),
  elStatus: document.getElementById("status-save"),
  estaJogando: () => jogando,
  obterRomAtual: () => ({ bytes: romBytesAtual, sha1: romSha1Atual }),
  solicitarSaveWorker,
  onCarregarSave: iniciarSessaoComSave,
  onErro: mostrarErroJogo,
  onFocarTela: focarTela,
});

// 8. Worker de simulacao Wasm
function ativarInterfaceJogo() {
  jogando = true;
  limparErroJogo();
  if (deckInicial) deckInicial.hidden = true;
  if (dropzone) dropzone.hidden = true;
  if (containerTela) containerTela.hidden = false;
  const tela = apresentadorVideo.obterTela();
  if (tela) tela.hidden = false;
  if (barraControles) barraControles.hidden = false;
  if (painelTeclas) painelTeclas.hidden = false;
  if (controlesToque) controlesToque.hidden = !controladorEntrada.toqueEstaAtivo();
  if (elAviso) elAviso.hidden = false;
  atualizarUiPausa();
  atualizarVelocidadeWorker();
  controladorSavesUi.atualizarRotulosSlots();
  apresentadorVideo.ajustarEscala();
  focarTela();
  hud.resetar();
  ultimaMascaraJogo = -1;
  ultimaMascaraTitulo = -1;
  sincronizarEntradaWorker();
  if (!document.hidden && !apresentadorVideo.estaPerdido() && !animacao) {
    animacao = requestAnimationFrame(lacoApresentacao);
  }
}

function inicializarWorker() {
  if (typeof Worker === "undefined") {
    mostrarErroJogo("Este navegador não suporta Web Workers.");
    apresentadorVideo.inicializarThreadPrincipal(false);
    return;
  }
  try {
    worker = new Worker(new URL("./sim-worker.js", import.meta.url), { type: "module" });
  } catch (e) {
    mostrarErroJogo("Falha ao criar Worker de simulação: " + e.message);
    apresentadorVideo.inicializarThreadPrincipal(false);
    return;
  }

  worker.onerror = () => {
    wasmPronto = false;
    if (entradaArquivo) entradaArquivo.disabled = true;
    for (const resolver of promessasIdent.values()) {
      resolver({
        valida: false,
        motivo: "Worker de simulação indisponível",
        sha1: "(erro)",
        tamanho: 0,
      });
    }
    promessasIdent.clear();
    for (const p of promessasSave.values()) {
      p.reject(new Error("Worker de simulação indisponível."));
    }
    promessasSave.clear();
    mostrarErroJogo("Falha ao carregar o módulo WebAssembly. Rode 'cargo xtask site' para gerar site/pkg.");
  };

  worker.onmessage = (e) => {
    const msg = e.data;
    if (!msg || !msg.tipo) return;

    switch (msg.tipo) {
      case "pronto":
        wasmPronto = true;
        if (entradaArquivo) entradaArquivo.disabled = false;
        limparErroJogo();
        break;

      case "canvas-anexado":
        apresentadorVideo.confirmarAnexoWorker(msg.ok);
        break;

      case "gpu-contexto-perdido":
        apresentadorVideo.notificarPerdaNoWorker();
        break;

      case "gpu-contexto-restaurado":
        apresentadorVideo.notificarRestauracaoNoWorker(msg.ok);
        break;

      case "erro-wasm":
        wasmPronto = false;
        if (entradaArquivo) entradaArquivo.disabled = true;
        mostrarErroJogo("Falha ao carregar WebAssembly: " + msg.mensagem);
        break;

      case "identificado": {
        const resolver = promessasIdent.get(msg.id);
        if (resolver) {
          promessasIdent.delete(msg.id);
          resolver(msg);
        }
        break;
      }

      case "save-pronto": {
        const p = promessasSave.get(msg.id);
        if (p) {
          promessasSave.delete(msg.id);
          if (msg.erro) p.reject(new Error(msg.erro));
          else p.resolve(msg);
        }
        break;
      }

      case "save-carregado":
        if (msg.sessao !== sessaoAtiva) return;
        ativarInterfaceJogo();
        controladorSavesUi.mostrarStatus(`Estado restaurado (q. ${msg.quadro})`);
        break;

      case "erro-save":
        if (msg.sessao !== sessaoAtiva) return;
        mostrarErroJogo("Falha ao carregar save state: " + msg.mensagem);
        break;

      case "iniciado":
        if (msg.sessao !== sessaoAtiva) return;
        ativarInterfaceJogo();
        break;

      case "erro-iniciar":
        if (msg.sessao !== sessaoAtiva) return;
        jogando = false;
        if (deckInicial) deckInicial.hidden = false;
        if (dropzone) dropzone.hidden = false;
        mostrarErroJogo("Falha ao iniciar a simulação: " + msg.mensagem);
        break;

      case "quadro-gpu":
        if (msg.sessao !== sessaoAtiva || !jogando) return;
        quadroNumAtual = msg.quadroNum;
        quadroGpuNovo = true;
        hud.registrarTempoWasm(msg.tempoWasm);
        if (msg.diag) hud.atualizarDiag(msg.diag, controladorAudio.obterContexto());
        break;

      case "quadro": {
        const novo = new Uint16Array(msg.buffer);
        if (msg.sessao !== sessaoAtiva || !jogando) {
          devolverBufferVideo(novo);
          return;
        }
        if (quadroPendente) devolverBufferVideo(quadroPendente);
        quadroPendente = novo;
        quadroPendenteNovo = true;
        quadroNumAtual = msg.quadroNum;
        hud.registrarTempoWasm(msg.tempoWasm);
        if (msg.diag) hud.atualizarDiag(msg.diag, controladorAudio.obterContexto());
        break;
      }

      case "worklet-status":
        if (msg.sessao === sessaoAtiva) hud.atualizarWorklet(msg.status);
        break;

      case "erro-nucleo":
        if (msg.sessao !== sessaoAtiva) return;
        jogando = false;
        cancelAnimationFrame(animacao);
        animacao = 0;
        if (quadroPendente) {
          if (quadroAtual) devolverBufferVideo(quadroAtual);
          quadroAtual = quadroPendente;
          quadroPendente = null;
          quadroPendenteNovo = false;
          apresentadorVideo.apresentar(quadroAtual);
        }
        mostrarErroJogo("O núcleo parou: " + msg.mensagem);
        break;
    }
  };

  worker.postMessage({ tipo: "init" });
  apresentadorVideo.tentarAnexarNoWorker(worker);
}

inicializarWorker();

// 9. Controles da barra superior, preferencias visuais persistentes e laco de apresentacao
const storageVisual = typeof window !== "undefined" ? window.localStorage : null;
const prefsVisuais = carregarPreferenciasVisuais(storageVisual);

function aplicarPreferenciasVisuais() {
  fpsDesbloqueado = prefsVisuais.fps === "desbloqueado";
  if (seletorFps) seletorFps.value = prefsVisuais.fps;
  if (seletorCrt) seletorCrt.value = prefsVisuais.crt;
  if (camadaCrt) camadaCrt.classList.toggle("ativo", prefsVisuais.crt === "ligado");
  if (seletorCram) seletorCram.value = prefsVisuais.cram;
}

aplicarPreferenciasVisuais();

if (seletorFps) {
  seletorFps.addEventListener("change", () => {
    prefsVisuais.fps = seletorFps.value;
    fpsDesbloqueado = prefsVisuais.fps === "desbloqueado";
    salvarPreferenciasVisuais(storageVisual, prefsVisuais);
    focarTela();
  });
}

if (seletorCrt && camadaCrt) {
  seletorCrt.addEventListener("change", () => {
    prefsVisuais.crt = seletorCrt.value;
    camadaCrt.classList.toggle("ativo", prefsVisuais.crt === "ligado");
    salvarPreferenciasVisuais(storageVisual, prefsVisuais);
    focarTela();
  });
}

if (seletorCram) {
  seletorCram.addEventListener("change", () => {
    prefsVisuais.cram = seletorCram.value;
    salvarPreferenciasVisuais(storageVisual, prefsVisuais);
    if (worker) {
      worker.postMessage({ tipo: "pontos-cram", valor: prefsVisuais.cram === "hardware" });
    }
    focarTela();
  });
}

if (btnFullscreen) {
  btnFullscreen.addEventListener("click", () => {
    apresentadorVideo.alternarTelaCheia();
    focarTela();
  });
}

if (btnResetar) {
  btnResetar.addEventListener("click", () => {
    if (romBytesAtual) iniciarSessao(romBytesAtual);
  });
}

if (btnTrocarRom) {
  btnTrocarRom.addEventListener("click", () => {
    encerrarSessaoAtual();
    if (deckInicial) deckInicial.hidden = false;
    if (dropzone) dropzone.hidden = false;
    if (caixaResultado) caixaResultado.hidden = true;
    if (containerTela) containerTela.hidden = true;
    if (barraControles) barraControles.hidden = true;
    if (painelTeclas) painelTeclas.hidden = true;
    if (controlesToque) controlesToque.hidden = true;
    if (entradaArquivo) entradaArquivo.value = "";
    if (elAviso) elAviso.hidden = true;
    limparErroJogo();
  });
}

function lacoApresentacao(agora) {
  animacao = 0;
  if (!jogando || document.hidden || apresentadorVideo.estaPerdido()) return;

  const inicioQuadro = performance.now();

  if (controladorEntrada.temGamepad()) {
    sincronizarEntradaWorker();
  }

  let apresentouNovo = false;
  if (apresentadorVideo.estaNoWorker()) {
    if (quadroGpuNovo) {
      quadroGpuNovo = false;
      apresentouNovo = true;
    }
  } else if (quadroPendenteNovo && quadroPendente) {
    if (quadroAtual) devolverBufferVideo(quadroAtual);
    quadroAtual = quadroPendente;
    quadroPendente = null;
    quadroPendenteNovo = false;
    apresentadorVideo.apresentar(quadroAtual);
    apresentouNovo = true;
  } else if (fpsDesbloqueado && quadroAtual) {
    apresentadorVideo.apresentar(quadroAtual);
  }

  const frameTimeMs = performance.now() - inicioQuadro;
  if (apresentouNovo || fpsDesbloqueado) {
    hud.registrarApresentacao(agora, frameTimeMs, quadroNumAtual);
  }

  animacao = requestAnimationFrame(lacoApresentacao);
}

// 10. Carga de ROM e ciclo de sessao
function identificarNoWorker(bytes) {
  return new Promise((resolve) => {
    const id = proxIdReq++;
    promessasIdent.set(id, resolve);
    const copia = bytes.slice();
    worker.postMessage({ tipo: "identificar", id, bytes: copia }, [copia.buffer]);
  });
}

async function carregarArquivoRom(arquivo) {
  if (!arquivo || !wasmPronto) return;
  const leitura = ++idLeituraRom;
  if (arquivo.size > LIMITE_BYTES) {
    mostrarResultadoRom(false, "Recusada: arquivo grande demais para ser a REV01.", "(não calculado)", arquivo.size);
    return;
  }
  mostrarResultadoRom(true, "Lendo e validando ROM...", "calculando...", arquivo.size);
  try {
    const bytes = new Uint8Array(await arquivo.arrayBuffer());
    if (leitura !== idLeituraRom) return;
    const r = await identificarNoWorker(bytes);
    if (leitura !== idLeituraRom) return;
    mostrarResultadoRom(r.valida, r.valida ? "ROM REV01 válida." : "Recusada: " + r.motivo + ".", r.sha1, r.tamanho);
    if (r.valida) {
      romBytesAtual = bytes;
      romSha1Atual = r.sha1;
      iniciarSessao(bytes);
    }
  } catch (err) {
    if (leitura !== idLeituraRom) return;
    const msg = err && err.message ? err.message : String(err);
    mostrarResultadoRom(false, "Falha ao ler arquivo: " + msg, "(erro)", arquivo.size || 0);
  }
}

if (entradaArquivo) {
  entradaArquivo.addEventListener("change", () => carregarArquivoRom(entradaArquivo.files[0]));
}

if (dropzone) {
  dropzone.addEventListener("dragover", (e) => {
    e.preventDefault();
    dropzone.classList.add("arrastando");
  });
  dropzone.addEventListener("dragleave", () => dropzone.classList.remove("arrastando"));
  dropzone.addEventListener("drop", (e) => {
    e.preventDefault();
    dropzone.classList.remove("arrastando");
    const arquivos = e.dataTransfer?.files;
    if (arquivos && arquivos.length > 0) carregarArquivoRom(arquivos[0]);
  });
}

function encerrarSessaoAtual() {
  sessaoAtiva++;
  jogando = false;
  pausadoUsuario = false;
  fastForwardToggle = false;
  fastForwardHold = false;
  quadroGpuNovo = false;
  atualizarUiPausa();
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

async function enviarSessaoWorker(tipo, bytes, extras = {}) {
  const sessao = sessaoAtiva;
  await controladorAudio.iniciar();
  if (sessao !== sessaoAtiva) return;

  const transferiveis = [];
  const portaAudio = controladorAudio.criarPortaParaWorker();
  if (portaAudio) transferiveis.push(portaAudio);

  const copiaRom = bytes.slice();
  transferiveis.push(copiaRom.buffer);

  worker.postMessage(
    {
      tipo,
      sessao,
      bytes: copiaRom,
      taxaBase: controladorAudio.obterTaxa(),
      pontosCram: Boolean(seletorCram && seletorCram.value === "hardware"),
      audioAtivo: controladorAudio.estaAtivo(),
      diagVisivel: hud.estaDiagAberto(),
      portaAudio,
      ...extras,
    },
    transferiveis,
  );
}

async function iniciarSessao(bytes) {
  encerrarSessaoAtual();
  await enviarSessaoWorker("iniciar", bytes);
}

async function iniciarSessaoComSave(bytes, entradasRle) {
  const manterPausado = pausadoUsuario;
  encerrarSessaoAtual();
  pausadoUsuario = manterPausado;
  await enviarSessaoWorker("carregar-save", bytes, { entradasRle, manterPausado });
}

document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    cancelAnimationFrame(animacao);
    animacao = 0;
    if (worker && jogando) worker.postMessage({ tipo: "pausar" });
    controladorAudio.suspenderSeAtivo();
  } else if (jogando && !apresentadorVideo.estaPerdido()) {
    if (!pausadoUsuario) controladorAudio.desbloquear();
    if (worker) worker.postMessage({ tipo: "retomar" });
    if (!animacao) animacao = requestAnimationFrame(lacoApresentacao);
  }
});
