//! Barramento e programa do Z80 (sub-etapas 6.1 e 6.2).
//!
//! O Z80 do Mega Drive do Sonic 1 roda dois programas pequenos: o código de partida (limpa a
//! RAM com uma cópia em bloco e fica num salto para si mesmo) e o driver de DAC (bateria e voz
//! SEGA). Aqui eles são máquinas de estados nativas. Nenhuma instrução é decodificada: cada
//! trecho do programa é uma função com o mesmo efeito e o custo em ciclos da instrução.
//!
//! O modelo de tempo copia o do oráculo: a instrução inteira acontece no primeiro ciclo e os
//! `custo - 1` ciclos seguintes só esperam (`docs/conhecimento/som.md`). Com BUSREQ ou RESET
//! ativos o Z80 não executa, só gasta 4 ciclos por tentativa.
//!
//! Custos: Zilog, Z80 CPU User Manual (UM0080), tabelas de instruções.
//! Rotinas do disassembly que este módulo espelha: `SetupValues_Z80` (programa A) e o programa
//! de `sound/z80.asm` (programa B). Os endereços dos trechos são os do programa montado; foram
//! conferidos contra os desvios do Z80 executados pelo oráculo (`docs/inventario-som.md`).

use crate::memoria::Saida;
use crate::quadro::tempo::{AcessoZ80, EventoTempo};

/// Tamanho da RAM do Z80.
pub const TAMANHO_RAM: usize = 0x2000;

/// Registrador de banco da janela de ROM do Z80 (9 bits, escrito um bit por vez).
///
/// Cada escrita no endereço `6000` do Z80 empurra o bit 0 do byte pela esquerda: o valor
/// anterior anda uma casa para a direita e o bit novo entra no topo (bit 8).
/// Fonte: Genesis Programming, "Z80 Bank Register" (wikibooks); conferido com o oráculo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Banco(u16);

impl Banco {
    pub fn valor(self) -> u16 {
        self.0
    }

    pub fn escrever_bit(&mut self, bit: bool) {
        self.0 = (self.0 >> 1) | (u16::from(bit) << 8);
    }

    /// Endereço do 68000 que o Z80 enxerga em `8000` a `FFFF`.
    pub fn endereco_68000(self, z80: u16) -> u32 {
        (u32::from(self.0) << 15) | u32::from(z80 & 0x7FFF)
    }
}

/// Um ponto do Z80 gravado pelo oráculo: onde o Z80 estava quando o 68000 o tocou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PontoZ80 {
    pub acesso: AcessoZ80,
    pub pc: u16,
    pub espera: u16,
    /// Ciclos do Z80 executados desde o ponto anterior.
    pub ativos: u32,
    pub banco: u16,
    pub endereco: u16,
    pub byte: u8,
}

impl PontoZ80 {
    pub fn de_evento(e: &EventoTempo) -> Option<Self> {
        match *e {
            EventoTempo::PontoZ80 {
                acesso,
                pc,
                espera,
                ativos,
                banco,
                endereco,
                byte,
                ..
            } => Some(Self {
                acesso,
                pc,
                espera,
                ativos,
                banco,
                endereco,
                byte,
            }),
            _ => None,
        }
    }
}

/// Diferença entre o Z80 do núcleo e o ponto gravado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DivergenciaZ80 {
    /// O núcleo fez um acesso de tipo diferente do próximo ponto gravado.
    AcessoDiferente {
        esperado: AcessoZ80,
        obtido: AcessoZ80,
    },
    /// O acesso é o mesmo, mas a máquina não está onde o oráculo viu o Z80.
    Posicao {
        acesso: AcessoZ80,
        esperado: (u16, u16, u16),
        obtido: (u16, u16, u16),
    },
}

/// O que a máquina do Z80 enxerga do resto do console durante um avanço.
pub struct Ambiente<'a> {
    pub ram: &'a mut [u8; TAMANHO_RAM],
    pub rom: &'a [u8],
    pub busreq: bool,
    pub reset: bool,
    /// Escritas do Z80 nos chips de som saem aqui, na ordem.
    pub saidas: &'a mut Vec<Saida>,
}

