// Save states deterministicos por registro de entrada RLE + persistencia em IndexedDB por SHA-1 da ROM.

export const FORMATO_SAVE = "websonic-save";
export const VERSAO_SAVE = 1;
export const SLOTS_PADRAO = [1, 2, 3];
const LIMITE_QUADROS_SAVE = 60 * 60 * 10; // 10 horas a 60 Hz (2.160.000 quadros)

const DB_NOME = "websonic-saves-v1";
const DB_VERSAO = 1;
const STORE_SLOTS = "slots";

/**
 * Codifica um vetor de mascaras de 8 bits (1 byte por quadro) usando RLE:
 * retorna lista de pares [mascara, quantidade].
 */
export function codificarEntradasRle(quadros) {
  const n = quadros ? quadros.length : 0;
  if (n === 0) return [];

  const saida = [];
  let atual = quadros[0] & 0xFF;
  let contagem = 1;

  for (let i = 1; i < n; i++) {
    const valor = quadros[i] & 0xFF;
    if (valor === atual) {
      contagem++;
    } else {
      saida.push([atual, contagem]);
      atual = valor;
      contagem = 1;
    }
  }
  saida.push([atual, contagem]);
  return saida;
}

/**
 * Decodifica pares [mascara, quantidade] de volta em um Uint8Array de mascaras por quadro.
 */
export function decodificarEntradasRle(rle, limiteQuadros = LIMITE_QUADROS_SAVE) {
  if (!Array.isArray(rle)) {
    throw new Error("Formato de entradas RLE inválido.");
  }

  let total = 0;
  for (let i = 0; i < rle.length; i++) {
    const par = rle[i];
    if (!Array.isArray(par) || par.length !== 2) {
      throw new Error("Entrada RLE corrompida.");
    }
    const mascara = par[0];
    const contagem = par[1];
    if (
      !Number.isInteger(mascara) ||
      mascara < 0 ||
      mascara > 255 ||
      !Number.isInteger(contagem) ||
      contagem <= 0
    ) {
      throw new Error("Valores inválidos na entrada RLE.");
    }
    total += contagem;
    if (total > limiteQuadros) {
      throw new Error("Arquivo de save excede o limite de quadros permitido.");
    }
  }

  const saida = new Uint8Array(total);
  let pos = 0;
  for (let i = 0; i < rle.length; i++) {
    const [mascara, contagem] = rle[i];
    saida.fill(mascara, pos, pos + contagem);
    pos += contagem;
  }
  return saida;
}

/**
 * Serializa um save state deterministico para JSON (.s1sav).
 */
export function serializarSave({ sha1, quadro, entradasRle, salvoEm = Date.now() }) {
  if (typeof sha1 !== "string" || sha1.length === 0) {
    throw new Error("SHA-1 da ROM é obrigatório para salvar.");
  }
  if (!Number.isInteger(quadro) || quadro < 0) {
    throw new Error("Número de quadro inválido.");
  }
  const rleValido = Array.isArray(entradasRle) ? entradasRle : [];
  return JSON.stringify(
    {
      formato: FORMATO_SAVE,
      versao: VERSAO_SAVE,
      sha1,
      quadro,
      salvoEm,
      entradasRle: rleValido,
    },
    null,
    2,
  );
}

/**
 * Valida e desserializa um texto JSON (.s1sav).
 */
