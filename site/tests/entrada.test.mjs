// Testes unitarios da logica pura de entrada (teclado, gamepad, toque e cancelamento de opostas).
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  calcularMascaraTeclado,
  calcularMascaraGamepad,
  calcularMascaraToque,
  combinarMascaras,
  cancelarOpostas,
  ehModoTituloOuDemo,
  construirMapaTeclas,
  carregarMapaPersonalizado,
  salvarMapaPersonalizado,
  formatarCodigoTecla,
  MAPA_PADRAO_ACAO,
  criarControladorEntrada,
} from "../entrada.js";

test("ehModoTituloOuDemo identifica Sega, Titulo e Demo", () => {
  assert.equal(ehModoTituloOuDemo(0x00), true);
  assert.equal(ehModoTituloOuDemo(0x04), true);
  assert.equal(ehModoTituloOuDemo(0x08), true);
  assert.equal(ehModoTituloOuDemo(0x0C), false);
  assert.equal(ehModoTituloOuDemo(0x10), false);
});

test("teclado mapeia Space para Start no titulo e para pulo (C) na fase", () => {
  const teclas = new Set(["Space"]);
  assert.equal(calcularMascaraTeclado(teclas, 0x04), 0x80);
  assert.equal(calcularMascaraTeclado(teclas, 0x00), 0x80);
  assert.equal(calcularMascaraTeclado(teclas, 0x0C), 0x20);
});

test("cancelamento de direcoes opostas funciona em teclado e na combinacao", () => {
  assert.equal(calcularMascaraTeclado(new Set(["ArrowLeft", "ArrowRight"]), 0x0C), 0x00);
  assert.equal(calcularMascaraTeclado(new Set(["KeyW", "KeyS", "KeyZ"]), 0x0C), 0x40);
  assert.equal(cancelarOpostas(0x0F), 0x00);
  // Esquerda no teclado + Direita no toque devem se cancelar na combinacao final.
  assert.equal(combinarMascaras(0x04, 0x00, 0x08 | 0x20), 0x20);
});

test("gamepad le D-pad, analogico, DirectInput e botoes de acao (e mapeia acao para Start no titulo)", () => {
  assert.equal(calcularMascaraGamepad(null, 0x0C), 0);

  const botoes = Array.from({ length: 17 }, () => ({ pressed: false, value: 0 }));
  botoes[0] = { pressed: true, value: 1 }; // A -> B (0x10)
  const gp = [{ connected: true, buttons: botoes, axes: [0.8, 0] }];

  assert.equal(calcularMascaraGamepad(gp, 0x0C), 0x08 | 0x10);
  assert.equal(calcularMascaraGamepad(gp, 0x04), 0x08 | 0x10 | 0x80);

  // DirectInput axes 4/5 + botoes secundarios + gamepad desconectado ignorado.
  const b2 = Array.from({ length: 17 }, () => ({ pressed: false, value: 0 }));
  b2[1] = { pressed: true, value: 1 };
  b2[2] = { pressed: true, value: 1 };
  b2[3] = { pressed: true, value: 1 };
  b2[4] = { pressed: true, value: 1 };
  b2[9] = { pressed: true, value: 1 };
  const gp2 = [
    null,
    { connected: false, buttons: b2, axes: [0, 0] },
    { connected: true, buttons: b2, axes: [0, 0, 0, 0, -0.9, -0.9] },
  ];
  assert.equal(calcularMascaraGamepad(gp2, 0x0C), 0x01 | 0x04 | 0x10 | 0x20 | 0x40 | 0x80);

  const gp3 = [{ connected: true, buttons: [], axes: [0, 0, 0, 0, 0.9, 0.9] }];
  assert.equal(calcularMascaraGamepad(gp3, 0x0C), 0x02 | 0x08);
});

