// Relógio de 60 Hz exatos conforme o requisito do projeto.
export const PERIODO_QUADRO = 1000 / 60;

export class Relogio {
  constructor() {
    this.ultimo = 0;
    this.acumulado = 0;
  }

  iniciar(agora) {
    this.acumulado = 0;
    this.ultimo = agora;
  }

  retomar(agora) {
    this.ultimo = agora;
  }

  avancar(agora) {
    // O vigia (app.js) chama com performance.now() entre dois rAF: o tempo do relogio nunca volta.
    this.acumulado += Math.max(0, agora - this.ultimo);
    this.ultimo = Math.max(this.ultimo, agora);
    if (this.acumulado > PERIODO_QUADRO * 10) {
      this.acumulado = PERIODO_QUADRO * 10;
    }
    // Janela de estabilizacao (entre 0,8 e 1,75 quadro). O limite e 0,8 e nao 0,75: com 60 Hz exatos, tres ticks de 240 Hz somam 0,75 quadro em ponto flutuante e disparariam um passo antes da hora (teste "janela de 0,8 quadro").
    let passos = 0;
    if (this.acumulado >= PERIODO_QUADRO * 0.8 && this.acumulado < PERIODO_QUADRO * 1.75) {
      passos = 1;
      this.acumulado -= PERIODO_QUADRO;
    } else {
      passos = Math.min(4, Math.floor(this.acumulado / PERIODO_QUADRO));
      this.acumulado -= passos * PERIODO_QUADRO;
    }
    return passos;
  }
}