export function desserializarSave(texto) {
  let dados;
  try {
    dados = typeof texto === "string" ? JSON.parse(texto) : texto;
  } catch {
    throw new Error("Arquivo de save não é um JSON válido.");
  }

  if (!dados || typeof dados !== "object") {
    throw new Error("Conteúdo de save inválido.");
  }
  if (dados.formato !== FORMATO_SAVE) {
    throw new Error("Formato de arquivo desconhecido (esperado websonic-save).");
  }
  if (dados.versao !== VERSAO_SAVE) {
    throw new Error(`Versão de save não suportada: ${dados.versao}.`);
  }
  if (typeof dados.sha1 !== "string" || dados.sha1.length === 0) {
    throw new Error("Save sem assinatura SHA-1 da ROM.");
  }
  if (!Number.isInteger(dados.quadro) || dados.quadro < 0) {
    throw new Error("Número de quadros inválido no save.");
  }

  // Valida integridade do RLE e confere se o total de quadros bate com o cabecalho.
  const decodificado = decodificarEntradasRle(dados.entradasRle);
  if (decodificado.length !== dados.quadro) {
    throw new Error(
      `Save inconsistente: cabeçalho indica ${dados.quadro} quadros, mas o log contém ${decodificado.length}.`,
    );
  }

  return {
    formato: FORMATO_SAVE,
    versao: VERSAO_SAVE,
    sha1: dados.sha1,
    quadro: dados.quadro,
    salvoEm: typeof dados.salvoEm === "number" ? dados.salvoEm : Date.now(),
    entradasRle: dados.entradasRle,
  };
}

function abrirBancoIdb(idb) {
  return new Promise((resolve, reject) => {
    const fabrica = idb !== undefined ? idb : (typeof indexedDB !== "undefined" ? indexedDB : null);
    if (!fabrica || typeof fabrica.open !== "function") {
      reject(new Error("IndexedDB indisponível"));
      return;
    }
    const req = fabrica.open(DB_NOME, DB_VERSAO);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_SLOTS)) {
        db.createObjectStore(STORE_SLOTS, { keyPath: "chave" });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error || new Error("Erro ao abrir IndexedDB"));
  });
}

/**
 * Gerenciador de 3 slots por SHA-1 da ROM usando IndexedDB com fallback em memoria.
 */
export function criarGerenciadorSaves({ idb } = {}) {
  const memoria = new Map();
  let promessaDb = null;

  function obterDb() {
    if (!promessaDb) {
      promessaDb = abrirBancoIdb(idb).catch(() => null);
    }
    return promessaDb;
  }

  function chaveSlot(sha1, slot) {
    return `${sha1}:${slot}`;
  }

  async function salvarSlot(sha1, slot, payload) {
    const registro = {
      chave: chaveSlot(sha1, slot),
      sha1,
      slot,
      quadro: payload.quadro,
      salvoEm: payload.salvoEm || Date.now(),
      entradasRle: payload.entradasRle,
      formato: FORMATO_SAVE,
      versao: VERSAO_SAVE,
    };
    memoria.set(registro.chave, registro);

    const db = await obterDb();
    if (!db) return registro;

    await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_SLOTS, "readwrite");
      tx.objectStore(STORE_SLOTS).put(registro);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error || new Error("Falha ao gravar slot"));
    }).catch(() => {});
    return registro;
  }

  async function carregarSlot(sha1, slot) {
    const chave = chaveSlot(sha1, slot);
    const db = await obterDb();
    if (!db) return memoria.get(chave) || null;

    const encontrado = await new Promise((resolve) => {
      const tx = db.transaction(STORE_SLOTS, "readonly");
      const req = tx.objectStore(STORE_SLOTS).get(chave);
      req.onsuccess = () => resolve(req.result || null);
      req.onerror = () => resolve(null);
    });
    if (encontrado) {
      memoria.set(chave, encontrado);
      return encontrado;
    }
    return memoria.get(chave) || null;
  }

  async function listarSlots(sha1) {
    const resultados = {};
    for (const slot of SLOTS_PADRAO) {
      const item = await carregarSlot(sha1, slot);
      if (item) {
        resultados[slot] = {
          slot,
          quadro: item.quadro,
          salvoEm: item.salvoEm,
        };
      }
    }
    return resultados;
  }

  async function removerSlot(sha1, slot) {
    const chave = chaveSlot(sha1, slot);
    memoria.delete(chave);
    const db = await obterDb();
    if (!db) return;
    await new Promise((resolve) => {
      const tx = db.transaction(STORE_SLOTS, "readwrite");
      tx.objectStore(STORE_SLOTS).delete(chave);
      tx.oncomplete = () => resolve();
      tx.onerror = () => resolve();
    });
  }

  return {
    salvarSlot,
    carregarSlot,
    listarSlots,
    removerSlot,
  };
}
