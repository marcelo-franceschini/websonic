import test from "node:test";
import assert from "node:assert/strict";
import {
  CHAVE_STORAGE_AUDIO,
  normalizarVolume,
  carregarPreferenciasAudio,
  salvarPreferenciasAudio,
  criarControladorAudio,
} from "../audio.js";

function criarStorageMemoria() {
  const mapa = new Map();
  return {
    getItem(k) {
      return mapa.has(k) ? mapa.get(k) : null;
    },
    setItem(k, v) {
      mapa.set(k, String(v));
    },
  };
}

test("normalizarVolume limita valores ao intervalo [0, 1] e trata entradas invalidas", () => {
  assert.equal(normalizarVolume(0.5), 0.5);
  assert.equal(normalizarVolume("0.35"), 0.35);
  assert.equal(normalizarVolume(-0.2), 0.0);
  assert.equal(normalizarVolume(2.5), 1.0);
  assert.equal(normalizarVolume(Number.NaN), 1.0);
  assert.equal(normalizarVolume(undefined), 1.0);
  assert.equal(normalizarVolume("abc"), 1.0);
});

test("carregarPreferenciasAudio retorna padrao para storage ausente, vazio ou corrompido", () => {
  const padrao = { volume: 1.0, volumeMusica: 1.0, volumeFx: 1.0, mudo: false };
  assert.deepEqual(carregarPreferenciasAudio(null), padrao);

  const vazio = criarStorageMemoria();
  assert.deepEqual(carregarPreferenciasAudio(vazio), padrao);

  vazio.setItem(CHAVE_STORAGE_AUDIO, "null");
  assert.deepEqual(carregarPreferenciasAudio(vazio), padrao);

  vazio.setItem(CHAVE_STORAGE_AUDIO, "{json-invalido");
  assert.deepEqual(carregarPreferenciasAudio(vazio), padrao);

  // Retrocompatibilidade com formato antigo sem volumeMusica/volumeFx
  vazio.setItem(CHAVE_STORAGE_AUDIO, JSON.stringify({ volume: 0.7, mudo: true }));
  assert.deepEqual(carregarPreferenciasAudio(vazio), {
    volume: 0.7,
    volumeMusica: 1.0,
    volumeFx: 1.0,
    mudo: true,
  });

  const comErro = {
    getItem() {
      throw new Error("quota");
    },
  };
  assert.deepEqual(carregarPreferenciasAudio(comErro), padrao);
});

test("salvarPreferenciasAudio e carregarPreferenciasAudio preservam volume geral, musica, FX e mudo normalizados", () => {
  assert.equal(salvarPreferenciasAudio(null, { volume: 0.5, mudo: false }), false);
  const storage = criarStorageMemoria();
  const salvo = salvarPreferenciasAudio(storage, {
    volume: "0.45",
    volumeMusica: "0.3",
    volumeFx: "0.85",
    mudo: true,
  });
  assert.equal(salvo, true);
  assert.deepEqual(carregarPreferenciasAudio(storage), {
    volume: 0.45,
    volumeMusica: 0.3,
    volumeFx: 0.85,
    mudo: true,
  });

  salvarPreferenciasAudio(storage, { volume: 5, volumeMusica: -1, volumeFx: 2, mudo: 0 });
  assert.deepEqual(carregarPreferenciasAudio(storage), {
    volume: 1.0,
    volumeMusica: 0.0,
    volumeFx: 1.0,
    mudo: false,
  });

  const semEscrita = {
    setItem() {
      throw new Error("SecurityError");
    },
  };
  assert.equal(salvarPreferenciasAudio(semEscrita, { volume: 0.5, mudo: false }), false);
});

function criarControleStub() {
  const ouvintes = new Map();
  const classes = new Set();
  const attrs = new Map();
  return {
    value: "1",
    classList: {
      toggle(c, f) {
        if (f) classes.add(c);
        else classes.delete(c);
      },
      contains: (c) => classes.has(c),
    },
    setAttribute(k, v) {
      attrs.set(k, String(v));
    },
    getAttribute: (k) => attrs.get(k),
    addEventListener(tipo, fn) {
      ouvintes.set(tipo, fn);
    },
    disparar(tipo, evt = {}) {
      const fn = ouvintes.get(tipo);
      if (fn) fn(evt);
    },
  };
}