test("toque mapeia direcoes, botoes e Start no titulo", () => {
  assert.equal(calcularMascaraToque(null, 0x0C), 0);
  assert.equal(calcularMascaraToque(new Set(), 0x0C), 0);
  const toques = new Set(["direita", "a"]);
  assert.equal(calcularMascaraToque(toques, 0x0C), 0x08 | 0x40);
  assert.equal(calcularMascaraToque(toques, 0x04), 0x08 | 0x40 | 0x80);

  const todos = new Set(["cima", "esquerda", "b", "c", "start"]);
  assert.equal(calcularMascaraToque(todos, 0x0C), 0x01 | 0x04 | 0x10 | 0x20 | 0x80);
});

test("remapeamento personalizado sobrescreve teclas, formata codigos e tolera storage falho", () => {
  assert.deepEqual(carregarMapaPersonalizado(null), MAPA_PADRAO_ACAO);
  salvarMapaPersonalizado(null, MAPA_PADRAO_ACAO);

  const storageQuebrado = {
    getItem() {
      throw new Error("read-err");
    },
    setItem() {
      throw new Error("write-err");
    },
  };
  assert.deepEqual(carregarMapaPersonalizado(storageQuebrado), MAPA_PADRAO_ACAO);
  salvarMapaPersonalizado(storageQuebrado, MAPA_PADRAO_ACAO);

  const mem = new Map();
  const storageFalso = {
    getItem: (k) => (mem.has(k) ? mem.get(k) : null),
    setItem: (k, v) => mem.set(k, v),
  };

  const custom = { ...MAPA_PADRAO_ACAO, cima: "KeyI", direita: "KeyM", a: "KeyQ" };
  salvarMapaPersonalizado(storageFalso, custom);
  const recarregado = carregarMapaPersonalizado(storageFalso);
  assert.equal(recarregado.cima, "KeyI");
  assert.equal(recarregado.direita, "KeyM");
  assert.equal(recarregado.a, "KeyQ");

  const mapaTeclas = construirMapaTeclas(recarregado);
  assert.equal(calcularMascaraTeclado(new Set(["KeyI", "KeyQ"]), 0x0C, mapaTeclas), 0x01 | 0x40);
  assert.equal(formatarCodigoTecla(""), "-");
  assert.equal(formatarCodigoTecla("KeyQ"), "Q");
  assert.equal(formatarCodigoTecla("Digit5"), "5");
  assert.equal(formatarCodigoTecla("Numpad8"), "Num 8");
  assert.equal(formatarCodigoTecla("Space"), "Espaço");
  assert.equal(formatarCodigoTecla("F10"), "F10");
});

function criarElementoStub(attrsIniciais = {}) {
  const classes = new Set();
  const attrs = new Map(Object.entries(attrsIniciais));
  const ouvintes = new Map();
  return {
    textContent: "",
    className: "",
    hidden: false,
    tagName: "BUTTON",
    blurred: false,
    focused: false,
    blur() {
      this.blurred = true;
    },
    focus() {
      this.focused = true;
    },
    getAttribute(k) {
      return attrs.has(k) ? attrs.get(k) : null;
    },
    setAttribute(k, v) {
      attrs.set(k, String(v));
    },
    setPointerCapture() {},
    releasePointerCapture() {},
    classList: {
      add(c) {
        classes.add(c);
      },
      remove(c) {
        classes.delete(c);
      },
      toggle(c, f) {
        const ativo = f !== undefined ? Boolean(f) : !classes.has(c);
        if (ativo) classes.add(c);
        else classes.delete(c);
      },
      contains: (c) => classes.has(c),
    },
    addEventListener(tipo, fn) {
      ouvintes.set(tipo, fn);
    },
    disparar(tipo, evt = {}) {
      const fn = ouvintes.get(tipo);
      if (fn) fn({ preventDefault() {}, stopImmediatePropagation() {}, ...evt });
    },
  };
}

