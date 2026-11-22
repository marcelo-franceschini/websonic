// Modal acessivel de remapeamento de teclas do controle.
import { ACOES_REMAP, formatarCodigoTecla } from "./entrada.js";

export function criarModalControles({
  modal,
  listaRemap,
  botoesAbrir,
  btnFechar,
  btnConcluir,
  btnRestaurar,
  controladorEntrada,
  onFechar,
}) {
  let ultimoFoco = null;

  function renderizarLista(acaoGravando = null, acaoFocar = null) {
    if (!listaRemap) return;
    const mapa = controladorEntrada.obterMapaAcoes();
    listaRemap.innerHTML = "";
    let botaoParaFocar = null;

    for (const acao of ACOES_REMAP) {
      const linha = document.createElement("div");
      linha.className = "remap-linha";

      const rotulo = document.createElement("span");
      rotulo.className = "remap-rotulo";
      rotulo.textContent = acao.rotulo;

      const btn = document.createElement("button");
      btn.type = "button";
      const gravando = acaoGravando === acao.id;
      const nomeTecla = formatarCodigoTecla(mapa[acao.id]);
      btn.className = gravando
        ? "btn btn-cinza btn-remap-tecla gravando"
        : "btn btn-cinza btn-remap-tecla";
      btn.textContent = gravando ? "Pressione..." : nomeTecla;
      btn.setAttribute("aria-pressed", String(gravando));
      btn.setAttribute(
        "aria-label",
        gravando
          ? `Aguardando nova tecla para ${acao.rotulo}`
          : `Remapear ${acao.rotulo}: ${nomeTecla}`,
      );
      btn.addEventListener("click", () => {
        renderizarLista(acao.id, acao.id);
        controladorEntrada.iniciarCapturaRemap(acao.id, () => {
          renderizarLista(null, acao.id);
        });
      });

      if (acaoFocar === acao.id) {
        botaoParaFocar = btn;
      }

      linha.appendChild(rotulo);
      linha.appendChild(btn);
      listaRemap.appendChild(linha);
    }

    if (botaoParaFocar && typeof botaoParaFocar.focus === "function") {
      botaoParaFocar.focus();
    }
  }

  function abrir() {
    if (!modal) return;
    ultimoFoco = typeof document !== "undefined" ? document.activeElement : null;
    renderizarLista(null);
    modal.hidden = false;
    if (btnFechar) btnFechar.focus();
  }

  function fechar() {
    if (!modal) return;
    controladorEntrada.cancelarCapturaRemap();
    modal.hidden = true;
    if (ultimoFoco && typeof ultimoFoco.focus === "function") {
      ultimoFoco.focus();
    }
    ultimoFoco = null;
    if (onFechar) onFechar();
  }

  for (const btn of botoesAbrir || []) {
    if (btn) btn.addEventListener("click", abrir);
  }
  if (btnFechar) btnFechar.addEventListener("click", fechar);
  if (btnConcluir) btnConcluir.addEventListener("click", fechar);
  if (btnRestaurar) {
    btnRestaurar.addEventListener("click", () => {
      controladorEntrada.cancelarCapturaRemap();
      controladorEntrada.restaurarPadrao();
      renderizarLista(null);
    });
  }
  if (modal) {
    modal.addEventListener("click", (e) => {
      if (e.target === modal) fechar();
    });
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !modal.hidden) {
        fechar();
      }
    });
  }

  return { abrir, fechar };
}