test("criarControladorAudio gerencia AudioContext, AudioWorklet, ganho, mudo, porta e bipe", async () => {
  const origWindow = globalThis.window;
  const origWorklet = globalThis.AudioWorkletNode;
  const origChannel = globalThis.MessageChannel;
  const origWarn = console.warn;

  const windowOuvintes = new Map();
  const msgsWorker = [];
  const msgsPortWorklet = [];
  const pipelines = [];
  const errosWorklet = [];
  const estadosHud = [];
  const errosUi = [];
  let focosTela = 0;
  let pausadoUsuario = false;
  let podeRetomar = true;
  let falharResume = false;
  let falharAddModule = false;
  let falharWorklet3Args = true;
  let usarTargetAtTime = true;
  let oscCriado = null;
  let ultimoWorkletNode = null;

  class FakeAudioContext {
    constructor() {
      this.state = "suspended";
      this.sampleRate = 48000;
      this.currentTime = 1.0;
      this.destination = { dest: true };
      this._listeners = new Map();
      this.audioWorklet = {
        addModule: async () => {
          if (falharAddModule) throw new Error("sem worklet");
        },
      };
    }
    addEventListener(tipo, fn) {
      this._listeners.set(tipo, fn);
    }
    dispararStateChange() {
      const fn = this._listeners.get("statechange");
      if (fn) fn();
    }
    async resume() {
      if (falharResume) throw new Error("bloqueado");
      this.state = "running";
      this.dispararStateChange();
    }
    async suspend() {
      this.state = "suspended";
      this.dispararStateChange();
    }
    createGain() {
      const gain = {
        valores: [],
        setValueAtTime(v) {
          this.valores.push(["set", v]);
        },
        exponentialRampToValueAtTime(v) {
          this.valores.push(["exp", v]);
        },
      };
      if (usarTargetAtTime) {
        gain.cancelScheduledValues = () => gain.valores.push(["cancel"]);
        gain.setTargetAtTime = (v) => gain.valores.push(["target", v]);
      }
      return {
        gain,
        connect() {},
        disconnect() {},
      };
    }
    createOscillator() {
      oscCriado = {
        type: "",
        iniciado: false,
        parado: false,
        desconectado: false,
        frequency: { setValueAtTime() {} },
        connect() {},
        disconnect() {
          this.desconectado = true;
        },
        start() {
          this.iniciado = true;
        },
        stop() {
          this.parado = true;
        },
        onended: null,
      };
      return oscCriado;
    }
  }

  class FakeWorkletNode {
    constructor(_ctx, _name, opts) {
      if (opts && falharWorklet3Args) {
        throw new Error("3-arg unsupported");
      }
      this.port = {
        postMessage(msg) {
          msgsPortWorklet.push(msg);
        },
      };
      this.onprocessorerror = null;
      ultimoWorkletNode = this;
    }
    connect() {}
  }

  class FakeMessageChannel {
    constructor() {
      this.port1 = { id: "p1" };
      this.port2 = { id: "p2" };
    }
  }

  globalThis.window = {
    AudioContext: FakeAudioContext,
    addEventListener(tipo, fn) {
      windowOuvintes.set(tipo, fn);
    },
  };
  globalThis.AudioWorkletNode = FakeWorkletNode;
  globalThis.MessageChannel = FakeMessageChannel;
  console.warn = () => {};

  try {
    const storage = criarStorageMemoria();
    salvarPreferenciasAudio(storage, { volume: 0.6, volumeMusica: 0.75, volumeFx: 0.5, mudo: true });

    const btnMudo = criarControleStub();
    const sliderVolume = criarControleStub();
    const sliderMusica = criarControleStub();
    const sliderFx = criarControleStub();
    const elValVolume = { textContent: "" };
    const elValMusica = { textContent: "" };
    const elValFx = { textContent: "" };
    const btnAudio = criarControleStub();
    const btnBipe = criarControleStub();
    const hud = {
      definirPipeline: (t, c) => pipelines.push([t, c]),
      definirErroWorklet: (t) => errosWorklet.push(t),
      atualizarEstadoAudio: (ctx) => estadosHud.push(ctx?.state),
    };

    const ctrl = criarControladorAudio({
      btnMudo,
      sliderVolume,
      sliderMusica,
      sliderFx,
      elValVolume,
      elValMusica,
      elValFx,
      btnAudio,
      btnBipe,
      hud,
      obterTela: () => ({ focus: () => { focosTela++; } }),
      obterWorker: () => ({ postMessage: (m) => msgsWorker.push(m) }),
      estaPausadoUsuario: () => pausadoUsuario,
      podeRetomarAudio: () => podeRetomar,
      onErro: (e) => errosUi.push(e),
      storage,
    });

    // Valores iniciais sincronizados com UI e taxa padrao 48000 antes do init.
    assert.equal(sliderVolume.value, "0.6");
    assert.equal(sliderMusica.value, "0.75");
    assert.equal(sliderFx.value, "0.5");
    assert.equal(elValVolume.textContent, "0%");
    assert.equal(elValMusica.textContent, "75%");
    assert.equal(elValFx.textContent, "50%");
    assert.deepEqual(ctrl.obterVolumes(), { musica: 0.75, fx: 0.5 });
    assert.equal(btnMudo.classList.contains("mudo"), true);
    assert.equal(btnMudo.getAttribute("aria-label"), "Ativar som");
    assert.equal(ctrl.obterTaxa(), 48000);
    assert.equal(ctrl.estaAtivo(), false);
    assert.equal(ctrl.criarPortaParaWorker(), null);

    // Iniciar cria contexto, worklet (com fallback de construtor 2 args) e desbloqueia.
    await ctrl.iniciar();
    await Promise.resolve();
    assert.equal(ctrl.estaAtivo(), true);
    assert.equal(ctrl.obterContexto().sampleRate, 48000);
    assert.equal(pipelines[0][0], "AudioWorklet (Worker direto)");
    assert.ok(msgsWorker.some((m) => m.tipo === "audio-ativo" && m.ativo === true));
    assert.ok(windowOuvintes.has("click"));

    // Segunda chamada a iniciar() e idempotente.
    await ctrl.iniciar();
    assert.equal(pipelines.length, 1);

    // Erro no processador AudioWorklet reporta no HUD.
    ultimoWorkletNode.onprocessorerror(new Error("proc-err"));
    assert.ok(errosWorklet.includes("Erro no AudioWorkletProcessor!"));

    // criarPortaParaWorker cria MessageChannel e envia port1 ao worklet.
    const porta = ctrl.criarPortaParaWorker();
    assert.deepEqual(porta, { id: "p2" });
    assert.equal(msgsPortWorklet.length, 1);

    // Slider acima de 0 desativa o mudo e persiste.
    sliderVolume.disparar("input", { target: { value: "0.8" } });
    assert.equal(elValVolume.textContent, "80%");
    assert.equal(btnMudo.classList.contains("mudo"), false);
    assert.equal(btnMudo.getAttribute("aria-label"), "Silenciar som");
    assert.deepEqual(carregarPreferenciasAudio(storage), {
      volume: 0.8,
      volumeMusica: 0.75,
      volumeFx: 0.5,
      mudo: false,
    });

    // Sliders de Musica e FX atualizam UI, persistem e enviam mensagem ao Worker.
    sliderMusica.disparar("input", { target: { value: "0.4" } });
    sliderFx.disparar("input", { target: { value: "0.9" } });
    assert.equal(elValMusica.textContent, "40%");
    assert.equal(elValFx.textContent, "90%");
    assert.deepEqual(ctrl.obterVolumes(), { musica: 0.4, fx: 0.9 });
    assert.ok(msgsWorker.some((m) => m.tipo === "volumes" && m.musica === 0.4 && m.fx === 0.9));

    // Clique em btnMudo alterna mudo e foca tela.
    btnMudo.disparar("click");
    assert.equal(btnMudo.classList.contains("mudo"), true);
    assert.equal(focosTela, 1);

    // suspenderSeAtivo suspende e retoma quando podeRetomarAudio() === true.
    ctrl.suspenderSeAtivo();
    await new Promise((r) => setTimeout(r, 0));
    assert.equal(ctrl.estaAtivo(), true);

    // Quando pausado pelo usuario, desbloquear nao retoma.
    podeRetomar = false;
    pausadoUsuario = true;
    ctrl.suspenderSeAtivo();
    await new Promise((r) => setTimeout(r, 0));
    assert.equal(ctrl.estaAtivo(), false);
    btnAudio.disparar("click");
    assert.equal(ctrl.estaAtivo(), false);
    assert.equal(focosTela, 2);

    // Bipe de teste toca oscilador e desconecta ao finalizar.
    btnBipe.disparar("click");
    await new Promise((r) => setTimeout(r, 0));
    assert.ok(oscCriado && oscCriado.iniciado && oscCriado.parado);
    oscCriado.onended();
    assert.equal(oscCriado.desconectado, true);

    // Erro no resume do bipe vai para onErro.
    falharResume = true;
    btnBipe.disparar("click");
    await new Promise((r) => setTimeout(r, 0));
    assert.ok(errosUi.some((e) => /retomar áudio/.test(e)));

    // Fallback de AudioWorklet indisponivel e sem setTargetAtTime.
    falharAddModule = true;
    usarTargetAtTime = false;
    falharResume = false;
    const ctrlSemWorklet = criarControladorAudio({
      btnMudo,
      sliderVolume,
      hud,
      storage,
    });
    await ctrlSemWorklet.iniciar();
    assert.ok(errosWorklet.includes("AudioWorklet indisponivel"));
    sliderVolume.disparar("input", { target: { value: "0.4" } });

    // Falha fatal ao criar AudioContext (ex: construtor lanca).
    globalThis.window.AudioContext = function Falha() {
      throw new Error("audio bloqueado");
    };
    const btnBipeErro = criarControleStub();
    const ctrlErro = criarControladorAudio({
      btnBipe: btnBipeErro,
      hud,
      onErro: (e) => errosUi.push(e),
      storage,
    });
    await ctrlErro.iniciar();
    assert.ok(pipelines.some((p) => /Falha ao iniciar audio/.test(p[0])));
    btnBipeErro.disparar("click");
    assert.ok(errosUi.some((e) => /tocar bipe de teste/.test(e)));
  } finally {
    globalThis.window = origWindow;
    globalThis.AudioWorkletNode = origWorklet;
    globalThis.MessageChannel = origChannel;
    console.warn = origWarn;
  }
});