/// Qual programa está rodando. É reconhecido no endereço zero depois de um reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Programa {
    /// Ainda não buscou nada depois do reset.
    Nenhum,
    /// Código de partida (`SetupValues_Z80`).
    Partida,
    /// Salto para si mesmo em zero, onde o código de partida termina.
    SaltoParaSi,
    /// Driver de DAC (`sound/z80.asm`).
    Dac,
    /// Primeiro byte desconhecido: a máquina só gasta ciclos.
    Desconhecido,
}

/// A máquina de estados do Z80.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaquinaZ80 {
    /// Endereço do próximo trecho. É o estado da máquina, não um registrador.
    pc: u16,
    /// Ciclos que faltam para o trecho em curso terminar.
    espera: u32,
    programa: Programa,
    banco: Banco,
    // Variáveis do driver de DAC, nomeadas pelo papel que têm no programa.
    base_ym: u16,
    valor: u8,
    contador: u8,
    tabela_dac: u16,
    ponteiro: u16,
    posicao: u16,
    restante: u16,
    passo: u8,
    acumulador_pcm: u8,
    comando_dac: u8,
    ponteiro_status: u16,
    // Variáveis do código de partida.
    origem_copia: u16,
    destino_copia: u16,
    quantidade_copia: u16,
    /// Quantas vezes um trecho desconhecido foi alcançado (aviso para o relatório).
    pub desconhecidos: u64,
}

fn baixo(v: u16) -> u8 {
    v.to_le_bytes()[0]
}

fn alto(v: u16) -> u8 {
    v.to_le_bytes()[1]
}

impl Default for MaquinaZ80 {
    fn default() -> Self {
        Self::nova()
    }
}

