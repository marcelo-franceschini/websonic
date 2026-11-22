// Controlador de audio do navegador: AudioContext, GainNode, AudioWorklet, volume, mudo e bipe de teste.

export const CHAVE_STORAGE_AUDIO = "websonic:audio:v1";

/**
 * Normaliza um valor numerico ou string de volume para o intervalo [0, 1].
 */
export function normalizarVolume(valor) {
  const n = typeof valor === "number" ? valor : Number.parseFloat(valor);
  if (!Number.isFinite(n)) return 1.0;
  if (n <= 0) return 0.0;
  if (n >= 1) return 1.0;
  return Math.round(n * 1000) / 1000;
}

/**
 * Carrega preferencias persistidas de volume (geral, musica e FX) e mudo de forma segura.
 */
export function carregarPreferenciasAudio(storage) {
  const padrao = { volume: 1.0, volumeMusica: 1.0, volumeFx: 1.0, mudo: false };
  if (!storage || typeof storage.getItem !== "function") return padrao;
  try {
    const bruto = storage.getItem(CHAVE_STORAGE_AUDIO);
    if (!bruto) return padrao;
    const dados = JSON.parse(bruto);
    if (!dados || typeof dados !== "object") return padrao;
    return {
      volume: normalizarVolume(dados.volume),
      volumeMusica: normalizarVolume(dados.volumeMusica),
      volumeFx: normalizarVolume(dados.volumeFx),
      mudo: Boolean(dados.mudo),
    };
  } catch {
    return padrao;
  }
}

/**
 * Persiste preferencias de volume (geral, musica e FX) e mudo no armazenamento fornecido.
 */
export function salvarPreferenciasAudio(storage, prefs) {
  if (!storage || typeof storage.setItem !== "function") return false;
  try {
    const payload = {
      volume: normalizarVolume(prefs && prefs.volume),
      volumeMusica: normalizarVolume(prefs && prefs.volumeMusica),
      volumeFx: normalizarVolume(prefs && prefs.volumeFx),
      mudo: Boolean(prefs && prefs.mudo),
    };
    storage.setItem(CHAVE_STORAGE_AUDIO, JSON.stringify(payload));
    return true;
  } catch {
    return false;
  }
}

