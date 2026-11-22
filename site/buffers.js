// Estruturas puras de buffers reciclaveis e historico deterministico de entradas.

/**
 * Historico contiguo de mascaras de entrada (1 byte por quadro) para Save States deterministicos.
 * Cresce por dobramento geometrico e permite pre-reserva antes de replays longos.
 */
export class HistoricoEntradas {
  constructor(capacidadeInicial = 36000) {
    this._buf = new Uint8Array(Math.max(16, capacidadeInicial | 0));
    this._len = 0;
  }

  get tamanho() {
    return this._len;
  }

  get capacidade() {
    return this._buf.length;
  }

  garantirCapacidade(minimo) {
    const alvo = Math.max(0, minimo | 0);
    if (alvo <= this._buf.length) return;
    let novaCap = this._buf.length;
    while (novaCap < alvo) {
      novaCap *= 2;
    }
    const novo = new Uint8Array(novaCap);
    if (this._len > 0) {
      novo.set(this._buf.subarray(0, this._len));
    }
    this._buf = novo;
  }

  registrar(mascara) {
    if (this._len >= this._buf.length) {
      this.garantirCapacidade(this._buf.length * 2);
    }
    this._buf[this._len++] = mascara & 0xFF;
  }

  limpar() {
    this._len = 0;
  }

  fatiaAtiva() {
    return this._buf.subarray(0, this._len);
  }
}

/**
 * Pool limitado de TypedArrays reutilizaveis para evitar alocacoes e GC em transferencias
 * entre Worker, thread principal e AudioWorklet.
 */
export class PoolBuffers {
  constructor(Ctor, limite, tamanhoPadrao) {
    this._Ctor = Ctor;
    this._limite = Math.max(1, limite | 0);
    this._tamanhoPadrao = Math.max(1, tamanhoPadrao | 0);
    this._itens = [];
    for (let i = 0; i < this._limite; i++) {
      this._itens.push(new Ctor(this._tamanhoPadrao));
    }
  }

  get disponiveis() {
    return this._itens.length;
  }

  adquirir(tamanhoMinimo = this._tamanhoPadrao, exato = false) {
    for (let i = this._itens.length - 1; i >= 0; i--) {
      const buf = this._itens[i];
      const compativel = exato ? buf.length === tamanhoMinimo : buf.length >= tamanhoMinimo;
      if (compativel) {
        const ultimo = this._itens.length - 1;
        if (i !== ultimo) {
          this._itens[i] = this._itens[ultimo];
        }
        this._itens.pop();
        return buf;
      }
    }
    const tam = exato ? tamanhoMinimo : Math.max(this._tamanhoPadrao, tamanhoMinimo);
    return new this._Ctor(tam);
  }

  devolverBuffer(arrayBuffer) {
    if (!(arrayBuffer instanceof ArrayBuffer)) return false;
    if (this._itens.length >= this._limite) return false;
    this._itens.push(new this._Ctor(arrayBuffer));
    return true;
  }
}
