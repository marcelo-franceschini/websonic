//! Memória espelhada do console (sub-etapa 5.1).
//!
//! O núcleo guarda a RAM do 68000, a RAM do Z80, o estado das portas de controle e os dois bits
//! do barramento do Z80. Não há VDP, YM2612 nem PSG aqui: escrever neles produz [`Saida`], que a
//! camada de vídeo e de som consome. Leituras de estado de hardware (BUSACK, estado da VDP,
//! estado do YM2612) vêm do tempo gravado, na ordem em que o oráculo as viu.
//!
//! O mapa e o tratamento de acesso inválido reproduzem o oráculo (ver
//! `docs/conhecimento/nucleo.md`). Nenhum dado da ROM mora aqui: a ROM entra por [`Memoria::nova`].
//! Rotinas do disassembly que este módulo espelha: nenhuma (é o próprio barramento).

pub mod porta;

use crate::quadro::tempo::AcessoZ80;
use crate::som::z80::{Ambiente, DivergenciaZ80, MaquinaZ80, PontoZ80};
use porta::{Dispositivo, Porta};
use std::collections::{BTreeMap, VecDeque};

/// Tamanho da RAM do 68000.
pub const TAMANHO_RAM: usize = 0x1_0000;
/// Tamanho da RAM do Z80.
pub const TAMANHO_RAM_Z80: usize = 0x2000;

/// Tamanho de um acesso.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tamanho {
    Byte,
    Word,
    Long,
}

/// Erro de acesso. O 68000 trata palavra ou longa em endereço ímpar como exceção (vetor 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroAcesso {
    EnderecoImpar(u32),
}

/// Coisas que o jogo não deveria fazer, ou que o núcleo ainda não modela. Cada uma é contada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Aviso {
    LeituraSemDispositivo,
    EscritaSemDispositivo,
    EscritaNaRom,
    /// Acesso à janela do Z80 sem o barramento cedido.
    Z80SemBarramento,
    /// Acesso a uma parte da janela do Z80 que o núcleo não modela.
    Z80JanelaNaoModelada,
    /// Leitura da porta de dados da VDP ou do contador HV (o jogo não faz).
    VdpLeituraNaoModelada,
    /// Escrita no registrador de banco do Z80 pelo 68000.
    Z80Banco,
    /// Leitura de hardware sem valor gravado na fila (o último valor foi repetido).
    LeituraSemGravacao,
}

/// Classe de leitura de hardware que depende do tempo e vem do arquivo gravado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClasseLeitura {
    Busack,
    VdpStatus,
    YmStatus,
}

/// Porta da VDP escrita pelo 68000.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortaVdp {
    Dados,
    Controle,
}

/// Efeito de uma escrita do 68000 que sai do núcleo. Mesma lógica dos eventos 5 a 8 do trace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Saida {
    Vdp {
        porta: PortaVdp,
        valor: u16,
        tamanho: Tamanho,
    },
    Psg(u8),
    /// Escrita do Z80 no YM2612 (o programa do DAC). `porta` é o endereço baixo (0 a 3).
    YmZ80 {
        porta: u8,
        valor: u8,
    },
    /// Escrita do 68000 no YM2612 pela janela do Z80. `porta` é o endereço baixo (0 a 3).
    Ym {
        porta: u8,
        valor: u8,
    },
    /// Escrita em BUSREQ: `true` pede o barramento.
    BusReq(bool),
    /// Escrita em RESET do Z80: `true` segura o Z80 em reset.
    ResetZ80(bool),
}

/// A memória.
#[derive(Clone, Debug)]
pub struct Memoria {
    rom: Vec<u8>,
    ram: Box<[u8; TAMANHO_RAM]>,
    ram_z80: Box<[u8; TAMANHO_RAM_Z80]>,
    busreq: bool,
    reset_z80: bool,
    portas: [Porta; 3],
    barramento_aberto: u16,
    saidas: Vec<Saida>,
    avisos: BTreeMap<Aviso, u64>,
    gravadas: BTreeMap<ClasseLeitura, VecDeque<u16>>,
    ultima_gravada: BTreeMap<ClasseLeitura, u16>,
    z80: MaquinaZ80,
    pontos_z80: VecDeque<PontoZ80>,
    divergencias_z80: Vec<DivergenciaZ80>,
}

fn indice(endereco: u32, mascara: u32) -> usize {
    usize::try_from(endereco & mascara).unwrap_or(0)
}

