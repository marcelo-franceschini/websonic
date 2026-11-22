// Persistencia pura das preferencias visuais (FPS, filtro CRT, proporcao e pontos de CRAM).

export const CHAVE_STORAGE_VISUAL = "websonic:visual:v1";

export const PREFERENCIAS_VISUAIS_PADRAO = Object.freeze({
  fps: "60",
  crt: "desligado",
  cram: "hardware",
  proporcao: "auto",
});

const OPCOES_FPS = new Set(["60", "desbloqueado"]);
const OPCOES_CRT = new Set(["desligado", "ligado"]);
const OPCOES_CRAM = new Set(["hardware", "limpo"]);
const OPCOES_PROPORCAO = new Set(["auto", "4:3", "16:9", "21:9", "32:9"]);

/**
 * Normaliza preferencias visuais garantindo apenas valores reconhecidos pela UI.
 */
export function normalizarPreferenciasVisuais(dados) {
  const obj = dados && typeof dados === "object" ? dados : {};
  return {
    fps: OPCOES_FPS.has(obj.fps) ? obj.fps : PREFERENCIAS_VISUAIS_PADRAO.fps,
    crt: OPCOES_CRT.has(obj.crt) ? obj.crt : PREFERENCIAS_VISUAIS_PADRAO.crt,
    cram: OPCOES_CRAM.has(obj.cram) ? obj.cram : PREFERENCIAS_VISUAIS_PADRAO.cram,
    proporcao: OPCOES_PROPORCAO.has(obj.proporcao)
      ? obj.proporcao
      : PREFERENCIAS_VISUAIS_PADRAO.proporcao,
  };
}

/**
 * Carrega preferencias visuais do storage de forma resiliente.
 */
export function carregarPreferenciasVisuais(storage) {
  if (!storage || typeof storage.getItem !== "function") {
    return { ...PREFERENCIAS_VISUAIS_PADRAO };
  }
  try {
    const bruto = storage.getItem(CHAVE_STORAGE_VISUAL);
    if (!bruto) return { ...PREFERENCIAS_VISUAIS_PADRAO };
    return normalizarPreferenciasVisuais(JSON.parse(bruto));
  } catch {
    return { ...PREFERENCIAS_VISUAIS_PADRAO };
  }
}

/**
 * Persiste preferencias visuais normalizadas no storage fornecido.
 */
export function salvarPreferenciasVisuais(storage, prefs) {
  if (!storage || typeof storage.setItem !== "function") return false;
  try {
    const payload = normalizarPreferenciasVisuais(prefs);
    storage.setItem(CHAVE_STORAGE_VISUAL, JSON.stringify(payload));
    return true;
  } catch {
    return false;
  }
}
