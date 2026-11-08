//! Abertura do jogo (Etapa 7): do ligar do console à primeira demo.
//!
//! Cada modo do jogo é uma máquina de estados. O laço principal do original roda como trechos
//! que terminam numa **marca**: um endereço de rótulo que o oráculo viu o 68000 alcançar. O
//! tempo gravado (nível 1) diz em que quadro cada marca chegou, e o trecho roda nesse quadro.
//! Interrupções gravadas rodam o tratador correspondente. Isto é infraestrutura e não tem par no
//! original. Cada rotina de `boot` cita o rótulo do disassembly que reproduz.
//!
//! Limite (D7.10 do plano): um trecho é atômico. Seu efeito aparece de uma vez, no quadro da marca
//! final. O comparador separa os quadros dentro de um trecho atômico dos demais.

pub mod boot;
pub mod ram;
pub mod sega;
pub mod vblank;

use crate::memoria::{ClasseLeitura, ErroAcesso, Memoria, Tamanho};
use crate::som::z80::PontoZ80;

/// Pontos do original que o oráculo marca (chegada do PC ao endereço do rótulo).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Marca {
    /// `GameProgram`: fim da preparação do console e começo da verificação da ROM.
    GameProgram,
    /// `CheckSumOk`: fim da verificação da ROM.
    ChecksumOk,
    /// `VDPSetupGame`: a limpeza da RAM de `GameInit` acabou.
    VdpSetupGame,
    /// `DACDriverLoad`: o `VDPSetupGame` acabou.
    DacDriverLoad,
    /// `JoypadInit`: a carga do programa do Z80 acabou.
    JoypadInit,
    /// `MainGameLoop`: `GameInit` acabou.
    MainGameLoop,
    /// `GM_Sega`: começo da tela da Sega.
    GmSega,
    /// `Sega_WaitPal`: a preparação da tela acabou e o ciclo de paleta começa.
    SegaWaitPal,
    /// `Sega_WaitEnd`: a voz acabou.
    SegaWaitEnd,
}

/// Rótulo e endereço de cada marca (metadado de `refs/rotulos.txt`, conferido por teste).
pub const MARCAS: [(Marca, &str, u32); 9] = [
    (Marca::GameProgram, "GameProgram", 0x300),
    (Marca::ChecksumOk, "CheckSumOk", 0x33E),
    (Marca::VdpSetupGame, "VDPSetupGame", 0x1222),
    (Marca::DacDriverLoad, "DACDriverLoad", 0x134A),
    (Marca::JoypadInit, "JoypadInit", 0x11B6),
    (Marca::MainGameLoop, "MainGameLoop", 0x388),
    (Marca::GmSega, "GM_Sega", 0x2DD2),
    (Marca::SegaWaitPal, "Sega_WaitPal", 0x2EAC),
    (Marca::SegaWaitEnd, "Sega_WaitEnd", 0x2ED4),
];

type Passo = fn(&mut Memoria) -> Result<(), ErroAbertura>;

/// O laço principal do boot, em ordem. Cada passo roda quando a marca do rótulo seguinte chega,
/// isto é, no fim do trecho que ele reproduz.
const SEQUENCIA: [(Marca, Passo); 6] = [
    (Marca::GameProgram, |m| {
        boot::entry_point(m)?;
        boot::game_program(m)
    }),
    (Marca::ChecksumOk, boot::check_sum_ok),
    (Marca::VdpSetupGame, boot::game_init_limpar),
    (Marca::DacDriverLoad, boot::vdp_setup_game),
    (Marca::JoypadInit, boot::dac_driver_load),
    (Marca::MainGameLoop, boot::game_init_fim),
];

/// Um fato do tempo gravado dentro de um quadro, na ordem em que aconteceu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventoQuadro {
    Interrupcao { nivel: u8, pc: u32 },
    Marca(Marca),
    Leitura { classe: ClasseLeitura, valor: u16 },
    PontoZ80(PontoZ80),
}

/// Erros que param a abertura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroAbertura {
    /// A soma de verificação da ROM não bate com o cabeçalho (`CheckSumError`).
    ChecksumErrado,
    /// Um laço de espera não terminou: faltou valor gravado.
    EsperaSemFim,
    Acesso(ErroAcesso),
    Rotina(crate::rotinas::ErroRotina),
    /// O trecho pedido não existe nesta versão do núcleo.
    NaoImplementado(&'static str),
}

impl From<ErroAcesso> for ErroAbertura {
    fn from(e: ErroAcesso) -> Self {
        Self::Acesso(e)
    }
}

