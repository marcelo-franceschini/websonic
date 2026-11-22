// Worker dedicado a validacao da ROM, simulacao Wasm a 60 Hz, WebGL2 em OffscreenCanvas,
// envio direto de audio ao AudioWorklet e save states deterministicos por replay RLE.
import init, { identificar, Jogo } from "./pkg/sonic1_web.js";
import { Relogio, PERIODO_QUADRO } from "./relogio.js";
import { ehModoTituloOuDemo } from "./entrada.js";
import { criarRenderizadorGPU } from "./gpu.js";
import { codificarEntradasRle, decodificarEntradasRle } from "./saves.js";
import { HistoricoEntradas, PoolBuffers } from "./buffers.js";

const TAXA_NTSC = 53693175 / 896040;
const POOL_AUDIO_TAMANHO = 8;
const POOL_AUDIO_FLOATS = 4096;
const POOL_VIDEO_TAMANHO = 2;
const PIXELS_TELA = 320 * 224;
const HISTORICO_CAPACIDADE_INICIAL = 36000;

let wasmInstance = null;
let jogo = null;
let sessaoAtiva = 0;
let timer = 0;
let pausadoSistema = false;
let pausadoUsuario = false;
let multiplicadorVelocidade = 1;
let diagVisivel = false;
let audioAtivo = false;

let entradaJogo = 0;
let entradaTitulo = 0;

let portaAudio = null;
let vistaAudio = null;
let vistaImagem = null;

// Renderizacao zero-copy via OffscreenCanvas (quando suportada pelo navegador)
let canvasWorker = null;
let compositorWorker = null;
let contextoWorkerPerdido = false;

// Historico deterministico de entradas (1 byte por quadro) e pools reciclaveis
const historico = new HistoricoEntradas(HISTORICO_CAPACIDADE_INICIAL);
const poolAudio = new PoolBuffers(Float32Array, POOL_AUDIO_TAMANHO, POOL_AUDIO_FLOATS);
const poolVideo = new PoolBuffers(Uint16Array, POOL_VIDEO_TAMANHO, PIXELS_TELA);

const relogio = new Relogio();
const resultadoQuadro = { tempoWasm: 0, diag: null };
let quadroNum = 0;
let picoHistorico = 0;
let quadrosComSom = 0;

function destruirCompositorWorker() {
  if (compositorWorker && typeof compositorWorker.destruir === "function") {
    try {
      compositorWorker.destruir();
    } catch {
      // Contexto ja pode estar perdido.
    }
  }
  compositorWorker = null;
}

function obterVistaImagem() {
  const mem = wasmInstance.memory.buffer;
  const ptr = jogo.imagem_ptr();
  const len = jogo.imagem_len();
  if (
    !vistaImagem ||
    vistaImagem.buffer !== mem ||
    vistaImagem.byteOffset !== ptr ||
    vistaImagem.length !== len
  ) {
    vistaImagem = new Uint16Array(mem, ptr, len);
  }
  return vistaImagem;
}

function pararTimer() {
  if (timer) {
    clearInterval(timer);
    timer = 0;
  }
}

function fecharPortaAudio() {
  if (portaAudio) {
    portaAudio.onmessage = null;
    if (typeof portaAudio.close === "function") portaAudio.close();
    portaAudio = null;
  }
}

function encerrarJogo() {
  pararTimer();
  fecharPortaAudio();
  if (jogo) {
    jogo.free();
    jogo = null;
  }
  vistaImagem = null;
  vistaAudio = null;
  historico.limpar();
}

function enviarQuadroVideo(tempoWasm, diag) {
  const pts = obterVistaImagem();

  // Caminho zero-copy: desenha direto da memoria Wasm no OffscreenCanvas WebGL2 do Worker.
  if (compositorWorker && !contextoWorkerPerdido) {
    try {
      compositorWorker.renderizar(pts);
      self.postMessage({
        tipo: "quadro-gpu",
        sessao: sessaoAtiva,
        quadroNum,
        tempoWasm,
        diag,
      });
      return;
    } catch {
      destruirCompositorWorker();
    }
  }

  // Fallback ping-pong: transfere Uint16Array reciclavel para a thread principal.
  const buf = poolVideo.adquirir(pts.length, true);
  buf.set(pts);
  self.postMessage(
    {
      tipo: "quadro",
      sessao: sessaoAtiva,
      buffer: buf.buffer,
      quadroNum,
      tempoWasm,
      diag,
    },
    [buf.buffer],
  );
}

