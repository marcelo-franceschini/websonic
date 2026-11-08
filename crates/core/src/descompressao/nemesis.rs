//! Nemesis. Reproduz as rotinas `NemDec` e `NemDecToRAM` do disassembly (a lógica de
//! decodificação é a mesma, só o destino muda).
//!
//! Formato (descrição pública, ver `docs/conhecimento/descompressao.md`):
//! - Cabeçalho de 2 bytes big-endian: bit 15 liga o modo XOR, os 15 bits baixos são o número
//!   de tiles (32 bytes cada).
//! - Tabela de códigos, terminada por `0xFF`. Um byte com o bit 7 ligado define o nibble
//!   corrente (4 bits baixos) e é seguido de um byte de entrada. Uma entrada tem o tamanho
//!   do código nos 4 bits baixos, a contagem menos 1 nos bits 4 a 6, e é seguida de um byte
//!   com o código.
//! - Dados: fluxo de bits (do mais significativo ao menos). Um código da tabela produz uma
//!   corrida de nibbles. O prefixo `111111` é reservado: 3 bits de contagem menos 1 e 4 bits
//!   de nibble.
//! - Os nibbles formam bytes (alto primeiro) e a saída são tiles de 32 bytes. No modo XOR,
//!   cada linha de 4 bytes sai XOR da linha anterior já decodificada.
//!
//! A estrutura [`Nemesis`] é retomável: o jogo original decodifica a arte aos poucos e o
//! progresso fica na memória comparada (decisão D3 do plano da Etapa 3).

use super::{Bits, Bytes, Erro, Resultado, empurrar};

/// Uma entrada da tabela de códigos.
#[derive(Clone, Copy)]
struct Entrada {
    codigo: u8,
    tamanho: u8,
    nibble: u8,
    contagem: u8,
}

/// Bytes por tile.
const BYTES_POR_TILE: usize = 32;
/// Nibbles por tile.
const NIBBLES_POR_TILE: usize = 64;

/// Descompressor Nemesis que pode parar e continuar.
pub struct Nemesis<'a> {
    bits: Bits<'a>,
    tabela: Vec<Entrada>,
    xor: bool,
    tiles_total: usize,
    nibbles_feitos: usize,
    nibble_pendente: Option<u8>,
    saida: Vec<u8>,
    limite_saida: usize,
    linha_anterior: [u8; 4],
}

impl<'a> Nemesis<'a> {
    /// Lê o cabeçalho e a tabela. Falha se a saída completa passar de `limite_saida`.
    pub fn novo(entrada: &'a [u8], limite_saida: usize) -> Result<Self, Erro> {
        let mut bytes = Bytes::novo(entrada);
        let cabecalho = bytes.palavra_be()?;
        let xor = cabecalho & 0x8000 != 0;
        let tiles_total = usize::from(cabecalho & 0x7FFF);
        let tamanho = tiles_total
            .checked_mul(BYTES_POR_TILE)
            .ok_or(Erro::SaidaExcedeLimite)?;
        if tamanho > limite_saida {
            return Err(Erro::SaidaExcedeLimite);
        }
        let mut tabela = Vec::new();
        if tiles_total > 0 {
            let mut nibble = 0u8;
            loop {
                let mut b = bytes.byte()?;
                if b == 0xFF {
                    break;
                }
                if b & 0x80 != 0 {
                    nibble = b & 0x0F;
                    b = bytes.byte()?;
                }
                let tamanho = b & 0x0F;
                if tamanho == 0 || tamanho > 8 {
                    return Err(Erro::FormatoInvalido("tamanho de codigo fora de 1 a 8"));
                }
                let contagem = (b.wrapping_shr(4) & 7)
                    .checked_add(1)
                    .ok_or(Erro::FormatoInvalido("contagem"))?;
                let codigo = bytes.byte()?;
                tabela.push(Entrada {
                    codigo,
                    tamanho,
                    nibble,
                    contagem,
                });
            }
        }
        Ok(Self {
            bits: Bits::novo(bytes),
            tabela,
            xor,
            tiles_total,
            nibbles_feitos: 0,
            nibble_pendente: None,
            saida: Vec::new(),
            limite_saida,
            linha_anterior: [0; 4],
        })
    }

    /// Palavras que a rotina `NemDec_BuildCodeTable` grava no buffer de códigos da RAM
    /// (`v_ngfx_buffer`): par (deslocamento em bytes, palavra). Cada palavra junta o tamanho do
    /// código (byte alto), a contagem menos um (bits 4 a 6) e o nibble (bits 0 a 3). Um código
    /// curto ocupa todas as posições que começam com ele. Posições não citadas ficam como estavam.
    pub fn palavras_da_tabela(&self) -> Vec<(u32, u16)> {
        let mut saida = Vec::new();
        for e in &self.tabela {
            let palavra = u16::from(e.tamanho).wrapping_shl(8)
                | (u16::from(e.contagem.saturating_sub(1)) & 7).wrapping_shl(4)
                | u16::from(e.nibble & 0x0F);
            let extra = 8u32.saturating_sub(u32::from(e.tamanho));
            let base = u32::from(e.codigo).wrapping_shl(extra);
            let quantas = 1u32.wrapping_shl(extra);
            for k in 0..quantas {
                saida.push((base.wrapping_add(k).wrapping_mul(2), palavra));
            }
        }
        saida
    }