/// Passos da tela da Sega (`GM_Sega`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sega {
    /// Esperando a chegada a `GM_Sega`.
    Entrada,
    /// `PaletteFadeOut`: faltam tantos VBlanks.
    FadeOut(u8),
    /// Preparação da tela (sem interrupções) até `Sega_WaitPal`.
    Preparo,
    /// Ciclo de paleta, um passo por VBlank.
    WaitPal,
    /// A voz foi pedida: o VBlank seguinte roda o driver, que só volta no fim da voz.
    Voz,
    /// O tratador do VBlank da voz está em curso. Ele acaba na interrupção seguinte: o VBlank
    /// que ficou pendente é atendido logo depois do `rte`, antes de o laço principal rodar.
    VozEmCurso,
    /// A voz acabou; falta o laço principal chegar a `Sega_WaitEnd`.
    VozFim,
    /// Espera depois da voz, até o temporizador zerar ou o Start.
    WaitEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Modo {
    Boot,
    Sega(Sega),
    /// O título ainda não existe nesta versão do núcleo.
    Titulo,
}

/// O console no trecho da abertura.
pub struct Abertura {
    pub mem: Memoria,
    /// Próximo passo da `SEQUENCIA`; `SEQUENCIA.len()` quando o boot acabou.
    cursor: usize,
    modo: Modo,
    quadro: u32,
    pub erro: Option<ErroAbertura>,
    /// Quantas vezes cada tipo de evento chegou sem uso (diagnóstico do relatório).
    pub marcas_ignoradas: u32,
    pub interrupcoes_ignoradas: u32,
    /// VBlanks que rodaram o tratador (diagnóstico do relatório).
    pub vblanks_tratados: u32,
    /// Onde as interrupções ignoradas caíram: quadro e estado (diagnóstico).
    pub ignoradas_em: Vec<(u32, String)>,
    /// Diagnóstico: RAM do 68000 e do Z80 cada vez que o laço principal chega a uma espera de VBlank.
    esperas: Option<Vec<(Vec<u8>, Vec<u8>)>>,
}

impl Abertura {
    pub fn nova(rom: Vec<u8>) -> Self {
        Self {
            mem: Memoria::nova(rom),
            cursor: 0,
            modo: Modo::Boot,
            interrupcoes_ignoradas: 0,
            vblanks_tratados: 0,
            ignoradas_em: Vec::new(),
            esperas: None,
            quadro: 0,
            erro: None,
            marcas_ignoradas: 0,
        }
    }

    pub fn quadro(&self) -> u32 {
        self.quadro
    }

    /// Está dentro de um trecho atômico? (a marca final do trecho ainda não chegou)
    pub fn em_trecho_atomico(&self) -> bool {
        !matches!(
            self.modo,
            Modo::Sega(Sega::FadeOut(_) | Sega::WaitPal | Sega::WaitEnd) | Modo::Titulo
        )
    }

