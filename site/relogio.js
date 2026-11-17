// Relógio de 60 Hz exatos conforme o requisito do projeto.
export const PERIODO_QUADRO = 1000 / 60;

export class Relogio {
  constructor() {
    this.ultimo = 0;
    this.acumulado = 0;
    this.alpha = 0;
  }

  iniciar(agora) {
    this.acumulado = 0;
    this.ultimo = agora;
    this.alpha = 0;
  }

  retomar(agora) {
    this.ultimo = agora;
  }

  avancar(agora) {
    this.acumulado += Math.max(0, agora - this.ultimo);
    this.ultimo = agora;
    if (this.acumulado > PERIODO_QUADRO * 10) {
      this.acumulado = PERIODO_QUADRO * 10;
    }
    // Janela de estabilizacao para V-Sync em ~60 Hz (entre 0.8 e 1.75 quadros):
    // Elimina micro-stutter e jitter sem alterar a contagem acumulada em testes sinteticos.
    let passos = 0;
    if (this.acumulado >= PERIODO_QUADRO * 0.8 && this.acumulado < PERIODO_QUADRO * 1.75) {
      passos = 1;
      this.acumulado -= PERIODO_QUADRO;
    } else {
      passos = Math.min(4, Math.floor(this.acumulado / PERIODO_QUADRO));
      this.acumulado -= passos * PERIODO_QUADRO;
    }
    this.alpha = Math.max(0, Math.min(1, this.acumulado / PERIODO_QUADRO));
    return passos;
  }

  obterAlpha() {
    return this.alpha;
  }

  get fase() {
    return this.alpha;
  }

  interpolar(anterior, atual) {
    return anterior + (atual - anterior) * this.alpha;
  }

  passosEAlpha(agora) {
    const passos = this.avancar(agora);
    return { passos, alpha: this.alpha };
  }
}

export function interpolarLinear(anterior, atual, alpha) {
  return anterior + (atual - anterior) * alpha;
}

export function interpolarPosicao(anterior, atual, alpha) {
  return [
    anterior[0] + (atual[0] - anterior[0]) * alpha,
    anterior[1] + (atual[1] - anterior[1]) * alpha,
  ];
}
