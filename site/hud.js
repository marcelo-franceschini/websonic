// HUD de performance (FPS, grafico de frametime) e painel de diagnostico de audio.
import { PERIODO_QUADRO } from "./relogio.js";

const HISTORICO_TEMPOS_MAX = 80;

/**
 * Calcula media e percentil 95 de uma janela de tempos em ms sem alocar memoria no laco quente.
 */
export function calcularMediaEP95(amostras, quantidade, scratch) {
  const n = Math.min(quantidade, amostras.length);
  if (n <= 0) return { media: 0, p95: 0 };

  const destino = scratch && scratch.length >= n ? scratch : new Float32Array(n);
  let soma = 0;
  for (let i = 0; i < n; i++) {
    const v = amostras[i];
    soma += v;
    destino[i] = v;
  }
  const ordenados = destino.subarray(0, n).sort();
  const idxP95 = Math.min(n - 1, Math.floor(n * 0.95));
  return {
    media: soma / n,
    p95: ordenados[idxP95],
  };
}

/**
 * Classifica o tempo de quadro para a cor da barra do HUD (60 FPS / 30 FPS / lento).
 */
export function classificarFrametime(tempoMs) {
  if (tempoMs <= 16.67) return "#10b981";
  if (tempoMs <= 33.33) return "#ffbe0b";
  return "#ef4444";
}

