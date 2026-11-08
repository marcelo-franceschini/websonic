//! Harness do som (sub-etapa 6.5): o jogo real roda no oráculo e o driver do núcleo roda a mesma
//! chamada de `UpdateMusic`, quadro a quadro, de forma contínua. Compara o estado do driver, a
//! RAM do Z80, a posição do Z80 e a sequência de escritas nos chips de som.
//!
//! Roda só pelo Marcelo, com a ROM. O relatório tem números, hashes e vereditos. Nunca bytes de
//! áudio nem de dados da ROM.

use crate::acesso_de;
use oracle_trace::TempoRaw;
use sonic1_core::memoria::{ClasseLeitura, Memoria, Saida};
use sonic1_core::som::driver::Driver;
use sonic1_core::som::z80::PontoZ80;
use sonic1_fidelidade::hash::sha1_de_partes;
use sonic1_fidelidade::{Evento, Origem, Tipo};
use websonic_oracle::oraculo::{MotivoParada, Parada};
use websonic_oracle::{Entradas, Oraculo};

/// Início e tamanho da região do driver na RAM do 68000.
pub const BASE_RAM: u32 = 0xFF_F000;
pub const TAMANHO_REGIAO: usize = 0x5C0;
const V_SOUNDQUEUE0: u32 = BASE_RAM + 0x0A;
const V_SOUNDQUEUE1: u32 = BASE_RAM + 0x0B;
const V_SFX_TRACK_RAM: u32 = BASE_RAM + 0x220;
const V_SPCSFX_FIM: u32 = BASE_RAM + 0x3A0;
const LIMITE_PASSOS: u64 = 400_000_000;

/// Evento de som na forma comparável: (tipo, a, b). Tipo 0 é YM do 68000, 1 YM do Z80, 2 PSG.
pub type EventoSom = (u8, u8, u8);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Falha {
    /// A sequência de escritas difere. Índice do primeiro evento diferente.
    Escritas {
        indice: usize,
    },
    RegiaoDoDriver {
        deslocamento: usize,
    },
    RamDoZ80 {
        deslocamento: usize,
    },
    PosicaoDoZ80 {
        descricao: String,
    },
    TempoNaoConsumido {
        pontos: usize,
        leituras: usize,
    },
    EsperaEstourada,
}

/// Resultado de um ID.
#[derive(Clone, Debug)]
pub struct RelatorioId {
    pub id: u8,
    pub nome: String,
    pub quadros: u32,
    pub esperadas: usize,
    pub obtidas: usize,
    pub hash_esperado: [u8; 20],
    pub hash_obtido: [u8; 20],
    pub externas: u32,
    pub falha: Option<(u32, Falha)>,
}

impl RelatorioId {
    pub fn igual(&self) -> bool {
        self.falha.is_none()
    }
}

pub fn hex(h: &[u8; 20]) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}

fn hash_eventos(e: &[EventoSom]) -> [u8; 20] {
    let bytes: Vec<u8> = e.iter().flat_map(|(t, a, b)| [*t, *a, *b]).collect();
    sha1_de_partes(&[&bytes])
}

/// Escritas do oráculo na forma comparável, com o marcador de BUSREQ liberado.
#[derive(Clone, Copy, Debug)]
enum ItemOraculo {
    Som(EventoSom),
    Libera,
}

fn item_do_evento(e: &Evento) -> Option<ItemOraculo> {
    match e.tipo {
        Tipo::Ym => Some(ItemOraculo::Som((
            u8::from(e.origem == Origem::Z80),
            u8::try_from(e.a & 3).unwrap_or(0),
            u8::try_from(e.b & 0xFF).unwrap_or(0),
        ))),
        Tipo::Psg => Some(ItemOraculo::Som((
            2,
            u8::try_from(e.a & 0xFF).unwrap_or(0),
            0,
        ))),
        Tipo::BarramentoZ80 if e.a == 0 && e.b == 0 => Some(ItemOraculo::Libera),
        _ => None,
    }
}

