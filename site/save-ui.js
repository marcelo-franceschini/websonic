// Controlador da interface de Save States (slots IndexedDB e exportacao/importacao .s1sav).
import {
  criarGerenciadorSaves,
  serializarSave,
  desserializarSave,
} from "./saves.js";

export function formatarRotuloSlot(slot, info) {
  if (!info || !Number.isFinite(info.quadro)) {
    return `Slot ${slot} (vazio)`;
  }
  return `Slot ${slot} (${(info.quadro / 60).toFixed(1)}s)`;
}

export function nomeArquivoSave(slot, quadro) {
  const s = Math.max(1, Number(slot) || 1);
  const q = Math.max(0, Math.floor(Number(quadro) || 0));
  return `websonic-slot${s}-q${q}.s1sav`;
}

export function criarControladorSavesUi({
  seletorSlot,
  btnSalvar,
  btnCarregar,
  btnExportar,
  btnImportar,
  inputImportar,
  elStatus,
  estaJogando,
  obterRomAtual,
  solicitarSaveWorker,
  onCarregarSave,
  onErro,
  onFocarTela,
  gerenciador = criarGerenciadorSaves(),
}) {
  let timerStatus = 0;

  function mostrarStatus(texto) {
    if (!elStatus) return;
    elStatus.textContent = texto;
    elStatus.hidden = false;
    if (timerStatus) clearTimeout(timerStatus);
    timerStatus = setTimeout(() => {
      elStatus.hidden = true;
    }, 3500);
  }

  function obterSlotSelecionado() {
    return Number(seletorSlot ? seletorSlot.value : 1) || 1;
  }

  async function atualizarRotulosSlots() {
    const rom = obterRomAtual();
    if (!seletorSlot || !rom || !rom.sha1) return;
    try {
      const mapa = await gerenciador.listarSlots(rom.sha1);
      for (const opt of seletorSlot.options) {
        const slot = Number(opt.value);
        opt.textContent = formatarRotuloSlot(slot, mapa[slot]);
      }
    } catch (err) {
      if (onErro) onErro("Falha ao listar slots salvos: " + (err?.message || String(err)));
    }
  }

  async function salvarNoSlotAtual() {
    const rom = obterRomAtual();
    if (!estaJogando() || !rom || !rom.sha1) return;
    try {
      const dados = await solicitarSaveWorker();
      const slot = obterSlotSelecionado();
      await gerenciador.salvarSlot(rom.sha1, slot, {
        quadro: dados.quadro,
        entradasRle: dados.entradasRle,
        salvoEm: Date.now(),
      });
      await atualizarRotulosSlots();
      mostrarStatus(`Slot ${slot} salvo (q. ${dados.quadro})`);
    } catch (err) {
      if (onErro) onErro("Falha ao salvar estado: " + (err?.message || String(err)));
    }
  }

  async function carregarDoSlotAtual() {
    const rom = obterRomAtual();
    if (!rom || !rom.bytes || !rom.sha1) return;
    try {
      const slot = obterSlotSelecionado();
      const salvo = await gerenciador.carregarSlot(rom.sha1, slot);
      if (!salvo) {
        mostrarStatus(`Slot ${slot} vazio`);
        return;
      }
      mostrarStatus(`Carregando Slot ${slot}...`);
      await onCarregarSave(rom.bytes, salvo.entradasRle);
    } catch (err) {
      if (onErro) onErro("Falha ao carregar estado: " + (err?.message || String(err)));
    }
  }

  async function exportarArquivo() {
    const rom = obterRomAtual();
    if (!estaJogando() || !rom || !rom.sha1) return;
    try {
      const dados = await solicitarSaveWorker();
      const slot = obterSlotSelecionado();
      const texto = serializarSave({
        sha1: rom.sha1,
        quadro: dados.quadro,
        entradasRle: dados.entradasRle,
        salvoEm: Date.now(),
      });
      const blob = new Blob([texto], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = nomeArquivoSave(slot, dados.quadro);
      document.body.appendChild(link);
      link.click();
      link.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      mostrarStatus(`Exportado .s1sav (q. ${dados.quadro})`);
    } catch (err) {
      if (onErro) onErro("Falha ao exportar save: " + (err?.message || String(err)));
    }
  }

  async function importarArquivo(arquivo) {
    const rom = obterRomAtual();
    if (!arquivo || !rom || !rom.bytes || !rom.sha1) return;
    try {
      const texto = await arquivo.text();
      const save = desserializarSave(texto);
      if (save.sha1 !== rom.sha1) {
        throw new Error("Este arquivo .s1sav pertence a outra ROM (SHA-1 diferente).");
      }
      const slot = obterSlotSelecionado();
      await gerenciador.salvarSlot(rom.sha1, slot, save);
      await atualizarRotulosSlots();
      mostrarStatus(`Importado no Slot ${slot} (q. ${save.quadro})`);
      await onCarregarSave(rom.bytes, save.entradasRle);
    } catch (err) {
      if (onErro) onErro("Falha ao importar save: " + (err?.message || String(err)));
    }
  }

  if (btnSalvar) {
    btnSalvar.addEventListener("click", () => {
      salvarNoSlotAtual();
      if (onFocarTela) onFocarTela();
    });
  }
  if (btnCarregar) {
    btnCarregar.addEventListener("click", () => {
      carregarDoSlotAtual();
      if (onFocarTela) onFocarTela();
    });
  }
  if (btnExportar) {
    btnExportar.addEventListener("click", () => {
      exportarArquivo();
      if (onFocarTela) onFocarTela();
    });
  }
  if (btnImportar && inputImportar) {
    btnImportar.addEventListener("click", () => {
      inputImportar.value = "";
      inputImportar.click();
    });
    inputImportar.addEventListener("change", () => {
      const arquivo = inputImportar.files?.[0];
      if (arquivo) importarArquivo(arquivo);
    });
  }

  return {
    atualizarRotulosSlots,
    salvarNoSlotAtual,
    carregarDoSlotAtual,
    mostrarStatus,
  };
}