function executarUmQuadro(emitirAudio) {
  quadroNum++;

  const modo = typeof jogo.modo_jogo === "function" ? jogo.modo_jogo() : 0x0C;
  const mascara = ehModoTituloOuDemo(modo) ? entradaTitulo : entradaJogo;
  historico.registrar(mascara);
  jogo.botoes(mascara);

  const inicioWasm = performance.now();
  jogo.simular_quadro();
  const tempoWasm = performance.now() - inicioWasm;

  const calcularDiag = diagVisivel && (quadroNum % 6 === 0);
  const enviarAudio = Boolean(emitirAudio && audioAtivo && portaAudio);
  const precisaAudio = enviarAudio || calcularDiag || quadrosComSom < 1000;
  let lenAudio = 0;
  let picoQuadro = 0;

  if (precisaAudio) {
    lenAudio = jogo.audio_len();
    if (lenAudio > 0) {
      const ptrAudio = jogo.audio_ptr();
      const mem = wasmInstance.memory.buffer;
      if (!vistaAudio || vistaAudio.buffer !== mem) {
        vistaAudio = new Float32Array(mem);
      }
      const inicio = ptrAudio >> 2;
      const fatiaAudio = vistaAudio.subarray(inicio, inicio + lenAudio);

      if (calcularDiag) {
        for (let i = 0; i < lenAudio; i++) {
          const abs = Math.abs(fatiaAudio[i]);
          if (abs > picoQuadro) picoQuadro = abs;
        }
        if (picoQuadro > picoHistorico) picoHistorico = picoQuadro;
      }

      if (quadrosComSom < 1000 && (fatiaAudio[0] !== 0 || fatiaAudio[lenAudio - 1] !== 0)) {
        quadrosComSom++;
      }

      if (enviarAudio) {
        const buf = poolAudio.adquirir(lenAudio, false);
        buf.set(fatiaAudio);
        const sub = buf.subarray(0, lenAudio);
        portaAudio.postMessage(sub, [buf.buffer]);
      }
    }
  }

  let diag = null;
  if (calcularDiag) {
    diag = {
      quadroNum,
      lenAudio,
      picoQuadro,
      picoHistorico,
      quadrosComSom,
    };
  }

  resultadoQuadro.tempoWasm = tempoWasm;
  resultadoQuadro.diag = diag;
  return resultadoQuadro;
}

function tick() {
  if (!jogo || pausadoSistema || pausadoUsuario || contextoWorkerPerdido) return;

  const passosBase = relogio.avancar(performance.now());
  if (passosBase === 0) return;

  const passos = Math.min(passosBase * multiplicadorVelocidade, 4);
  let atualizou = false;
  let ultimoWasmTime = 0;
  let ultimoDiag = null;

  for (let passo = 0; passo < passos; passo++) {
    // Em fast-forward (2x), envia audio apenas no ultimo passo do lote para manter a fila limpa.
    const emitirAudio = multiplicadorVelocidade === 1 || passo === passos - 1;
    const res = executarUmQuadro(emitirAudio);
    ultimoWasmTime = res.tempoWasm;
    if (res.diag) ultimoDiag = res.diag;
    atualizou = true;

    if (jogo.tem_erro()) {
      const mensagem = jogo.erro();
      pararTimer();
      enviarQuadroVideo(ultimoWasmTime, ultimoDiag);
      self.postMessage({ tipo: "erro-nucleo", sessao: sessaoAtiva, mensagem });
      return;
    }

    if (ultimoWasmTime > 16 && passo < passos - 1) {
      break;
    }
  }

  if (atualizou) {
    enviarQuadroVideo(ultimoWasmTime, ultimoDiag);
  }
}

function configurarPortaAudio(porta) {
  if (!porta) return;
  portaAudio = porta;
  portaAudio.onmessage = (ev) => {
    const dados = ev.data;
    if (dados instanceof ArrayBuffer) {
      poolAudio.devolverBuffer(dados);
      return;
    }
    if (dados && dados.tipo === "status") {
      if (jogo) jogo.ocupacao_audio(dados.disponiveis);
      if (diagVisivel) {
        self.postMessage({ tipo: "worklet-status", sessao: sessaoAtiva, status: dados });
      }
    }
  };
}

