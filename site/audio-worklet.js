// AudioWorkletProcessor do Sonic 1 (Etapa 8).
// Fila circular sem SharedArrayBuffer (compativel com GitHub Pages).
// Recebe blocos Float32Array do Worker (via MessageChannel) ou da thread principal via transferencia.

class SonicAudioProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    // Fila circular potencia de 2: 16384 pares estereo (~340 ms a 48 kHz).
    this.tam = 16384;
    this.mascara = this.tam - 1;
    this.bufL = new Float32Array(this.tam);
    this.bufR = new Float32Array(this.tam);
    this.leitura = 0;
    this.escrita = 0;
    this.disponiveis = 0;
    this.falhas = 0;
    this.descartadas = 0;
    this.iniciado = false;
    this.preenchimentoInicial = Math.ceil(sampleRate * 3 / 60);
    this.preenchimentoRetomada = Math.min(1024, Math.ceil(sampleRate * 1.5 / 60));
    this.tevePrimeiraPartida = false;
    this.ultimoL = 0;
    this.ultimoR = 0;
    this.chamadasProcess = 0;
    // Porta direta do MessageChannel com o Worker de simulacao (quando ativa).
    this.portaDados = null;

    this.port.onmessage = (e) => this.receberMensagem(e, this.port);
  }

  reiniciarFila() {
    this.leitura = 0;
    this.escrita = 0;
    this.disponiveis = 0;
    this.ultimoL = 0;
    this.ultimoR = 0;
    this.falhas = 0;
    this.descartadas = 0;
    this.iniciado = false;
    this.tevePrimeiraPartida = false;
  }

  receberMensagem(e, origem) {
    const msg = e.data;
    if (!msg) return;

    if (msg.tipo === "porta" && msg.porta) {
      if (this.portaDados) {
        this.portaDados.onmessage = null;
        if (typeof this.portaDados.close === "function") this.portaDados.close();
      }
      this.portaDados = msg.porta;
      this.portaDados.onmessage = (ev) => this.receberMensagem(ev, this.portaDados);
      this.reiniciarFila();
      return;
    }

    if (msg.tipo === "reset") {
      this.reiniciarFila();
      return;
    }

    const dados = msg instanceof Float32Array ? msg : new Float32Array(msg);
    const pares = dados.length >> 1;
    const mascara = this.mascara;
    const tam = this.tam;
    for (let i = 0; i < pares; i++) {
      if (this.disponiveis < tam) {
        this.bufL[this.escrita] = dados[i * 2];
        this.bufR[this.escrita] = dados[i * 2 + 1];
        this.escrita = (this.escrita + 1) & mascara;
        this.disponiveis++;
      } else {
        this.descartadas++;
      }
    }
    if (dados.buffer) {
      origem.postMessage(dados.buffer, [dados.buffer]);
    }
  }

  process(_inputs, outputs) {
    this.chamadasProcess++;
    const out = outputs[0];
    if (!out || out.length === 0) return true;

    const outL = out[0];
    const outR = out.length > 1 ? out[1] : null;
    const n = outL.length; // Normalmente 128 amostras
    const mascara = this.mascara;

    const limiar = this.tevePrimeiraPartida ? this.preenchimentoRetomada : this.preenchimentoInicial;
    if (!this.iniciado && this.disponiveis >= limiar) {
      this.iniciado = true;
      this.tevePrimeiraPartida = true;
    }

    for (let i = 0; i < n; i++) {
      if (!this.iniciado) {
        this.ultimoL *= 0.95;
        this.ultimoR *= 0.95;
      } else if (this.disponiveis > 0) {
        this.ultimoL = this.bufL[this.leitura];
        this.ultimoR = this.bufR[this.leitura];
        this.leitura = (this.leitura + 1) & mascara;
        this.disponiveis--;
      } else {
        // Falta de dado: decaimento suave para evitar estalidos
        this.falhas++;
        this.iniciado = false;
        this.ultimoL *= 0.95;
        this.ultimoR *= 0.95;
      }

      if (outR) {
        outL[i] = this.ultimoL;
        outR[i] = this.ultimoR;
      } else {
        // Saida mono: faz a media dos dois canais
        outL[i] = (this.ultimoL + this.ultimoR) * 0.5;
      }
    }

    if (this.chamadasProcess % 24 === 0) {
      const destino = this.portaDados || this.port;
      destino.postMessage({
        tipo: "status",
        disponiveis: this.disponiveis,
        falhas: this.falhas,
        descartadas: this.descartadas,
        processados: this.chamadasProcess * n,
      });
    }

    return true; // Continua rodando
  }
}

registerProcessor("sonic-audio-processor", SonicAudioProcessor);