export function criarControladorAudio({
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
  obterTela,
  obterWorker,
  estaPausadoUsuario,
  podeRetomarAudio,
  onErro,
  storage = typeof window !== "undefined" ? window.localStorage : null,
}) {
  const prefsIniciais = carregarPreferenciasAudio(storage);
  let audioCtx = null;
  let ganhoMasterNode = null;
  let workletNode = null;
  let volumeAtual = prefsIniciais.volume;
  let volumeMusicaAtual = prefsIniciais.volumeMusica;
  let volumeFxAtual = prefsIniciais.volumeFx;
  let mudoAtivo = prefsIniciais.mudo;
  let ouvintesDesbloqueioRegistrados = false;

  function sincronizarControlesUi() {
    if (sliderVolume) {
      sliderVolume.value = String(volumeAtual);
    }
    if (sliderMusica) {
      sliderMusica.value = String(volumeMusicaAtual);
    }
    if (sliderFx) {
      sliderFx.value = String(volumeFxAtual);
    }
    if (elValVolume) {
      elValVolume.textContent = mudoAtivo ? "0%" : `${Math.round(volumeAtual * 100)}%`;
    }
    if (elValMusica) {
      elValMusica.textContent = `${Math.round(volumeMusicaAtual * 100)}%`;
    }
    if (elValFx) {
      elValFx.textContent = `${Math.round(volumeFxAtual * 100)}%`;
    }
    if (btnMudo) {
      btnMudo.classList.toggle("mudo", mudoAtivo);
      btnMudo.setAttribute("aria-pressed", String(mudoAtivo));
      btnMudo.setAttribute("aria-label", mudoAtivo ? "Ativar som" : "Silenciar som");
    }
  }

  function persistirPreferencias() {
    salvarPreferenciasAudio(storage, {
      volume: volumeAtual,
      volumeMusica: volumeMusicaAtual,
      volumeFx: volumeFxAtual,
      mudo: mudoAtivo,
    });
  }

  function enviarVolumesWorker() {
    const worker = obterWorker ? obterWorker() : null;
    if (worker) {
      worker.postMessage({
        tipo: "volumes",
        musica: volumeMusicaAtual,
        fx: volumeFxAtual,
      });
    }
  }

  function atualizarGanho() {
    if (!ganhoMasterNode || !audioCtx) return;
    const valor = mudoAtivo ? 0.0 : volumeAtual;
    const agora = audioCtx.currentTime;
    const param = ganhoMasterNode.gain;
    if (typeof param.cancelScheduledValues === "function") {
      param.cancelScheduledValues(agora);
    }
    if (typeof param.setTargetAtTime === "function") {
      param.setTargetAtTime(valor, agora, 0.008);
    } else {
      param.setValueAtTime(valor, agora);
    }
  }

  sincronizarControlesUi();

  function criarContexto() {
    if (!audioCtx) {
      const AudioContextClass = window.AudioContext || window.webkitAudioContext;
      audioCtx = new AudioContextClass();
      audioCtx.addEventListener("statechange", () => {
        const ativo = audioCtx.state === "running";
        const worker = obterWorker ? obterWorker() : null;
        if (worker) {
          worker.postMessage({ tipo: "audio-ativo", ativo, resetar: ativo });
        }
        hud.atualizarEstadoAudio(audioCtx);
      });
    }
    return audioCtx;
  }

  function desbloquear() {
    if (estaPausadoUsuario && estaPausadoUsuario()) return;
    if (audioCtx && audioCtx.state === "suspended") {
      audioCtx.resume().then(() => hud.atualizarEstadoAudio(audioCtx)).catch(() => {});
    }
  }

  function suspenderSeAtivo() {
    if (audioCtx && audioCtx.state === "running") {
      audioCtx
        .suspend()
        .then(() => {
          if (podeRetomarAudio && podeRetomarAudio()) {
            desbloquear();
          }
        })
        .catch(() => {});
    }
  }

  async function iniciar() {
    if (audioCtx && workletNode) return;
    try {
      criarContexto();

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
        window.addEventListener("click", desbloquear);
        window.addEventListener("keydown", desbloquear);
        window.addEventListener("pointerdown", desbloquear);
        window.addEventListener("touchstart", desbloquear);
      }

      desbloquear();
      hud.atualizarEstadoAudio(audioCtx);
    } catch (e) {
      hud.definirPipeline(`Falha ao iniciar audio: ${e.message}`, "diag-erro");
    }
  }

  function criarPortaParaWorker() {
    if (!workletNode || typeof MessageChannel === "undefined") {
      return null;
    }
    const canal = new MessageChannel();
    workletNode.port.postMessage({ tipo: "porta", porta: canal.port1 }, [canal.port1]);
    return canal.port2;
  }

  function focarTela() {
    const tela = obterTela ? obterTela() : null;
    if (tela) tela.focus();
  }

  if (sliderVolume) {
    sliderVolume.addEventListener("input", (e) => {
      volumeAtual = normalizarVolume(e.target.value);
      if (mudoAtivo && volumeAtual > 0) {
        mudoAtivo = false;
      }
      sincronizarControlesUi();
      persistirPreferencias();
      atualizarGanho();
    });
  }

  if (sliderMusica) {
    sliderMusica.addEventListener("input", (e) => {
      volumeMusicaAtual = normalizarVolume(e.target.value);
      sincronizarControlesUi();
      persistirPreferencias();
      enviarVolumesWorker();
    });
  }

  if (sliderFx) {
    sliderFx.addEventListener("input", (e) => {
      volumeFxAtual = normalizarVolume(e.target.value);
      sincronizarControlesUi();
      persistirPreferencias();
      enviarVolumesWorker();
    });
  }

  if (btnMudo) {
    btnMudo.addEventListener("click", () => {
      mudoAtivo = !mudoAtivo;
      sincronizarControlesUi();
      persistirPreferencias();
      atualizarGanho();
      focarTela();
    });
  }

  if (btnAudio) {
    btnAudio.addEventListener("click", () => {
      desbloquear();
      focarTela();
    });
  }

  if (btnBipe) {
    btnBipe.addEventListener("click", () => {
      try {
        const ctx = criarContexto();
        ctx
          .resume()
          .then(() => {
            hud.atualizarEstadoAudio(audioCtx);
            const osc = ctx.createOscillator();
            const ganho = ctx.createGain();
            osc.type = "sine";
            osc.frequency.setValueAtTime(440, ctx.currentTime);
            ganho.gain.setValueAtTime(0.25, ctx.currentTime);
            ganho.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.35);
            osc.connect(ganho);
            ganho.connect(ctx.destination);
            osc.onended = () => {
              osc.disconnect();
              ganho.disconnect();
            };
            osc.start();
            osc.stop(ctx.currentTime + 0.35);
          })
          .catch((e) => {
            if (onErro) onErro("Erro ao retomar áudio para bipe de teste: " + e.message);
          });
      } catch (e) {
        if (onErro) onErro("Erro ao tocar bipe de teste: " + e.message);
      }
    });
  }

  return {
    iniciar,
    desbloquear,
    suspenderSeAtivo,
    criarPortaParaWorker,
    obterContexto: () => audioCtx,
    obterTaxa: () => (audioCtx ? audioCtx.sampleRate : 48000),
    obterVolumes: () => ({ musica: volumeMusicaAtual, fx: volumeFxAtual }),
    estaAtivo: () => Boolean(audioCtx && audioCtx.state === "running"),
  };
}