/// Um trecho de `UpdateMusic` visto no oráculo.
struct Trecho {
    entrada_mclk: u64,
    saida_mclk: u64,
    regiao_entrada: Vec<u8>,
    regiao_saida: Vec<u8>,
    z80_saida: Vec<u8>,
    z80_entrada: Vec<u8>,
}

pub struct Config {
    pub rom: Vec<u8>,
    pub update_music: u32,
    pub quadros_base: u32,
    pub quadros_musica: u32,
    pub trechos_maximo_efeito: u32,
}

fn regiao(o: &Oraculo) -> Vec<u8> {
    let mut v = Vec::with_capacity(TAMANHO_REGIAO);
    for i in 0..TAMANHO_REGIAO {
        v.push(o.ler_byte_ram68k(BASE_RAM + u32::try_from(i).unwrap_or(0)));
    }
    v
}

fn sp_retorno(o: &Oraculo) -> u32 {
    let ssp = o.emulador().oracle_regs68().ssp;
    let b = |i: u32| u32::from(o.ler_byte_ram68k(ssp.wrapping_add(i)));
    (b(0) << 24) | (b(1) << 16) | (b(2) << 8) | b(3)
}

/// Roda o oráculo até a próxima entrada de `UpdateMusic`.
fn ate_entrada(o: &mut Oraculo, cfg: &Config, eventos: &mut Vec<Evento>) -> Result<(), String> {
    let r = o.executar_ate(&[Parada::Pc68(cfg.update_music)], LIMITE_PASSOS, &mut |q| {
        eventos.extend(q.eventos);
    })?;
    if r == MotivoParada::LimiteDePassos {
        return Err("limite de passos antes de UpdateMusic".into());
    }
    Ok(())
}

/// Roda `f` numa thread nova: os contadores do tempo gravado do oráculo são por thread e têm de
/// começar do zero em cada execução.
pub fn em_thread_nova<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(f)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|_| "a thread do oraculo entrou em panico".to_string())?
}

/// Estado calmo: o jogo roda até `quadros_base`, o som é parado e deixa-se passar alguns quadros.
pub fn preparar_base(cfg: &Config) -> Result<Vec<u8>, String> {
    oracle_trace::tempo_ligar();
    let mut o = Oraculo::novo(cfg.rom.clone(), Entradas::default())?;
    let mut lixo = Vec::new();
    loop {
        ate_entrada(&mut o, cfg, &mut lixo)?;
        if o.quadro_atual() >= cfg.quadros_base {
            break;
        }
    }
    o.emulador_mut()
        .oracle_escrever_ram68k_byte(V_SOUNDQUEUE0, 0xE4);
    for _ in 0..4 {
        ate_entrada(&mut o, cfg, &mut lixo)?;
    }
    o.salvar_estado()
}

fn injetar_id(o: &mut Oraculo, id: u8) {
    let alvo = if (0xA0..=0xDF).contains(&id) {
        V_SOUNDQUEUE1
    } else {
        V_SOUNDQUEUE0
    };
    o.emulador_mut().oracle_escrever_ram68k_byte(alvo, id);
}

fn faixa_de_efeitos_parada(o: &Oraculo) -> bool {
    (V_SFX_TRACK_RAM..V_SPCSFX_FIM)
        .step_by(0x30)
        .all(|a| o.ler_byte_ram68k(a) & 0x80 == 0)
}