impl Memoria {
    /// Console de referência: controle de 3 botões na porta 1, nada na porta 2 e na extensão,
    /// Z80 em reset e sem pedido de barramento, RAM zerada (como o oráculo).
    pub fn nova(rom: Vec<u8>) -> Self {
        Self {
            rom,
            ram: Box::new([0; TAMANHO_RAM]),
            ram_z80: Box::new([0; TAMANHO_RAM_Z80]),
            busreq: false,
            reset_z80: true,
            portas: [
                Porta::nova(Dispositivo::TresBotoes(0)),
                Porta::nova(Dispositivo::Nada),
                Porta::nova(Dispositivo::Nada),
            ],
            barramento_aberto: 0,
            saidas: Vec::new(),
            avisos: BTreeMap::new(),
            gravadas: BTreeMap::new(),
            ultima_gravada: BTreeMap::new(),
            z80: MaquinaZ80::nova(),
            pontos_z80: VecDeque::new(),
            divergencias_z80: Vec::new(),
        }
    }

    /// Os bytes da ROM carregada.
    pub fn rom(&self) -> &[u8] {
        &self.rom
    }

    pub fn ram(&self) -> &[u8; TAMANHO_RAM] {
        &self.ram
    }

    pub fn ram_mut(&mut self) -> &mut [u8; TAMANHO_RAM] {
        &mut self.ram
    }

    pub fn ram_z80(&self) -> &[u8; TAMANHO_RAM_Z80] {
        &self.ram_z80
    }

    /// Botões do controle 1 no formato do corpus.
    pub fn definir_botoes(&mut self, mascara: u8) {
        self.portas[0].definir_botoes(mascara);
    }

    /// O Z80 tem o barramento do 68000 liberado para o 68000 usar a janela dele.
    pub fn busack(&self) -> bool {
        self.busreq && !self.reset_z80
    }

    /// Saídas produzidas desde a última chamada de [`Memoria::tirar_saidas`].
    pub fn saidas(&self) -> &[Saida] {
        &self.saidas
    }

    pub fn tirar_saidas(&mut self) -> Vec<Saida> {
        std::mem::take(&mut self.saidas)
    }

    pub fn avisos(&self) -> &BTreeMap<Aviso, u64> {
        &self.avisos
    }

    /// Pontos do Z80 gravados pelo oráculo (tempo gravado, nível 1), na ordem. Cada acesso do 68000
    /// que toca o Z80 consome o próximo: avança a máquina do Z80 pelos ciclos gravados e confere
    /// onde ela está.
    pub fn enfileirar_ponto_z80(&mut self, p: PontoZ80) {
        self.pontos_z80.push_back(p);
    }

    pub fn pontos_z80_pendentes(&self) -> usize {
        self.pontos_z80.len()
    }

    pub fn divergencias_z80(&self) -> &[DivergenciaZ80] {
        &self.divergencias_z80
    }

    pub fn z80(&self) -> &MaquinaZ80 {
        &self.z80
    }

    pub fn z80_mut(&mut self) -> &mut MaquinaZ80 {
        &mut self.z80
    }

    /// Avança o Z80 por `ciclos` ciclos com o estado atual de BUSREQ e RESET.
    pub fn avancar_z80(&mut self, ciclos: u64) {
        let mut amb = Ambiente {
            ram: &mut self.ram_z80,
            rom: &self.rom,
            busreq: self.busreq,
            reset: self.reset_z80,
            saidas: &mut self.saidas,
        };
        self.z80.avancar(&mut amb, ciclos);
    }

    /// Estado dos sinais do Z80 (para começar de um estado salvo do oráculo).
    pub fn definir_estado_z80(&mut self, busreq: bool, reset: bool) {
        self.busreq = busreq;
        self.reset_z80 = reset;
    }

    pub fn ram_z80_mut(&mut self) -> &mut [u8; TAMANHO_RAM_Z80] {
        &mut self.ram_z80
    }