test("criarControladorEntrada integra teclado, atalhos, remap, gamepad, toque e blur", () => {
  const origWindow = globalThis.window;
  const origDocument = globalThis.document;
  const origNavigator = Object.getOwnPropertyDescriptor(globalThis, "navigator");
  const origLocalStorage = globalThis.localStorage;

  const winOuvintes = new Map();
  const storageMem = new Map();
  const keycapCima = criarElementoStub({ "data-acao": "cima", "data-code": "KeyW" });
  const keycapDireita = criarElementoStub({ "data-acao": "direita", "data-code": "KeyD" });
  const btnToqueDir = criarElementoStub({ "data-dir": "direita" });
  const btnToqueA = criarElementoStub({ "data-botao": "a" });
  const btnToqueToggle = criarElementoStub();
  const badgeGamepad = criarElementoStub();
  const tela = criarElementoStub({ tagName: "CANVAS" });
  const botaoFocado = criarElementoStub();
  const controlesToque = {
    hidden: true,
    querySelectorAll(sel) {
      if (sel === ".btn-toque") return [btnToqueDir, btnToqueA];
      if (sel === ".btn-toque.ativo") return [btnToqueDir];
      return [];
    },
  };

  let gamepadsAtuais = [];

  globalThis.window = {
    ontouchstart: null,
    addEventListener(tipo, fn) {
      winOuvintes.set(tipo, fn);
    },
  };
  globalThis.document = {
    body: { tagName: "BODY" },
    activeElement: botaoFocado,
    querySelectorAll(sel) {
      if (sel === ".tecla-cap[data-acao]" || sel === ".tecla-cap[data-code]") {
        return [keycapCima, keycapDireita];
      }
      if (sel === ".tecla-cap.ativa") {
        return [keycapCima];
      }
      return [];
    },
  };
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: {
      maxTouchPoints: 1,
      getGamepads: () => gamepadsAtuais,
    },
  });
  globalThis.localStorage = {
    getItem: (k) => (storageMem.has(k) ? storageMem.get(k) : null),
    setItem: (k, v) => storageMem.set(k, v),
  };

  const dispararJanela = (tipo, evt = {}) => {
    const fn = winOuvintes.get(tipo);
    if (fn) fn({ preventDefault() {}, stopImmediatePropagation() {}, ...evt });
  };

  try {
    const acoes = [];
    let interacoes = 0;
    let mudancas = 0;

    const ctrl = criarControladorEntrada({
      obterTela: () => tela,
      badgeGamepad,
      controlesToque,
      btnToque: btnToqueToggle,
      onAcaoEspecial: (a) => acoes.push(a),
      onInteracao: () => interacoes++,
      estaJogando: () => true,
      onMudancaEntrada: () => mudancas++,
    });

    assert.equal(ctrl.toqueEstaAtivo(), true);
    assert.equal(keycapCima.textContent, "W");

    // Tecla de jogo (KeyW) ilumina keycap, desfoca botao ativo e entra na mascara.
    dispararJanela("keydown", { code: "KeyW" });
    assert.equal(keycapCima.classList.contains("ativa"), true);
    assert.equal(botaoFocado.blurred, true);
    assert.equal(ctrl.lerMascara(0x0C), 0x01);
    assert.equal(interacoes, 1);

    // Repeat e ignorado.
    dispararJanela("keydown", { code: "KeyW", repeat: true });

    // Keyup solta a tecla.
    dispararJanela("keyup", { code: "KeyW" });
    assert.equal(keycapCima.classList.contains("ativa"), false);
    assert.equal(ctrl.lerMascara(0x0C), 0x00);

    // Atalhos especiais: F, F3, F5, F8, Tab down/up.
    dispararJanela("keydown", { code: "KeyF" });
    dispararJanela("keydown", { code: "F3" });
    dispararJanela("keydown", { code: "F5" });
    dispararJanela("keydown", { code: "F8" });
    dispararJanela("keydown", { code: "Tab" });
    dispararJanela("keyup", { code: "Tab" });
    assert.deepEqual(acoes, [
      "fullscreen",
      "hud",
      "salvar-rapido",
      "carregar-rapido",
      "fast-forward-inicio",
      "fast-forward-fim",
    ]);

    // Atalhos ignorados quando foco esta em INPUT.
    globalThis.document.activeElement = { tagName: "INPUT" };
    dispararJanela("keydown", { code: "KeyF" });
    assert.equal(acoes.length, 6);
    globalThis.document.activeElement = tela;

    // Captura de remapeamento: cancelar com Escape e depois trocar KeyW <-> KeyD.
    let capturado = "nenhum";
    ctrl.iniciarCapturaRemap("cima", (c) => {
      capturado = c;
    });
    dispararJanela("keydown", { code: "Escape" });
    assert.equal(capturado, null);

    ctrl.iniciarCapturaRemap("cima", (c) => {
      capturado = c;
    });
    dispararJanela("keydown", { code: "KeyD" });
    assert.equal(capturado, "KeyD");
    assert.equal(ctrl.obterMapaAcoes().cima, "KeyD");
    assert.equal(ctrl.obterMapaAcoes().direita, "KeyW");

    // Acao desconhecida e ignorada; restaurarPadrao volta ao original.
    ctrl.definirTeclaAcao("inexistente", "KeyQ");
    ctrl.cancelarCapturaRemap();
    ctrl.restaurarPadrao();
    assert.equal(ctrl.obterMapaAcoes().cima, "KeyW");

    // Gamepad conectado e desconectado.
    const botoesGp = Array.from({ length: 17 }, () => ({ pressed: false, value: 0 }));
    botoesGp[0] = { pressed: true, value: 1 };
    const gp1 = { connected: true, id: "Controle Muito Longo USB Gamepad", buttons: botoesGp, axes: [0, 0] };
    gamepadsAtuais = [gp1];
    dispararJanela("gamepadconnected", { gamepad: gp1 });
    assert.equal(ctrl.temGamepad(), true);
    assert.match(badgeGamepad.textContent, /\.\.\.$/);
    const par = ctrl.lerParMascaras();
    assert.equal(par.jogo, 0x10);
    assert.equal(par.titulo, 0x10 | 0x80);

    // Desconecta deixando outro gamepad curto conectado, depois desconecta todos.
    const gpCurto = { connected: true, id: "Xbox Pad", buttons: [], axes: [0, 0] };
    gamepadsAtuais = [gpCurto];
    dispararJanela("gamepaddisconnected");
    assert.equal(ctrl.temGamepad(), true);
    assert.equal(badgeGamepad.textContent, "Gamepad · Xbox Pad");

    gamepadsAtuais = [];
    dispararJanela("gamepaddisconnected");
    assert.equal(ctrl.temGamepad(), false);
    assert.equal(badgeGamepad.textContent, "Gamepad pronto");

    // Toque: pointerdown, pointerup, pointercancel e botao alternar toque.
    btnToqueDir.disparar("pointerdown", { pointerId: 1 });
    assert.equal(ctrl.lerMascara(0x0C), 0x08);
    btnToqueDir.disparar("pointerup", { pointerId: 1 });
    assert.equal(ctrl.lerMascara(0x0C), 0x00);

    btnToqueA.disparar("pointerdown", { pointerId: 2 });
    assert.equal(ctrl.lerMascara(0x0C), 0x40);
    btnToqueA.disparar("pointercancel", { pointerId: 2 });
    assert.equal(ctrl.lerMascara(0x0C), 0x00);

    // Blur limpa teclas e toques e dispara fast-forward-fim.
    btnToqueDir.disparar("pointerdown", { pointerId: 3 });
    dispararJanela("blur");
    assert.equal(ctrl.lerMascara(0x0C), 0x00);
    assert.equal(acoes[acoes.length - 1], "fast-forward-fim");

    // Alternar botao de toque.
    btnToqueToggle.disparar("click");
    assert.equal(ctrl.toqueEstaAtivo(), false);
    assert.equal(controlesToque.hidden, true);
    assert.equal(tela.focused, true);
    assert.ok(mudancas > 0);
  } finally {
    globalThis.window = origWindow;
    globalThis.document = origDocument;
    if (origNavigator) {
      Object.defineProperty(globalThis, "navigator", origNavigator);
    }
    globalThis.localStorage = origLocalStorage;
  }
});

