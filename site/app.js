// Cola mínima entre a página e o WebAssembly. A lógica fica em Rust (crate sonic1-rom).
import init, { identificar, Jogo } from "./pkg/sonic1_web.js";

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
  const valida = r.valida;
  r.free(); // libera o objeto do lado do WebAssembly
  if (valida) iniciar(bytes);
});

// ---- Jogo ----
// Tudo o que importa roda no WebAssembly. Aqui só: desenhar o quadro, ler o teclado e marcar o tempo.

const tela = document.getElementById("tela");
const ctx = tela.getContext("2d");
const imagem = ctx.createImageData(320, 224);
let jogo = null;
let apertadas = 0; // bits: cima, baixo, esquerda, direita, B, C, A, Start
let ultimo = 0;
let acumulado = 0;
const PASSO = 1000 / 60; // a lógica roda sempre a 60 Hz

let audioCtx = null;
let workletNode = null;

async function iniciarAudio() {
  if (audioCtx) return;
  try {
    audioCtx = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: 48000 });
    await audioCtx.audioWorklet.addModule("audio-worklet.js");
    workletNode = new AudioWorkletNode(audioCtx, "sonic-audio-processor");
    workletNode.connect(audioCtx.destination);
    const retomar = () => {
      if (audioCtx && audioCtx.state === "suspended") audioCtx.resume();
      window.removeEventListener("click", retomar);
      window.removeEventListener("keydown", retomar);
    };
    window.addEventListener("click", retomar);
    window.addEventListener("keydown", retomar);
    if (audioCtx.state === "suspended") audioCtx.resume();
  } catch (e) {
    console.warn("Falha ao inicializar AudioWorklet:", e);
  }
}

const TECLAS = {
  ArrowUp: 0x01, ArrowDown: 0x02, ArrowLeft: 0x04, ArrowRight: 0x08,
  x: 0x10, c: 0x20, z: 0x40, Enter: 0x80,
};
window.addEventListener("keydown", (e) => { const b = TECLAS[e.key]; if (b) { apertadas |= b; e.preventDefault(); } });
window.addEventListener("keyup", (e) => { const b = TECLAS[e.key]; if (b) { apertadas &= ~b; e.preventDefault(); } });

// Escala inteira: o maior fator que cabe na janela, no mínimo 1.
function ajustarEscala() {
  const f = Math.max(1, Math.floor(Math.min((window.innerWidth - 32) / 320, (window.innerHeight * 0.8) / 224)));
  tela.style.width = 320 * f + "px";
  tela.style.height = 224 * f + "px";
}
window.addEventListener("resize", ajustarEscala);

function laco(agora) {
  if (!jogo) return;
  acumulado += Math.min(agora - ultimo, 100); // não acumula atraso demais
  ultimo = agora;
  if (acumulado >= PASSO) { // no máximo um quadro por chamada
    acumulado -= PASSO;
    jogo.botoes(apertadas);
    imagem.data.set(jogo.quadro());
    ctx.putImageData(imagem, 0, 0);

    const audio = jogo.audio();
    if (workletNode && audio.length > 0) {
      const f32 = new Float32Array(audio);
      workletNode.port.postMessage(f32.buffer, [f32.buffer]);
    }

    const erro = jogo.erro();
    if (erro) {
      const p = document.getElementById("erro-jogo");
      p.hidden = false;
      p.textContent = "O núcleo parou: " + erro;
    }
  }
  requestAnimationFrame(laco);
}

// Chamado depois de a ROM ser aceita: cria o jogo com os mesmos bytes (que não saem da máquina).
async function iniciar(bytes) {
  try {
    jogo = Jogo.criar(bytes);
  } catch (e) {
    jogo = null;
    return;
  }
  await iniciarAudio();
  tela.hidden = false;
  document.getElementById("aviso").hidden = false;
  ajustarEscala();
  ultimo = performance.now();
  requestAnimationFrame(laco);
}