    /// Aplica um ponto do Z80 que veio de um acesso feito por código que o núcleo não roda (por
    /// exemplo, o resto da rotina de VBlank): avança a máquina, confere a posição e aplica o
    /// efeito do acesso nos sinais e na RAM do Z80.
    pub fn aplicar_ponto_externo(&mut self, p: &PontoZ80) {
        self.avancar_z80(u64::from(p.ativos));
        let obtido = (
            self.z80.pc(),
            u16::try_from(self.z80.espera()).unwrap_or(u16::MAX),
            self.z80.banco().valor(),
        );
        if obtido != (p.pc, p.espera, p.banco) {
            self.divergencias_z80.push(DivergenciaZ80::Posicao {
                acesso: p.acesso,
                esperado: (p.pc, p.espera, p.banco),
                obtido,
            });
        }
        match p.acesso {
            AcessoZ80::BusReqPedido => self.busreq = true,
            AcessoZ80::BusReqLiberado => self.busreq = false,
            AcessoZ80::ResetLigado => self.reset_z80 = true,
            AcessoZ80::ResetSolto => self.reset_z80 = false,
            AcessoZ80::EscritaJanela if p.endereco < 0x4000 => {
                self.ram_z80[indice(u32::from(p.endereco), 0x1FFF)] = p.byte;
            }
            AcessoZ80::EscritaJanela | AcessoZ80::LeituraJanela => {}
        }
    }

    fn sincronizar_z80(&mut self, acesso: AcessoZ80) {
        let Some(p) = self.pontos_z80.pop_front() else {
            return;
        };
        if p.acesso != acesso {
            self.divergencias_z80.push(DivergenciaZ80::AcessoDiferente {
                esperado: p.acesso,
                obtido: acesso,
            });
            return;
        }
        self.avancar_z80(u64::from(p.ativos));
        let obtido = (
            self.z80.pc(),
            u16::try_from(self.z80.espera()).unwrap_or(u16::MAX),
            self.z80.banco().valor(),
        );
        let esperado = (p.pc, p.espera, p.banco);
        if obtido != esperado {
            self.divergencias_z80.push(DivergenciaZ80::Posicao {
                acesso,
                esperado,
                obtido,
            });
        }
    }

    fn avisar(&mut self, aviso: Aviso) {
        let c = self.avisos.entry(aviso).or_insert(0);
        *c = c.saturating_add(1);
    }

    /// Acrescenta uma leitura de hardware gravada pelo oráculo (palavra, ou byte no menos
    /// significativo para YM).
    pub fn enfileirar_leitura(&mut self, classe: ClasseLeitura, valor: u16) {
        self.gravadas.entry(classe).or_default().push_back(valor);
    }

    /// Quantas leituras gravadas ainda não foram consumidas.
    pub fn leituras_pendentes(&self) -> usize {
        self.gravadas.values().map(VecDeque::len).sum()
    }

    fn leitura_gravada(&mut self, classe: ClasseLeitura) -> u16 {
        if let Some(v) = self.gravadas.get_mut(&classe).and_then(VecDeque::pop_front) {
            self.ultima_gravada.insert(classe, v);
            return v;
        }
        self.avisar(Aviso::LeituraSemGravacao);
        self.ultima_gravada.get(&classe).copied().unwrap_or(0)
    }

    fn byte_da_rom(&self, endereco: u32) -> Option<u8> {
        self.rom.get(indice(endereco, 0x3F_FFFF)).copied()
    }

    // ---- leitura ----

    /// Lê `tamanho` bytes em `endereco` (24 bits; os 8 bits altos são ignorados).
    /// Longa é feita como duas palavras, a alta primeiro, como o barramento do 68000.
    pub fn ler(&mut self, endereco: u32, tamanho: Tamanho) -> Result<u32, ErroAcesso> {
        let a = endereco & 0xFF_FFFF;
        match tamanho {
            Tamanho::Byte => Ok(u32::from(self.ler_byte(a))),
            Tamanho::Word => {
                if a & 1 != 0 {
                    return Err(ErroAcesso::EnderecoImpar(a));
                }
                Ok(u32::from(self.ler_palavra(a)))
            }
            Tamanho::Long => {
                if a & 1 != 0 {
                    return Err(ErroAcesso::EnderecoImpar(a));
                }
                let alta = u32::from(self.ler_palavra(a));
                let baixa = u32::from(self.ler_palavra(a.wrapping_add(2) & 0xFF_FFFF));
                Ok((alta << 16) | baixa)
            }
        }
    }

    fn ler_byte(&mut self, a: u32) -> u8 {
        let valor = self.ler_regiao(a, false);
        let [_, baixo] = valor.to_be_bytes();
        // O barramento aberto guarda o último byte lido no byte alto (como o oráculo).
        let [_, b] = self.barramento_aberto.to_be_bytes();
        self.barramento_aberto = u16::from_be_bytes([baixo, b]);
        baixo
    }

