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
//!
//! Quadros de lag (D7.25): o laço só está na espera do VBlank depois da marca `WaitForVBlank`.
//! Uma interrupção que chega antes dela pega o laço no meio do trecho, e o tratador roda como no
//! original nesse caso: com a rotina zero, só o driver de som e o contador.

pub mod afogamento;
pub mod animacao;
pub mod badniks;
pub mod barreira_invisivel;
pub mod basaran;
pub mod boot;
pub mod boss_ghz;
pub mod botao;
pub mod buzz;
pub mod capsula;
pub mod caterkiller;
pub mod cenario;
pub mod chopper;
pub mod collapse_ledge;
pub mod continue_screen;
pub mod crabmeat;
pub mod desenho;
pub mod edge_walls;
pub mod fase;
pub mod fisica;
pub mod game_over;
pub mod ghz_extras;
pub mod got_through;
pub mod hud;
pub mod labirinto;
pub mod labirinto_cenario;
pub mod lamppost;
pub mod marble_bloco;
pub mod marble_fogo;
pub mod marble_plataformas;
pub mod marble_stomper;
pub mod motobug;
pub mod objetos;
pub mod objpos;
pub mod paleta;
pub mod parede_quebravel;
pub mod plataforma_basica;
pub mod plataforma_oscilante;
pub mod plc;
pub mod ponte;
pub mod ram;
pub mod sega;
pub mod signpost;
pub mod sonic_agua;
pub mod special_result;
pub mod special_stage;
pub mod spikes;
pub mod splash;
mod spring_yard_cenario;
pub mod tijolo_marble;
pub mod titulo;
pub mod vblank;
pub mod vidro_marble;

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
    /// `GM_Title`: começo da tela do título.
    GmTitle,
    /// `PaletteFadeIn`: chamada do aparecimento da paleta.
    PaletteFadeIn,
    /// `PaletteFadeOut`: a segunda chamada em `GM_Title`, antes da parte B do título.
    PaletteFadeOut,
    /// `Tit_MainLoop`: começo do laço do título, depois da parte C.
    TitMainLoop,
    /// `WaitForVBlank`: o laço principal chegou à espera do VBlank. Uma interrupção que chega
    /// antes desta marca é quadro de lag (D7.25).
    WaitForVBlank,
    /// `GotoDemo`: o temporizador do título acabou. Só para o relatório: o núcleo decide pela
    /// própria RAM.
    GotoDemo,
    /// `GM_Level`: começo do modo de fase (e de demo).
    GmLevel,
    /// `Level_TtlCardLoop`: começo do laço dos cartões de título da fase.
    LevelTtlCardLoop,
    /// `Level_Delay`: a carga da fase acabou; começam os quatro VBlanks de espera.
    LevelDelay,
    /// `Level_StartGame`: o aparecimento acabou; fim da sequência antes da fase.
    LevelStartGame,
}

/// Rótulo e endereço de cada marca (metadado de `refs/rotulos.txt`, conferido por teste).
pub const MARCAS: [(Marca, &str, u32); 19] = [
    (Marca::GameProgram, "GameProgram", 0x300),
    (Marca::ChecksumOk, "CheckSumOk", 0x33E),
    (Marca::VdpSetupGame, "VDPSetupGame", 0x1222),
    (Marca::DacDriverLoad, "DACDriverLoad", 0x134A),
    (Marca::JoypadInit, "JoypadInit", 0x11B6),
    (Marca::MainGameLoop, "MainGameLoop", 0x388),
    (Marca::GmSega, "GM_Sega", 0x2DD2),
    (Marca::SegaWaitPal, "Sega_WaitPal", 0x2EAC),
    (Marca::SegaWaitEnd, "Sega_WaitEnd", 0x2ED4),
    (Marca::GmTitle, "GM_Title", 0x2EF4),
    (Marca::PaletteFadeIn, "PaletteFadeIn", 0x1DA4),
    (Marca::PaletteFadeOut, "PaletteFadeOut", 0x1E4A),
    (Marca::TitMainLoop, "Tit_MainLoop", 0x315E),
    (Marca::WaitForVBlank, "WaitForVBlank", 0x29A0),
    (Marca::GotoDemo, "GotoDemo", 0x33A4),
    (Marca::GmLevel, "GM_Level", 0x379E),
    (Marca::LevelTtlCardLoop, "Level_TtlCardLoop", 0x3912),
    (Marca::LevelDelay, "Level_Delay", 0x3A8C),
    (Marca::LevelStartGame, "Level_StartGame", 0x3AD8),
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
    /// O ID de um objeto que a fase pediu e que o núcleo ainda não tem.
    ObjetoNaoImplementado(u8),
    /// Os IDs de objeto que a partida criou e que o núcleo não tem, como máscara de 256 bits
    /// (bit `id % 128` da palavra `id / 128`).
    ObjetosNaoImplementados([u128; 2]),
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
    Titulo(Titulo),
    Fase(Fase),
    Especial(Especial),
}

/// Etapas de `GM_Special` e da tela de resultados.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Especial {
    EntradaBranco(u8),
    Aparecimento(u8),
    Jogando,
    Saida,
    Resultados,
    SaidaBranco(u8),
}

/// Passos do modo de fase e de demo (`GM_Level`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fase {
    /// O `GotoDemo` preparou a demo; falta a chegada a `GM_Level`.
    Entrada,
    /// `PaletteFadeOut` do começo de `GM_Level`: faltam tantos VBlanks.
    FadeOut(u8),
    /// Preparo dos cartões de título (sem espera) até a marca `Level_TtlCardLoop`.
    PreparoCartoes,
    /// `Level_TtlCardLoop`: os cartões entram e a fila de arte da zona anda, um quadro por VBlank.
    Cartoes,
    /// Carga final da fase (sem espera) até a marca `Level_Delay`.
    PreparoFinal,
    /// `Level_DelayLoop`: faltam tantos VBlanks da fase.
    Atraso(u8),
    /// `PalFadeIn_Alt`: aparecimento das linhas 2 a 4 da paleta, faltam tantos VBlanks.
    FadeIn(u8),
    /// Espera a marca `Level_StartGame`.
    PreparoFim,
    /// Laço principal da fase, com os objetos e controles já implementados.
    Jogando,
    ContinueFadeOut(u8),
    ContinueFadeIn(u8),
    ContinueLaco,
    DemoFade,
    /// Fim da abertura para o comparador histórico.
    Parado,
}