/// Roda um ID: o oráculo primeiro (guardando tudo), depois o núcleo sobre o que foi gravado.
pub fn rodar_id(cfg: &Config, base: &[u8], id: u8, nome: &str) -> Result<RelatorioId, String> {
    oracle_trace::tempo_ligar();
    let mut o = Oraculo::novo(cfg.rom.clone(), Entradas::default())?;
    o.carregar_estado(base)?;
    let mut descarte = Vec::new();
    oracle_trace::drain_tempo(&mut descarte);
    injetar_id(&mut o, id);
    let e_musica = (0x81..=0x9F).contains(&id);
    let teto = if e_musica {
        cfg.quadros_musica
    } else {
        cfg.trechos_maximo_efeito
    };

    // Estado inicial do núcleo, copiado do oráculo na entrada do primeiro trecho.
    let mut mem = Memoria::nova(cfg.rom.clone());
    mem.ram_mut().copy_from_slice(&o.ram68k());
    mem.ram_z80_mut().copy_from_slice(&o.ram_z80());
    mem.definir_estado_z80(false, false);
    let pc80 = o.emulador().oracle_pc80();
    let valor = o.ram_z80()[0x1FFF];
    mem.z80_mut()
        .injetar_driver_dac_ocioso(pc80, 0, 0x00F, valor);

    let mut eventos: Vec<Evento> = Vec::new();
    let mut trechos: Vec<Trecho> = Vec::new();
    for n in 0..teto {
        let entrada_mclk = o.emulador().oracle_mclk();
        let regiao_entrada = regiao(&o);
        let z80_entrada = o.ram_z80();
        let ret = sp_retorno(&o);
        let r = o.executar_ate(&[Parada::Pc68(ret)], LIMITE_PASSOS, &mut |q| {
            eventos.extend(q.eventos);
        })?;
        if r == MotivoParada::LimiteDePassos {
            return Err(format!("{nome}: UpdateMusic nao retornou (trecho {n})"));
        }
        let saida_mclk = o.emulador().oracle_mclk();
        trechos.push(Trecho {
            entrada_mclk,
            saida_mclk,
            regiao_entrada,
            regiao_saida: regiao(&o),
            z80_entrada,
            z80_saida: o.ram_z80(),
        });
        if !e_musica && n >= 20 && faixa_de_efeitos_parada(&o) {
            break;
        }
        ate_entrada(&mut o, cfg, &mut eventos)?;
    }
    // Fecha o quadro em andamento para que os eventos do último trecho cheguem à lista.
    for _ in 0..2 {
        eventos.extend(o.executar_quadro()?.eventos);
    }
    let mut brutos = Vec::new();
    oracle_trace::drain_tempo(&mut brutos);
    Ok(comparar(id, nome, &trechos, &eventos, &brutos, mem))
}

