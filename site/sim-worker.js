// Worker dedicado a validacao da ROM, simulacao Wasm a 60 Hz e envio direto de audio ao AudioWorklet.
import init, { identificar, Jogo } from "./pkg/sonic1_web.js";
import { Relogio, PERIODO_QUADRO } from "./relogio.js";

const TAXA_NTSC = 53693175 / 896040;
const POOL_AUDIO_TAMANHO = 8;
const POOL_AUDIO_FLOATS = 4096;
const POOL_VIDEO_TAMANHO = 2;
const PIXELS_TELA = 320 * 224;

let wasmInstance = null;
let jogo = null;
let sessaoAtiva = 0;
let timer = 0;
let pausado = false;
let diagVisivel = false;
let audioAtivo = false;

let entradaJogo = 0;
let entradaTitulo = 0;

let portaAudio = null;
let vistaAudio = null;
let vistaImagem = null;

const poolAudio = [];
for (let i = 0; i < POOL_AUDIO_TAMANHO; i++) {
  poolAudio.push(new Float32Array(POOL_AUDIO_FLOATS));
}

const poolVideo = [];
for (let i = 0; i < POOL_VIDEO_TAMANHO; i++) {
  poolVideo.push(new Uint16Array(PIXELS_TELA));
}

const relogio = new Relogio();
let quadroNum = 0;
let picoHistorico = 0;
let quadrosComSom = 0;

function ehModoTituloOuDemo(modo) {
  return modo === 0x00 || modo === 0x04 || modo === 0x08;
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
}

function enviarQuadroVideo(tempoWasm, diag) {
  const pts = obterVistaImagem();
  let buf = poolVideo.pop();
  if (!buf || buf.length !== pts.length) {
    buf = new Uint16Array(pts.length);
  }
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

function tick() {
  if (!jogo || pausado) return;

  const passos = relogio.avancar(performance.now());
  if (passos === 0) return;

  let atualizou = false;
  let ultimoWasmTime = 0;
  let ultimoDiag = null;

  for (let passo = 0; passo < passos; passo++) {
    quadroNum++;

    const modo = typeof jogo.modo_jogo === "function" ? jogo.modo_jogo() : 0x0C;
    jogo.botoes(ehModoTituloOuDemo(modo) ? entradaTitulo : entradaJogo);

    const inicioWasm = performance.now();
    jogo.simular_quadro();
    ultimoWasmTime = performance.now() - inicioWasm;
    atualizou = true;

    const ptrAudio = jogo.audio_ptr();
    const lenAudio = jogo.audio_len();
    let picoQuadro = 0;
    const calcularDiag = diagVisivel && (quadroNum % 6 === 0);

    if (lenAudio > 0) {
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

      if (audioAtivo && portaAudio) {
        let buf = poolAudio.pop();
        if (!buf || buf.length < lenAudio) {
          buf = new Float32Array(Math.max(POOL_AUDIO_FLOATS, lenAudio));
        }
        buf.set(fatiaAudio);
        const sub = buf.subarray(0, lenAudio);
        portaAudio.postMessage(sub, [buf.buffer]);
      }
    }

    if (calcularDiag) {
      ultimoDiag = {
        quadroNum,
        lenAudio,
        picoQuadro,
        picoHistorico,
        quadrosComSom,
      };
    }

    if (jogo.tem_erro()) {
      const mensagem = jogo.erro();
      pararTimer();
      enviarQuadroVideo(ultimoWasmTime, ultimoDiag);
      self.postMessage({ tipo: "erro-nucleo", sessao: sessaoAtiva, mensagem });
      return;
    }

    if (ultimoWasmTime > 16 && passo < passos - 1) {
      // Amortecimento: quadro pesado (ex: carga Kosinski/Enigma); interrompe a rajada
      // para entregar audio e imagem sem engasgos.
      break;
    }
  }

  if (atualizou) {
    enviarQuadroVideo(ultimoWasmTime, ultimoDiag);
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
      encerrarJogo();
      sessaoAtiva = msg.sessao;
      pausado = false;
      audioAtivo = Boolean(msg.audioAtivo);
      diagVisivel = Boolean(msg.diagVisivel);
      quadroNum = 0;
      picoHistorico = 0;
      quadrosComSom = 0;

      if (msg.portaAudio) {
        portaAudio = msg.portaAudio;
        portaAudio.onmessage = (ev) => {
          const dados = ev.data;
          if (dados instanceof ArrayBuffer) {
            if (poolAudio.length < POOL_AUDIO_TAMANHO) {
              poolAudio.push(new Float32Array(dados));
            }
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

      try {
        const bytes = msg.bytes instanceof Uint8Array ? msg.bytes : new Uint8Array(msg.bytes);
        const taxaBase = msg.taxaBase || 48000;
        // O console roda a 53693175 / 896040 = 59,92 Hz; o site roda a 60 Hz exatos.
        const taxaAjustada = (taxaBase * TAXA_NTSC) / 60;
        jogo = Jogo.criar_com_audio(bytes, taxaAjustada);
        if (typeof jogo.definir_pontos_cram === "function") {
          jogo.definir_pontos_cram(Boolean(msg.pontosCram));
        }
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

    case "entrada": {
      entradaJogo = msg.mascaraJogo | 0;
      entradaTitulo = msg.mascaraTitulo | 0;
      break;
    }

    case "pausar": {
      pausado = true;
      break;
    }

    case "retomar": {
      if (jogo && pausado) {
        pausado = false;
        relogio.retomar(performance.now());
      }
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
      if (msg.buffer instanceof ArrayBuffer && poolVideo.length < POOL_VIDEO_TAMANHO) {
        poolVideo.push(new Uint16Array(msg.buffer));
      }
      break;
    }

    case "parar": {
      encerrarJogo();
      break;
    }
  }
};
