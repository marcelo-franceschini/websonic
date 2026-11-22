// Apresentador de video: OffscreenCanvas zero-copy no Worker, fallback WebGL2/2D, escala inteira e tela cheia.
import { criarRenderizadorGPU, LARGURA, ALTURA } from "./gpu.js";

/**
 * Gera a tabela de conversao de cor de 9 bits (BBBGGGRRR) para RGBA (Uint32 little-endian),
 * usada apenas no fallback de renderizacao 2D por software.
 */
export function gerarTabelaCor9Bits() {
  const tabela = new Uint32Array(512);
  for (let c = 0; c < 512; c++) {
    const r = c & 7;
    const g = (c >> 3) & 7;
    const b = (c >> 6) & 7;
    const r8 = (r << 5) | (r << 2) | (r >> 1);
    const g8 = (g << 5) | (g << 2) | (g >> 1);
    const b8 = (b << 5) | (b << 2) | (b >> 1);
    tabela[c] = 0xFF000000 | (b8 << 16) | (g8 << 8) | r8;
  }
  return tabela;
}

/**
 * Calcula o maior fator de escala inteiro (>= 1) que cabe na janela ou em tela cheia
 * preservando pixels perfeitamente quadrados.
 */
export function calcularFatorEscalaInteira({
  larguraJanela,
  alturaJanela,
  emTelaCheia = false,
  larguraBase = LARGURA,
  alturaBase = ALTURA,
}) {
  if (emTelaCheia) {
    return Math.max(1, Math.floor(Math.min(larguraJanela / larguraBase, alturaJanela / alturaBase)));
  }
  const dispW = Math.min(Math.max(larguraBase, larguraJanela - 32), 960);
  const dispH = alturaJanela * 0.72;
  return Math.max(1, Math.floor(Math.min(dispW / larguraBase, dispH / alturaBase)));
}