    fn ler_palavra(&mut self, a: u32) -> u16 {
        let v = self.ler_regiao(a, true);
        self.barramento_aberto = v;
        v
    }

    fn aberto(&self, a: u32, palavra: bool) -> u16 {
        if palavra {
            self.barramento_aberto
        } else {
            let [alto, baixo] = self.barramento_aberto.to_be_bytes();
            u16::from(if a & 1 == 0 { alto } else { baixo })
        }
    }

    /// Leitura de ROM além do tamanho do arquivo: o oráculo devolve o barramento aberto.
    fn rom_ausente(&mut self, a: u32, palavra: bool) -> u16 {
        self.avisar(Aviso::LeituraSemDispositivo);
        self.aberto(a, palavra)
    }

    /// Define o valor que o barramento aberto devolve. Serve aos testes: no oráculo esse valor é
    /// a última palavra buscada pelo 68000, e o núcleo não busca instruções.
    pub fn definir_barramento_aberto(&mut self, valor: u16) {
        self.barramento_aberto = valor;
    }

    fn ler_regiao(&mut self, a: u32, palavra: bool) -> u16 {
        match a {
            0x00_0000..=0x3F_FFFF => {
                let alto = self.byte_da_rom(a);
                if palavra {
                    let baixo = self.byte_da_rom(a | 1);
                    match (alto, baixo) {
                        (Some(x), Some(y)) => u16::from_be_bytes([x, y]),
                        _ => self.rom_ausente(a, true),
                    }
                } else {
                    match alto {
                        Some(x) => u16::from(x),
                        None => self.rom_ausente(a, false),
                    }
                }
            }
            0xA0_0000..=0xA0_FFFF if self.busack() => {
                self.sincronizar_z80(AcessoZ80::LeituraJanela);
                let b = self.ler_janela_z80(a);
                if palavra {
                    u16::from_be_bytes([b, b])
                } else {
                    u16::from(b)
                }
            }
            0xA1_0000..=0xA1_001F => {
                let b = self.ler_io(a | 1);
                if palavra {
                    u16::from_be_bytes([b, b])
                } else {
                    u16::from(b)
                }
            }
            0xA1_1100 => {
                let v = self.leitura_gravada(ClasseLeitura::Busack);
                if palavra {
                    v
                } else {
                    u16::from(v.to_be_bytes()[0])
                }
            }
            0xC0_0000..=0xC0_001F => self.ler_vdp(a, palavra),
            0xE0_0000..=0xFF_FFFF => {
                let i = indice(a, 0xFFFF);
                if palavra {
                    u16::from_be_bytes([self.ram[i], self.ram[i | 1]])
                } else {
                    u16::from(self.ram[i])
                }
            }
            _ => {
                self.avisar(Aviso::LeituraSemDispositivo);
                self.aberto(a, palavra)
            }
        }
    }

    fn ler_janela_z80(&mut self, a: u32) -> u8 {
        // Para o 68000, $8000 a $FFFF espelha $0000 a $7FFF.
        match a & 0x7FFF {
            0x0000..=0x3FFF => self.ram_z80[indice(a, 0x1FFF)],
            0x4000..=0x5FFF => {
                let v = self.leitura_gravada(ClasseLeitura::YmStatus);
                v.to_be_bytes()[1]
            }
            0x6000..=0x7EFF => 0xFF,
            _ => {
                self.avisar(Aviso::Z80JanelaNaoModelada);
                0xFF
            }
        }
    }

    fn ler_io(&self, a: u32) -> u8 {
        match a {
            // Versão: bit 7 região (0 = Japão), bit 6 PAL (0), bit 5 sem Sega CD (1).
            0xA1_0001 => 0x20,
            0xA1_0003 => self.portas[0].ler_dado(),
            0xA1_0005 => self.portas[1].ler_dado(),
            0xA1_0007 => self.portas[2].ler_dado(),
            0xA1_0009 => self.portas[0].ler_controle(),
            0xA1_000B => self.portas[1].ler_controle(),
            0xA1_000D => self.portas[2].ler_controle(),
            0xA1_000F => self.portas[0].ler_tx(),
            0xA1_0015 => self.portas[1].ler_tx(),
            0xA1_001B => self.portas[2].ler_tx(),
            _ => 0,
        }
    }