function prepararNovaSessao(msg, manterPausado) {
  encerrarJogo();
  sessaoAtiva = msg.sessao;
  pausadoSistema = false;
  pausadoUsuario = Boolean(manterPausado);
  multiplicadorVelocidade = 1;
  audioAtivo = Boolean(msg.audioAtivo);
  diagVisivel = Boolean(msg.diagVisivel);
  quadroNum = 0;
  picoHistorico = 0;
  quadrosComSom = 0;
  configurarPortaAudio(msg.portaAudio);
}

function inicializarInstanciaJogo(msg) {
  const bytes = msg.bytes instanceof Uint8Array ? msg.bytes : new Uint8Array(msg.bytes);
  const taxaBase = msg.taxaBase || 48000;
  // O console roda a 53693175 / 896040 = 59,92 Hz; o site roda a 60 Hz exatos.
  const taxaAjustada = (taxaBase * TAXA_NTSC) / 60;
  jogo = Jogo.criar_com_audio(bytes, taxaAjustada);
  if (typeof jogo.definir_pontos_cram === "function") {
    jogo.definir_pontos_cram(Boolean(msg.pontosCram));
  }
}

self.onmessage = async (e) => {
  const msg = e.data;
  if (!msg || !msg.tipo) return;

  switch (msg.tipo) {
    case "init": {
      try {
        wasmInstance = await init();
        self.postMessage({ tipo: "pronto" });
      } catch (err) {
        self.postMessage({
          tipo: "erro-wasm",
          mensagem: err && err.message ? err.message : String(err),
        });
      }
      break;
    }

    case "anexar-canvas": {
      destruirCompositorWorker();
      canvasWorker = msg.canvas || null;
      contextoWorkerPerdido = false;
      if (canvasWorker) {
        try {
          compositorWorker = criarRenderizadorGPU(canvasWorker);
          canvasWorker.addEventListener("webglcontextlost", (ev) => {
            ev.preventDefault();
            contextoWorkerPerdido = true;
            compositorWorker = null;
            self.postMessage({ tipo: "gpu-contexto-perdido" });
          });
          canvasWorker.addEventListener("webglcontextrestored", () => {
            try {
              destruirCompositorWorker();
              compositorWorker = criarRenderizadorGPU(canvasWorker);
              contextoWorkerPerdido = !compositorWorker;
              self.postMessage({ tipo: "gpu-contexto-restaurado", ok: Boolean(compositorWorker) });
            } catch {
              destruirCompositorWorker();
              contextoWorkerPerdido = false;
              self.postMessage({ tipo: "gpu-contexto-restaurado", ok: false });
            }
          });
        } catch {
          destruirCompositorWorker();
        }
      }
      self.postMessage({
        tipo: "canvas-anexado",
        ok: Boolean(compositorWorker),
        renderer: compositorWorker ? compositorWorker.renderer : null,
      });
      break;
    }

    case "identificar": {
      if (!wasmInstance) {
        self.postMessage({
          tipo: "identificado",
          id: msg.id,
          valida: false,
          motivo: "WebAssembly ainda não foi inicializado",
          sha1: "(indisponivel)",
          tamanho: msg.bytes ? msg.bytes.byteLength : 0,
        });
        return;
      }
      try {
        const bytes = msg.bytes instanceof Uint8Array ? msg.bytes : new Uint8Array(msg.bytes);
        const r = identificar(bytes);
        const resposta = {
          tipo: "identificado",
          id: msg.id,
          valida: r.valida,
          motivo: r.motivo,
          sha1: r.sha1,
          tamanho: r.tamanho,
        };
        r.free();
        self.postMessage(resposta);
      } catch (err) {
        self.postMessage({
          tipo: "identificado",
          id: msg.id,
          valida: false,
          motivo: err && err.message ? err.message : String(err),
          sha1: "(erro)",
          tamanho: msg.bytes ? msg.bytes.byteLength : 0,
        });
      }
      break;
    }

    case "iniciar": {
      prepararNovaSessao(msg, false);

      try {
        inicializarInstanciaJogo(msg);
        relogio.iniciar(performance.now());
        timer = setInterval(tick, PERIODO_QUADRO / 2);
        self.postMessage({ tipo: "iniciado", sessao: sessaoAtiva });
      } catch (err) {
        encerrarJogo();
        self.postMessage({
          tipo: "erro-iniciar",
          sessao: sessaoAtiva,
          mensagem: err && err.message ? err.message : String(err),
        });
      }
      break;
    }

    case "exportar-save": {
      if (!jogo) {
        self.postMessage({
          tipo: "save-pronto",
          id: msg.id,
          erro: "Nenhuma sessão ativa para salvar.",
        });
        return;
      }
      const entradasRle = codificarEntradasRle(historico.fatiaAtiva());
      self.postMessage({
        tipo: "save-pronto",
        id: msg.id,
        quadro: historico.tamanho,
        entradasRle,
      });
      break;
    }

    case "carregar-save": {
      prepararNovaSessao(msg, msg.manterPausado);

      try {
        const entradas = decodificarEntradasRle(msg.entradasRle);
        historico.garantirCapacidade(entradas.length);
        inicializarInstanciaJogo(msg);

        // Reproduz deterministicamente os quadros salvos em modo silencioso (maxima velocidade).
        for (let i = 0; i < entradas.length; i++) {
          const m = entradas[i];
          historico.registrar(m);
          jogo.botoes(m);
          jogo.simular_quadro();
          quadroNum++;
          if (jogo.tem_erro()) {
            throw new Error(jogo.erro());
          }
        }

        if (portaAudio) {
          portaAudio.postMessage({ tipo: "reset" });
        }

        relogio.iniciar(performance.now());
        timer = setInterval(tick, PERIODO_QUADRO / 2);
        enviarQuadroVideo(0, null);
        self.postMessage({
          tipo: "save-carregado",
          sessao: sessaoAtiva,
          quadro: quadroNum,
        });
      } catch (err) {
        encerrarJogo();
        self.postMessage({
          tipo: "erro-save",
          sessao: sessaoAtiva,
          mensagem: err && err.message ? err.message : String(err),
        });
      }
      break;
    }

    case "entrada": {
      entradaJogo = msg.mascaraJogo | 0;
      entradaTitulo = msg.mascaraTitulo | 0;
      break;
    }

    case "pausar": {
      pausadoSistema = true;
      break;
    }

    case "retomar": {
      if (jogo && pausadoSistema) {
        pausadoSistema = false;
        if (!pausadoUsuario) {
          relogio.retomar(performance.now());
        }
      }
      break;
    }

    case "pausar-usuario": {
      pausadoUsuario = Boolean(msg.pausado);
      if (jogo && !pausadoUsuario && !pausadoSistema) {
        relogio.retomar(performance.now());
      }
      break;
    }

    case "passo-unico": {
      if (!jogo || pausadoSistema || contextoWorkerPerdido) return;
      pausadoUsuario = true;
      const res = executarUmQuadro(false);
      enviarQuadroVideo(res.tempoWasm, res.diag);
      if (jogo.tem_erro()) {
        const mensagem = jogo.erro();
        pararTimer();
        self.postMessage({ tipo: "erro-nucleo", sessao: sessaoAtiva, mensagem });
      }
      break;
    }

    case "velocidade": {
      const mult = Number(msg.multiplicador) >= 2 ? 2 : 1;
      multiplicadorVelocidade = mult;
      break;
    }

    case "audio-ativo": {
      audioAtivo = Boolean(msg.ativo);
      if (msg.resetar && portaAudio) {
        portaAudio.postMessage({ tipo: "reset" });
      }
      break;
    }

    case "pontos-cram": {
      if (jogo && typeof jogo.definir_pontos_cram === "function") {
        jogo.definir_pontos_cram(Boolean(msg.valor));
      }
      break;
    }

    case "diag-visivel": {
      diagVisivel = Boolean(msg.visivel);
      break;
    }

    case "devolver-video": {
      poolVideo.devolverBuffer(msg.buffer);
      break;
    }

    case "parar": {
      encerrarJogo();
      break;
    }
  }
};