/// Passos da tela do título (`GM_Title`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Titulo {
    /// Esperando a chegada a `GM_Title`.
    Entrada,
    /// Primeiro `PaletteFadeOut`: faltam tantos VBlanks.
    FadeOut(u8),
    /// Carga do "SONIC TEAM PRESENTS" (sem interrupções) até `PaletteFadeIn`.
    PreparoA,
    /// `PaletteFadeIn` da tela do "SONIC TEAM PRESENTS".
    FadeIn(u8),
    /// Parte B, fatia 1: espera a marca `PaletteFadeOut` (a fatia roda nela, de uma vez).
    PreparoB,
    /// `PaletteFadeOut` da parte B: faltam tantos VBlanks.
    FadeOutB(u8),
    /// Parte C: espera a marca `PaletteFadeIn` do aparecimento (a parte C roda nela).
    PreparoC,
    /// `PaletteFadeIn` do aparecimento do título: faltam tantos VBlanks. A fila de arte anda aqui.
    FadeInC(u8),
    /// Espera a marca `Tit_MainLoop`, que é o começo do laço do título.
    PreparoTit,
    /// `Tit_MainLoop`: um quadro do título por VBlank, até o temporizador acabar.
    Laco,
    /// `GotoDemo_PreDelayLoop`: os 30 quadros de espera antes da demo.
    PreDemo,
}

/// O que o laço principal espera (para o tempo nominal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Espera {
    /// Está num trecho que termina na marca.
    Marca(Marca),
    /// Está esperando o próximo VBlank.
    VBlank,
    /// O tratador do VBlank da voz está rodando; o fim dele é a próxima interrupção.
    VozAcabando,
    /// Nada mais a rodar nesta versão do núcleo.
    Nenhuma,
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
    /// Quantos desses VBlanks foram quadros de lag (diagnóstico do relatório).
    pub quadros_de_lag: u32,
    /// Onde as interrupções ignoradas caíram: quadro e estado (diagnóstico).
    pub ignoradas_em: Vec<(u32, String)>,
    /// O laço pediu o VBlank e vai esperar, mas a marca `WaitForVBlank` ainda não chegou.
    espera_pedida: bool,
    /// O laço está parado em `WaitForVBlank`: a próxima interrupção é o VBlank que ele pediu.
    na_espera: bool,
    /// Diagnóstico: RAM do 68000 e do Z80 cada vez que o laço principal chega a uma espera de VBlank.
    esperas: Option<Vec<(Vec<u8>, Vec<u8>)>>,
    /// Eventos do quadro anterior, ainda não rodados (olhar adiante de um quadro).
    pendentes: Vec<EventoQuadro>,
    /// Com tempo gravado os eventos rodam um quadro depois; no modo nominal rodam na hora.
    atraso: bool,
    hblank_disponivel: bool,
    /// O comparador histórico encerra as saídas em Level_StartGame.
    parar_no_fim_abertura: bool,
    /// Chamado depois de cada evento (marca ou interrupção) com a memória: o comparador recolhe
    /// as saídas da VDP na hora em que saem, quando a RAM ainda é a do evento (o DMA lê dela).
    observador: Option<Observador>,
    observador_temporal: Option<ObservadorTemporal>,
    saidas_observadas: usize,
}

/// Função chamada depois de cada evento processado.
pub type Observador = Box<dyn FnMut(&mut Memoria)>;

/// Trecho que produziu as saídas recolhidas pelo observador temporal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentoSaida {
    VBlank,
    HBlank,
    Principal,
}

/// Observa as saídas antes de o próximo trecho alterar a memória.
pub type ObservadorTemporal = Box<dyn FnMut(&mut Memoria, MomentoSaida, &[crate::memoria::Saida])>;

impl Abertura {
    pub fn nova(rom: Vec<u8>) -> Self {
        Self {
            mem: Memoria::nova(rom),
            cursor: 0,
            modo: Modo::Boot,
            interrupcoes_ignoradas: 0,
            vblanks_tratados: 0,
            quadros_de_lag: 0,
            ignoradas_em: Vec::new(),
            espera_pedida: false,
            na_espera: false,
            esperas: None,
            pendentes: Vec::new(),
            atraso: true,
            hblank_disponivel: false,
            parar_no_fim_abertura: true,
            observador: None,
            observador_temporal: None,
            saidas_observadas: 0,
            quadro: 0,
            erro: None,
            marcas_ignoradas: 0,
        }
    }

    /// Liga o observador de eventos (só para o comparador).
    pub fn definir_observador(&mut self, o: Observador) {
        self.observador = Some(o);
    }

    /// Instala observador separado, com identificação do trecho que gerou as saídas.
    pub fn definir_observador_temporal(&mut self, o: ObservadorTemporal) {
        self.observador_temporal = Some(o);
    }

    /// Console sem tempo gravado (navegador): os eventos rodam na hora do quadro em que chegam.
    pub fn nova_nominal(rom: Vec<u8>) -> Self {
        let mut a = Self::nova(rom);
        a.atraso = false;
        a.parar_no_fim_abertura = false;
        a
    }

    /// Permite comparar a fase com eventos gravados após a abertura.
    pub fn continuar_apos_abertura(&mut self) {
        self.parar_no_fim_abertura = false;
    }

    /// O que o laço principal espera agora. O modo nominal usa isto para criar o tempo.
    pub fn espera(&self) -> Espera {
        match self.modo {
            Modo::Boot => SEQUENCIA
                .get(self.cursor)
                .map_or(Espera::Nenhuma, |(m, _)| Espera::Marca(*m)),
            Modo::Sega(Sega::Entrada) => Espera::Marca(Marca::GmSega),
            Modo::Sega(Sega::Preparo) => Espera::Marca(Marca::SegaWaitPal),
            Modo::Sega(Sega::VozFim) => Espera::Marca(Marca::SegaWaitEnd),
            Modo::Sega(Sega::VozEmCurso) => Espera::VozAcabando,
            Modo::Sega(_) => Espera::VBlank,
            Modo::Titulo(Titulo::Entrada) => Espera::Marca(Marca::GmTitle),
            Modo::Titulo(Titulo::PreparoA) => Espera::Marca(Marca::PaletteFadeIn),
            Modo::Titulo(Titulo::PreparoB) => Espera::Marca(Marca::PaletteFadeOut),
            Modo::Titulo(Titulo::PreparoC) => Espera::Marca(Marca::PaletteFadeIn),
            Modo::Titulo(Titulo::PreparoTit) => Espera::Marca(Marca::TitMainLoop),
            Modo::Titulo(
                Titulo::FadeOut(_)
                | Titulo::FadeIn(_)
                | Titulo::FadeOutB(_)
                | Titulo::FadeInC(_)
                | Titulo::Laco
                | Titulo::PreDemo,
            ) => Espera::VBlank,
            Modo::Fase(Fase::Entrada) => Espera::Marca(Marca::GmLevel),
            Modo::Fase(Fase::PreparoCartoes) => Espera::Marca(Marca::LevelTtlCardLoop),
            Modo::Fase(Fase::PreparoFinal) => Espera::Marca(Marca::LevelDelay),
            Modo::Fase(Fase::PreparoFim) => Espera::Marca(Marca::LevelStartGame),
            Modo::Fase(
                Fase::FadeOut(_)
                | Fase::Cartoes
                | Fase::Atraso(_)
                | Fase::FadeIn(_)
                | Fase::ContinueFadeOut(_)
                | Fase::ContinueFadeIn(_)
                | Fase::ContinueLaco
                | Fase::DemoFade
                | Fase::Jogando,
            ) => Espera::VBlank,
            Modo::Fase(Fase::Parado) => Espera::Nenhuma,
            Modo::Especial(_) => Espera::VBlank,
        }
    }