    fn ler_vdp(&mut self, a: u32, palavra: bool) -> u16 {
        let w = match a & 0x1F {
            0x00..=0x03 | 0x08..=0x0B => {
                self.avisar(Aviso::VdpLeituraNaoModelada);
                0
            }
            0x04..=0x07 => self.leitura_gravada(ClasseLeitura::VdpStatus),
            _ => self.barramento_aberto,
        };
        if palavra {
            w
        } else {
            u16::from(w.to_be_bytes()[usize::from(a & 1 != 0)])
        }
    }

    // ---- escrita ----

    /// Escreve `valor` (nos bits baixos, conforme o tamanho) em `endereco`.
    pub fn escrever(
        &mut self,
        endereco: u32,
        tamanho: Tamanho,
        valor: u32,
    ) -> Result<(), ErroAcesso> {
        let a = endereco & 0xFF_FFFF;
        match tamanho {
            Tamanho::Byte => {
                self.escrever_regiao(a, u16::from(valor.to_be_bytes()[3]), false);
                Ok(())
            }
            Tamanho::Word => {
                if a & 1 != 0 {
                    return Err(ErroAcesso::EnderecoImpar(a));
                }
                let [_, _, h, l] = valor.to_be_bytes();
                self.escrever_regiao(a, u16::from_be_bytes([h, l]), true);
                Ok(())
            }
            Tamanho::Long => {
                if a & 1 != 0 {
                    return Err(ErroAcesso::EnderecoImpar(a));
                }
                let [a0, a1, b0, b1] = valor.to_be_bytes();
                self.escrever_regiao(a, u16::from_be_bytes([a0, a1]), true);
                self.escrever_regiao(
                    a.wrapping_add(2) & 0xFF_FFFF,
                    u16::from_be_bytes([b0, b1]),
                    true,
                );
                Ok(())
            }
        }
    }

    fn escrever_regiao(&mut self, a: u32, valor: u16, palavra: bool) {
        let [alto, baixo] = valor.to_be_bytes();
        match a {
            0x00_0000..=0x3F_FFFF => self.avisar(Aviso::EscritaNaRom),
            0xA0_0000..=0xA0_FFFF if self.busack() => {
                // Escrita em palavra grava o byte alto, como o oráculo.
                self.sincronizar_z80(AcessoZ80::EscritaJanela);
                let b = if palavra { alto } else { baixo };
                self.escrever_janela_z80(a, b);
            }
            0xA1_0000..=0xA1_001F => self.escrever_io(a | 1, baixo),
            0xA1_1100 => {
                let pedido = if palavra {
                    alto & 1 != 0
                } else {
                    baixo & 1 != 0
                };
                self.sincronizar_z80(if pedido {
                    AcessoZ80::BusReqPedido
                } else {
                    AcessoZ80::BusReqLiberado
                });
                self.busreq = pedido;
                self.saidas.push(Saida::BusReq(self.busreq));
            }
            0xA1_1200 => {
                let liberado = if palavra {
                    alto & 1 != 0
                } else {
                    baixo & 1 != 0
                };
                self.sincronizar_z80(if liberado {
                    AcessoZ80::ResetSolto
                } else {
                    AcessoZ80::ResetLigado
                });
                self.reset_z80 = !liberado;
                self.saidas.push(Saida::ResetZ80(self.reset_z80));
            }
            0xC0_0000..=0xC0_001F => self.escrever_vdp(a, valor, palavra),
            0xE0_0000..=0xFF_FFFF => {
                let i = indice(a, 0xFFFF);
                if palavra {
                    self.ram[i] = alto;
                    self.ram[i | 1] = baixo;
                } else {
                    self.ram[i] = baixo;
                }
            }
            _ => self.avisar(Aviso::EscritaSemDispositivo),
        }
    }

    fn escrever_janela_z80(&mut self, a: u32, b: u8) {
        match a & 0x7FFF {
            0x0000..=0x3FFF => self.ram_z80[indice(a, 0x1FFF)] = b,
            0x4000..=0x5FFF => self.saidas.push(Saida::Ym {
                porta: b_porta(a),
                valor: b,
            }),
            0x6000..=0x60FF => self.avisar(Aviso::Z80Banco),
            0x6100..=0x7EFF | 0x7F20..=0x7FFF => {}
            _ => self.avisar(Aviso::Z80JanelaNaoModelada),
        }
    }

