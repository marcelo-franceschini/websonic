// Cola mínima entre a página e o WebAssembly. A lógica fica em Rust (crate sonic1-rom).
import init, { identificar } from "./pkg/sonic1_web.js";

// Maior arquivo aceito para leitura. Uma ROM de Mega Drive tem no máximo 4 MiB.
// Acima disso nem lemos o arquivo: ele não pode ser a REV01.
const LIMITE_BYTES = 16 * 1024 * 1024;

const entrada = document.getElementById("arquivo");
const caixa = document.getElementById("resultado");

// Mostra o resultado. Usa textContent (nunca innerHTML), então o nome do arquivo
// ou qualquer texto nunca vira HTML.
function mostrar(valida, veredito, sha1, tamanho) {
  caixa.hidden = false;
  caixa.className = valida ? "ok" : "erro";
  document.getElementById("veredito").textContent = veredito;
  document.getElementById("sha1").textContent = sha1;
  document.getElementById("tamanho").textContent = String(tamanho);
}

await init();
entrada.disabled = false;

entrada.addEventListener("change", async () => {
  const arquivo = entrada.files[0];
  if (!arquivo) return;
  if (arquivo.size > LIMITE_BYTES) {
    mostrar(false, "Recusada: arquivo grande demais para ser a REV01.", "(não calculado)", arquivo.size);
    return;
  }
  const bytes = new Uint8Array(await arquivo.arrayBuffer());
  const r = identificar(bytes);
  mostrar(r.valida, r.valida ? "ROM REV01 válida." : "Recusada: " + r.motivo + ".", r.sha1, r.tamanho);
  r.free(); // libera o objeto do lado do WebAssembly
});