impl MaquinaZ80 {
    /// Estado de um Z80 recém-ligado, ainda em reset (como o oráculo).
    pub fn nova() -> Self {
        Self {
            pc: 0,
            espera: 0,
            programa: Programa::Nenhum,
            banco: Banco::default(),
            base_ym: 0,
            valor: 0,
            contador: 0,
            tabela_dac: 0,
            ponteiro: 0,
            posicao: 0,
            restante: 0,
            passo: 0,
            acumulador_pcm: 0,
            comando_dac: 0,
            ponteiro_status: 0,
            origem_copia: 0,
            destino_copia: 0,
            quantidade_copia: 0,
            desconhecidos: 0,
        }
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn espera(&self) -> u32 {
        self.espera
    }

    pub fn banco(&self) -> Banco {
        self.banco
    }

    pub fn programa(&self) -> Programa {
        self.programa
    }

    /// Ponto de partida para testes e para o harness: põe a máquina num trecho conhecido do
    /// driver de DAC, com as variáveis do laço de espera de amostra.
    pub fn injetar_driver_dac_ocioso(&mut self, pc: u16, espera: u32, banco: u16) {
        self.programa = Programa::Dac;
        self.pc = pc;
        self.espera = espera;
        self.banco = Banco(banco & 0x1FF);
        self.base_ym = 0x4000;
        self.ponteiro = 0x1FFF;
    }

    // --- Leitura e escrita no espaço de endereços do Z80 ---

    fn ler(&self, amb: &Ambiente<'_>, a: u16) -> u8 {
        match a {
            0x0000..=0x3FFF => amb.ram[usize::from(a & 0x1FFF)],
            0x8000..=0xFFFF => {
                let i = usize::try_from(self.banco.endereco_68000(a) & 0x3F_FFFF).unwrap_or(0);
                amb.rom.get(i).copied().unwrap_or(0xFF)
            }
            // YM2612 (o programa não lê), registrador de banco e área sem nada: 0xFF.
            _ => 0xFF,
        }
    }

    fn escrever(&mut self, amb: &mut Ambiente<'_>, a: u16, v: u8) {
        match a {
            0x0000..=0x3FFF => amb.ram[usize::from(a & 0x1FFF)] = v,
            0x4000..=0x5FFF => amb.saidas.push(Saida::YmZ80 {
                porta: baixo(a) & 3,
                valor: v,
            }),
            0x6000..=0x60FF => self.banco.escrever_bit(v & 1 != 0),
            _ => {}
        }
    }

    fn imediato16(amb: &Ambiente<'_>, a: u16) -> u16 {
        let l = amb.ram[usize::from(a & 0x1FFF)];
        let h = amb.ram[usize::from(a.wrapping_add(1) & 0x1FFF)];
        u16::from_le_bytes([l, h])
    }

    fn imediato8(amb: &Ambiente<'_>, a: u16) -> u8 {
        amb.ram[usize::from(a & 0x1FFF)]
    }

    // --- Avanço no tempo ---

    /// Avança `ciclos` ciclos do Z80, como `ciclos` chamadas de `tick` do oráculo.
    pub fn avancar(&mut self, amb: &mut Ambiente<'_>, ciclos: u64) {
        let mut resta = ciclos;
        while resta > 0 {
            if self.espera > 0 {
                let d = u64::from(self.espera).min(resta);
                self.espera = self
                    .espera
                    .saturating_sub(u32::try_from(d).unwrap_or(u32::MAX));
                resta = resta.saturating_sub(d);
                continue;
            }
            // Parado por BUSREQ ou RESET: cada tentativa gasta 4 ciclos (1 e 3 de espera).
            // Salta de quatro em quatro em tempo constante.
            if amb.reset || amb.busreq {
                self.parado_por_barramento(amb.reset);
                let tentativas = resta / 4;
                resta = resta.saturating_sub(tentativas.saturating_mul(4));
                if resta > 0 {
                    resta = resta.saturating_sub(1);
                    self.espera = 3;
                }
                continue;
            }
            resta = resta.saturating_sub(1);
            let custo = self.executar(amb);
            self.espera = custo.saturating_sub(1);
        }
    }

    /// Efeito de uma tentativa de execução com o barramento tomado ou em reset.
    fn parado_por_barramento(&mut self, reset: bool) {
        if reset {
            // O reset do oráculo zera o PC e esquece o programa em curso.
            self.pc = 0;
            self.programa = Programa::Nenhum;
        }
    }

    /// Executa o trecho em `pc` (efeito no início) e devolve o custo em ciclos.
    fn executar(&mut self, amb: &mut Ambiente<'_>) -> u32 {
        if self.pc == 0 {
            self.programa = match amb.ram[0] {
                0xAF => Programa::Partida,
                0xF3 => Programa::Dac,
                0xE9 => Programa::SaltoParaSi,
                _ => Programa::Desconhecido,
            };
        }
        match self.programa {
            Programa::Partida => self.partida(amb),
            Programa::SaltoParaSi => 4,
            Programa::Dac => self.dac(amb),
            Programa::Nenhum | Programa::Desconhecido => {
                self.desconhecidos = self.desconhecidos.saturating_add(1);
                4
            }
        }
    }

    // --- Programa A: código de partida (SetupValues_Z80) ---

    fn partida(&mut self, amb: &mut Ambiente<'_>) -> u32 {
        let pc = self.pc;
        match pc {
            0x00 => {
                self.valor = 0;
                self.pc = 0x01;
                4
            }
            0x01 => {
                self.quantidade_copia = Self::imediato16(amb, 0x02);
                self.pc = 0x04;
                10
            }
            0x04 => {
                self.destino_copia = Self::imediato16(amb, 0x05);
                self.pc = 0x07;
                10
            }
            0x07 => {
                self.origem_copia = Self::imediato16(amb, 0x08);
                self.pc = 0x0A;
                10
            }
            // ld sp,hl
            0x0A => {
                self.pc = 0x0B;
                6
            }
            // ld (hl),a
            0x0B => {
                let o = self.origem_copia;
                let v = self.valor;
                self.escrever(amb, o, v);
                self.pc = 0x0C;
                7
            }
            // ldir: uma volta por execução; repete até o contador chegar a zero.
            0x0C => {
                let v = self.ler(amb, self.origem_copia);
                let d = self.destino_copia;
                self.escrever(amb, d, v);
                self.origem_copia = self.origem_copia.wrapping_add(1);
                self.destino_copia = self.destino_copia.wrapping_add(1);
                self.quantidade_copia = self.quantidade_copia.wrapping_sub(1);
                if self.quantidade_copia != 0 {
                    21
                } else {
                    self.pc = 0x0E;
                    16
                }
            }
            // pop ix, pop iy
            0x0E | 0x10 => {
                self.pc = pc.wrapping_add(2);
                14
            }
            // ld i,a ; ld r,a
            0x12 | 0x14 => {
                self.pc = pc.wrapping_add(2);
                9
            }
            // pop de ; pop hl ; pop af
            0x16..=0x18 => {
                self.pc = pc.wrapping_add(1);
                10
            }
            // ex af,af' ; exx
            0x19 | 0x1A => {
                self.pc = pc.wrapping_add(1);
                4
            }
            // pop bc ; pop de ; pop hl ; pop af. Os valores lidos são zeros que a cópia gravou.
            0x1B..=0x1E => {
                self.origem_copia = 0;
                self.pc = pc.wrapping_add(1);
                10
            }
            // ld sp,hl
            0x1F => {
                self.pc = 0x20;
                6
            }
            // di
            0x20 => {
                self.pc = 0x21;
                4
            }
            // im 1
            0x21 => {
                self.pc = 0x23;
                8
            }
            // ld (hl),$E9: o primeiro byte vira um salto para si mesmo.
            0x23 => {
                let o = self.origem_copia;
                self.escrever(amb, o, 0xE9);
                self.pc = 0x25;
                10
            }
            // jp (hl): vai para o endereço em HL, que é zero.
            0x25 => {
                self.pc = self.origem_copia;
                4
            }
            _ => self.trecho_desconhecido(),
        }
    }

    fn trecho_desconhecido(&mut self) -> u32 {
        self.desconhecidos = self.desconhecidos.saturating_add(1);
        self.programa = Programa::Desconhecido;
        4
    }

    // --- Programa B: driver de DAC (sound/z80.asm) ---

    fn dac(&mut self, amb: &mut Ambiente<'_>) -> u32 {
        let pc = self.pc;
        // Quase todo trecho segue para o endereço seguinte; os de desvio sobrescrevem.
        match pc {
            // di, di, di
            0x00..=0x02 => self.seguir(pc.wrapping_add(1), 4),
            // ld sp,nn
            0x03 => self.seguir(0x06, 10),
            // ld ix,nn
            0x06 => {
                self.base_ym = Self::imediato16(amb, 0x08);
                self.seguir(0x0A, 14)
            }
            // xor a
            0x0A => {
                self.valor = 0;
                self.seguir(0x0B, 4)
            }
            // ld (zDAC_Status),a
            0x0B => {
                let (a, v) = (Self::imediato16(amb, 0x0C), self.valor);
                self.escrever(amb, a, v);
                self.seguir(0x0E, 13)
            }
            // ld (zDAC_Sample),a
            0x0E => {
                let (a, v) = (Self::imediato16(amb, 0x0F), self.valor);
                self.escrever(amb, a, v);
                self.seguir(0x11, 13)
            }
            // ld a,n (bit menos significativo do banco)
            0x11 => {
                self.valor = Self::imediato8(amb, 0x12);
                self.seguir(0x13, 7)
            }
            // ld (zBankRegister),a
            0x13 => {
                let (a, v) = (Self::imediato16(amb, 0x14), self.valor);
                self.escrever(amb, a, v);
                self.seguir(0x16, 13)
            }
            // ld b,8
            0x16 => {
                self.contador = Self::imediato8(amb, 0x17);
                self.seguir(0x18, 7)
            }
            // ld a,n (resto do banco)
            0x18 => {
                self.valor = Self::imediato8(amb, 0x19);
                self.seguir(0x1A, 7)
            }
            // zBankSwitchLoop: ld (zBankRegister),a
            0x1A => {
                let (a, v) = (Self::imediato16(amb, 0x1B), self.valor);
                self.escrever(amb, a, v);
                self.seguir(0x1D, 13)
            }
            // rrca
            0x1D => {
                self.valor = self.valor.rotate_right(1);
                self.seguir(0x1E, 4)
            }
            // djnz zBankSwitchLoop
            0x1E => {
                self.contador = self.contador.wrapping_sub(1);
                if self.contador != 0 {
                    self.seguir(0x1A, 13)
                } else {
                    self.seguir(0x20, 8)
                }
            }
            // jr zCheckForSamples
            0x20 => self.seguir(0x32, 12),
            // zCheckForSamples: ld hl,zDAC_Sample
            0x32 => {
                self.ponteiro = Self::imediato16(amb, 0x33);
                self.seguir(0x35, 10)
            }
            // zWaitDACLoop: ld a,(hl)
            0x35 => {
                self.valor = self.ler(amb, self.ponteiro);
                self.seguir(0x36, 7)
            }
            // or a
            0x36 => self.seguir(0x37, 4),
            // jp p,zWaitDACLoop (continua esperando enquanto o bit 7 está limpo)
            0x37 => {
                if self.valor & 0x80 == 0 {
                    self.seguir(0x35, 10)
                } else {
                    self.seguir(0x3A, 10)
                }
            }
            // sub $81
            0x3A => {
                self.valor = self.valor.wrapping_sub(Self::imediato8(amb, 0x3B));
                self.seguir(0x3C, 7)
            }
            // ld (hl),a: marca a amostra como em reprodução
            0x3C => {
                let (a, v) = (self.ponteiro, self.valor);
                self.escrever(amb, a, v);
                self.seguir(0x3D, 7)
            }
            // cp 6
            0x3D => self.seguir(0x3F, 7),
            // jr nc,zPlay_SegaPCM
            0x3F => {
                if self.valor >= Self::imediato8(amb, 0x3E) {
                    self.seguir(0xB9, 12)
                } else {
                    self.seguir(0x41, 7)
                }
            }
            // ld de,0
            0x41 => {
                self.posicao = 0;
                self.seguir(0x44, 10)
            }
            // ld iy,zPCM_Table
            0x44 => {
                self.tabela_dac = Self::imediato16(amb, 0x46);
                self.seguir(0x48, 14)
            }
            // sla a (três vezes)
            0x48 | 0x4A | 0x4C => {
                self.valor = self.valor.wrapping_shl(1);
                self.seguir(pc.wrapping_add(2), 8)
            }
            // ld b,0
            0x4E => {
                self.contador = 0;
                self.seguir(0x50, 7)
            }
            // ld c,a
            0x50 => {
                self.passo = self.valor;
                self.seguir(0x51, 4)
            }
            // add iy,bc
            0x51 => {
                let bc = u16::from_le_bytes([self.passo, self.contador]);
                self.tabela_dac = self.tabela_dac.wrapping_add(bc);
                self.seguir(0x53, 15)
            }
            // ld e,(iy+0)
            0x53 => {
                let b = self.ler(amb, self.tabela_dac);
                self.posicao = u16::from_le_bytes([b, alto(self.posicao)]);
                self.seguir(0x56, 19)
            }
            // ld d,(iy+1)
            0x56 => {
                let b = self.ler(amb, self.tabela_dac.wrapping_add(1));
                self.posicao = u16::from_le_bytes([baixo(self.posicao), b]);
                self.seguir(0x59, 19)
            }
            // ld c,(iy+2)
            0x59 => {
                let b = self.ler(amb, self.tabela_dac.wrapping_add(2));
                self.restante = u16::from_le_bytes([b, alto(self.restante)]);
                self.seguir(0x5C, 19)
            }
            // ld b,(iy+3)
            0x5C => {
                let b = self.ler(amb, self.tabela_dac.wrapping_add(3));
                self.restante = u16::from_le_bytes([baixo(self.restante), b]);
                self.seguir(0x5F, 19)
            }
            // exx
            0x5F | 0x74 | 0x82 | 0x90 | 0x98 | 0xA6 => self.seguir(pc.wrapping_add(1), 4),
            // ld d,$80
            0x60 => {
                self.acumulador_pcm = Self::imediato8(amb, 0x61);
                self.seguir(0x62, 7)
            }
            // ld hl,zDAC_Status
            0x62 => {
                self.ponteiro_status = Self::imediato16(amb, 0x63);
                self.seguir(0x65, 10)
            }
            // ld (hl),d: o driver não aceita comandos
            0x65 => {
                let (a, v) = (self.ponteiro_status, self.acumulador_pcm);
                self.escrever(amb, a, v);
                self.seguir(0x66, 7)
            }
            // ld (ix+0),$2B
            0x66 => {
                let (a, v) = (self.base_ym, Self::imediato8(amb, 0x69));
                self.escrever(amb, a, v);
                self.seguir(0x6A, 19)
            }
            // ld e,$2A
            0x6A => {
                self.comando_dac = Self::imediato8(amb, 0x6B);
                self.seguir(0x6C, 7)
            }
            // ld c,(iy+4): o pitch da amostra
            0x6C => {
                self.passo = self.ler(amb, self.tabela_dac.wrapping_add(4));
                self.seguir(0x6F, 19)
            }
            // ld (ix+1),d: liga o DAC
            0x6F => {
                let (a, v) = (self.base_ym.wrapping_add(1), self.acumulador_pcm);
                self.escrever(amb, a, v);
                self.seguir(0x72, 19)
            }
            // ld (hl),0: aceita comandos
            0x72 => {
                let a = self.ponteiro_status;
                self.escrever(amb, a, 0);
                self.seguir(0x74, 10)
            }
            // ld h,(tabela de decodificação)>>8
            0x75 => {
                let h = Self::imediato8(amb, 0x76);
                self.ponteiro = u16::from_le_bytes([baixo(self.ponteiro), h]);
                self.seguir(0x77, 7)
            }
            // zPlayPCMLoop: ld a,(de)
            0x77 | 0x91 => {
                self.valor = self.ler(amb, self.posicao);
                self.seguir(pc.wrapping_add(1), 7)
            }
            // and $F0
            0x78 => {
                self.valor &= Self::imediato8(amb, 0x79);
                self.seguir(0x7A, 7)
            }
            // rrca (quatro vezes)
            0x7A..=0x7D => {
                self.valor = self.valor.rotate_right(1);
                self.seguir(pc.wrapping_add(1), 4)
            }
            // add a,(parte baixa da tabela)
            0x7E => {
                self.valor = self.valor.wrapping_add(Self::imediato8(amb, 0x7F));
                self.seguir(0x80, 7)
            }
            // ld l,a
            0x80 | 0x96 => {
                self.ponteiro = u16::from_le_bytes([self.valor, alto(self.ponteiro)]);
                self.seguir(pc.wrapping_add(1), 4)
            }
            // ld a,(hl)
            0x81 | 0x97 => {
                self.valor = self.ler(amb, self.ponteiro);
                self.seguir(pc.wrapping_add(1), 7)
            }
            // add a,d
            0x83 | 0x99 => {
                self.valor = self.valor.wrapping_add(self.acumulador_pcm);
                self.seguir(pc.wrapping_add(1), 4)
            }
            // ld d,a
            0x84 | 0x9A => {
                self.acumulador_pcm = self.valor;
                self.seguir(pc.wrapping_add(1), 4)
            }
            // ld (hl),l: marca ocupado (parte baixa do endereço do status)
            0x85 | 0x9B => {
                let (a, v) = (self.ponteiro_status, baixo(self.ponteiro_status));
                self.escrever(amb, a, v);
                self.seguir(pc.wrapping_add(1), 7)
            }
            // ld (ix+0),e: seleciona o registrador 2A
            0x86 | 0x9C => {
                let (a, v) = (self.base_ym, self.comando_dac);
                self.escrever(amb, a, v);
                self.seguir(pc.wrapping_add(3), 19)
            }
            // ld (ix+1),d: envia a amostra
            0x89 | 0x9F => {
                let (a, v) = (self.base_ym.wrapping_add(1), self.acumulador_pcm);
                self.escrever(amb, a, v);
                self.seguir(pc.wrapping_add(3), 19)
            }
            // ld (hl),h: libera (parte alta do endereço do status)
            0x8C | 0xA2 => {
                let (a, v) = (self.ponteiro_status, alto(self.ponteiro_status));
                self.escrever(amb, a, v);
                self.seguir(pc.wrapping_add(1), 7)
            }
            // ld b,c
            0x8D | 0xA3 => {
                self.contador = self.passo;
                self.seguir(pc.wrapping_add(1), 4)
            }
            // djnz $ (laço de pitch)
            0x8E | 0xA4 | 0xCA => {
                self.contador = self.contador.wrapping_sub(1);
                if self.contador != 0 {
                    self.seguir(pc, 13)
                } else {
                    self.seguir(pc.wrapping_add(2), 8)
                }
            }
            // and $0F
            0x92 => {
                self.valor &= Self::imediato8(amb, 0x93);
                self.seguir(0x94, 7)
            }
            // add a,(parte baixa da tabela)
            0x94 => {
                self.valor = self.valor.wrapping_add(Self::imediato8(amb, 0x95));
                self.seguir(0x96, 7)
            }
            // ld a,(zDAC_Sample)
            0xA7 => {
                self.valor = self.ler(amb, Self::imediato16(amb, 0xA8));
                self.seguir(0xAA, 13)
            }
            // bit 7,a
            0xAA => self.seguir(0xAC, 8),
            // jp nz,zCheckForSamples (chegou um comando novo)
            0xAC => {
                if self.valor & 0x80 != 0 {
                    self.seguir(0x32, 10)
                } else {
                    self.seguir(0xAF, 10)
                }
            }
            // inc de
            0xAF | 0xCC => {
                self.posicao = self.posicao.wrapping_add(1);
                self.seguir(pc.wrapping_add(1), 6)
            }
            // dec bc
            0xB0 => {
                self.restante = self.restante.wrapping_sub(1);
                self.seguir(0xB1, 6)
            }
            // ld a,c
            0xB1 => {
                self.valor = baixo(self.restante);
                self.seguir(0xB2, 4)
            }
            // or b
            0xB2 => {
                self.valor |= alto(self.restante);
                self.seguir(0xB3, 4)
            }
            // jp nz,zPlayPCMLoop
            0xB3 => {
                if self.valor != 0 {
                    self.seguir(0x77, 10)
                } else {
                    self.seguir(0xB6, 10)
                }
            }
            // jp zCheckForSamples
            0xB6 | 0xD3 => self.seguir(0x32, 10),
            // zPlay_SegaPCM: ld de,(posição da voz SEGA na janela)
            0xB9 => {
                self.posicao = Self::imediato16(amb, 0xBA);
                self.seguir(0xBC, 10)
            }
            // ld hl,(tamanho da voz SEGA)
            0xBC => {
                self.ponteiro = Self::imediato16(amb, 0xBD);
                self.seguir(0xBF, 10)
            }
            // ld c,$2A
            0xBF => {
                self.passo = Self::imediato8(amb, 0xC0);
                self.seguir(0xC1, 7)
            }
            // zPlaySEGAPCMLoop: ld a,(de)
            0xC1 => {
                self.valor = self.ler(amb, self.posicao);
                self.seguir(0xC2, 7)
            }
            // ld (ix+0),c
            0xC2 => {
                let (a, v) = (self.base_ym, self.passo);
                self.escrever(amb, a, v);
                self.seguir(0xC5, 19)
            }
            // ld (ix+1),a
            0xC5 => {
                let (a, v) = (self.base_ym.wrapping_add(1), self.valor);
                self.escrever(amb, a, v);
                self.seguir(0xC8, 19)
            }
            // ld b,(pitch da voz SEGA)
            0xC8 => {
                self.contador = Self::imediato8(amb, 0xC9);
                self.seguir(0xCA, 7)
            }
            // dec hl
            0xCD => {
                self.ponteiro = self.ponteiro.wrapping_sub(1);
                self.seguir(0xCE, 6)
            }
            // ld a,l
            0xCE => {
                self.valor = baixo(self.ponteiro);
                self.seguir(0xCF, 4)
            }
            // or h
            0xCF => {
                self.valor |= alto(self.ponteiro);
                self.seguir(0xD0, 4)
            }
            // jp nz,zPlaySEGAPCMLoop
            0xD0 => {
                if self.valor != 0 {
                    self.seguir(0xC1, 10)
                } else {
                    self.seguir(0xD3, 10)
                }
            }
            _ => self.trecho_desconhecido(),
        }
    }

    /// Define o próximo trecho e devolve o custo.
    fn seguir(&mut self, proximo: u16, custo: u32) -> u32 {
        self.pc = proximo;
        custo
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn amb<'a>(
        ram: &'a mut [u8; TAMANHO_RAM],
        saidas: &'a mut Vec<Saida>,
        busreq: bool,
        reset: bool,
    ) -> Ambiente<'a> {
        Ambiente {
            ram,
            rom: &[],
            busreq,
            reset,
            saidas,
        }
    }

