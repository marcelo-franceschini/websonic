// Testes unitarios do modulo de Save States deterministicos (RLE, serializacao .s1sav e slots).
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  FORMATO_SAVE,
  VERSAO_SAVE,
  codificarEntradasRle,
  decodificarEntradasRle,
  serializarSave,
  desserializarSave,
  criarGerenciadorSaves,
} from "../saves.js";

test("codificarEntradasRle e decodificarEntradasRle preservam sequencias variadas", () => {
  assert.deepEqual(codificarEntradasRle(new Uint8Array(0)), []);
  assert.deepEqual(decodificarEntradasRle([]), new Uint8Array(0));

  const original = new Uint8Array([0, 0, 0, 0x08, 0x08, 0x28, 0x20, 0x20, 0x20, 0x20]);
  const rle = codificarEntradasRle(original);
  assert.deepEqual(rle, [
    [0x00, 3],
    [0x08, 2],
    [0x28, 1],
    [0x20, 4],
  ]);

  const restaurado = decodificarEntradasRle(rle);
  assert.deepEqual(restaurado, original);
});

test("decodificarEntradasRle rejeita payloads corrompidos ou acima do limite", () => {
  assert.deepEqual(codificarEntradasRle(null), []);
  assert.throws(() => decodificarEntradasRle("invalido"), /inválido/i);
  assert.throws(() => decodificarEntradasRle([[1]]), /corrompida/i);
  assert.throws(() => decodificarEntradasRle([[256, 1]]), /inválidos/i);
  assert.throws(() => decodificarEntradasRle([[0, 0]]), /inválidos/i);
  assert.throws(() => decodificarEntradasRle([[0, 100]], 50), /limite/i);
});

test("serializarSave e desserializarSave fazem ida e volta fiel e validam integridade", () => {
  const entradas = new Uint8Array([0, 0, 0x80, 0x08, 0x08, 0x08]);
  const rle = codificarEntradasRle(entradas);
  const texto = serializarSave({
    sha1: "6ddb7de1e17e7f6cdb88927bd906352030daa194",
    quadro: entradas.length,
    entradasRle: rle,
    salvoEm: 1700000000000,
  });

  const lido = desserializarSave(texto);
  assert.equal(lido.formato, FORMATO_SAVE);
  assert.equal(lido.versao, VERSAO_SAVE);
  assert.equal(lido.sha1, "6ddb7de1e17e7f6cdb88927bd906352030daa194");
  assert.equal(lido.quadro, 6);
  assert.equal(lido.salvoEm, 1700000000000);
  assert.deepEqual(decodificarEntradasRle(lido.entradasRle), entradas);

  // serializarSave com entradasRle ausente usa [] e valida campos obrigatorios.
  const vazio = serializarSave({ sha1: "abc", quadro: 0, entradasRle: null });
  const lidoVazio = desserializarSave(JSON.parse(vazio));
  assert.equal(lidoVazio.quadro, 0);
  assert.throws(() => serializarSave({ sha1: "", quadro: 0, entradasRle: [] }), /SHA-1/i);
  assert.throws(() => serializarSave({ sha1: "abc", quadro: -1, entradasRle: [] }), /quadro/i);

  // desserializarSave sem salvoEm numerico gera timestamp atual.
  const semData = desserializarSave({
    formato: FORMATO_SAVE,
    versao: VERSAO_SAVE,
    sha1: "abc",
    quadro: 2,
    entradasRle: [[0, 2]],
  });
  assert.equal(typeof semData.salvoEm, "number");

  // Validacoes de erro em desserializarSave.
  assert.throws(() => desserializarSave(null), /inválido/i);
  assert.throws(() => desserializarSave("{}"), /desconhecido/i);
  assert.throws(
    () => desserializarSave({ formato: FORMATO_SAVE, versao: 99, sha1: "abc", quadro: 0, entradasRle: [] }),
    /Versão/i,
  );
  assert.throws(
    () => desserializarSave({ formato: FORMATO_SAVE, versao: VERSAO_SAVE, sha1: "", quadro: 0, entradasRle: [] }),
    /SHA-1/i,
  );
  assert.throws(
    () => desserializarSave({ formato: FORMATO_SAVE, versao: VERSAO_SAVE, sha1: "abc", quadro: -2, entradasRle: [] }),
    /quadros/i,
  );

  // Cabecalho inconsistente com a soma do RLE deve falhar.
  const quebrado = JSON.stringify({
    formato: FORMATO_SAVE,
    versao: VERSAO_SAVE,
    sha1: "abc",
    quadro: 10,
    entradasRle: [[0, 3]],
  });
  assert.throws(() => desserializarSave(quebrado), /inconsistente/i);
  assert.throws(() => desserializarSave("nao-eh-json"), /JSON válido/i);
});

