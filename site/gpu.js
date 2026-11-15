// Composição inteira das camadas da VDP. A captura temporal continua no Rust.
export const LARGURA_NOMINAL = 320;
export const ALTURA_NOMINAL = 224;
export const LARGURA_WIDESCREEN = 426;
export const ALTURA_EXPANDIDA = 240;

const VERTICE = `#version 300 es
void main() {
  vec2 p = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

const FRAGMENTO = `#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;
uniform usampler2D camadas;
uniform usampler2D sprites;
uniform usampler2D fundos;
uniform usampler2D seletores;
uniform usampler2D paletas;
uniform int quantidadeSprites;
uniform int larguraTela;
uniform int alturaTela;
out vec4 saida;
uint ponto(uint b, uint atributo) {
  uint cor = (atributo & 128u) != 0u ? b >> 4u : b & 15u;
  return (atributo & 112u) | cor;
}
void main() {
  ivec2 p = ivec2(int(gl_FragCoord.x), (alturaTela - 1) - int(gl_FragCoord.y));
  uvec4 c = texelFetch(camadas, p, 0);
  uint sel = texelFetch(seletores, p, 0).r;
  uint cor;
  if ((sel & 32768u) != 0u) {
    cor = sel & 511u;
  } else {
    uint indice = texelFetch(fundos, p, 0).r;
    uint sprite = 0u;
    for (int i = 0; i < 20; i++) {
      if (i >= quantidadeSprites) break;
      int linhaSprite = p.y * quantidadeSprites + i;
      uvec2 s = texelFetch(sprites, ivec2(p.x + (linhaSprite % 4) * larguraTela, linhaSprite / 4), 0).rg;
      uint candidato = ponto(s.r, s.g);
      if ((candidato & 15u) != 0u) { sprite = candidato; break; }
    }
    uvec3 candidatos = uvec3(sprite, ponto(c.r, c.g), ponto(c.b, c.a));
    bool achou = false;
    for (int prioridade = 1; prioridade >= 0; prioridade--) {
      for (int camada = 0; camada < 3; camada++) {
        uint valor = candidatos[camada];
        if (!achou && (valor & 15u) != 0u && ((valor >> 6u) & 1u) == uint(prioridade)) {
          indice = valor & 63u;
          achou = true;
        }
      }
    }
    cor = texelFetch(paletas, ivec2(int(indice + (sel % 32u) * 64u), int(sel / 32u)), 0).r;
  }
  uvec3 rgb = uvec3(cor & 7u, (cor >> 3u) & 7u, (cor >> 6u) & 7u);
  rgb = (rgb << 5u) | (rgb << 2u) | (rgb >> 1u);
  saida = vec4(vec3(rgb) / 255.0, 1.0);
}`;

export function criarRenderizadorGPU(canvas) {
  const gl = canvas.getContext("webgl2", { alpha: false, antialias: false, depth: false, stencil: false });
  if (!gl) return null;
  function shader(tipo, codigo) {
    const s = gl.createShader(tipo);
    gl.shaderSource(s, codigo);
    gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s));
    return s;
  }
  const programa = gl.createProgram();
  gl.attachShader(programa, shader(gl.VERTEX_SHADER, VERTICE));
  gl.attachShader(programa, shader(gl.FRAGMENT_SHADER, FRAGMENTO));
  gl.linkProgram(programa);
  if (!gl.getProgramParameter(programa, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(programa));
  gl.useProgram(programa);
  gl.bindVertexArray(gl.createVertexArray());
  gl.disable(gl.BLEND);
  gl.disable(gl.DITHER);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
  const texturas = ["camadas", "sprites", "fundos", "seletores", "paletas"].map((nome, i) => {
    const t = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0 + i);
    gl.bindTexture(gl.TEXTURE_2D, t);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.uniform1i(gl.getUniformLocation(programa, nome), i);
    return t;
  });
  const limite = gl.getParameter(gl.MAX_TEXTURE_SIZE);
  const debug = gl.getExtension("WEBGL_debug_renderer_info");
  const locQuantidadeSprites = gl.getUniformLocation(programa, "quantidadeSprites");
  const locLarguraTela = gl.getUniformLocation(programa, "larguraTela");
  const locAlturaTela = gl.getUniformLocation(programa, "alturaTela");
  function atualizar(i, formato, externo, largura, altura, tipo, dados) {
    gl.activeTexture(gl.TEXTURE0 + i);
    gl.bindTexture(gl.TEXTURE_2D, texturas[i]);
    gl.texImage2D(gl.TEXTURE_2D, 0, formato, largura, altura, 0, externo, tipo, dados);
  }
  return {
    renderizar(camadas, sprites, fundos, seletores, paletas, largura, altura) {
      if (!largura || !altura) {
        if (camadas.length === 426 * 224 * 4) {
          largura = 426;
          altura = 224;
        } else if (camadas.length === 426 * 240 * 4) {
          largura = 426;
          altura = 240;
        } else if (camadas.length === 320 * 240 * 4) {
          largura = 320;
          altura = 240;
        } else {
          largura = 320;
          altura = 224;
        }
      }
      const totalPontos = largura * altura;
      const totalPaletas = paletas.length / 64;
      const alturaPaletas = Math.ceil(totalPaletas / 32);
      const quantidadeSprites = sprites.length / (totalPontos * 2);
      if (camadas.length !== totalPontos * 4 || !Number.isInteger(quantidadeSprites) ||
          quantidadeSprites < 1 || quantidadeSprites > 20 ||
          fundos.length !== totalPontos || seletores.length !== totalPontos ||
          !Number.isInteger(totalPaletas) || totalPaletas < 1 || alturaPaletas > limite || limite < 2048) {
        throw new Error("Dados de vídeo incompatíveis com WebGL2");
      }
      if (canvas.width !== largura || canvas.height !== altura) {
        canvas.width = largura;
        canvas.height = altura;
      }
      const paletasEmpacotadas = new Uint16Array(2048 * alturaPaletas);
      paletasEmpacotadas.set(paletas);
      gl.uniform1i(locQuantidadeSprites, quantidadeSprites);
      gl.uniform1i(locLarguraTela, largura);
      gl.uniform1i(locAlturaTela, altura);
      atualizar(0, gl.RGBA8UI, gl.RGBA_INTEGER, largura, altura, gl.UNSIGNED_BYTE, camadas);
      atualizar(1, gl.RG8UI, gl.RG_INTEGER, largura * 4, Math.ceil(altura / 4) * quantidadeSprites, gl.UNSIGNED_BYTE, sprites);
      atualizar(2, gl.R8UI, gl.RED_INTEGER, largura, altura, gl.UNSIGNED_BYTE, fundos);
      atualizar(3, gl.R16UI, gl.RED_INTEGER, largura, altura, gl.UNSIGNED_SHORT, seletores);
      atualizar(4, gl.R16UI, gl.RED_INTEGER, 2048, alturaPaletas, gl.UNSIGNED_SHORT, paletasEmpacotadas);
      gl.viewport(0, 0, largura, altura);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      const erro = gl.getError();
      if (erro !== gl.NO_ERROR) throw new Error(`WebGL2: erro ${erro}`);
    },
    // Usado na validação: framebuffer em ordem inferior para superior.
    lerPixels(largura = canvas.width || 320, altura = canvas.height || 224) {
      const pixels = new Uint8Array(largura * altura * 4);
      gl.readPixels(0, 0, largura, altura, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      return pixels;
    },
    redimensionar(largura, altura) {
      canvas.width = largura;
      canvas.height = altura;
    },
    get largura() {
      return canvas.width;
    },
    get altura() {
      return canvas.height;
    },
    sincronizar() { gl.finish(); },
    renderer: gl.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER),
  };
}
