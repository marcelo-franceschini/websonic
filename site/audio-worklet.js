// AudioWorkletProcessor do Sonic 1 (Etapa 8).
// Fila circular sem SharedArrayBuffer (compativel com GitHub Pages).
// Recebe blocos Float32Array do Worker (via MessageChannel) ou da thread principal via transferencia.

class SonicAudioProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    // Fila circular potencia de 2: 32768 pares estereo (~682 ms a 48 kHz),
    // absorvendo transicoes de fase e picos do navegador sem estalos nem descartes.
    this.tam = 32768;
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
    this.aposFalta = false;
    this.rampaRestante = 0;
    this.rampaTotal = 32;
    this.ultimoL = 0;
    this.ultimoR = 0;
    this.chamadasProcess = 0;
    // Porta direta do MessageChannel com o Worker de simulacao (quando ativa).
    this.portaDados = null;
    // Objeto reutilizado para telemetria (evita alocacao no thread de audio em tempo real).
    this.msgStatus = {
      tipo: "status",
      disponiveis: 0,
      falhas: 0,
      descartadas: 0,
      processados: 0,
    };

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
    this.aposFalta = false;
    this.rampaRestante = 0;
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
    const livres = this.tam - this.disponiveis;
    const copiar = pares <= livres ? pares : livres;
    if (pares > livres) {
      this.descartadas += pares - livres;
    }

    let escrita = this.escrita;
    const tam = this.tam;
    const bufL = this.bufL;
    const bufR = this.bufR;
    const ateFim = tam - escrita;
    const primeiraParte = copiar <= ateFim ? copiar : ateFim;
    for (let i = 0; i < primeiraParte; i++) {
      const idx = i << 1;
      bufL[escrita + i] = dados[idx];
      bufR[escrita + i] = dados[idx + 1];
    }
    const restante = copiar - primeiraParte;
    if (restante > 0) {
      const baseDados = primeiraParte << 1;
      for (let i = 0; i < restante; i++) {
        const idx = baseDados + (i << 1);
        bufL[i] = dados[idx];
        bufR[i] = dados[idx + 1];
      }
      escrita = restante;
    } else {
      escrita = (escrita + primeiraParte) & this.mascara;
    }
    this.escrita = escrita;
    this.disponiveis += copiar;

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
      if (this.aposFalta) {
        this.rampaRestante = this.rampaTotal;
        this.aposFalta = false;
      }
    }

    // Caminho rapido sem ramificacoes por amostra (99.9% dos callbacks em estereo estavel).
    if (this.iniciado && this.disponiveis >= n && outR) {
      const bufL = this.bufL;
      const bufR = this.bufR;
      let leitura = this.leitura;
      if (leitura + n <= this.tam) {
        outL.set(bufL.subarray(leitura, leitura + n));
        outR.set(bufR.subarray(leitura, leitura + n));
        leitura = (leitura + n) & mascara;
      } else {
        const primeiraParte = this.tam - leitura;
        outL.set(bufL.subarray(leitura, this.tam), 0);
        outR.set(bufR.subarray(leitura, this.tam), 0);
        const restante = n - primeiraParte;
        outL.set(bufL.subarray(0, restante), primeiraParte);
        outR.set(bufR.subarray(0, restante), primeiraParte);
        leitura = restante;
      }
      if (this.rampaRestante > 0) {
        // Suaviza o retorno apos uma falta de dados para eliminar estalos (de-click).
        const passos = this.rampaRestante < n ? this.rampaRestante : n;
        const total = this.rampaTotal;
        const baseL = this.ultimoL;
        const baseR = this.ultimoR;
        for (let i = 0; i < passos; i++) {
          const ganho = (total - this.rampaRestante + 1) / total;
          const inv = 1 - ganho;
          outL[i] = baseL * inv + outL[i] * ganho;
          outR[i] = baseR * inv + outR[i] * ganho;
          this.rampaRestante--;
        }
      }
      this.leitura = leitura;
      this.disponiveis -= n;
      this.ultimoL = outL[n - 1];
      this.ultimoR = outR[n - 1];
    } else {
      for (let i = 0; i < n; i++) {
        if (!this.iniciado) {
          this.ultimoL *= 0.95;
          this.ultimoR *= 0.95;
        } else if (this.disponiveis > 0) {
          let l = this.bufL[this.leitura];
          let r = this.bufR[this.leitura];
          this.leitura = (this.leitura + 1) & mascara;
          this.disponiveis--;
          if (this.rampaRestante > 0) {
            const total = this.rampaTotal;
            const ganho = (total - this.rampaRestante + 1) / total;
            const inv = 1 - ganho;
            l = this.ultimoL * inv + l * ganho;
            r = this.ultimoR * inv + r * ganho;
            this.rampaRestante--;
          }
          this.ultimoL = l;
          this.ultimoR = r;
        } else {
          // Falta de dado: decaimento suave para evitar estalidos
          this.falhas++;
          this.iniciado = false;
          this.aposFalta = true;
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
    }

    if (this.chamadasProcess % 24 === 0) {
      const destino = this.portaDados || this.port;
      const msg = this.msgStatus;
      msg.disponiveis = this.disponiveis;
      msg.falhas = this.falhas;
      msg.descartadas = this.descartadas;
      msg.processados = this.chamadasProcess * n;
      destino.postMessage(msg);
    }

    return true; // Continua rodando
  }
}

registerProcessor("sonic-audio-processor", SonicAudioProcessor);