    fn escrever_io(&mut self, a: u32, v: u8) {
        match a {
            0xA1_0003 => self.portas[0].escrever_dado(v),
            0xA1_0005 => self.portas[1].escrever_dado(v),
            0xA1_0007 => self.portas[2].escrever_dado(v),
            0xA1_0009 => self.portas[0].escrever_controle(v),
            0xA1_000B => self.portas[1].escrever_controle(v),
            0xA1_000D => self.portas[2].escrever_controle(v),
            0xA1_000F => self.portas[0].escrever_tx(v),
            0xA1_0015 => self.portas[1].escrever_tx(v),
            0xA1_001B => self.portas[2].escrever_tx(v),
            _ => {}
        }
    }

    fn escrever_vdp(&mut self, a: u32, valor: u16, palavra: bool) {
        // Escrita em byte na VDP repete o byte nos dois lados (o evento guarda o byte).
        let (v, t) = if palavra {
            (valor, Tamanho::Word)
        } else {
            (valor & 0x00FF, Tamanho::Byte)
        };
        match a & 0x1F {
            0x00..=0x03 => self.saidas.push(Saida::Vdp {
                porta: PortaVdp::Dados,
                valor: v,
                tamanho: t,
            }),
            0x04..=0x07 => self.saidas.push(Saida::Vdp {
                porta: PortaVdp::Controle,
                valor: v,
                tamanho: t,
            }),
            0x10..=0x17 if palavra || a & 1 != 0 => {
                self.saidas.push(Saida::Psg(valor.to_be_bytes()[1]))
            }
            _ => {}
        }
    }
}

