// Apresenta a imagem de 9 bits do renderizador fiel (CPU). A GPU so converte a cor e amplia
// (docs/requisitos.md, Video fiel em software).
export const LARGURA = 320;
export const LARGURA_MAX = 640;
export const ALTURA = 224;

const VERTICE = `#version 300 es
void main() {
  vec2 p = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

// Cor de 9 bits (BBBGGGRRR) para 8 bits por canal, replicando os bits altos
// como Ponte::fechar_quadro (crates/console/src/lib.rs).
const FRAGMENTO = `#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;
uniform usampler2D imagem;
out vec4 saida;
void main() {
  ivec2 p = ivec2(int(gl_FragCoord.x), ${ALTURA - 1} - int(gl_FragCoord.y));
  uint cor = texelFetch(imagem, p, 0).r;
  uvec3 rgb = uvec3(cor & 7u, (cor >> 3u) & 7u, (cor >> 6u) & 7u);
  rgb = (rgb << 5u) | (rgb << 2u) | (rgb >> 1u);
  saida = vec4(vec3(rgb) * (1.0 / 255.0), 1.0);
}`;

function normalizarLarguraGpu(largura) {
  const n = Number.isFinite(largura) ? Math.floor(largura) : LARGURA;
  return Math.max(LARGURA, Math.min(LARGURA_MAX, n)) & ~1;
}

export function criarRenderizadorGPU(canvas, larguraInicial = LARGURA) {
  const gl = canvas.getContext("webgl2", {
    alpha: false,
    antialias: false,
    depth: false,
    stencil: false,
    preserveDrawingBuffer: false,
    desynchronized: true,
    powerPreference: "high-performance",
  });
  if (!gl) return null;
  const compilarShader = (tipo, codigo) => {
    const s = gl.createShader(tipo);
    gl.shaderSource(s, codigo);
    gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
      const log = gl.getShaderInfoLog(s);
      gl.deleteShader(s);
      throw new Error(log);
    }
    return s;
  };
  const vs = compilarShader(gl.VERTEX_SHADER, VERTICE);
  let fs;
  try {
    fs = compilarShader(gl.FRAGMENT_SHADER, FRAGMENTO);
  } catch (err) {
    gl.deleteShader(vs);
    throw err;
  }
  const programa = gl.createProgram();
  gl.attachShader(programa, vs);
  gl.attachShader(programa, fs);
  gl.linkProgram(programa);
  // Shaders intermediarios podem ser descartados logo apos o link do programa.
  gl.detachShader(programa, vs);
  gl.detachShader(programa, fs);
  gl.deleteShader(vs);
  gl.deleteShader(fs);
  if (!gl.getProgramParameter(programa, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(programa);
    gl.deleteProgram(programa);
    throw new Error(log);
  }
  gl.useProgram(programa);
  const vao = gl.createVertexArray();
  gl.bindVertexArray(vao);
  gl.disable(gl.BLEND);
  gl.disable(gl.DITHER);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 2);
  gl.uniform1i(gl.getUniformLocation(programa, "imagem"), 0);

  let larguraAtual = normalizarLarguraGpu(larguraInicial);
  let textura = null;

  function alocarTextura(largura) {
    if (textura) {
      gl.deleteTexture(textura);
    }
    // Textura imutavel alocada por dimensao: cada quadro so atualiza via texSubImage2D.
    textura = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, textura);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texStorage2D(gl.TEXTURE_2D, 1, gl.R16UI, largura, ALTURA);
    canvas.width = largura;
    canvas.height = ALTURA;
    gl.viewport(0, 0, largura, ALTURA);
    larguraAtual = largura;
  }

  alocarTextura(larguraAtual);

  const debug = gl.getExtension("WEBGL_debug_renderer_info");
  const renderer = gl.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER);
  let destruido = false;
  return {
    redimensionar(novaLargura) {
      if (destruido) return larguraAtual;
      const w = normalizarLarguraGpu(novaLargura);
      if (w !== larguraAtual) {
        alocarTextura(w);
      }
      return larguraAtual;
    },
    obterLargura() {
      return larguraAtual;
    },
    // `imagem`: Uint16Array de largura x 224 pontos, view da memoria Wasm (sem copia no JS).
    renderizar(imagem) {
      if (destruido) return;
      const esperado = larguraAtual * ALTURA;
      if (imagem.length !== esperado && imagem.length % ALTURA === 0) {
        const w = normalizarLarguraGpu(imagem.length / ALTURA);
        if (w * ALTURA === imagem.length) {
          alocarTextura(w);
        }
      }
      gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, larguraAtual, ALTURA, gl.RED_INTEGER, gl.UNSIGNED_SHORT, imagem);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
    },
    // Usado na validacao: framebuffer em ordem inferior para superior.
    lerPixels() {
      const pixels = new Uint8Array(larguraAtual * ALTURA * 4);
      gl.readPixels(0, 0, larguraAtual, ALTURA, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      return pixels;
    },
    sincronizar() { gl.finish(); },
    destruir() {
      if (destruido) return;
      destruido = true;
      gl.bindTexture(gl.TEXTURE_2D, null);
      gl.bindVertexArray(null);
      gl.useProgram(null);
      if (textura) {
        gl.deleteTexture(textura);
        textura = null;
      }
      gl.deleteVertexArray(vao);
      gl.deleteProgram(programa);
    },
    renderer,
  };
}
