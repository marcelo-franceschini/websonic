import test from "node:test";
import assert from "node:assert/strict";
import {
  formatarRotuloSlot,
  nomeArquivoSave,
  criarControladorSavesUi,
} from "../save-ui.js";

test("formatarRotuloSlot formata slot vazio e ocupado em segundos", () => {
  assert.equal(formatarRotuloSlot(1, null), "Slot 1 (vazio)");
  assert.equal(formatarRotuloSlot(2, undefined), "Slot 2 (vazio)");
  assert.equal(formatarRotuloSlot(3, { quadro: 360 }), "Slot 3 (6.0s)");
  assert.equal(formatarRotuloSlot(1, { quadro: 90 }), "Slot 1 (1.5s)");
});

test("nomeArquivoSave gera nome higienizado para exportacao .s1sav", () => {
  assert.equal(nomeArquivoSave(2, 1800), "websonic-slot2-q1800.s1sav");
  assert.equal(nomeArquivoSave(0, -5), "websonic-slot1-q0.s1sav");
});

test("criarControladorSavesUi encaminha falhas de leitura/listagem para onErro sem rejeicao solta", async () => {
  const erros = [];
  const seletorFalso = { value: "2", options: [{ value: "2", textContent: "" }] };
  const gerenciadorFalhando = {
    async listarSlots() {
      throw new Error("banco indisponivel");
    },
    async carregarSlot() {
      throw new Error("registro corrompido");
    },
  };

  const ui = criarControladorSavesUi({
    seletorSlot: seletorFalso,
    estaJogando: () => true,
    obterRomAtual: () => ({ bytes: new Uint8Array([1, 2, 3]), sha1: "abc123" }),
    onErro: (msg) => erros.push(msg),
    gerenciador: gerenciadorFalhando,
  });

  await ui.atualizarRotulosSlots();
  await ui.carregarDoSlotAtual();

  assert.equal(erros.length, 2);
  assert.match(erros[0], /banco indisponivel/);
  assert.match(erros[1], /registro corrompido/);
});

function criarBotaoStub() {
  const ouvintes = new Map();
  return {
    cliques: 0,
    value: "",
    files: [],
    addEventListener(tipo, fn) {
      ouvintes.set(tipo, fn);
    },
    click() {
      this.cliques++;
      const fn = ouvintes.get("click");
      if (fn) fn();
    },
    disparar(tipo) {
      const fn = ouvintes.get(tipo);
      if (fn) fn();
    },
  };
}

