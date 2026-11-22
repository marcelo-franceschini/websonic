import test from "node:test";
import assert from "node:assert/strict";
import {
  CHAVE_STORAGE_VISUAL,
  PREFERENCIAS_VISUAIS_PADRAO,
  normalizarPreferenciasVisuais,
  carregarPreferenciasVisuais,
  salvarPreferenciasVisuais,
} from "../preferencias.js";

function criarStorageMemoria() {
  const mapa = new Map();
  return {
    getItem(k) {
      return mapa.has(k) ? mapa.get(k) : null;
    },
    setItem(k, v) {
      mapa.set(k, String(v));
    },
  };
}

test("normalizarPreferenciasVisuais valida opcoes permitidas e aplica padroes", () => {
  assert.deepEqual(normalizarPreferenciasVisuais(null), PREFERENCIAS_VISUAIS_PADRAO);
  assert.deepEqual(
    normalizarPreferenciasVisuais({ fps: "desbloqueado", crt: "ligado", cram: "limpo", proporcao: "21:9" }),
    { fps: "desbloqueado", crt: "ligado", cram: "limpo", proporcao: "21:9" },
  );
  assert.deepEqual(
    normalizarPreferenciasVisuais({ fps: "120", crt: "invalido", cram: "limpo", proporcao: "99:9" }),
    { fps: "60", crt: "desligado", cram: "limpo", proporcao: "auto" },
  );
});

test("carregarPreferenciasVisuais e salvarPreferenciasVisuais fazem ida e volta segura", () => {
  const storage = criarStorageMemoria();
  assert.deepEqual(carregarPreferenciasVisuais(storage), PREFERENCIAS_VISUAIS_PADRAO);

  const ok = salvarPreferenciasVisuais(storage, {
    fps: "desbloqueado",
    crt: "ligado",
    cram: "limpo",
    proporcao: "32:9",
  });
  assert.equal(ok, true);
  assert.deepEqual(carregarPreferenciasVisuais(storage), {
    fps: "desbloqueado",
    crt: "ligado",
    cram: "limpo",
    proporcao: "32:9",
  });

  storage.setItem(CHAVE_STORAGE_VISUAL, "{corrompido");
  assert.deepEqual(carregarPreferenciasVisuais(storage), PREFERENCIAS_VISUAIS_PADRAO);

  const bloqueado = {
    getItem() {
      throw new Error("blocked");
    },
    setItem() {
      throw new Error("quota");
    },
  };
  assert.deepEqual(carregarPreferenciasVisuais(null), PREFERENCIAS_VISUAIS_PADRAO);
  assert.deepEqual(carregarPreferenciasVisuais({}), PREFERENCIAS_VISUAIS_PADRAO);
  assert.equal(salvarPreferenciasVisuais(null, PREFERENCIAS_VISUAIS_PADRAO), false);
  assert.equal(salvarPreferenciasVisuais({}, PREFERENCIAS_VISUAIS_PADRAO), false);

  assert.deepEqual(carregarPreferenciasVisuais(bloqueado), PREFERENCIAS_VISUAIS_PADRAO);
  assert.equal(salvarPreferenciasVisuais(bloqueado, PREFERENCIAS_VISUAIS_PADRAO), false);
});