    /// Roda um quadro com os fatos gravados dele.
    pub fn executar_quadro(&mut self, eventos: &[EventoQuadro]) {
        // O tempo gravado do quadro inteiro fica disponível antes de qualquer trecho rodar.
        for e in eventos {
            match *e {
                EventoQuadro::Leitura { classe, valor } => {
                    self.mem.enfileirar_leitura(classe, valor)
                }
                EventoQuadro::PontoZ80(p) => self.mem.enfileirar_ponto_z80(p),
                _ => {}
            }
        }
        for e in eventos {
            if self.erro.is_some() {
                break;
            }
            let r = match *e {
                EventoQuadro::Marca(m) => self.marca(m),
                EventoQuadro::Interrupcao { nivel: 6, .. } => self.vblank(),
                EventoQuadro::Interrupcao { .. } => {
                    self.interrupcoes_ignoradas = self.interrupcoes_ignoradas.saturating_add(1);
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(erro) = r {
                self.erro = Some(erro);
            }
        }
        self.quadro = self.quadro.saturating_add(1);
    }

    fn marca(&mut self, m: Marca) -> Result<(), ErroAbertura> {
        if self.modo == Modo::Boot {
            match SEQUENCIA.get(self.cursor) {
                Some((esperada, passo)) if *esperada == m => {
                    passo(&mut self.mem)?;
                    self.cursor = self.cursor.saturating_add(1);
                    if self.cursor == SEQUENCIA.len() {
                        self.modo = Modo::Sega(Sega::Entrada);
                    }
                }
                _ => self.marcas_ignoradas = self.marcas_ignoradas.saturating_add(1),
            }
            return Ok(());
        }
        match (self.modo, m) {
            (Modo::Sega(Sega::Entrada), Marca::GmSega) => {
                sega::gm_sega_inicio(&mut self.mem);
                self.espera();
                self.modo = Modo::Sega(Sega::FadeOut(22));
            }
            (Modo::Sega(Sega::Preparo), Marca::SegaWaitPal) => {
                sega::gm_sega_preparo(&mut self.mem)?;
                wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                self.espera();
                self.modo = Modo::Sega(Sega::WaitPal);
            }
            (Modo::Sega(Sega::VozFim), Marca::SegaWaitEnd) => {
                // O laço principal volta a rodar depois do VBlank atrasado.
                ww(&mut self.mem, ram::V_GENERICTIMER, 30);
                wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                self.espera();
                self.modo = Modo::Sega(Sega::WaitEnd);
            }
            _ => self.marcas_ignoradas = self.marcas_ignoradas.saturating_add(1),
        }
        Ok(())
    }

    /// Liga o registro do estado em cada espera de VBlank (só para o comparador).
    pub fn registrar_esperas(&mut self) {
        self.esperas.get_or_insert_with(Vec::new);
    }

    /// Entrega e esvazia os estados registrados nas esperas.
    pub fn tirar_esperas(&mut self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.esperas.as_mut().map(std::mem::take).unwrap_or_default()
    }

    /// O laço principal chegou a `WaitForVBlank`: o estado de hoje é o de comparação.
    fn espera(&mut self) {
        if let Some(l) = &mut self.esperas {
            l.push((self.mem.ram().to_vec(), self.mem.ram_z80().to_vec()));
        }
    }

    fn ignorar(&mut self) {
        self.interrupcoes_ignoradas = self.interrupcoes_ignoradas.saturating_add(1);
        self.ignoradas_em
            .push((self.quadro, format!("{:?}", self.modo)));
    }

    /// Interrupção de VBlank gravada: roda o tratador e, em seguida, o laço principal até a
    /// próxima espera.
    fn vblank(&mut self) -> Result<(), ErroAbertura> {
        let Modo::Sega(passo) = self.modo else {
            self.ignorar();
            return Ok(());
        };
        match passo {
            Sega::FadeOut(n) => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                vblank::tratar(&mut self.mem)?;
                sega::fade_out_to_black(&mut self.mem);
                let n = n.saturating_sub(1);
                if n > 0 {
                    wb(
                        &mut self.mem,
                        ram::V_VBLANK_ROUTINE,
                        vblank::ROTINA_PALETTE_FADE,
                    );
                    self.espera();
                    self.modo = Modo::Sega(Sega::FadeOut(n));
                } else {
                    self.modo = Modo::Sega(Sega::Preparo);
                }
            }
            Sega::WaitPal => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                vblank::tratar(&mut self.mem)?;
                if sega::pal_cycle_sega(&mut self.mem) {
                    wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                    self.espera();
                } else {
                    wb(&mut self.mem, ram::V_SOUNDQUEUE1, ram::SFX_SEGA);
                    wb(
                        &mut self.mem,
                        ram::V_VBLANK_ROUTINE,
                        vblank::ROTINA_SEGA_PCM,
                    );
                    self.espera();
                    self.modo = Modo::Sega(Sega::Voz);
                }
            }
            Sega::Voz => self.modo = Modo::Sega(Sega::VozEmCurso),
            Sega::VozEmCurso => {
                // O tratador da voz acabou (o driver só volta no fim da voz). Em seguida o
                // VBlank pendente roda como quadro de lag: `v_vblank_routine` é zero, então
                // só o driver de som roda.
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(2);
                vblank::tratar(&mut self.mem)?;
                vblank::tratar(&mut self.mem)?;
                self.modo = Modo::Sega(Sega::VozFim);
            }
            Sega::WaitEnd => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                vblank::tratar(&mut self.mem)?;
                // `Sega_WaitEnd`: temporizador zerado ou Start apertado leva ao título.
                let acabou = rw(&mut self.mem, ram::V_GENERICTIMER) == 0;
                let start = if acabou {
                    0x80
                } else {
                    let v = rb(&mut self.mem, ram::V_JPADHOLD1.wrapping_add(1)) & 0x80;
                    wb(&mut self.mem, ram::V_JPADHOLD1.wrapping_add(1), v);
                    v
                };
                if acabou || start != 0 {
                    wb(&mut self.mem, ram::V_GAMEMODE, 4);
                    self.modo = Modo::Titulo;
                } else {
                    wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                    self.espera();
                }
            }
            Sega::Entrada | Sega::Preparo | Sega::VozFim => self.ignorar(),
        }
        Ok(())
    }
}

/// Atalhos de acesso à memória no estilo do 68000: erro de alinhamento nunca acontece nos
/// endereços fixos que as rotinas usam.
pub(crate) fn wb(m: &mut Memoria, a: u32, v: u8) {
    let _ = m.escrever(a, Tamanho::Byte, u32::from(v));
}
pub(crate) fn ww(m: &mut Memoria, a: u32, v: u16) {
    let _ = m.escrever(a, Tamanho::Word, u32::from(v));
}
pub(crate) fn wl(m: &mut Memoria, a: u32, v: u32) {
    let _ = m.escrever(a, Tamanho::Long, v);
}
pub(crate) fn rb(m: &mut Memoria, a: u32) -> u8 {
    m.ler(a, Tamanho::Byte).map_or(0, |v| v.to_le_bytes()[0])
}
pub(crate) fn rw(m: &mut Memoria, a: u32) -> u16 {
    m.ler(a, Tamanho::Word).map_or(0, |v| {
        let b = v.to_le_bytes();
        u16::from_le_bytes([b[0], b[1]])
    })
}

pub(crate) fn rl(m: &mut Memoria, a: u32) -> u32 {
    m.ler(a, Tamanho::Long).unwrap_or(0)
}