export function criarHud(el) {
  const ctxHudGrafico = el.canvasHudGrafico ? el.canvasHudGrafico.getContext("2d") : null;
  let hudVisivel = false;

  const historicoFrameTime = new Float32Array(HISTORICO_TEMPOS_MAX);
  const historicoWasmTime = new Float32Array(HISTORICO_TEMPOS_MAX);
  const copiaWasmTime = new Float32Array(HISTORICO_TEMPOS_MAX);
  let indiceFrame = 0;
  let totalFrame = 0;
  let indiceWasm = 0;
  let totalWasm = 0;

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
    if (el.hudPerformance) el.hudPerformance.hidden = !hudVisivel;
    if (el.btnHud) {
      el.btnHud.classList.toggle("btn-azul", hudVisivel);
      el.btnHud.classList.toggle("btn-cinza", !hudVisivel);
      el.btnHud.setAttribute("aria-pressed", String(hudVisivel));
    }
  }

  if (el.btnHud) {
    el.btnHud.addEventListener("click", alternarHud);
  }

  if (el.btnToggleDiag && el.painelDiag) {
    el.btnToggleDiag.addEventListener("click", () => {
      const abrir = el.painelDiag.hidden;
      el.painelDiag.hidden = !abrir;
      el.btnToggleDiag.setAttribute("aria-expanded", String(abrir));
      if (el.diagSeta) el.diagSeta.textContent = abrir ? "▲" : "▼";
      if (el.onToggleDiag) el.onToggleDiag(abrir);
    });
  }

  function desenharGraficoHud() {
    if (!ctxHudGrafico || !hudVisivel) return;
    const w = el.canvasHudGrafico.width;
    const h = el.canvasHudGrafico.height;
    ctxHudGrafico.clearRect(0, 0, w, h);

    // Linhas de referencia: 16.67ms (60 FPS) e 33.33ms (30 FPS)
    const y16 = Math.round(h - (16.67 * h / 40));
    const y33 = Math.round(h - (33.33 * h / 40));

    ctxHudGrafico.fillStyle = "rgba(59, 125, 248, 0.45)";
    ctxHudGrafico.fillRect(0, y16, w, 1);

    ctxHudGrafico.fillStyle = "rgba(239, 68, 68, 0.35)";
    ctxHudGrafico.fillRect(0, y33, w, 1);

    const n = Math.min(totalFrame, HISTORICO_TEMPOS_MAX);
    const larguraBarra = w / HISTORICO_TEMPOS_MAX;

    for (let i = 0; i < n; i++) {
      const idx = (indiceFrame - n + i + HISTORICO_TEMPOS_MAX) % HISTORICO_TEMPOS_MAX;
      const tempo = historicoFrameTime[idx];
      const altura = Math.min(h, Math.max(1, Math.round((tempo / 40) * h)));
      const x = i * larguraBarra;
      const y = h - altura;

      ctxHudGrafico.fillStyle = classificarFrametime(tempo);
      ctxHudGrafico.fillRect(x, y, Math.max(1, larguraBarra - 0.5), altura);
    }
  }

  function registrarApresentacao(agora, frameTimeMs, quadroNum) {
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

    ultimoFrameTime = frameTimeMs;
    historicoFrameTime[indiceFrame] = frameTimeMs;
    indiceFrame = (indiceFrame + 1) % HISTORICO_TEMPOS_MAX;
    totalFrame = Math.min(totalFrame + 1, HISTORICO_TEMPOS_MAX);

    if (ultimoTempoHz === 0) {
      ultimoTempoHz = agora;
      ultimoQuadroSim = quadroNum;
    } else if (agora - ultimoTempoHz >= 500) {
      const deltaQuadros = quadroNum - ultimoQuadroSim;
      const deltaTempo = agora - ultimoTempoHz;
      simulacaoHz = deltaTempo > 0 ? (deltaQuadros * 1000) / deltaTempo : 60.0;
      ultimoQuadroSim = quadroNum;
      ultimoTempoHz = agora;
    }

    if (hudVisivel) {
      if (el.elHudFps) el.elHudFps.textContent = `${fpsApresentado.toFixed(1)} FPS`;
      if (el.elHudHz) el.elHudHz.textContent = `${simulacaoHz.toFixed(1)} Hz sim`;
      if (el.elHudFrameTime) el.elHudFrameTime.textContent = `${ultimoFrameTime.toFixed(1)} ms`;
      if (el.elHudWasmTime) el.elHudWasmTime.textContent = `${ultimoWasmTime.toFixed(1)} ms`;
      desenharGraficoHud();
    }
  }

  function registrarTempoWasm(ms) {
    ultimoWasmTime = ms;
    historicoWasmTime[indiceWasm] = ms;
    indiceWasm = (indiceWasm + 1) % HISTORICO_TEMPOS_MAX;
    totalWasm = Math.min(totalWasm + 1, HISTORICO_TEMPOS_MAX);
  }

  function atualizarDiag(diag, audioCtx) {
    if (!el.painelDiag || el.painelDiag.hidden) return;

    const seg = (diag.quadroNum * PERIODO_QUADRO / 1000).toFixed(1);
    const strQuadro = `${diag.quadroNum} (${seg}s de jogo)`;
    if (el.elDiagQuadro && el.elDiagQuadro.textContent !== strQuadro) {
      el.elDiagQuadro.textContent = strQuadro;
    }

    const strAmostras = String(diag.lenAudio);
    if (el.elDiagAmostras && el.elDiagAmostras.textContent !== strAmostras) {
      el.elDiagAmostras.textContent = strAmostras;
    }

    const strPico = diag.picoQuadro.toFixed(4);
    if (el.elDiagPico && el.elDiagPico.textContent !== strPico) {
      el.elDiagPico.textContent = strPico;
    }

    const scaleStr = `scaleX(${Math.min(1, diag.picoQuadro).toFixed(3)})`;
    if (el.elBarraVolume && el.elBarraVolume.style.transform !== scaleStr) {
      el.elBarraVolume.style.transform = scaleStr;
    }

    const strPicoMax = diag.picoHistorico.toFixed(4);
    if (el.elDiagPicoMax && el.elDiagPicoMax.textContent !== strPicoMax) {
      el.elDiagPicoMax.textContent = strPicoMax;
    }

    const strQuadrosSom = `${diag.quadrosComSom} quadros`;
    if (el.elDiagQuadrosSom && el.elDiagQuadrosSom.textContent !== strQuadrosSom) {
      el.elDiagQuadrosSom.textContent = strQuadrosSom;
    }

    if (el.elDiagFps) {
      const strFps = `${fpsApresentado.toFixed(1)} FPS (simulação: ${simulacaoHz.toFixed(1)} Hz)`;
      if (el.elDiagFps.textContent !== strFps) el.elDiagFps.textContent = strFps;
    }

    if (el.elDiagTempoWasm) {
      const { media, p95 } = calcularMediaEP95(historicoWasmTime, totalWasm, copiaWasmTime);
      const strTempo = `${media.toFixed(2)} ms (p95: ${p95.toFixed(2)} ms)`;
      if (el.elDiagTempoWasm.textContent !== strTempo) el.elDiagTempoWasm.textContent = strTempo;
    }

    atualizarEstadoAudio(audioCtx);
  }

  function atualizarWorklet(status) {
    if (!el.painelDiag || el.painelDiag.hidden || !el.elDiagWorklet) return;
    el.elDiagWorklet.textContent = `Fila: ${status.disponiveis} amostras | Falhas: ${status.falhas} | Descartadas: ${status.descartadas}`;
  }

  function definirPipeline(texto, classe) {
    if (!el.elDiagPipeline) return;
    el.elDiagPipeline.textContent = texto;
    el.elDiagPipeline.className = classe;
  }

  function definirErroWorklet(texto) {
    if (el.elDiagWorklet) el.elDiagWorklet.textContent = texto;
  }

  function atualizarEstadoAudio(audioCtx) {
    if (!audioCtx) {
      if (el.elDiagEstado && el.elDiagEstado.textContent !== "Não inicializado") {
        el.elDiagEstado.textContent = "Não inicializado";
        el.elDiagEstado.className = "diag-mudo";
      }
      if (el.badgeAudio && el.badgeAudio.textContent !== "Áudio inativo") {
        el.badgeAudio.className = "status-item";
        el.badgeAudio.textContent = "Áudio inativo";
      }
      return;
    }

    const st = audioCtx.state;
    if (st === "running") {
      if (el.elDiagEstado && el.elDiagEstado.textContent !== "Ativo (running)") {
        el.elDiagEstado.textContent = "Ativo (running)";
        el.elDiagEstado.className = "diag-ativo";
      }
      if (el.btnAudio && el.btnAudio.textContent !== "Áudio ativo") {
        el.btnAudio.textContent = "Áudio ativo";
        el.btnAudio.classList.remove("btn-verde", "btn-audio-aviso");
        el.btnAudio.classList.add("btn-audio-ativo");
      }
      if (el.badgeAudio && el.badgeAudio.textContent !== "Áudio ativo") {
        el.badgeAudio.className = "status-item ativo";
        el.badgeAudio.textContent = "Áudio ativo";
      }
    } else if (st === "suspended") {
      if (el.elDiagEstado && el.elDiagEstado.textContent !== "Suspenso pelo navegador") {
        el.elDiagEstado.textContent = "Suspenso pelo navegador";
        el.elDiagEstado.className = "diag-aviso";
      }
      if (el.btnAudio && el.btnAudio.textContent !== "Desbloquear áudio") {
        el.btnAudio.textContent = "Desbloquear áudio";
        el.btnAudio.classList.remove("btn-verde", "btn-audio-ativo");
        el.btnAudio.classList.add("btn-audio-aviso");
      }
      if (el.badgeAudio && el.badgeAudio.textContent !== "Clique para ativar áudio") {
        el.badgeAudio.className = "status-item aviso";
        el.badgeAudio.textContent = "Clique para ativar áudio";
      }
    } else if (el.elDiagEstado && el.elDiagEstado.textContent !== st) {
      el.elDiagEstado.textContent = st;
      el.elDiagEstado.className = "diag-mudo";
    }

    const taxaStr = audioCtx.sampleRate + " Hz";
    if (el.elDiagTaxa && el.elDiagTaxa.textContent !== taxaStr) {
      el.elDiagTaxa.textContent = taxaStr;
    }
  }

  function resetar() {
    indiceFrame = 0;
    totalFrame = 0;
    indiceWasm = 0;
    totalWasm = 0;
    ultimoTimestampRaf = 0;
    contadorApresentacoes = 0;
    acumuladorFpsTempo = 0;
    ultimoQuadroSim = 0;
    ultimoTempoHz = 0;
  }

  function estaDiagAberto() {
    return Boolean(el.painelDiag && !el.painelDiag.hidden);
  }

  return {
    alternarHud,
    registrarApresentacao,
    registrarTempoWasm,
    atualizarDiag,
    atualizarWorklet,
    atualizarEstadoAudio,
    definirPipeline,
    definirErroWorklet,
    resetar,
    estaDiagAberto,
  };
}