fn comparar(
    id: u8,
    nome: &str,
    trechos: &[Trecho],
    eventos: &[Evento],
    brutos: &[TempoRaw],
    mut mem: Memoria,
) -> RelatorioId {
    let itens: Vec<(u64, ItemOraculo)> = eventos
        .iter()
        .filter_map(|e| item_do_evento(e).map(|i| (e.mclk, i)))
        .collect();
    let pontos: Vec<PontoZ80> = brutos
        .iter()
        .filter_map(|r| match *r {
            TempoRaw::Z80Ponto {
                acesso,
                pc,
                espera,
                ativos,
                banco,
                endereco,
                byte,
                mclk,
            } => acesso_de(acesso).map(|acesso| PontoZ80 {
                acesso,
                pc,
                espera,
                ativos,
                banco,
                endereco,
                byte,
                mclk,
            }),
            _ => None,
        })
        .collect();
    let leituras: Vec<(ClasseLeitura, u16, u64)> = brutos
        .iter()
        .filter_map(|r| match *r {
            TempoRaw::Leitura {
                classe,
                valor,
                mclk,
            } => {
                let c = match classe {
                    oracle_trace::leitura::BUSACK => ClasseLeitura::Busack,
                    oracle_trace::leitura::VDP_STATUS => ClasseLeitura::VdpStatus,
                    oracle_trace::leitura::YM_STATUS => ClasseLeitura::YmStatus,
                    _ => return None,
                };
                Some((c, valor, mclk))
            }
            _ => None,
        })
        .collect();
    let mut rel = RelatorioId {
        id,
        nome: nome.to_string(),
        quadros: u32::try_from(trechos.len()).unwrap_or(u32::MAX),
        esperadas: 0,
        obtidas: 0,
        hash_esperado: [0; 20],
        hash_obtido: [0; 20],
        externas: 0,
        falha: None,
    };
    let mut todas_esperadas: Vec<EventoSom> = Vec::new();
    let mut todas_obtidas: Vec<EventoSom> = Vec::new();
    let (mut ip, mut il, mut ie) = (0usize, 0usize, 0usize);
    let calibrado = calibrar_espera(&mut mem, pontos.first());
    let _ = calibrado;
    // Depois de um trecho longo (voz SEGA) o Z80 do oráculo anda sozinho; o do núcleo só alcança
    // no primeiro ponto do trecho seguinte. Enquanto isso a RAM do Z80 não é comparável.
    let mut z80_defasado = false;
    for (k, t) in trechos.iter().enumerate() {
        let k32 = u32::try_from(k).unwrap_or(u32::MAX);
        // Acessos ao Z80 feitos por código que o núcleo não roda.
        while ip < pontos.len() && pontos[ip].mclk < t.entrada_mclk {
            mem.aplicar_ponto_externo(&pontos[ip]);
            ip += 1;
        }
        // Escritas feitas por fora do driver entre dois trechos.
        for i in 0..TAMANHO_REGIAO {
            let a = BASE_RAM.wrapping_add(u32::try_from(i).unwrap_or(0));
            let atual = mem.ram()[usize::try_from(a & 0xFFFF).unwrap_or(0)];
            if atual != t.regiao_entrada[i] {
                mem.ram_mut()[usize::try_from(a & 0xFFFF).unwrap_or(0)] = t.regiao_entrada[i];
                rel.externas = rel.externas.saturating_add(1);
            }
        }
        if !z80_defasado
            && let Some(d) =
                (0..t.z80_entrada.len()).find(|&d| mem.ram_z80()[d] != t.z80_entrada[d])
        {
            rel.falha = Some((k32, Falha::RamDoZ80 { deslocamento: d }));
            break;
        }
        // Tempo gravado do trecho.
        while ip < pontos.len() && pontos[ip].mclk <= t.saida_mclk {
            mem.enfileirar_ponto_z80(pontos[ip]);
            ip += 1;
        }
        while il < leituras.len() && leituras[il].2 <= t.saida_mclk {
            if leituras[il].2 >= t.entrada_mclk {
                mem.enfileirar_leitura(leituras[il].0, leituras[il].1);
            }
            il += 1;
        }
        let ultimo_ponto_mclk = pontos[..ip]
            .iter()
            .rev()
            .map(|p| p.mclk)
            .find(|m| *m >= t.entrada_mclk);
        let trecho_longo = ultimo_ponto_mclk.is_some_and(|m| t.saida_mclk.saturating_sub(m) > 3000);
        let esperas = {
            let mut d = Driver::novo(&mut mem);
            d.update_music();
            d.esperas_estouradas
        };
        // Escritas esperadas: do ponto onde parou até o fim do trecho. O fim é o último BUSREQ
        // liberado (depois dele o Z80 anda sozinho e esses eventos entram no trecho seguinte),
        // a menos que haja escrita do 68000 depois dele (um trecho que tomou o barramento de
        // novo e não o devolveu).
        let ultimo_libera = (ie..itens.len())
            .rfind(|&i| matches!(itens[i].1, ItemOraculo::Libera) && itens[i].0 <= t.saida_mclk);
        let ultimo_68000 = (ie..itens.len()).rfind(|&i| {
            matches!(itens[i].1, ItemOraculo::Som((tipo, _, _)) if tipo != 1)
                && itens[i].0 <= t.saida_mclk
        });
        let fim = match (ultimo_libera, ultimo_68000) {
            (Some(l), Some(s)) => {
                if l > s {
                    l
                } else {
                    s.saturating_add(1)
                }
            }
            (Some(l), None) => l,
            (None, Some(s)) => s.saturating_add(1),
            (None, None) => ie,
        };
        let esperadas: Vec<EventoSom> = itens[ie..fim.min(itens.len())]
            .iter()
            .filter_map(|(_, i)| match i {
                ItemOraculo::Som(e) => Some(*e),
                ItemOraculo::Libera => None,
            })
            .collect();
        if std::env::var("WEBSONIC_DEBUG").is_ok() && k < 3 {
            let dentro = itens
                .iter()
                .filter(|(m, _)| *m >= t.entrada_mclk && *m <= t.saida_mclk)
                .count();
            let libera = itens
                .iter()
                .filter(|(m, i)| {
                    *m >= t.entrada_mclk && *m <= t.saida_mclk && matches!(i, ItemOraculo::Libera)
                })
                .count();
            eprintln!(
                "trecho {k}: entrada={} saida={} ie={ie} fim={fim} itens={} dentro={dentro} libera_dentro={libera}",
                t.entrada_mclk,
                t.saida_mclk,
                itens.len()
            );
        }
        ie = (fim + 1).min(itens.len());
        let obtidas: Vec<EventoSom> = mem
            .tirar_saidas()
            .into_iter()
            .filter_map(|s| match s {
                Saida::Ym { porta, valor } => Some((0, porta, valor)),
                Saida::YmZ80 { porta, valor } => Some((1, porta, valor)),
                Saida::Psg(v) => Some((2, v, 0)),
                _ => None,
            })
            .collect();
        todas_esperadas.extend_from_slice(&esperadas);
        todas_obtidas.extend_from_slice(&obtidas);
        if rel.falha.is_none() {
            if esperas > 0 {
                rel.falha = Some((k32, Falha::EsperaEstourada));
            } else if let Some(d) = mem.divergencias_z80().first() {
                rel.falha = Some((
                    k32,
                    Falha::PosicaoDoZ80 {
                        descricao: format!("{d:?}"),
                    },
                ));
            } else if esperadas != obtidas {
                let i = esperadas
                    .iter()
                    .zip(obtidas.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(esperadas.len().min(obtidas.len()));
                rel.falha = Some((
                    k32,
                    Falha::Escritas {
                        indice: todas_esperadas
                            .len()
                            .saturating_sub(esperadas.len())
                            .saturating_add(i),
                    },
                ));
            } else if let Some(d) = (0..TAMANHO_REGIAO).find(|&i| {
                mem.ram()[usize::try_from(
                    BASE_RAM.wrapping_add(u32::try_from(i).unwrap_or(0)) & 0xFFFF,
                )
                .unwrap_or(0)]
                    != t.regiao_saida[i]
            }) {
                rel.falha = Some((k32, Falha::RegiaoDoDriver { deslocamento: d }));
            } else if let Some(d) =
                (0..t.z80_saida.len()).find(|&d| mem.ram_z80()[d] != t.z80_saida[d])
            {
                rel.falha = Some((k32, Falha::RamDoZ80 { deslocamento: d }));
            } else if mem.pontos_z80_pendentes() > 0 {
                rel.falha = Some((
                    k32,
                    Falha::TempoNaoConsumido {
                        pontos: mem.pontos_z80_pendentes(),
                        leituras: mem.leituras_pendentes(),
                    },
                ));
            }
        }
        z80_defasado = trecho_longo;
        if rel.falha.is_some() {
            break;
        }
    }
    rel.esperadas = todas_esperadas.len();
    rel.obtidas = todas_obtidas.len();
    rel.hash_esperado = hash_eventos(&todas_esperadas);
    rel.hash_obtido = hash_eventos(&todas_obtidas);
    rel
}

/// O estado do Z80 injetado não traz os ciclos que faltam da instrução em curso. Escolhe o valor
/// (de 0 a 22) que leva o núcleo à posição gravada no primeiro ponto.
fn calibrar_espera(mem: &mut Memoria, primeiro: Option<&PontoZ80>) -> bool {
    let Some(p) = primeiro else { return false };
    for e in 0..=22u32 {
        let mut teste = mem.clone();
        teste.z80_mut().definir_espera(e);
        teste.avancar_z80(u64::from(p.ativos));
        if (
            teste.z80().pc(),
            u16::try_from(teste.z80().espera()).unwrap_or(u16::MAX),
            teste.z80().banco().valor(),
        ) == (p.pc, p.espera, p.banco)
        {
            mem.z80_mut().definir_espera(e);
            return true;
        }
    }
    false
}

/// Nome do campo da RAM do driver que contém o deslocamento (só nomes do mapa de rótulos).
pub fn nome_do_deslocamento(d: usize) -> String {
    const CAMPOS_RAM: [(usize, &str); 24] = [
        (0x00, "v_sndprio"),
        (0x01, "v_main_tempo_timeout"),
        (0x02, "v_main_tempo"),
        (0x03, "f_pausemusic"),
        (0x04, "v_fadeout_counter"),
        (0x06, "v_fadeout_delay"),
        (0x07, "v_communication_byte"),
        (0x08, "f_updating_dac"),
        (0x09, "v_sound_id"),
        (0x0A, "v_soundqueue0"),
        (0x0B, "v_soundqueue1"),
        (0x0C, "v_soundqueue2"),
        (0x0E, "f_voice_selector"),
        (0x18, "v_voice_ptr"),
        (0x20, "v_special_voice_ptr"),
        (0x24, "f_fadein_flag"),
        (0x25, "v_fadein_delay"),
        (0x26, "v_fadein_counter"),
        (0x27, "f_1up_playing"),
        (0x28, "v_tempo_mod"),
        (0x29, "v_speeduptempo"),
        (0x2A, "f_speedup"),
        (0x2B, "v_ring_speaker"),
        (0x2C, "f_push_playing"),
    ];
    const CAMPOS_TRILHA: [(usize, &str); 27] = [
        (0x00, "PlaybackControl"),
        (0x01, "VoiceControl"),
        (0x02, "TempoDivider"),
        (0x04, "DataPointer"),
        (0x08, "Transpose"),
        (0x09, "Volume"),
        (0x0A, "AMSFMSPan"),
        (0x0B, "VoiceIndex"),
        (0x0C, "VolEnvIndex"),
        (0x0D, "StackPointer"),
        (0x0E, "DurationTimeout"),
        (0x0F, "SavedDuration"),
        (0x10, "Freq"),
        (0x12, "NoteTimeout"),
        (0x13, "NoteTimeoutMaster"),
        (0x14, "ModulationPtr"),
        (0x18, "ModulationWait"),
        (0x19, "ModulationSpeed"),
        (0x1A, "ModulationDelta"),
        (0x1B, "ModulationSteps"),
        (0x1C, "ModulationVal"),
        (0x1E, "Detune"),
        (0x1F, "FeedbackAlgo"),
        (0x20, "VoicePtr"),
        (0x24, "LoopCounters"),
        (0x28, "LoopCounters+4"),
        (0x2C, "LoopCounters+8"),
    ];
    const TRILHAS: [&str; 18] = [
        "dac",
        "fm1",
        "fm2",
        "fm3",
        "fm4",
        "fm5",
        "fm6",
        "psg1",
        "psg2",
        "psg3",
        "sfx_fm3",
        "sfx_fm4",
        "sfx_fm5",
        "sfx_psg1",
        "sfx_psg2",
        "sfx_psg3",
        "spcsfx_fm4",
        "spcsfx_psg3",
    ];
    if d < 0x40 {
        let campo = CAMPOS_RAM
            .iter()
            .rev()
            .find(|(o, _)| *o <= d)
            .map_or("?", |c| c.1);
        return format!("SMPS_RAM.{campo}");
    }
    if d >= 0x3A0 {
        return format!("v_1up_ram_copy+{:#x}", d - 0x3A0);
    }
    let i = (d - 0x40) / 0x30;
    let o = (d - 0x40) % 0x30;
    let campo = CAMPOS_TRILHA
        .iter()
        .rev()
        .find(|(x, _)| *x <= o)
        .map_or("?", |c| c.1);
    format!(
        "trilha {} {campo}+{}",
        TRILHAS.get(i).copied().unwrap_or("?"),
        o
    )
}