    /// Número de tiles que o cabeçalho promete.
    pub fn tiles_total(&self) -> usize {
        self.tiles_total
    }

    /// Bytes já decodificados e definitivos. No modo XOR, uma linha de 4 bytes só vale depois
    /// de completa, por isso a linha em andamento não aparece.
    pub fn saida(&self) -> &[u8] {
        let util = if self.xor {
            self.saida.len() & !3
        } else {
            self.saida.len()
        };
        self.saida.get(..util).unwrap_or(&[])
    }

    /// Verdadeiro quando todos os tiles foram decodificados.
    pub fn terminou(&self) -> bool {
        self.nibbles_feitos >= self.alvo(self.tiles_total)
    }

    /// Bytes da entrada tocados até agora.
    pub fn consumidos(&self) -> usize {
        self.bits.posicao()
    }

    fn alvo(&self, tiles: usize) -> usize {
        tiles.min(self.tiles_total).saturating_mul(NIBBLES_POR_TILE)
    }

    /// Decodifica códigos até completar `tiles` tiles (no total, contando os já feitos).
    /// Uma corrida pode terminar um pouco além do tile pedido, mas nunca além do fim.
    pub fn executar_ate(&mut self, tiles: usize) -> Result<(), Erro> {
        let alvo = self.alvo(tiles);
        while self.nibbles_feitos < alvo {
            self.um_codigo()?;
        }
        Ok(())
    }

    fn um_codigo(&mut self) -> Result<(), Erro> {
        let mut codigo: u8 = 0;
        let mut tamanho: u8 = 0;
        loop {
            codigo = codigo.wrapping_shl(1) | self.bits.bit()?;
            tamanho = tamanho.checked_add(1).ok_or(Erro::EntradaTruncada)?;
            if tamanho == 6 && codigo == 0x3F {
                let contagem = u8::try_from(self.bits.bits(3)?)
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or(Erro::FormatoInvalido("contagem"))?;
                let nibble = u8::try_from(self.bits.bits(4)?).unwrap_or(0);
                return self.emitir(nibble, contagem);
            }
            let achada = self
                .tabela
                .iter()
                .find(|e| e.tamanho == tamanho && e.codigo == codigo)
                .copied();
            if let Some(e) = achada {
                return self.emitir(e.nibble, e.contagem);
            }
            if tamanho >= 8 {
                return Err(Erro::FormatoInvalido("codigo sem entrada na tabela"));
            }
        }
    }

    fn emitir(&mut self, nibble: u8, contagem: u8) -> Result<(), Erro> {
        let total = self.alvo(self.tiles_total);
        let depois = self
            .nibbles_feitos
            .checked_add(usize::from(contagem))
            .ok_or(Erro::SaidaExcedeLimite)?;
        if depois > total {
            return Err(Erro::FormatoInvalido("corrida passa do fim dos tiles"));
        }
        for _ in 0..contagem {
            match self.nibble_pendente.take() {
                None => self.nibble_pendente = Some(nibble),
                Some(alto) => {
                    let byte = alto.wrapping_shl(4) | nibble;
                    self.empurrar_byte(byte)?;
                }
            }
        }
        self.nibbles_feitos = depois;
        Ok(())
    }

    fn empurrar_byte(&mut self, byte: u8) -> Result<(), Erro> {
        empurrar(&mut self.saida, byte, self.limite_saida)?;
        if self.xor && self.saida.len().is_multiple_of(4) {
            // Fecha uma linha de 4 bytes: aplica o XOR com a linha anterior já resolvida.
            let inicio = self.saida.len().saturating_sub(4);
            let mut linha = [0u8; 4];
            for (i, destino) in linha.iter_mut().enumerate() {
                let bruto = self
                    .saida
                    .get(inicio.saturating_add(i))
                    .copied()
                    .unwrap_or(0);
                let anterior = self.linha_anterior.get(i).copied().unwrap_or(0);
                *destino = bruto ^ anterior;
            }
            for (i, valor) in linha.iter().enumerate() {
                if let Some(posicao) = self.saida.get_mut(inicio.saturating_add(i)) {
                    *posicao = *valor;
                }
            }
            self.linha_anterior = linha;
        }
        Ok(())
    }
}

/// Descomprime um fluxo Nemesis inteiro.
pub fn descomprimir(entrada: &[u8], limite_saida: usize) -> Result<Resultado, Erro> {
    let mut nemesis = Nemesis::novo(entrada, limite_saida)?;
    nemesis.executar_ate(nemesis.tiles_total())?;
    Ok(Resultado {
        consumidos: nemesis.consumidos(),
        saida: nemesis.saida,
    })
}
