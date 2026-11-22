// Testes unitarios do modal de remapeamento de controles (site/modal-controles.js).
import { test } from "node:test";
import assert from "node:assert/strict";
import { criarModalControles } from "../modal-controles.js";
import { ACOES_REMAP, MAPA_PADRAO_ACAO } from "../entrada.js";

function criarNoDom(tag = "div") {
  const ouvintes = new Map();
  const attrs = new Map();
  const filhos = [];
  return {
    tag,
    type: "",
    className: "",
    textContent: "",
    hidden: true,
    filhos,
    focado: 0,
    set innerHTML(v) {
      if (v === "") filhos.length = 0;
    },
    appendChild(el) {
      filhos.push(el);
    },
    setAttribute(k, v) {
      attrs.set(k, String(v));
    },
    getAttribute(k) {
      return attrs.get(k);
    },
    focus() {
      this.focado++;
    },
    addEventListener(tipo, fn) {
      ouvintes.set(tipo, fn);
    },
    disparar(tipo, evt = {}) {
      const fn = ouvintes.get(tipo);
      if (fn) fn(evt);
    },
  };
}

test("criarModalControles abre, renderiza, grava remapeamento, restaura padrao e fecha com foco", () => {
  const origWindow = globalThis.window;
  const origDocument = globalThis.document;
  const winOuvintes = new Map();

  const focoAnterior = criarNoDom("button");
  globalThis.window = {
    addEventListener(tipo, fn) {
      winOuvintes.set(tipo, fn);
    },
  };
  globalThis.document = {
    activeElement: focoAnterior,
    createElement: (tag) => criarNoDom(tag),
  };

  try {
    const modal = criarNoDom("div");
    const listaRemap = criarNoDom("div");
    const btnAbrir = criarNoDom("button");
    const btnFechar = criarNoDom("button");
    const btnConcluir = criarNoDom("button");
    const btnRestaurar = criarNoDom("button");

    let mapa = { ...MAPA_PADRAO_ACAO };
    let capturaCallback = null;
    let cancelamentos = 0;
    let restauracoes = 0;
    let fechamentos = 0;

    const controladorEntrada = {
      obterMapaAcoes: () => ({ ...mapa }),
      iniciarCapturaRemap(_acaoId, cb) {
        capturaCallback = cb;
      },
      cancelarCapturaRemap() {
        cancelamentos++;
      },
      restaurarPadrao() {
        restauracoes++;
        mapa = { ...MAPA_PADRAO_ACAO };
      },
    };

    const api = criarModalControles({
      modal,
      listaRemap,
      botoesAbrir: [btnAbrir, null],
      btnFechar,
      btnConcluir,
      btnRestaurar,
      controladorEntrada,
      onFechar: () => fechamentos++,
    });

    // Abrir via clique no botao de abertura.
    btnAbrir.disparar("click");
    assert.equal(modal.hidden, false);
    assert.equal(btnFechar.focado, 1);
    assert.equal(listaRemap.filhos.length, ACOES_REMAP.length);

    // Clicar no primeiro botao (cima) inicia gravacao e re-renderiza com foco.
    const primeiroBtn = listaRemap.filhos[0].filhos[1];
    assert.equal(primeiroBtn.textContent, "W");
    primeiroBtn.disparar("click");

    const btnGravando = listaRemap.filhos[0].filhos[1];
    assert.equal(btnGravando.textContent, "Pressione...");
    assert.equal(btnGravando.getAttribute("aria-pressed"), "true");
    assert.match(btnGravando.className, /gravando/);
    assert.equal(btnGravando.focado, 1);

    // Concluir captura re-renderiza com a nova tecla.
    mapa.cima = "KeyI";
    capturaCallback("KeyI");
    const btnAtualizado = listaRemap.filhos[0].filhos[1];
    assert.equal(btnAtualizado.textContent, "I");
    assert.equal(btnAtualizado.getAttribute("aria-pressed"), "false");
    assert.equal(btnAtualizado.focado, 1);

    // Restaurar padrao cancela captura e re-renderiza.
    btnRestaurar.disparar("click");
    assert.equal(restauracoes, 1);
    assert.equal(cancelamentos, 1);
    assert.equal(listaRemap.filhos[0].filhos[1].textContent, "W");

    // Clique no fundo do modal fecha e devolve foco ao elemento anterior.
    modal.disparar("click", { target: modal });
    assert.equal(modal.hidden, true);
    assert.equal(focoAnterior.focado, 1);
    assert.equal(fechamentos, 1);

    // Escape fecha quando modal esta aberto.
    api.abrir();
    assert.equal(modal.hidden, false);
    winOuvintes.get("keydown")({ key: "Escape" });
    assert.equal(modal.hidden, true);
    assert.equal(fechamentos, 2);

    // Botao concluir fecha modal.
    api.abrir();
    btnConcluir.disparar("click");
    assert.equal(modal.hidden, true);

    // Guarda segura quando modal ou listaRemap sao nulos.
    const nulo = criarModalControles({
      modal: null,
      listaRemap: null,
      controladorEntrada,
    });
    nulo.abrir();
    nulo.fechar();
  } finally {
    globalThis.window = origWindow;
    globalThis.document = origDocument;
  }
});