fn b_porta(a: u32) -> u8 {
    u8::try_from(a & 3).unwrap_or(0)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn mem() -> Memoria {
        Memoria::nova(vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])
    }

    #[test]
    fn ram_e_big_endian_e_espelha_a_cada_64k() {
        let mut m = mem();
        m.escrever(0xFF_0010, Tamanho::Long, 0xAABB_CCDD).unwrap();
        assert_eq!(m.ler(0xFF_0010, Tamanho::Byte).unwrap(), 0xAA);
        assert_eq!(m.ler(0xFF_0012, Tamanho::Word).unwrap(), 0xCCDD);
        assert_eq!(m.ler(0xFF_0010, Tamanho::Long).unwrap(), 0xAABB_CCDD);
        // Espelhos: $E00010, $EF0010 e $FF0010 são o mesmo byte.
        assert_eq!(m.ler(0xE0_0010, Tamanho::Byte).unwrap(), 0xAA);
        assert_eq!(m.ler(0xEF_0010, Tamanho::Byte).unwrap(), 0xAA);
        assert_eq!(m.ram()[0x10], 0xAA);
    }

    #[test]
    fn palavra_e_longa_em_endereco_impar_dao_erro() {
        let mut m = mem();
        assert_eq!(
            m.ler(0xFF_0001, Tamanho::Word),
            Err(ErroAcesso::EnderecoImpar(0xFF_0001))
        );
        assert_eq!(
            m.ler(0xFF_0003, Tamanho::Long),
            Err(ErroAcesso::EnderecoImpar(0xFF_0003))
        );
        assert_eq!(
            m.escrever(0xFF_0001, Tamanho::Word, 1),
            Err(ErroAcesso::EnderecoImpar(0xFF_0001))
        );
        // Byte em endereco impar e normal.
        assert!(m.ler(0xFF_0001, Tamanho::Byte).is_ok());
    }

    #[test]
    fn rom_so_leitura() {
        let mut m = mem();
        assert_eq!(m.ler(2, Tamanho::Word).unwrap(), 0x3344);
        m.escrever(2, Tamanho::Word, 0).unwrap();
        assert_eq!(m.ler(2, Tamanho::Word).unwrap(), 0x3344);
        assert_eq!(m.avisos().get(&Aviso::EscritaNaRom), Some(&1));
    }

    #[test]
    fn janela_do_z80_so_com_barramento() {
        let mut m = mem();
        m.escrever(0xA0_0010, Tamanho::Byte, 0x5A).unwrap();
        // Sem BUSREQ e com o Z80 em reset a escrita e ignorada.
        assert_eq!(m.ram_z80()[0x10], 0);
        // Libera o reset e pede o barramento.
        m.escrever(0xA1_1200, Tamanho::Word, 0x0100).unwrap();
        m.escrever(0xA1_1100, Tamanho::Word, 0x0100).unwrap();
        assert!(m.busack());
        m.escrever(0xA0_0010, Tamanho::Byte, 0x5A).unwrap();
        assert_eq!(m.ram_z80()[0x10], 0x5A);
        // Espelho em $2010 e em $8010.
        assert_eq!(m.ler(0xA0_2010, Tamanho::Byte).unwrap(), 0x5A);
        assert_eq!(m.ler(0xA0_8010, Tamanho::Byte).unwrap(), 0x5A);
        // Escrita em palavra grava o byte alto e a leitura em palavra repete o byte.
        m.escrever(0xA0_0012, Tamanho::Word, 0x7700).unwrap();
        assert_eq!(m.ram_z80()[0x12], 0x77);
        assert_eq!(m.ler(0xA0_0012, Tamanho::Word).unwrap(), 0x7777);
        assert_eq!(m.saidas(), &[Saida::ResetZ80(false), Saida::BusReq(true)]);
    }

    #[test]
    fn escritas_de_hardware_viram_saidas() {
        let mut m = mem();
        m.escrever(0xC0_0004, Tamanho::Word, 0x8F02).unwrap();
        m.escrever(0xC0_0000, Tamanho::Long, 0x1234_5678).unwrap();
        m.escrever(0xC0_0011, Tamanho::Byte, 0x9F).unwrap();
        assert_eq!(
            m.tirar_saidas(),
            vec![
                Saida::Vdp {
                    porta: PortaVdp::Controle,
                    valor: 0x8F02,
                    tamanho: Tamanho::Word
                },
                Saida::Vdp {
                    porta: PortaVdp::Dados,
                    valor: 0x1234,
                    tamanho: Tamanho::Word
                },
                Saida::Vdp {
                    porta: PortaVdp::Dados,
                    valor: 0x5678,
                    tamanho: Tamanho::Word
                },
                Saida::Psg(0x9F),
            ]
        );
        assert!(m.saidas().is_empty());
    }

    #[test]
    fn leituras_de_hardware_vem_da_fila_gravada() {
        let mut m = mem();
        m.enfileirar_leitura(ClasseLeitura::VdpStatus, 0x3400);
        m.enfileirar_leitura(ClasseLeitura::Busack, 0x0100);
        assert_eq!(m.ler(0xC0_0004, Tamanho::Word).unwrap(), 0x3400);
        assert_eq!(m.ler(0xA1_1100, Tamanho::Byte).unwrap(), 0x01);
        assert_eq!(m.leituras_pendentes(), 0);
        // Fila vazia: repete o ultimo valor e avisa.
        assert_eq!(m.ler(0xC0_0004, Tamanho::Word).unwrap(), 0x3400);
        assert_eq!(m.avisos().get(&Aviso::LeituraSemGravacao), Some(&1));
    }

    #[test]
    fn versao_e_portas_do_console_de_referencia() {
        let mut m = mem();
        assert_eq!(m.ler(0xA1_0001, Tamanho::Byte).unwrap(), 0x20);
        // Endereco par espelha o impar.
        assert_eq!(m.ler(0xA1_0000, Tamanho::Byte).unwrap(), 0x20);
        m.escrever(0xA1_0009, Tamanho::Byte, 0x40).unwrap();
        m.escrever(0xA1_0003, Tamanho::Byte, 0x40).unwrap();
        m.definir_botoes(0x01);
        assert_eq!(m.ler(0xA1_0003, Tamanho::Byte).unwrap() & 0x3F, 0x3E);
        // Porta 2 sem nada: tudo alto.
        assert_eq!(m.ler(0xA1_0005, Tamanho::Byte).unwrap(), 0xFF);
    }

    #[test]
    fn sem_dispositivo_le_barramento_aberto_e_ignora_escrita() {
        let mut m = mem();
        m.ler(0xFF_0000, Tamanho::Word).unwrap();
        m.escrever(0xFF_0000, Tamanho::Word, 0xBEEF).unwrap();
        m.ler(0xFF_0000, Tamanho::Word).unwrap();
        // Regiao sem dispositivo: devolve o ultimo valor do barramento.
        assert_eq!(m.ler(0x50_0000, Tamanho::Word).unwrap(), 0xBEEF);
        m.escrever(0x50_0000, Tamanho::Word, 1).unwrap();
        assert_eq!(m.avisos().get(&Aviso::LeituraSemDispositivo), Some(&1));
        assert_eq!(m.avisos().get(&Aviso::EscritaSemDispositivo), Some(&1));
    }
}