    #[test]
    fn banco_forma_o_valor_com_nove_escritas() {
        // Escreve 0b1_0000_1111 do bit menos significativo para o mais significativo.
        let alvo: u16 = 0b1_0000_1111;
        let mut b = Banco::default();
        for i in 0..9 {
            b.escrever_bit((alvo >> i) & 1 != 0);
        }
        assert_eq!(b.valor(), alvo);
        assert_eq!(b.endereco_68000(0x8123), (u32::from(alvo) << 15) | 0x0123);
    }

    #[test]
    fn em_reset_o_pc_volta_a_zero_e_nada_executa() {
        let mut m = MaquinaZ80::nova();
        let (mut ram, mut s) = (Box::new([0u8; TAMANHO_RAM]), Vec::new());
        m.pc = 0x35;
        m.avancar(&mut amb(&mut ram, &mut s, false, true), 1000);
        assert_eq!(m.pc(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn avanco_parado_por_barramento_consome_ciclos_em_tempo_constante() {
        let mut m = MaquinaZ80::nova();
        let (mut ram, mut s) = (Box::new([0u8; TAMANHO_RAM]), Vec::new());
        ram[0] = 0xF3;
        m.avancar(&mut amb(&mut ram, &mut s, true, false), 7_870_947);
        // Parado: o PC não anda, e o resto dos ciclos é gasto em esperas de tentativa.
        assert_eq!(m.pc(), 0);
        assert!(m.espera() <= 3);
    }

    #[test]
    fn dividir_um_avanco_em_dois_da_o_mesmo_estado() {
        // Propriedade: avancar(a + b) == avancar(a); avancar(b), em qualquer ponto.
        let mut ram0 = Box::new([0u8; TAMANHO_RAM]);
        ram0[0] = 0xAF;
        ram0[2] = 0x07; // bc = 7
        ram0[5] = 0x40; // de = 0x4000? (destino dentro da RAM por espelho)
        ram0[8] = 0x10;
        for corte in [0u64, 1, 3, 4, 10, 25, 61, 100] {
            let mut a = MaquinaZ80::nova();
            let mut b = MaquinaZ80::nova();
            let (mut ra, mut rb) = (ram0.clone(), ram0.clone());
            let (mut sa, mut sb) = (Vec::new(), Vec::new());
            a.avancar(&mut amb(&mut ra, &mut sa, false, false), 300);
            b.avancar(&mut amb(&mut rb, &mut sb, false, false), corte);
            b.avancar(&mut amb(&mut rb, &mut sb, false, false), 300 - corte);
            assert_eq!(a, b, "corte {corte}");
            assert_eq!(ra[..], rb[..]);
        }
    }
}