export function criarApresentadorVideo({
  canvasInicial,
  containerTela,
  badgeGpu,
  onErro,
  onLimparErro,
  onContextoPerdido,
  onContextoRestaurado,
  onInteracao,
}) {
  let tela = canvasInicial;
  let compositor = null;
  let ctx2d = null;
  let renderNoWorker = false;
  let contextoPerdido = false;
  let imagem2d = null;
  let imagem2dU32 = null;
  let tabelaCor = null;

  function registrarClique(canvas) {
    if (!canvas) return;
    canvas.addEventListener("click", () => {
      canvas.focus();
      if (onInteracao) onInteracao();
    });
  }

  function substituirElementoCanvas() {
    const nova = tela.cloneNode(false);
    tela.replaceWith(nova);
    tela = nova;
    tela.tabIndex = 0;
    registrarClique(tela);
    return tela;
  }

  function usarCpu(motivo) {
    renderNoWorker = false;
    if (compositor && typeof compositor.destruir === "function") {
      compositor.destruir();
    }
    compositor = null;
    substituirElementoCanvas();
    ctx2d = tela.getContext("2d");
    if (!imagem2d) {
      imagem2d = new ImageData(LARGURA, ALTURA);
      imagem2dU32 = new Uint32Array(imagem2d.data.buffer);
    }
    if (!tabelaCor) tabelaCor = gerarTabelaCor9Bits();
    contextoPerdido = false;
    if (badgeGpu) badgeGpu.textContent = "Wasm · 2D";
    if (onErro) onErro(`Vídeo por software: ${motivo}`);
  }

  function inicializarThreadPrincipal(jaTransferido) {
    renderNoWorker = false;
    if (jaTransferido) {
      substituirElementoCanvas();
    }
    try {
      compositor = criarRenderizadorGPU(tela);
      if (!compositor) {
        usarCpu("WebGL2 indisponível.");
        return;
      }
      if (badgeGpu) badgeGpu.textContent = "WebGL2 · Wasm";
    } catch (erro) {
      usarCpu(`falha ao iniciar WebGL2 (${erro.message}).`);
      return;
    }

    tela.addEventListener("webglcontextlost", (e) => {
      e.preventDefault();
      contextoPerdido = true;
      if (onContextoPerdido) onContextoPerdido();
      if (onErro) onErro("Contexto gráfico perdido. Aguardando recuperação.");
    });

    tela.addEventListener("webglcontextrestored", () => {
      try {
        if (compositor && typeof compositor.destruir === "function") {
          compositor.destruir();
        }
        compositor = criarRenderizadorGPU(tela);
        if (!compositor) throw new Error("WebGL2 indisponível");
        contextoPerdido = false;
        if (onLimparErro) onLimparErro();
      } catch (erro) {
        usarCpu(`falha ao restaurar WebGL2 (${erro.message}).`);
      }
      if (onContextoRestaurado) onContextoRestaurado();
    });
  }

  function tentarAnexarNoWorker(worker) {
    if (worker && tela && typeof tela.transferControlToOffscreen === "function") {
      try {
        const offscreen = tela.transferControlToOffscreen();
        worker.postMessage({ tipo: "anexar-canvas", canvas: offscreen }, [offscreen]);
        return true;
      } catch {
        // Fallback para thread principal.
      }
    }
    inicializarThreadPrincipal(false);
    return false;
  }

  function confirmarAnexoWorker(ok) {
    if (ok) {
      renderNoWorker = true;
      if (badgeGpu) badgeGpu.textContent = "WebGL2 · Zero-Copy";
    } else {
      inicializarThreadPrincipal(true);
    }
  }

  function notificarPerdaNoWorker() {
    contextoPerdido = true;
    if (onContextoPerdido) onContextoPerdido();
    if (onErro) onErro("Contexto gráfico perdido no Worker. Aguardando recuperação.");
  }

  function notificarRestauracaoNoWorker(ok) {
    if (ok) {
      contextoPerdido = false;
      if (onLimparErro) onLimparErro();
      if (onContextoRestaurado) onContextoRestaurado();
    } else {
      inicializarThreadPrincipal(true);
    }
  }

  function apresentar(pts) {
    if (renderNoWorker || contextoPerdido || !pts) return;
    if (compositor) {
      try {
        compositor.renderizar(pts);
      } catch (erro) {
        usarCpu(`falha de WebGL2 (${erro.message}).`);
      }
    } else if (ctx2d && imagem2d && imagem2dU32 && tabelaCor) {
      const d32 = imagem2dU32;
      const total = LARGURA * ALTURA;
      for (let i = 0; i < total; i++) {
        d32[i] = tabelaCor[pts[i] & 0x1FF];
      }
      ctx2d.putImageData(imagem2d, 0, 0);
    }
  }

  function ajustarEscala() {
    if (!tela) return;
    const f = calcularFatorEscalaInteira({
      larguraJanela: window.innerWidth,
      alturaJanela: window.innerHeight,
      emTelaCheia: Boolean(document.fullscreenElement),
    });
    tela.style.width = `${LARGURA * f}px`;
    tela.style.height = `${ALTURA * f}px`;
  }

  function alternarTelaCheia() {
    const alvo = containerTela || tela;
    if (!alvo) return;
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

  registrarClique(tela);
  window.addEventListener("resize", ajustarEscala);
  document.addEventListener("fullscreenchange", ajustarEscala);
  document.addEventListener("webkitfullscreenchange", ajustarEscala);

  return {
    tentarAnexarNoWorker,
    inicializarThreadPrincipal,
    confirmarAnexoWorker,
    notificarPerdaNoWorker,
    notificarRestauracaoNoWorker,
    apresentar,
    ajustarEscala,
    alternarTelaCheia,
    obterTela: () => tela,
    estaNoWorker: () => renderNoWorker,
    estaPerdido: () => contextoPerdido,
  };
}
