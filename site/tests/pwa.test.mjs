// Testes unitarios do modulo PWA (site/pwa.js): deteccao standalone/iOS, SW, rede e instalacao.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  detectarModoStandalone,
  detectarIosSafari,
  inicializarPwa,
} from "../pwa.js";

function criarElStub(hidden = true) {
  const ouvintes = new Map();
  return {
    hidden,
    focado: 0,
    focus() {
      this.focado++;
    },
    addEventListener(tipo, fn) {
      ouvintes.set(tipo, fn);
    },
    async disparar(tipo, evt = {}) {
      const fn = ouvintes.get(tipo);
      if (fn) return await fn(evt);
    },
  };
}

test("detectarModoStandalone e detectarIosSafari identificam plataformas corretamente", () => {
  const origWindow = globalThis.window;
  const origNavigator = Object.getOwnPropertyDescriptor(globalThis, "navigator");

  try {
    globalThis.window = undefined;
    assert.equal(detectarModoStandalone(), false);

    globalThis.window = {
      matchMedia: () => ({ matches: true }),
      navigator: {},
    };
    assert.equal(detectarModoStandalone(), true);

    globalThis.window = {
      matchMedia: () => ({ matches: false }),
      navigator: { standalone: true },
    };
    assert.equal(detectarModoStandalone(), true);

    globalThis.window = {
      matchMedia: () => ({ matches: false }),
      navigator: { standalone: false },
    };
    assert.equal(detectarModoStandalone(), false);

    Object.defineProperty(globalThis, "navigator", {
      configurable: true,
      value: {
        userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Safari/604.1",
      },
    });
    assert.equal(detectarIosSafari(), true);

    Object.defineProperty(globalThis, "navigator", {
      configurable: true,
      value: {
        userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 CriOS/120.0",
      },
    });
    assert.equal(detectarIosSafari(), false);

    Object.defineProperty(globalThis, "navigator", {
      configurable: true,
      value: undefined,
    });
    assert.equal(detectarIosSafari(), false);
  } finally {
    globalThis.window = origWindow;
    if (origNavigator) {
      Object.defineProperty(globalThis, "navigator", origNavigator);
    }
  }
});

test("inicializarPwa registra SW, monitora online/offline e gerencia beforeinstallprompt", async () => {
  const origWindow = globalThis.window;
  const origDocument = globalThis.document;
  const origNavigator = Object.getOwnPropertyDescriptor(globalThis, "navigator");

  const winOuvintes = new Map();
  const registrosSw = [];

  globalThis.window = {
    matchMedia: () => ({ matches: false }),
    navigator: {},
    addEventListener(tipo, fn) {
      winOuvintes.set(tipo, fn);
    },
  };
  globalThis.document = {
    readyState: "loading",
    activeElement: null,
  };
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: {
      userAgent: "Mozilla/5.0 Chrome/125.0",
      onLine: true,
      serviceWorker: {
        register(url) {
          registrosSw.push(url);
          return Promise.reject(new Error("iframe-sw-blocked"));
        },
      },
    },
  });

  try {
    const btnInstalar = criarElStub(true);
    const badgeOffline = criarElStub(true);

    inicializarPwa({
      btnInstalar,
      badgeOffline,
      modalIos: null,
      btnFecharModalIos: null,
    });

    // SW so registra no evento load quando readyState === "loading".
    assert.equal(registrosSw.length, 0);
    winOuvintes.get("load")();
    assert.deepEqual(registrosSw, ["./sw.js"]);

    // Status de rede online/offline.
    assert.equal(badgeOffline.hidden, true);
    winOuvintes.get("offline")();
    assert.equal(badgeOffline.hidden, false);
    winOuvintes.get("online")();
    assert.equal(badgeOffline.hidden, true);

    // beforeinstallprompt exibe botao e aceita instalacao.
    let prevenido = false;
    let promptChamado = 0;
    winOuvintes.get("beforeinstallprompt")({
      preventDefault() {
        prevenido = true;
      },
      prompt() {
        promptChamado++;
      },
      userChoice: Promise.resolve({ outcome: "accepted" }),
    });
    assert.equal(prevenido, true);
    assert.equal(btnInstalar.hidden, false);

    await btnInstalar.disparar("click");
    assert.equal(promptChamado, 1);
    assert.equal(btnInstalar.hidden, true);

    // Erro no prompt e ignorado sem quebrar, e appinstalled esconde o botao.
    winOuvintes.get("beforeinstallprompt")({
      preventDefault() {},
      prompt() {
        throw new Error("cancelado");
      },
    });
    assert.equal(btnInstalar.hidden, false);
    await btnInstalar.disparar("click");
    winOuvintes.get("appinstalled")();
    assert.equal(btnInstalar.hidden, true);
  } finally {
    globalThis.window = origWindow;
    globalThis.document = origDocument;
    if (origNavigator) {
      Object.defineProperty(globalThis, "navigator", origNavigator);
    }
  }
});

test("inicializarPwa registra SW imediatamente em complete e abre/fecha guia iOS Safari", async () => {
  const origWindow = globalThis.window;
  const origDocument = globalThis.document;
  const origNavigator = Object.getOwnPropertyDescriptor(globalThis, "navigator");

  const winOuvintes = new Map();
  const registrosSw = [];
  const focoAnterior = criarElStub(false);

  globalThis.window = {
    matchMedia: () => ({ matches: false }),
    navigator: { standalone: false },
    addEventListener(tipo, fn) {
      winOuvintes.set(tipo, fn);
    },
  };
  globalThis.document = {
    readyState: "complete",
    activeElement: focoAnterior,
  };
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: {
      userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Safari/604.1",
      onLine: false,
      serviceWorker: {
        register(url) {
          registrosSw.push(url);
          return Promise.resolve();
        },
      },
    },
  });

  try {
    const btnInstalar = criarElStub(true);
    const badgeOffline = criarElStub(true);
    const modalIos = criarElStub(true);
    const btnFecharModalIos = criarElStub(false);

    inicializarPwa({
      btnInstalar,
      badgeOffline,
      modalIos,
      btnFecharModalIos,
    });

    assert.deepEqual(registrosSw, ["./sw.js"]);
    assert.equal(badgeOffline.hidden, false);
    assert.equal(btnInstalar.hidden, false);

    // Clicar em instalar no iOS Safari abre o modal de guia.
    await btnInstalar.disparar("click");
    assert.equal(modalIos.hidden, false);
    assert.equal(btnFecharModalIos.focado, 1);

    // Fechar pelo botao restaura foco.
    await btnFecharModalIos.disparar("click");
    assert.equal(modalIos.hidden, true);
    assert.equal(focoAnterior.focado, 1);

    // Abrir novamente e fechar pelo backdrop.
    await btnInstalar.disparar("click");
    await modalIos.disparar("click", { target: modalIos });
    assert.equal(modalIos.hidden, true);

    // Abrir novamente e fechar pela tecla Escape.
    await btnInstalar.disparar("click");
    winOuvintes.get("keydown")({ key: "Escape" });
    assert.equal(modalIos.hidden, true);
  } finally {
    globalThis.window = origWindow;
    globalThis.document = origDocument;
    if (origNavigator) {
      Object.defineProperty(globalThis, "navigator", origNavigator);
    }
  }
});
