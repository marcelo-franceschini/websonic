// Apresentador de video: OffscreenCanvas zero-copy no Worker, fallback WebGL2/2D, widescreen/ultrawide e tela cheia.
import { criarRenderizadorGPU, LARGURA, LARGURA_MAX, ALTURA } from "./gpu.js";

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
 * Normaliza a largura logica para um numero par dentro do intervalo suportado pelo VDP [320, 640].
 */
export function normalizarLarguraLogica(largura) {
  const n = Number.isFinite(largura) ? Math.round(largura) : LARGURA;
  return Math.max(LARGURA, Math.min(LARGURA_MAX, n)) & ~1;
}

/**
 * Calcula a largura logica em pixels (altura fixa 224) para a proporcao escolhida ou para o monitor atual.
 */
export function calcularLarguraLogica({
  proporcao = "4:3",
  larguraJanela = 1280,
  alturaJanela = 720,
}) {
  switch (proporcao) {
    case "4:3":
      return LARGURA;
    case "16:9":
      return 398;
    case "21:9":
      return 524;
    case "32:9":
      return LARGURA_MAX;
    case "auto": {
      const h = Math.max(1, alturaJanela);
      const razao = larguraJanela / h;
      return normalizarLarguraLogica(ALTURA * razao);
    }
    default:
      return LARGURA;
  }
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

/**
 * Calcula dimensoes finais em CSS pixels, liberando toda a largura do monitor em widescreen/ultrawide
 * e preservando escala inteira pixel-perfect no modo 4:3 em janela.
 */
export function calcularDimensoesExibicao({
  larguraJanela,
  alturaJanela,
  emTelaCheia = false,
  proporcao = "4:3",
  larguraBase = LARGURA,
  alturaBase = ALTURA,
}) {
  if (proporcao === "4:3" && !emTelaCheia && larguraBase === LARGURA) {
    const f = calcularFatorEscalaInteira({
      larguraJanela,
      alturaJanela,
      emTelaCheia: false,
      larguraBase,
      alturaBase,
    });
    return { largura: larguraBase * f, altura: alturaBase * f };
  }

  if (emTelaCheia) {
    const escala = Math.max(1, Math.min(larguraJanela / larguraBase, alturaJanela / alturaBase));
    return {
      largura: Math.round(larguraBase * escala),
      altura: Math.round(alturaBase * escala),
    };
  }

  const dispW = Math.max(larguraBase, larguraJanela - 48);
  const dispH = Math.max(alturaBase, alturaJanela * 0.72);
  const escala = Math.max(1, Math.min(dispW / larguraBase, dispH / alturaBase));
  return {
    largura: Math.round(larguraBase * escala),
    altura: Math.round(alturaBase * escala),
  };
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
  onMudancaLargura,
  proporcaoInicial = "4:3",
}) {
  let tela = canvasInicial;
  let compositor = null;
  let ctx2d = null;
  let renderNoWorker = false;
  let contextoPerdido = false;
  let imagem2d = null;
  let imagem2dU32 = null;
  let tabelaCor = null;
  let proporcaoAtual = proporcaoInicial;
  let larguraLogicaAtual = LARGURA;

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

  function garantirBuffer2d(largura) {
    if (!imagem2d || imagem2d.width !== largura || imagem2d.height !== ALTURA) {
      imagem2d = new ImageData(largura, ALTURA);
      imagem2dU32 = new Uint32Array(imagem2d.data.buffer);
    }
    if (tela && (tela.width !== largura || tela.height !== ALTURA)) {
      tela.width = largura;
      tela.height = ALTURA;
    }
  }

  function usarCpu(motivo) {
    renderNoWorker = false;
    if (compositor && typeof compositor.destruir === "function") {
      compositor.destruir();
    }
    compositor = null;
    substituirElementoCanvas();
    ctx2d = tela.getContext("2d");
    garantirBuffer2d(larguraLogicaAtual);
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
      compositor = criarRenderizadorGPU(tela, larguraLogicaAtual);
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
        compositor = criarRenderizadorGPU(tela, larguraLogicaAtual);
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
        worker.postMessage(
          { tipo: "anexar-canvas", canvas: offscreen, largura: larguraLogicaAtual },
          [offscreen],
        );
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
    } else if (ctx2d && tabelaCor) {
      const larguraQuadro = pts.length % ALTURA === 0
        ? normalizarLarguraLogica(pts.length / ALTURA)
        : larguraLogicaAtual;
      garantirBuffer2d(larguraQuadro);
      const d32 = imagem2dU32;
      const total = Math.min(d32.length, pts.length);
      for (let i = 0; i < total; i++) {
        d32[i] = tabelaCor[pts[i] & 0x1FF];
      }
      ctx2d.putImageData(imagem2d, 0, 0);
    }
  }

  function ajustarEscala() {
    if (!tela) return larguraLogicaAtual;
    const emTelaCheia = Boolean(document.fullscreenElement);
    const novaLargura = calcularLarguraLogica({
      proporcao: proporcaoAtual,
      larguraJanela: window.innerWidth,
      alturaJanela: window.innerHeight,
    });
    const mudouLargura = novaLargura !== larguraLogicaAtual;
    larguraLogicaAtual = novaLargura;

    if (!renderNoWorker) {
      if (compositor && typeof compositor.redimensionar === "function") {
        compositor.redimensionar(larguraLogicaAtual);
      } else if (ctx2d) {
        garantirBuffer2d(larguraLogicaAtual);
      }
    }

    const dim = calcularDimensoesExibicao({
      larguraJanela: window.innerWidth,
      alturaJanela: window.innerHeight,
      emTelaCheia,
      proporcao: proporcaoAtual,
      larguraBase: larguraLogicaAtual,
      alturaBase: ALTURA,
    });
    tela.style.width = `${dim.largura}px`;
    tela.style.height = `${dim.altura}px`;

    if (mudouLargura && onMudancaLargura) {
      onMudancaLargura(larguraLogicaAtual);
    }
    return larguraLogicaAtual;
  }

  function definirProporcao(novaProporcao) {
    proporcaoAtual = novaProporcao || "4:3";
    return ajustarEscala();
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
    definirProporcao,
    alternarTelaCheia,
    obterTela: () => tela,
    obterLarguraLogica: () => larguraLogicaAtual,
    estaNoWorker: () => renderNoWorker,
    estaPerdido: () => contextoPerdido,
  };
}