    pub fn quadro(&self) -> u32 {
        self.quadro
    }

    /// Está dentro de um trecho atômico? (a marca final do trecho ainda não chegou)
    pub fn em_trecho_atomico(&self) -> bool {
        !matches!(
            self.modo,
            Modo::Sega(Sega::FadeOut(_) | Sega::WaitPal | Sega::WaitEnd)
                | Modo::Titulo(
                    Titulo::FadeOut(_)
                        | Titulo::FadeIn(_)
                        | Titulo::FadeOutB(_)
                        | Titulo::FadeInC(_)
                        | Titulo::Laco
                        | Titulo::PreDemo
                )
                | Modo::Especial(_)
                | Modo::Fase(
                    Fase::FadeOut(_)
                        | Fase::Cartoes
                        | Fase::Atraso(_)
                        | Fase::FadeIn(_)
                        | Fase::ContinueFadeOut(_)
                        | Fase::ContinueFadeIn(_)
                        | Fase::ContinueLaco
                        | Fase::DemoFade
                        | Fase::Jogando
                )
        )
    }

    /// Entrega os fatos gravados de um quadro. Os fatos de leitura e os pontos do Z80 ficam na
    /// fila na hora. Os eventos (marcas e interrupções) rodam um quadro depois: uma interrupção
    /// no fim de um quadro faz acessos que o oráculo grava no quadro seguinte, e o tratador
    /// precisa deles na fila. Chame [`Abertura::terminar`] depois do último quadro.
    pub fn executar_quadro(&mut self, eventos: &[EventoQuadro]) {
        if self.mem.saidas().is_empty() {
            self.saidas_observadas = 0;
        }
        for e in eventos {
            match *e {
                EventoQuadro::Leitura { classe, valor } => {
                    self.mem.enfileirar_leitura(classe, valor)
                }
                EventoQuadro::PontoZ80(p) => self.mem.enfileirar_ponto_z80(p),
                _ => {}
            }
        }
        let anteriores = if self.atraso {
            std::mem::replace(&mut self.pendentes, eventos.to_vec())
        } else {
            eventos.to_vec()
        };
        for (indice, e) in anteriores.iter().copied().enumerate() {
            self.hblank_disponivel = anteriores
                .iter()
                .skip(indice.wrapping_add(1))
                .find_map(|evento| match evento {
                    EventoQuadro::Interrupcao { nivel: 4, .. } => Some(true),
                    EventoQuadro::Interrupcao { nivel: 6, .. } => Some(false),
                    _ => None,
                })
                .unwrap_or(false);
            if self.erro.is_some() {
                break;
            }
            let r = match e {
                EventoQuadro::Marca(m) => self.marca(m),
                EventoQuadro::Interrupcao { nivel: 6, .. } => self.vblank(),
                EventoQuadro::Interrupcao { nivel: 4, .. } => vblank::tratar_hblank(&mut self.mem),
                EventoQuadro::Interrupcao { .. } => {
                    self.interrupcoes_ignoradas = self.interrupcoes_ignoradas.saturating_add(1);
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(erro) = r {
                self.erro = Some(erro);
            }
            let momento = if matches!(e, EventoQuadro::Interrupcao { nivel: 4, .. }) {
                MomentoSaida::HBlank
            } else {
                MomentoSaida::Principal
            };
            self.notificar_observadores(momento);
        }
        self.quadro = self.quadro.saturating_add(1);
    }

    /// Roda os eventos do último quadro entregue.
    pub fn terminar(&mut self) {
        self.executar_quadro(&[]);
    }

    fn marca(&mut self, m: Marca) -> Result<(), ErroAbertura> {
        if m == Marca::WaitForVBlank {
            self.chegar_na_espera();
            return Ok(());
        }
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
                self.pedir_espera();
                self.modo = Modo::Sega(Sega::FadeOut(22));
            }
            (Modo::Sega(Sega::Preparo), Marca::SegaWaitPal) => {
                sega::gm_sega_preparo(&mut self.mem)?;
                wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                self.pedir_espera();
                self.modo = Modo::Sega(Sega::WaitPal);
            }
            (Modo::Sega(Sega::VozFim), Marca::SegaWaitEnd) => {
                // O laço principal volta a rodar depois do VBlank atrasado.
                ww(&mut self.mem, ram::V_GENERICTIMER, 30);
                wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                self.pedir_espera();
                self.modo = Modo::Sega(Sega::WaitEnd);
            }
            (Modo::Titulo(Titulo::Entrada), Marca::GmTitle) => {
                sega::gm_sega_inicio(&mut self.mem);
                self.pedir_espera();
                self.modo = Modo::Titulo(Titulo::FadeOut(22));
            }
            (Modo::Titulo(Titulo::PreparoA), Marca::PaletteFadeIn) => {
                titulo::gm_title_preparo_a(&mut self.mem)?;
                titulo::palette_fade_in_inicio(&mut self.mem);
                self.pedir_espera();
                self.modo = Modo::Titulo(Titulo::FadeIn(22));
            }
            (Modo::Titulo(Titulo::PreparoC), Marca::PaletteFadeIn) => {
                // A parte C roda de uma vez, até a chamada de `PaletteFadeIn` (D7.10).
                titulo::parte_c(&mut self.mem)?;
                titulo::palette_fade_in_inicio(&mut self.mem);
                self.pedir_espera();
                self.modo = Modo::Titulo(Titulo::FadeInC(22));
            }
            (Modo::Titulo(Titulo::PreparoTit), Marca::TitMainLoop) => {
                // `Tit_MainLoop` pede o VBlank do título; o laço anda um quadro por VBlank.
                wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_TITULO);
                self.pedir_espera();
                self.modo = Modo::Titulo(Titulo::Laco);
            }
            (Modo::Fase(Fase::Entrada), Marca::GmLevel) => {
                fase::gm_level_inicio(&mut self.mem);
                self.pedir_espera();
                self.modo = Modo::Fase(Fase::FadeOut(22));
            }
            (Modo::Fase(Fase::PreparoFinal), Marca::LevelDelay) => {
                fase::preparo_final(&mut self.mem)?;
                self.pedir_vblank(vblank::ROTINA_NIVEL);
                self.modo = Modo::Fase(Fase::Atraso(4));
            }
            (Modo::Fase(Fase::PreparoFim), Marca::LevelStartGame) => {
                fase::level_start_game(&mut self.mem)?;
                if self.parar_no_fim_abertura {
                    self.modo = Modo::Fase(Fase::Parado);
                } else {
                    self.pedir_vblank(vblank::ROTINA_NIVEL);
                    self.modo = Modo::Fase(Fase::Jogando);
                }
            }
            (Modo::Fase(Fase::PreparoCartoes), Marca::LevelTtlCardLoop) => {
                // O preparo dos cartões roda de uma vez (D7.10); os VBlanks no meio dele chegam
                // antes da marca e rodam como quadros de lag (D7.25).
                fase::preparo_dos_cartoes(&mut self.mem)?;
                self.pedir_vblank(vblank::ROTINA_CARTOES);
                self.modo = Modo::Fase(Fase::Cartoes);
            }
            (Modo::Titulo(Titulo::PreparoB), Marca::PaletteFadeOut) => {
                // A parte B roda de uma vez (D7.10). Depois, o desvanecimento pede o VBlank.
                titulo::parte_b1(&mut self.mem)?;
                titulo::palette_fade_out_inicio(&mut self.mem);
                self.pedir_espera();
                self.modo = Modo::Titulo(Titulo::FadeOutB(22));
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
        self.esperas
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    }

    /// O laço principal chegou a `WaitForVBlank`: o estado de hoje é o de comparação.
    fn registrar_espera(&mut self) {
        if let Some(l) = &mut self.esperas {
            l.push((self.mem.ram().to_vec(), self.mem.ram_z80().to_vec()));
        }
    }

    /// O laço principal terminou o trecho, pediu o VBlank (a rotina já está em `v_vblank_routine`)
    /// e vai esperar. A espera só começa com a marca `WaitForVBlank` do tempo gravado: até lá,
    /// uma interrupção é quadro de lag (D7.25).
    fn pedir_espera(&mut self) {
        self.espera_pedida = true;
    }

    /// Marca `WaitForVBlank`: o laço chegou à espera que pediu. Sem pedido (laço que o núcleo não
    /// roda, como o do título depois da fatia), a marca não muda nada.
    fn chegar_na_espera(&mut self) {
        if self.espera_pedida {
            self.espera_pedida = false;
            self.na_espera = true;
            self.registrar_espera();
        }
    }

    /// Quadro de lag: a interrupção chegou com o laço principal no meio do trecho, antes da
    /// marca `WaitForVBlank`. No original o laço ainda não pediu a rotina (o `move.b` vem logo
    /// antes de `WaitForVBlank`), então `v_vblank_routine` está em zero: o tratador só roda o
    /// driver de som e soma o contador. O núcleo roda o trecho de uma vez e já pode ter gravado o
    /// pedido; o tratador roda com zero, e o pedido volta depois, como o laço faria. O estado do
    /// laço não muda.
    fn vblank_de_lag(&mut self) -> Result<(), ErroAbertura> {
        self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
        self.quadros_de_lag = self.quadros_de_lag.saturating_add(1);
        let pedido = rb(&mut self.mem, ram::V_VBLANK_ROUTINE);
        wb(&mut self.mem, ram::V_VBLANK_ROUTINE, 0);
        self.tratar_vblank()?;
        wb(&mut self.mem, ram::V_VBLANK_ROUTINE, pedido);
        Ok(())
    }

    /// VBlank durante a tela do título.
    fn vblank_titulo(&mut self, t: Titulo) -> Result<(), ErroAbertura> {
        match t {
            Titulo::FadeOut(n) => {
                let restam = self.vblank_fade_out(n)?;
                let proximo = if restam > 0 {
                    Titulo::FadeOut(restam)
                } else {
                    Titulo::PreparoA
                };
                self.modo = Modo::Titulo(proximo);
            }
            Titulo::FadeOutB(n) => {
                let restam = self.vblank_fade_out(n)?;
                let proximo = if restam > 0 {
                    Titulo::FadeOutB(restam)
                } else {
                    Titulo::PreparoC
                };
                self.modo = Modo::Titulo(proximo);
            }
            Titulo::FadeIn(n) => {
                let restam = self.vblank_fade_in(n)?;
                let proximo = if restam > 0 {
                    Titulo::FadeIn(restam)
                } else {
                    Titulo::PreparoB
                };
                self.modo = Modo::Titulo(proximo);
            }
            Titulo::FadeInC(n) => {
                let restam = self.vblank_fade_in(n)?;
                let proximo = if restam > 0 {
                    Titulo::FadeInC(restam)
                } else {
                    Titulo::PreparoTit
                };
                self.modo = Modo::Titulo(proximo);
            }
            Titulo::Laco => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                match titulo::laco_do_titulo(&mut self.mem)? {
                    titulo::DepoisDoTitulo::Continua => self.pedir_vblank_do_titulo(),
                    titulo::DepoisDoTitulo::GotoDemo => {
                        // `GotoDemo`: meio segundo no último quadro do aceno, antes da demo.
                        ww(&mut self.mem, ram::V_GENERICTIMER, 30);
                        self.pedir_vblank_do_titulo();
                        self.modo = Modo::Titulo(Titulo::PreDemo);
                    }
                    titulo::DepoisDoTitulo::VoltaSega => {
                        wb(&mut self.mem, ram::V_GAMEMODE, 0);
                        self.modo = Modo::Sega(Sega::Entrada);
                    }
                    titulo::DepoisDoTitulo::Jogar => {
                        titulo::play_level(&mut self.mem);
                        self.modo = Modo::Fase(Fase::Entrada);
                    }
                }
            }
            Titulo::PreDemo => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                match titulo::goto_demo_passo(&mut self.mem)? {
                    titulo::DepoisDoGotoDemo::Continua => self.pedir_vblank_do_titulo(),
                    titulo::DepoisDoGotoDemo::Demo => {
                        // O `rts` volta ao `MainGameLoop`, que chama o modo novo.
                        if rb(&mut self.mem, ram::V_GAMEMODE) == ram::ID_SPECIAL {
                            self.iniciar_especial();
                        } else {
                            self.modo = Modo::Fase(Fase::Entrada);
                        }
                    }
                    titulo::DepoisDoGotoDemo::VoltaSega => {
                        wb(&mut self.mem, ram::V_GAMEMODE, 0);
                        self.modo = Modo::Sega(Sega::Entrada);
                    }
                    titulo::DepoisDoGotoDemo::Jogar => {
                        titulo::play_level(&mut self.mem);
                        self.modo = Modo::Fase(Fase::Entrada);
                    }
                }
            }
            Titulo::Entrada
            | Titulo::PreparoA
            | Titulo::PreparoB
            | Titulo::PreparoC
            | Titulo::PreparoTit => self.ignorar(),
        }
        Ok(())
    }

    /// `move.b #id_VBlank_Title,(v_vblank_routine).w` antes de `WaitForVBlank`.
    fn pedir_vblank_do_titulo(&mut self) {
        wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_TITULO);
        self.pedir_espera();
    }

    /// VBlank no modo de fase.
    fn vblank_fase(&mut self, f: Fase) -> Result<(), ErroAbertura> {
        match f {
            Fase::FadeOut(n) => {
                let restam = self.vblank_fade_out(n)?;
                self.modo = Modo::Fase(if restam > 0 {
                    Fase::FadeOut(restam)
                } else {
                    Fase::PreparoCartoes
                });
            }
            Fase::Cartoes => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                match fase::laco_dos_cartoes(&mut self.mem)? {
                    fase::DepoisDosCartoes::Continua => self.pedir_vblank(vblank::ROTINA_CARTOES),
                    fase::DepoisDosCartoes::Fim => self.modo = Modo::Fase(Fase::PreparoFinal),
                }
            }
            Fase::Atraso(n) => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                let restam = n.saturating_sub(1);
                if restam > 0 {
                    self.pedir_vblank(vblank::ROTINA_NIVEL);
                    self.modo = Modo::Fase(Fase::Atraso(restam));
                } else {
                    // `move.w #$202F,(v_pfade_start).w` e `PalFadeIn_Alt`.
                    ww(&mut self.mem, ram::V_PFADE_START, 0x202F);
                    titulo::palette_fade_in_alt(&mut self.mem);
                    self.pedir_espera();
                    self.modo = Modo::Fase(Fase::FadeIn(22));
                }
            }
            Fase::FadeIn(n) => {
                let restam = self.vblank_fade_in(n)?;
                self.modo = Modo::Fase(if restam > 0 {
                    Fase::FadeIn(restam)
                } else {
                    Fase::PreparoFim
                });
            }
            Fase::Jogando => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                fase::level_main_loop(&mut self.mem)?;
                if rw(&mut self.mem, ram::F_RESTART) != 0 {
                    fase::gm_level_inicio(&mut self.mem);
                    self.pedir_espera();
                    self.modo = Modo::Fase(Fase::FadeOut(22));
                    return Ok(());
                }
                if rb(&mut self.mem, ram::V_GAMEMODE) == 0 {
                    self.modo = Modo::Sega(Sega::Entrada);
                    return Ok(());
                }
                if rb(&mut self.mem, ram::V_GAMEMODE) == 0x14 {
                    titulo::palette_fade_out_inicio(&mut self.mem);
                    self.pedir_espera();
                    self.modo = Modo::Fase(Fase::ContinueFadeOut(22));
                    return Ok(());
                }
                if rb(&mut self.mem, ram::V_GAMEMODE) == ram::ID_SPECIAL {
                    self.iniciar_especial();
                    return Ok(());
                }
                if rb(&mut self.mem, ram::V_GAMEMODE) == ram::GM_TITLE_ID {
                    self.modo = Modo::Titulo(Titulo::Entrada);
                    return Ok(());
                }
                if rw(&mut self.mem, ram::F_DEMO) != 0
                    && rw(&mut self.mem, ram::V_GENERICTIMER) == 0
                {
                    fase::demo_fade_inicio(&mut self.mem);
                    self.pedir_vblank(vblank::ROTINA_NIVEL);
                    self.modo = Modo::Fase(Fase::DemoFade);
                    return Ok(());
                }
                self.pedir_vblank(vblank::ROTINA_NIVEL);
            }
            Fase::ContinueFadeOut(n) => {
                let restam = self.vblank_fade_out(n)?;
                if restam != 0 {
                    self.modo = Modo::Fase(Fase::ContinueFadeOut(restam));
                } else {
                    continue_screen::preparar(&mut self.mem)?;
                    titulo::palette_fade_in_inicio(&mut self.mem);
                    self.pedir_espera();
                    self.modo = Modo::Fase(Fase::ContinueFadeIn(22));
                }
            }
            Fase::ContinueFadeIn(n) => {
                let restam = self.vblank_fade_in(n)?;
                if restam != 0 {
                    self.modo = Modo::Fase(Fase::ContinueFadeIn(restam));
                } else {
                    self.pedir_vblank(vblank::ROTINA_CONTINUE);
                    self.modo = Modo::Fase(Fase::ContinueLaco);
                }
            }
            Fase::ContinueLaco => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                match continue_screen::passo(&mut self.mem)? {
                    continue_screen::Resultado::Aguardar => {
                        self.pedir_vblank(vblank::ROTINA_CONTINUE)
                    }
                    continue_screen::Resultado::Sega => self.modo = Modo::Sega(Sega::Entrada),
                    continue_screen::Resultado::Fase => self.modo = Modo::Fase(Fase::Entrada),
                }
            }
            Fase::DemoFade => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                if fase::demo_fade_passo(&mut self.mem)? {
                    self.modo = if rb(&mut self.mem, ram::V_GAMEMODE) == ram::GM_TITLE_ID {
                        Modo::Titulo(Titulo::Entrada)
                    } else {
                        Modo::Sega(Sega::Entrada)
                    };
                } else {
                    self.pedir_vblank(vblank::ROTINA_NIVEL);
                }
            }
            Fase::Entrada
            | Fase::PreparoCartoes
            | Fase::PreparoFinal
            | Fase::PreparoFim
            | Fase::Parado => self.ignorar(),
        }
        Ok(())
    }

    fn iniciar_especial(&mut self) {
        wb(&mut self.mem, ram::V_SOUNDQUEUE1, 0xCA);
        ww(&mut self.mem, ram::V_PFADE_START, 0x003F);
        self.pedir_vblank(vblank::ROTINA_PALETTE_FADE);
        self.modo = Modo::Especial(Especial::EntradaBranco(22));
    }

    fn vblank_especial(&mut self, estado: Especial) -> Result<(), ErroAbertura> {
        self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
        self.tratar_vblank()?;
        match estado {
            Especial::EntradaBranco(n) | Especial::SaidaBranco(n) => {
                special_result::clarear(&mut self.mem);
                plc::run_plc(&mut self.mem)?;
                if n > 1 {
                    self.pedir_vblank(vblank::ROTINA_PALETTE_FADE);
                    self.modo = Modo::Especial(if matches!(estado, Especial::EntradaBranco(_)) {
                        Especial::EntradaBranco(n.wrapping_sub(1))
                    } else {
                        Especial::SaidaBranco(n.wrapping_sub(1))
                    });
                } else if matches!(estado, Especial::EntradaBranco(_)) {
                    special_stage::preparar(&mut self.mem)?;
                    special_result::iniciar_branco(&mut self.mem);
                    self.pedir_vblank(vblank::ROTINA_PALETTE_FADE);
                    self.modo = Modo::Especial(Especial::Aparecimento(22));
                } else {
                    self.modo = Modo::Fase(Fase::Entrada);
                }
            }
            Especial::Aparecimento(n) => {
                special_result::escurecer(&mut self.mem);
                plc::run_plc(&mut self.mem)?;
                if n > 1 {
                    self.pedir_vblank(vblank::ROTINA_PALETTE_FADE);
                    self.modo = Modo::Especial(Especial::Aparecimento(n.wrapping_sub(1)));
                } else {
                    fase::pause_game(&mut self.mem);
                    self.pedir_vblank(vblank::ROTINA_SPECIAL);
                    self.modo = Modo::Especial(Especial::Jogando);
                }
            }
            Especial::Jogando | Especial::Saida => {
                if estado == Especial::Jogando && rw(&mut self.mem, ram::F_PAUSE) != 0 {
                    fase::pause_game(&mut self.mem);
                    self.pedir_vblank(vblank::ROTINA_SPECIAL);
                    return Ok(());
                }
                special_stage::passo(&mut self.mem)?;
                if estado == Especial::Saida {
                    let contador = rw(&mut self.mem, 0xFF_F794).wrapping_sub(1);
                    ww(&mut self.mem, 0xFF_F794, contador);
                    if contador & 0x8000 != 0 {
                        ww(&mut self.mem, 0xFF_F794, 2);
                        special_result::clarear(&mut self.mem);
                    }
                    if rw(&mut self.mem, ram::V_GENERICTIMER) == 0 {
                        special_result::preparar(&mut self.mem)?;
                        fase::pause_game(&mut self.mem);
                        self.pedir_vblank(vblank::ROTINA_CARTOES);
                        self.modo = Modo::Especial(Especial::Resultados);
                    } else {
                        self.pedir_vblank(vblank::ROTINA_CONTINUE);
                    }
                } else if rw(&mut self.mem, ram::F_DEMO) != 0
                    && rw(&mut self.mem, ram::V_GENERICTIMER) == 0
                {
                    wb(&mut self.mem, ram::V_GAMEMODE, 0);
                    self.modo = Modo::Sega(Sega::Entrada);
                } else if rb(&mut self.mem, ram::V_GAMEMODE) != ram::ID_SPECIAL {
                    if rw(&mut self.mem, ram::F_DEMO) != 0 {
                        self.modo = if rb(&mut self.mem, ram::V_GAMEMODE) == ram::GM_TITLE_ID {
                            Modo::Titulo(Titulo::Entrada)
                        } else {
                            wb(&mut self.mem, ram::V_GAMEMODE, 0);
                            Modo::Sega(Sega::Entrada)
                        };
                    } else {
                        wb(&mut self.mem, ram::V_GAMEMODE, ram::ID_LEVEL);
                        if rw(&mut self.mem, ram::V_ZONE_ACT) >= 0x0502 {
                            ww(&mut self.mem, ram::V_ZONE_ACT, 0);
                        }
                        ww(&mut self.mem, ram::V_GENERICTIMER, 60);
                        ww(&mut self.mem, ram::V_PFADE_START, 0x003F);
                        ww(&mut self.mem, 0xFF_F794, 0);
                        self.pedir_vblank(vblank::ROTINA_CONTINUE);
                        self.modo = Modo::Especial(Especial::Saida);
                    }
                } else {
                    fase::pause_game(&mut self.mem);
                    self.pedir_vblank(vblank::ROTINA_SPECIAL);
                }
            }
            Especial::Resultados => {
                if rw(&mut self.mem, ram::F_PAUSE) == 0 {
                    let mut a = ram::V_OBJSPACE;
                    while a < ram::V_OBJSPACE_END {
                        match rb(&mut self.mem, a + ram::OB_ID) {
                            0x7E => special_result::cartao(&mut self.mem, a)?,
                            0x7F => special_result::esmeraldas(&mut self.mem, a)?,
                            0 => {}
                            id => return Err(ErroAbertura::ObjetoNaoImplementado(id)),
                        }
                        a = a.wrapping_add(ram::OBJECT_SIZE);
                    }
                    objetos::build_sprites(&mut self.mem)?;
                    plc::run_plc(&mut self.mem)?;
                }
                if rw(&mut self.mem, ram::F_RESTART) != 0
                    && rl(&mut self.mem, ram::V_PLC_BUFFER) == 0
                {
                    wb(&mut self.mem, ram::V_SOUNDQUEUE1, 0xCA);
                    ww(&mut self.mem, ram::V_PFADE_START, 0x003F);
                    self.pedir_vblank(vblank::ROTINA_PALETTE_FADE);
                    self.modo = Modo::Especial(Especial::SaidaBranco(22));
                } else {
                    fase::pause_game(&mut self.mem);
                    self.pedir_vblank(vblank::ROTINA_CARTOES);
                }
            }
        }
        Ok(())
    }

    /// `move.b #rotina,(v_vblank_routine).w` antes de `WaitForVBlank`.
    fn pedir_vblank(&mut self, rotina: u8) {
        wb(&mut self.mem, ram::V_VBLANK_ROUTINE, rotina);
        self.pedir_espera();
    }

    /// Um VBlank de desvanecimento para preto (`PaletteFadeOut`): roda o tratador e um passo de
    /// `FadeOut_ToBlack`. Devolve os VBlanks que faltam, zero quando o desvanecimento acabou.
    fn vblank_fade_out(&mut self, n: u8) -> Result<u8, ErroAbertura> {
        self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
        self.tratar_vblank()?;
        sega::fade_out_to_black(&mut self.mem);
        // `RunPLC` roda depois de cada passo do desvanecimento (fila vazia não faz nada).
        plc::run_plc(&mut self.mem)?;
        let restam = n.saturating_sub(1);
        if restam > 0 {
            wb(
                &mut self.mem,
                ram::V_VBLANK_ROUTINE,
                vblank::ROTINA_PALETTE_FADE,
            );
            self.pedir_espera();
        }
        Ok(restam)
    }

    /// Um VBlank do aparecimento (`PaletteFadeIn`): roda o tratador, um passo de `FadeIn_FromBlack`
    /// e `RunPLC`. Devolve os VBlanks que faltam, zero quando o aparecimento acabou.
    fn vblank_fade_in(&mut self, n: u8) -> Result<u8, ErroAbertura> {
        self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
        self.tratar_vblank()?;
        titulo::fade_in_from_black(&mut self.mem);
        plc::run_plc(&mut self.mem)?;
        let restam = n.saturating_sub(1);
        if restam > 0 {
            wb(
                &mut self.mem,
                ram::V_VBLANK_ROUTINE,
                vblank::ROTINA_PALETTE_FADE,
            );
            self.pedir_espera();
        }
        Ok(restam)
    }

    /// O tratador de VBlank e, em seguida, o observador: as escritas do tratador (DMA) saem antes
    /// do passo do laço principal, que muda a RAM de onde o DMA leu.
    fn tratar_vblank(&mut self) -> Result<(), ErroAbertura> {
        vblank::tratar_com_hblank(&mut self.mem, self.hblank_disponivel)?;
        self.notificar_observadores(MomentoSaida::VBlank);
        Ok(())
    }

    fn notificar_observadores(&mut self, momento: MomentoSaida) {
        let total = self.mem.saidas().len();
        if self.saidas_observadas > total {
            self.saidas_observadas = 0;
        }
        if let Some(o) = self.observador_temporal.as_mut() {
            let novas = self.mem.saidas()[self.saidas_observadas..].to_vec();
            o(&mut self.mem, momento, &novas);
            self.saidas_observadas = total;
        }
        if let Some(o) = self.observador.as_mut() {
            o(&mut self.mem);
        }
        if self.mem.saidas().len() < self.saidas_observadas {
            self.saidas_observadas = 0;
        }
    }

    fn ignorar(&mut self) {
        self.interrupcoes_ignoradas = self.interrupcoes_ignoradas.saturating_add(1);
        self.ignoradas_em
            .push((self.quadro, format!("{:?}", self.modo)));
    }

    /// Interrupção de VBlank gravada: roda o tratador e, em seguida, o laço principal até a
    /// próxima espera. Se o laço ainda não chegou à espera, é quadro de lag.
    fn vblank(&mut self) -> Result<(), ErroAbertura> {
        match self.modo {
            // Boot ainda não chegou ao laço de interrupções.
            Modo::Boot | Modo::Fase(Fase::Parado) => {
                self.ignorar();
                return Ok(());
            }
            // O tratador da voz foi adiado; esta interrupção o fecha, sem passar pela espera.
            Modo::Sega(Sega::VozEmCurso) => {}
            _ if !self.na_espera => return self.vblank_de_lag(),
            _ => {}
        }
        self.na_espera = false;
        if let Modo::Titulo(t) = self.modo {
            return self.vblank_titulo(t);
        }
        if let Modo::Fase(f) = self.modo {
            return self.vblank_fase(f);
        }
        if let Modo::Especial(e) = self.modo {
            return self.vblank_especial(e);
        }
        let Modo::Sega(passo) = self.modo else {
            self.ignorar();
            return Ok(());
        };
        match passo {
            Sega::FadeOut(n) => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                sega::fade_out_to_black(&mut self.mem);
                let n = n.saturating_sub(1);
                if n > 0 {
                    wb(
                        &mut self.mem,
                        ram::V_VBLANK_ROUTINE,
                        vblank::ROTINA_PALETTE_FADE,
                    );
                    self.pedir_espera();
                    self.modo = Modo::Sega(Sega::FadeOut(n));
                } else {
                    self.modo = Modo::Sega(Sega::Preparo);
                }
            }
            Sega::WaitPal => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
                if sega::pal_cycle_sega(&mut self.mem) {
                    wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                    self.pedir_espera();
                } else {
                    wb(&mut self.mem, ram::V_SOUNDQUEUE1, ram::SFX_SEGA);
                    wb(
                        &mut self.mem,
                        ram::V_VBLANK_ROUTINE,
                        vblank::ROTINA_SEGA_PCM,
                    );
                    self.pedir_espera();
                    self.modo = Modo::Sega(Sega::Voz);
                }
            }
            Sega::Voz => self.modo = Modo::Sega(Sega::VozEmCurso),
            Sega::VozEmCurso => {
                // O tratador da voz acabou (o driver só volta no fim da voz). Em seguida o
                // VBlank pendente roda como quadro de lag: `v_vblank_routine` é zero, então
                // só o driver de som roda.
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(2);
                self.tratar_vblank()?;
                self.tratar_vblank()?;
                self.modo = Modo::Sega(Sega::VozFim);
            }
            Sega::WaitEnd => {
                self.vblanks_tratados = self.vblanks_tratados.saturating_add(1);
                self.tratar_vblank()?;
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
                    self.modo = Modo::Titulo(Titulo::Entrada);
                } else {
                    wb(&mut self.mem, ram::V_VBLANK_ROUTINE, vblank::ROTINA_SEGA);
                    self.pedir_espera();
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

#[cfg(test)]
mod testes_transicoes {
    use super::*;

    #[test]
    fn evento_horizontal_entrega_troca_de_paleta() {
        let mut a = Abertura::nova_nominal(vec![0; 0x10000]);
        ww(&mut a.mem, ram::F_HBLANK_PAL, 1);
        a.executar_quadro(&[EventoQuadro::Interrupcao { nivel: 4, pc: 0 }]);
        assert!(a.erro.is_none());
        assert_eq!(rw(&mut a.mem, ram::F_HBLANK_PAL), 0);
        assert_eq!(a.interrupcoes_ignoradas, 0);
    }

    #[test]
    #[ignore = "requer SONIC1_ROM com a ROM REV01"]
    fn game_over_continue_fades_e_retorno_a_fase_com_rom_real() {
        let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
        let rom = std::fs::read(caminho).expect("leitura da ROM");
        let mut a = Abertura::nova_nominal(rom);
        a.modo = Modo::Fase(Fase::Jogando);
        a.na_espera = true;
        wb(&mut a.mem, ram::V_GAMEMODE, ram::ID_LEVEL);
        wb(&mut a.mem, ram::V_PLAYER + ram::OB_ID, ram::ID_SONIC);
        wb(&mut a.mem, ram::V_PLAYER + ram::OB_ROUTINE, 8);
        wb(&mut a.mem, ram::V_TITLECARD + ram::OB_ID, 0x39);
        wb(&mut a.mem, ram::V_TITLECARD + ram::OB_ROUTINE, 4);
        wb(&mut a.mem, ram::V_CONTINUES, 2);
        wb(&mut a.mem, ram::V_OPL_ROUTINE, 2);
        a.vblank().unwrap();
        for _ in 0..44 {
            a.chegar_na_espera();
            a.vblank().unwrap();
        }
        assert_eq!(a.modo, Modo::Fase(Fase::ContinueLaco));
        for _ in 0..80 {
            a.chegar_na_espera();
            a.vblank().unwrap();
        }
        assert_eq!(rb(&mut a.mem, ram::V_PLAYER + ram::OB_ROUTINE), 4);
        wb(&mut a.mem, 0xA1_0009, 0x40);
        a.mem.definir_botoes(ram::BTN_START);
        a.chegar_na_espera();
        a.vblank().unwrap();
        a.mem.definir_botoes(0);
        for _ in 0..250 {
            if a.modo == Modo::Fase(Fase::Entrada) {
                break;
            }
            a.chegar_na_espera();
            a.vblank().unwrap();
        }
        assert_eq!(a.modo, Modo::Fase(Fase::Entrada));
        assert_eq!(rb(&mut a.mem, ram::V_CONTINUES), 1);
        assert_eq!(rb(&mut a.mem, ram::V_LIVES), 3);
    }

    #[test]
    #[ignore = "requer SONIC1_ROM com a ROM REV01"]
    fn especial_via_fase_com_rom_real() {
        let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
        let rom = std::fs::read(caminho).expect("leitura da ROM");
        let mut a = Abertura::nova_nominal(rom);
        a.modo = Modo::Fase(Fase::Jogando);
        a.na_espera = true;
        wb(&mut a.mem, ram::V_GAMEMODE, ram::ID_SPECIAL);
        wb(&mut a.mem, ram::V_LIVES, 3);
        a.executar_quadro(&[EventoQuadro::Interrupcao { nivel: 6, pc: 0 }]);
        assert_eq!(a.erro, None);
        assert_eq!(a.modo, Modo::Especial(Especial::EntradaBranco(22)));
        for _ in 0..44 {
            a.executar_quadro(&[
                EventoQuadro::Marca(Marca::WaitForVBlank),
                EventoQuadro::Interrupcao { nivel: 6, pc: 0 },
            ]);
            assert_eq!(a.erro, None);
        }
        assert_eq!(a.modo, Modo::Especial(Especial::Jogando));
        assert_eq!(rb(&mut a.mem, ram::V_PLAYER + ram::OB_ID), 9);
        assert_eq!(
            rb(&mut a.mem, ram::V_VBLANK_ROUTINE),
            vblank::ROTINA_SPECIAL
        );
        for _ in 0..10 {
            a.executar_quadro(&[
                EventoQuadro::Marca(Marca::WaitForVBlank),
                EventoQuadro::Interrupcao { nivel: 6, pc: 0 },
            ]);
            assert_eq!(a.erro, None);
            assert_eq!(a.modo, Modo::Especial(Especial::Jogando));
        }
        assert_eq!(rw(&mut a.mem, ram::V_GENERICTIMER), 1790);
        wb(&mut a.mem, ram::V_GAMEMODE, ram::ID_LEVEL);
        a.executar_quadro(&[
            EventoQuadro::Marca(Marca::WaitForVBlank),
            EventoQuadro::Interrupcao { nivel: 6, pc: 0 },
        ]);
        assert_eq!(a.erro, None);
        assert_eq!(a.modo, Modo::Especial(Especial::Saida));
        for _ in 0..60 {
            a.executar_quadro(&[
                EventoQuadro::Marca(Marca::WaitForVBlank),
                EventoQuadro::Interrupcao { nivel: 6, pc: 0 },
            ]);
            assert_eq!(a.erro, None);
        }
        assert_eq!(a.modo, Modo::Especial(Especial::Resultados));
        for _ in 0..10 {
            a.executar_quadro(&[
                EventoQuadro::Marca(Marca::WaitForVBlank),
                EventoQuadro::Interrupcao { nivel: 6, pc: 0 },
            ]);
            assert_eq!(a.erro, None);
        }
        assert_eq!(a.modo, Modo::Especial(Especial::Resultados));
        for _ in 0..1500 {
            if a.modo == Modo::Fase(Fase::Entrada) {
                break;
            }
            a.executar_quadro(&[
                EventoQuadro::Marca(Marca::WaitForVBlank),
                EventoQuadro::Interrupcao { nivel: 6, pc: 0 },
            ]);
            assert_eq!(a.erro, None);
        }
        assert_eq!(a.modo, Modo::Fase(Fase::Entrada));
    }

    #[test]
    fn especial_inicia_fade_branco_e_aguarda_vblank() {
        let mut a = Abertura::nova_nominal(vec![0; 0x10000]);
        a.iniciar_especial();
        assert_eq!(a.modo, Modo::Especial(Especial::EntradaBranco(22)));
        assert_eq!(a.espera(), Espera::VBlank);
        assert_eq!(rb(&mut a.mem, ram::V_SOUNDQUEUE1), 0xCA);
        assert_eq!(rw(&mut a.mem, ram::V_PFADE_START), 0x003F);
        a.chegar_na_espera();
        a.vblank().unwrap();
        assert_eq!(a.modo, Modo::Especial(Especial::EntradaBranco(21)));
        assert_eq!(rw(&mut a.mem, ram::V_PALETTE), 2);
    }

    #[test]
    fn resultado_espera_fila_e_retorna_a_fase_apos_fade() {
        let mut a = Abertura::nova_nominal(vec![0; 0x10000]);
        a.modo = Modo::Especial(Especial::Resultados);
        a.na_espera = true;
        ww(&mut a.mem, ram::F_RESTART, 1);
        a.vblank().unwrap();
        assert_eq!(a.modo, Modo::Especial(Especial::SaidaBranco(22)));
        for _ in 0..22 {
            a.chegar_na_espera();
            a.vblank().unwrap();
        }
        assert_eq!(a.modo, Modo::Fase(Fase::Entrada));
    }

    #[test]
    fn game_over_com_credito_inicia_fade_do_continue() {
        let mut a = Abertura::nova_nominal(vec![0; 0x40000]);
        a.modo = Modo::Fase(Fase::Jogando);
        a.na_espera = true;
        wb(&mut a.mem, ram::V_GAMEMODE, ram::ID_LEVEL);
        wb(&mut a.mem, ram::V_PLAYER + ram::OB_ID, ram::ID_SONIC);
        wb(&mut a.mem, ram::V_PLAYER + ram::OB_ROUTINE, 8);
        wb(&mut a.mem, ram::V_TITLECARD + ram::OB_ID, 0x39);
        wb(&mut a.mem, ram::V_TITLECARD + ram::OB_ROUTINE, 4);
        wb(&mut a.mem, ram::V_CONTINUES, 1);
        wb(&mut a.mem, ram::V_OPL_ROUTINE, 2);
        a.vblank().unwrap();
        assert_eq!(a.modo, Modo::Fase(Fase::ContinueFadeOut(22)));
        assert_eq!(rb(&mut a.mem, ram::V_GAMEMODE), 0x14);
    }

    #[test]
    fn vblank_titulo_jogar_inicia_fase_e_zera_demo() {
        let mut a = Abertura::nova_nominal(vec![0; 0x10000]);
        a.modo = Modo::Titulo(Titulo::Laco);
        a.na_espera = true;
        wb(&mut a.mem, ram::V_JPADPRESS1, ram::BTN_START);
        ww(&mut a.mem, ram::V_GENERICTIMER, 100);

        a.vblank().unwrap();
        assert_eq!(a.modo, Modo::Fase(Fase::Entrada));
        assert_eq!(rb(&mut a.mem, ram::V_GAMEMODE), ram::ID_LEVEL);
        assert_eq!(rw(&mut a.mem, ram::F_DEMO), 0);
        assert_eq!(rb(&mut a.mem, ram::V_LIVES), 3);
    }

    #[test]
    fn fase_jogando_com_reinicio_reinicia_fase() {
        let mut a = Abertura::nova_nominal(vec![0; 0x10000]);
        a.modo = Modo::Fase(Fase::Jogando);
        a.na_espera = true;
        ww(&mut a.mem, ram::F_RESTART, 1);

        a.vblank().unwrap();
        assert_eq!(a.modo, Modo::Fase(Fase::FadeOut(22)));
    }
}
