// AudioWorkletProcessor do Sonic 1 (Etapa 8).
// Fila circular sem SharedArrayBuffer (compativel com GitHub Pages).
// Recebe blocos Float32Array do thread principal via postMessage com transferencia.

class SonicAudioProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    // Fila circular para 16384 pares estereo (~340 ms a 48 kHz).
    this.tam = 16384;
    this.bufL = new Float32Array(this.tam);
    this.bufR = new Float32Array(this.tam);
    this.leitura = 0;
    this.escrita = 0;
    this.disponiveis = 0;
    this.falhas = 0;
    this.ultimoL = 0;
    this.ultimoR = 0;

    this.port.onmessage = (e) => {
      const dados = new Float32Array(e.data); // Intercalado: L, R, L, R...
      const pares = dados.length / 2;
      for (let i = 0; i < pares; i++) {
        if (this.disponiveis < this.tam) {
          this.bufL[this.escrita] = dados[i * 2];
          this.bufR[this.escrita] = dados[i * 2 + 1];
          this.escrita = (this.escrita + 1) % this.tam;
          this.disponiveis++;
        }
      }
    };
  }

  process(inputs, outputs) {
    const outL = outputs[0][0];
    const outR = outputs[0][1];
    const n = outL.length; // Normalmente 128 amostras

    for (let i = 0; i < n; i++) {
      if (this.disponiveis > 0) {
        this.ultimoL = this.bufL[this.leitura];
        this.ultimoR = this.bufR[this.leitura];
        this.leitura = (this.leitura + 1) % this.tam;
        this.disponiveis--;
      } else {
        // Falta de dado: decaimento suave para evitar estalidos
        this.falhas++;
        this.ultimoL *= 0.95;
        this.ultimoR *= 0.95;
      }
      outL[i] = this.ultimoL;
      outR[i] = this.ultimoR;
    }

    return true; // Continua rodando
  }
}

registerProcessor("sonic-audio-processor", SonicAudioProcessor);