test("criarGerenciadorSaves funciona com fallback em memoria em ambiente Node", async () => {
  const mgr = criarGerenciadorSaves();
  const sha1 = "6ddb7de1e17e7f6cdb88927bd906352030daa194";

  assert.equal(await mgr.carregarSlot(sha1, 1), null);

  await mgr.salvarSlot(sha1, 1, {
    quadro: 120,
    entradasRle: [[0, 100], [0x08, 20]],
  });

  const carregado = await mgr.carregarSlot(sha1, 1);
  assert.ok(carregado);
  assert.equal(carregado.quadro, 120);
  assert.deepEqual(carregado.entradasRle, [[0, 100], [0x08, 20]]);

  const slots = await mgr.listarSlots(sha1);
  assert.equal(slots[1].quadro, 120);
  assert.equal(slots[2], undefined);

  await mgr.removerSlot(sha1, 1);
  assert.equal(await mgr.carregarSlot(sha1, 1), null);
});

function criarIdbFalso({ falharAbrir = false, falharPut = false, falharGet = false, falharDelete = false, jaTemStore = false } = {}) {
  const tabela = new Map();
  let storesCriadas = 0;

  const db = {
    objectStoreNames: {
      contains: () => jaTemStore || storesCriadas > 0,
    },
    createObjectStore() {
      storesCriadas++;
    },
    transaction(_store, _mode) {
      const tx = {
        oncomplete: null,
        onerror: null,
        error: new Error("tx-err"),
        objectStore() {
          return {
            put(registro) {
              queueMicrotask(() => {
                if (falharPut) {
                  if (tx.onerror) tx.onerror();
                } else {
                  tabela.set(registro.chave, registro);
                  if (tx.oncomplete) tx.oncomplete();
                }
              });
            },
            get(chave) {
              const req = { result: null, onsuccess: null, onerror: null };
              queueMicrotask(() => {
                if (falharGet) {
                  if (req.onerror) req.onerror();
                } else {
                  req.result = tabela.get(chave);
                  if (req.onsuccess) req.onsuccess();
                }
              });
              return req;
            },
            delete(chave) {
              queueMicrotask(() => {
                if (falharDelete) {
                  if (tx.onerror) tx.onerror();
                } else {
                  tabela.delete(chave);
                  if (tx.oncomplete) tx.oncomplete();
                }
              });
            },
          };
        },
      };
      return tx;
    },
  };

  return {
    tabela,
    get storesCriadas() {
      return storesCriadas;
    },
    idb: {
      open() {
        const req = { result: db, error: null, onupgradeneeded: null, onsuccess: null, onerror: null };
        queueMicrotask(() => {
          if (falharAbrir) {
            if (req.onerror) req.onerror();
            return;
          }
          if (req.onupgradeneeded) req.onupgradeneeded();
          if (req.onsuccess) req.onsuccess();
        });
        return req;
      },
    },
  };
}

test("criarGerenciadorSaves persiste, carrega, lista e remove slots via IndexedDB injetado", async () => {
  const falso = criarIdbFalso();
  const mgr = criarGerenciadorSaves({ idb: falso.idb });
  const sha1 = "rom-sha1-idb";

  // Carregamento de slot inexistente consulta IDB e retorna null.
  assert.equal(await mgr.carregarSlot(sha1, 2), null);
  assert.equal(falso.storesCriadas, 1);

  await mgr.salvarSlot(sha1, 2, {
    quadro: 300,
    entradasRle: [[0x08, 300]],
    salvoEm: 1700000002000,
  });
  assert.ok(falso.tabela.has(`${sha1}:2`));

  // Novo gerenciador sobre o mesmo IDB le diretamente do banco (sem cache em memoria).
  const falso2 = criarIdbFalso({ jaTemStore: true });
  falso2.tabela.set(`${sha1}:2`, falso.tabela.get(`${sha1}:2`));
  const mgrLeitor = criarGerenciadorSaves({ idb: falso2.idb });
  const lido = await mgrLeitor.carregarSlot(sha1, 2);
  assert.equal(lido.quadro, 300);
  assert.equal(falso2.storesCriadas, 0);

  const lista = await mgrLeitor.listarSlots(sha1);
  assert.equal(lista[2].quadro, 300);

  await mgrLeitor.removerSlot(sha1, 2);
  assert.equal(falso2.tabela.has(`${sha1}:2`), false);
});

test("criarGerenciadorSaves tolera erros de abertura e transacao no IndexedDB", async () => {
  const errAbrir = criarIdbFalso({ falharAbrir: true });
  const mgr1 = criarGerenciadorSaves({ idb: errAbrir.idb });
  await mgr1.salvarSlot("sha", 1, { quadro: 10, entradasRle: [[0, 10]], salvoEm: 1 });
  assert.equal((await mgr1.carregarSlot("sha", 1)).quadro, 10);

  const errTx = criarIdbFalso({ falharPut: true, falharGet: true, falharDelete: true });
  const mgr2 = criarGerenciadorSaves({ idb: errTx.idb });
  await mgr2.salvarSlot("sha", 1, { quadro: 20, entradasRle: [[0, 20]], salvoEm: 2 });
  // Mesmo com falha no get do IDB, fallback em memoria preserva o registro.
  assert.equal((await mgr2.carregarSlot("sha", 1)).quadro, 20);
  await mgr2.removerSlot("sha", 1);
  assert.equal(await mgr2.carregarSlot("sha", 1), null);
});