test("criarControladorSavesUi salva, carrega, exporta e importa arquivos .s1sav", async () => {
  const erros = [];
  const carregados = [];
  let focos = 0;
  let jogando = true;
  const rom = { bytes: new Uint8Array([9, 8, 7]), sha1: "sha1-sonic-teste" };

  const seletorSlot = {
    value: "2",
    options: [
      { value: "1", textContent: "" },
      { value: "2", textContent: "" },
      { value: "3", textContent: "" },
    ],
  };
  const elStatus = { textContent: "", hidden: true };
  const btnSalvar = criarBotaoStub();
  const btnCarregar = criarBotaoStub();
  const btnExportar = criarBotaoStub();
  const btnImportar = criarBotaoStub();
  const inputImportar = criarBotaoStub();

  // Stubs de temporizador e DOM para exportacao.
  const timeouts = [];
  const origSetTimeout = globalThis.setTimeout;
  const origClearTimeout = globalThis.clearTimeout;
  const origDoc = globalThis.document;
  const origCreateObjUrl = URL.createObjectURL;
  const origRevokeObjUrl = URL.revokeObjectURL;

  const linksCriados = [];
  const urlsRevogadas = [];
  let clears = 0;
  globalThis.setTimeout = (fn) => {
    const id = timeouts.length + 1;
    timeouts.push(fn);
    return id;
  };
  globalThis.clearTimeout = () => {
    clears++;
  };
  globalThis.document = {
    createElement() {
      const link = {
        href: "",
        download: "",
        clicado: false,
        removido: false,
        click() {
          this.clicado = true;
        },
        remove() {
          this.removido = true;
        },
      };
      linksCriados.push(link);
      return link;
    },
    body: {
      appendChild() {},
    },
  };
  URL.createObjectURL = () => "blob:websonic-save";
  URL.revokeObjectURL = (u) => urlsRevogadas.push(u);

  try {
    let falharWorker = false;
    const ui = criarControladorSavesUi({
      seletorSlot,
      btnSalvar,
      btnCarregar,
      btnExportar,
      btnImportar,
      inputImportar,
      elStatus,
      estaJogando: () => jogando,
      obterRomAtual: () => rom,
      solicitarSaveWorker: async () => {
        if (falharWorker) throw new Error("worker ocupado");
        return { quadro: 120, entradasRle: [[0x08, 120]] };
      },
      onCarregarSave: async (bytes, rle) => {
        carregados.push({ bytes, rle });
      },
      onErro: (m) => erros.push(m),
      onFocarTela: () => {
        focos++;
      },
    });

    // Carregar slot vazio exibe status sem erro.
    await ui.carregarDoSlotAtual();
    assert.equal(elStatus.textContent, "Slot 2 vazio");
    assert.equal(elStatus.hidden, false);

    // Salvar via metodo direto e verificar rotulos + reset de timer.
    await ui.salvarNoSlotAtual();
    assert.equal(elStatus.textContent, "Slot 2 salvo (q. 120)");
    assert.equal(seletorSlot.options[1].textContent, "Slot 2 (2.0s)");
    assert.equal(seletorSlot.options[0].textContent, "Slot 1 (vazio)");
    assert.ok(clears >= 1);

    // Expirar status oculta o elemento.
    timeouts[timeouts.length - 1]();
    assert.equal(elStatus.hidden, true);

    // Carregar slot salvo chama onCarregarSave.
    await ui.carregarDoSlotAtual();
    assert.equal(carregados.length, 1);
    assert.deepEqual(carregados[0].rle, [[0x08, 120]]);

    // Exportar via clique no botao.
    btnExportar.click();
    await new Promise((r) => origSetTimeout(r, 0));
    assert.equal(linksCriados.length, 1);
    assert.equal(linksCriados[0].download, "websonic-slot2-q120.s1sav");
    assert.equal(linksCriados[0].clicado, true);
    assert.equal(linksCriados[0].removido, true);
    assert.equal(focos, 1);
    // Executa callback de revokeObjectURL.
    timeouts.forEach((fn) => fn());
    assert.deepEqual(urlsRevogadas, ["blob:websonic-save"]);

    // Cliques em btnSalvar e btnCarregar disparam onFocarTela.
    btnSalvar.click();
    btnCarregar.click();
    await new Promise((r) => origSetTimeout(r, 0));
    assert.equal(focos, 3);

    // Importar arquivo valido pelo fluxo de botao + input change.
    btnImportar.click();
    assert.equal(inputImportar.cliques, 1);
    const jsonValido = JSON.stringify({
      formato: "websonic-save",
      versao: 1,
      sha1: "sha1-sonic-teste",
      quadro: 60,
      salvoEm: 1700000000000,
      entradasRle: [[0x20, 60]],
    });
    inputImportar.files = [{ text: async () => jsonValido }];
    inputImportar.disparar("change");
    await new Promise((r) => origSetTimeout(r, 0));
    assert.equal(seletorSlot.options[1].textContent, "Slot 2 (1.0s)");
    assert.match(elStatus.textContent, /Importado no Slot 2/);

    // Importar arquivo de outra ROM dispara onErro.
    const jsonOutraRom = JSON.stringify({
      formato: "websonic-save",
      versao: 1,
      sha1: "outro-sha1",
      quadro: 10,
      salvoEm: 1700000000000,
      entradasRle: [[0, 10]],
    });
    inputImportar.files = [{ text: async () => jsonOutraRom }];
    inputImportar.disparar("change");
    await new Promise((r) => origSetTimeout(r, 0));
    assert.ok(erros.some((e) => /SHA-1 diferente/.test(e)));

    // Falha no worker ao salvar e exportar vai para onErro.
    falharWorker = true;
    await ui.salvarNoSlotAtual();
    btnExportar.click();
    await new Promise((r) => origSetTimeout(r, 0));
    assert.ok(erros.some((e) => /Falha ao salvar estado/.test(e)));
    assert.ok(erros.some((e) => /Falha ao exportar save/.test(e)));

    // Quando nao esta jogando, salvar e exportar sao no-op.
    jogando = false;
    const errosAntes = erros.length;
    await ui.salvarNoSlotAtual();
    btnExportar.click();
    await new Promise((r) => origSetTimeout(r, 0));
    assert.equal(erros.length, errosAntes);
  } finally {
    globalThis.setTimeout = origSetTimeout;
    globalThis.clearTimeout = origClearTimeout;
    globalThis.document = origDoc;
    URL.createObjectURL = origCreateObjUrl;
    URL.revokeObjectURL = origRevokeObjUrl;
  }
});

