// Teste local: a ROM é servida somente na interface loopback pelo comando Rust.
import init, { Jogo } from "/pkg/sonic1_web.js";
import { criarRenderizadorGPU } from "/gpu.js";

const texto = document.getElementById("resultado");
const opcoes = new URLSearchParams(location.search);
const total = Number(opcoes.get("quadros"));
const passo = Number(opcoes.get("passo"));
function p95(dados) {
  dados.sort((a, b) => a - b);
  return { mediana: dados[Math.floor(dados.length / 2)], p95: dados[Math.min(dados.length - 1, Math.floor(dados.length * 0.95))] };
}
function expandir(c) { return (c << 5) | (c << 2) | (c >> 1); }
function diferencas(cpu, gpu) {
  let total = 0;
  for (let y = 0; y < 224; y++) for (let x = 0; x < 320; x++) {
    const a = (y * 320 + x) * 4;
    const b = ((223 - y) * 320 + x) * 4;
    if (cpu[a] !== gpu[b] || cpu[a + 1] !== gpu[b + 1] || cpu[a + 2] !== gpu[b + 2]) total++;
  }
  return total;
}
function testarCandidatos(gpu) {
  const camadas = new Uint8Array(320 * 224 * 4);
  const sprites = new Uint8Array(320 * 224 * 20 * 2);
  const fundos = new Uint8Array(320 * 224);
  const seletores = new Uint16Array(320 * 224);
  const paletas = new Uint16Array(64);
  for (let i = 0; i < 64; i++) paletas[i] = i;
  for (let i = 0; i < 8; i++) {
    camadas.set([0x10, 0x80 | ((i & 2) ? 0x40 : 0), 0x20, 0x80 | ((i & 1) ? 0x40 : 0)], i * 4);
    sprites[i * 2] = 0x30;
    sprites[i * 2 + 1] = 0x80 | ((i & 4) ? 0x40 : 0);
  }
  // O primeiro sprite é transparente; o segundo sobreposto deve vencer.
  sprites[8 * 2] = 0;
  sprites[(320 + 8) * 2] = 0x40;
  sprites[(320 + 8) * 2 + 1] = 0x80;
  seletores[9] = 0x8000 | 0x1ff;
  gpu.renderizar(camadas, sprites, fundos, seletores, paletas);
  const pix = gpu.lerPixels();
  for (let i = 0; i < 10; i++) {
    let cor;
    if (i === 9) cor = 0x1ff;
    else if (i === 8) cor = 4;
    else {
      const candidatos = [{ cor: 3, p: Boolean(i & 4) }, { cor: 1, p: Boolean(i & 2) }, { cor: 2, p: Boolean(i & 1) }];
      cor = (candidatos.find(c => c.p) || candidatos[0]).cor;
    }
    const pos = (223 * 320 + i) * 4;
    if (pix[pos] !== expandir(cor & 7) || pix[pos + 1] !== expandir((cor >> 3) & 7) || pix[pos + 2] !== expandir((cor >> 6) & 7)) {
      throw new Error(`Cena sintética divergente no ponto ${i}`);
    }
  }
  const tempos = [];
  for (let i = 0; i < 10; i++) {
    const inicio = performance.now();
    gpu.renderizar(camadas, sprites, fundos, seletores, paletas);
    gpu.sincronizar();
    tempos.push(performance.now() - inicio);
  }
  return { bytes_por_quadro: camadas.byteLength + sprites.byteLength + fundos.byteLength + seletores.byteLength + 4096,
    upload_gpu_sincrono_ms: p95(tempos) };
}

try {
  await init();
  const rom = new Uint8Array(await (await fetch("/rom")).arrayBuffer());
  const cpu = Jogo.criar_com_audio(rom, 48000);
  const gpuJogo = Jogo.criar_com_audio(rom, 48000);
  gpuJogo.ativar_gpu_sem_referencia();
  const tela = document.createElement("canvas");
  tela.width = 320; tela.height = 224;
  const gpu = criarRenderizadorGPU(tela);
  if (!gpu) throw new Error("WebGL2 indisponível");
  const piorCasoSintetico = testarCandidatos(gpu);
  const simulacao = [], desenho = [];
  let comparados = 0, bytes = 0;
  for (let i = 0; i < total; i++) {
    const referencia = cpu.quadro();
    cpu.audio();
    const inicioSim = performance.now();
    gpuJogo.simular_quadro();
    gpuJogo.audio();
    simulacao.push(performance.now() - inicioSim);
    const erroCpu = cpu.erro(), erroGpu = gpuJogo.erro();
    if (erroCpu || erroGpu) throw new Error(`Núcleo no quadro ${i}: ${erroCpu || erroGpu}`);
    if (i % passo !== 0 && i + 1 !== total) continue;
    const camadas = gpuJogo.camadas_gpu(), sprites = gpuJogo.sprites_gpu(), fundos = gpuJogo.fundos_gpu();
    const seletores = gpuJogo.seletores_gpu(), paletas = gpuJogo.paletas_gpu();
    bytes = camadas.byteLength + sprites.byteLength + fundos.byteLength + seletores.byteLength + Math.ceil(paletas.length / 2048) * 4096;
    const inicioGpu = performance.now();
    gpu.renderizar(camadas, sprites, fundos, seletores, paletas);
    gpu.sincronizar();
    desenho.push(performance.now() - inicioGpu);
    const dif = diferencas(referencia, gpu.lerPixels());
    if (dif !== 0) throw new Error(`Quadro ${i}: ${dif} pixels diferentes`);
    comparados++;
  }
  cpu.free(); gpuJogo.free();
  texto.textContent = `GPU_TESTE_PASSOU ${JSON.stringify({ renderer: gpu.renderer, quadros: total, comparados, bytes_por_quadro: bytes, simulacao_ms: p95(simulacao), upload_gpu_sincrono_ms: p95(desenho), pior_caso_20_sprites: piorCasoSintetico })}`;
} catch (erro) {
  texto.textContent = `GPU_TESTE_FALHOU ${erro.message}`;
} finally {
  await fetch("/terminou").catch(() => {});
}
