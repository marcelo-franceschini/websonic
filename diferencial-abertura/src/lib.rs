//! Comparação da abertura (Etapa 7): o oráculo roda a ROM quadro a quadro; o núcleo recebe o
//! tempo gravado do mesmo quadro (marcas, leituras e pontos do Z80) e roda a abertura. O
//! relatório compara, por quadro, a RAM do 68000 (sem a pilha), a RAM do Z80 (sem `1FFD` e
//! `1FFF`) e as sequências de escritas de vídeo, som e barramento do Z80.
//!
//! Roda só pelo Marcelo, com a ROM. Saída: contagens, nomes, endereços e vereditos. Nunca
//! valores de memória, pixels nem áudio.

use oracle_trace::TempoRaw;
use sonic1_core::abertura::ram::{PILHA_FIM, PILHA_INICIO};
use sonic1_core::abertura::{Abertura, EventoQuadro, MARCAS};
use sonic1_core::memoria::{ClasseLeitura, PortaVdp, Saida, Tamanho};
use sonic1_core::nivel2::{laco_da_limpeza, laco_da_soma, laco_da_voz};
use sonic1_core::quadro::tempo::AcessoZ80;
use sonic1_core::som::z80::PontoZ80;
use sonic1_fidelidade::hash::{em_hex, sha1_de_partes};
use sonic1_fidelidade::{Origem, Tipo};
use std::collections::BTreeMap;
use websonic_oracle::{Entradas, Oraculo};

/// Posições da RAM do Z80 que o Z80 muda sozinho (decisão da Etapa 6).
const RACE_Z80: [usize; 2] = [0x1FFD, 0x1FFF];
/// `WaitForVBlank` (metadado de `refs/rotulos.txt`).
const WAIT_FOR_VBLANK: u32 = 0x29A0;
/// Estado do oráculo no fechamento de um quadro: RAM do 68000, RAM do Z80, número do quadro,
/// se o fechamento é comparável e se o núcleo estava em trecho atômico.
type Fechamento = (Vec<u8>, Vec<u8>, u32, bool, bool);
const NOMES_SEQ: [&str; 5] = ["video", "psg", "ym68", "ymz80", "z80"];
const LIMITE_PASSOS_QUADRO: u64 = 50_000_000;

/// Escrita comparável: (tipo, a, b, c). Tipo 0 vídeo, 1 PSG, 2 YM do 68000, 3 YM do Z80, 4 Z80.
pub type Escrita = (u8, u32, u32, u32);

#[derive(Default)]
pub struct Relatorio {
    pub quadros: u32,
    pub ram_iguais: u32,
    pub ram_diverge: u32,
    pub ram_diverge_atomico: u32,
    pub z80_iguais: u32,
    pub z80_diverge: u32,
    pub z80_diverge_atomico: u32,
    pub escritas_oraculo: [u64; 5],
    pub escritas_nucleo: [u64; 5],
    pub escritas_diverge: Option<String>,
    pub primeira_ram: Option<String>,
    pub primeira_z80: Option<String>,
    pub atraso_maximo: u64,
    pub erro_nucleo: Option<String>,
    pub divergencias_posicao_z80: usize,
    /// As primeiras divergências de posição do Z80 (PC, ciclos que faltam, banco): diagnóstico.
    pub z80_primeiras: Vec<String>,
    pub z80_por_acesso: BTreeMap<String, u32>,
    /// Nomes das variáveis que divergem na RAM do 68000, com a contagem de quadros.
    pub ram_variaveis: BTreeMap<String, u32>,
    /// Contexto do primeiro acesso ao Z80 que difere: índice, oráculo e núcleo.
    pub z80_contexto: Option<String>,
    pub z80_fila: Option<String>,
    pub int6_oraculo: u32,
    /// Experimento do nível 2: (nome, medido em ciclos do 68000, previsto).
    pub nivel2: Vec<(String, u64, u64)>,
    /// Comparação nas esperas de VBlank (D7.4): o laço principal chegou a `WaitForVBlank`.
    pub esperas_iguais: u32,
    pub esperas_diverge: u32,
    pub esperas_z80_diverge: u32,
    pub primeira_espera: Option<String>,
    pub esperas_variaveis: BTreeMap<String, u32>,
    pub esperas_oraculo: u32,
    pub esperas_nucleo: u32,
    /// Quadros cujo fechamento cai dentro do tratador de VBlank ou com VBlank no quadro.
    pub fechamento_nao_comparavel: u32,
    pub int6_nucleo_tratadas: u32,
    pub int6_nucleo_ignoradas: u32,
    pub ignoradas_em: Vec<(u32, String)>,
    /// Quadros em que cada marca chegou.
    pub marcas_vistas: BTreeMap<String, Vec<u32>>,
    /// Sequências do Z80 (sinal e estado), só para diagnóstico: oráculo e núcleo.
    pub z80_oraculo: Vec<(u32, u32)>,
    pub z80_nucleo: Vec<(u32, u32)>,
}

impl Relatorio {
    /// Zero divergência fora dos trechos atômicos e nas sequências de escritas.
    pub fn zero(&self) -> bool {
        self.ram_diverge == 0
            && self.esperas_diverge == 0
            && self.esperas_z80_diverge == 0
            && self.z80_diverge == 0
            && self.escritas_diverge.is_none()
            && self.erro_nucleo.is_none()
            && self.divergencias_posicao_z80 == 0
    }
}

fn acesso_de(codigo: u8) -> Option<AcessoZ80> {
    use oracle_trace::acesso_z80::*;
    Some(match codigo {
        BUSREQ_PEDIDO => AcessoZ80::BusReqPedido,
        BUSREQ_LIBERADO => AcessoZ80::BusReqLiberado,
        RESET_LIGADO => AcessoZ80::ResetLigado,
        RESET_SOLTO => AcessoZ80::ResetSolto,
        LEITURA_JANELA => AcessoZ80::LeituraJanela,
        ESCRITA_JANELA => AcessoZ80::EscritaJanela,
        _ => return None,
    })
}

/// Nome da variável que contém um endereço da RAM, pelo mapa de rótulos (só metadado).
pub struct Nomes(Vec<(u32, String)>);

impl Nomes {
    pub fn carregar(texto: &str) -> Self {
        let mut v: Vec<(u32, String)> = texto
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| {
                let mut c = l.split('\t');
                let nome = c.next()?;
                let tipo = c.next()?;
                let valor = u32::from_str_radix(c.next()?, 16).ok()?;
                (tipo == "variavel" && valor >= 0xFF_0000)
                    .then(|| (valor & 0xFF_FFFF, nome.to_string()))
            })
            .collect();
        v.sort();
        Self(v)
    }

    pub fn de(&self, endereco: u32) -> String {
        let i = self.0.partition_point(|(a, _)| *a <= endereco);
        match i.checked_sub(1).and_then(|i| self.0.get(i)) {
            Some((a, n)) => format!("{n}+{:#x}", endereco - a),
            None => "?".into(),
        }
    }
}

fn mascarar_ram(ram: &[u8]) -> Vec<u8> {
    let mut v = ram.to_vec();
    let (i, f) = (
        usize::try_from(PILHA_INICIO & 0xFFFF).unwrap_or(0),
        usize::try_from(PILHA_FIM & 0xFFFF).unwrap_or(0),
    );
    if let Some(r) = v.get_mut(i..f) {
        r.fill(0);
    }
    v
}

fn mascarar_z80(ram: &[u8]) -> Vec<u8> {
    let mut v = ram.to_vec();
    for p in RACE_Z80 {
        if let Some(b) = v.get_mut(p) {
            *b = 0;
        }
    }
    v
}

fn primeira_diferenca(a: &[u8], b: &[u8]) -> Option<(usize, usize)> {
    let primeira = a.iter().zip(b).position(|(x, y)| x != y)?;
    let total = a.iter().zip(b).filter(|(x, y)| x != y).count();
    Some((primeira, total))
}

/// Escritas do oráculo num quadro, na ordem do trace.
fn escritas_do_quadro(q: &sonic1_fidelidade::Quadro) -> Vec<Escrita> {
    let mut v = Vec::new();
    for e in &q.eventos {
        match (e.tipo, e.origem) {
            (Tipo::Vdp, _) => v.push((0, e.a, e.b, e.c)),
            (Tipo::Psg, _) => v.push((1, e.a, 0, 0)),
            (Tipo::Ym, Origem::Cpu68) => v.push((2, e.a, e.b, 0)),
            (Tipo::Ym, _) => v.push((3, e.a, e.b, 0)),
            (Tipo::BarramentoZ80, _) => v.push((4, e.a, e.b, 0)),
            _ => {}
        }
    }
    v
}

fn escrita_do_nucleo(s: &Saida) -> Option<Escrita> {
    Some(match *s {
        Saida::Vdp {
            porta,
            valor,
            tamanho,
        } => (
            0,
            u32::from(porta == PortaVdp::Controle),
            u32::from(valor),
            if tamanho == Tamanho::Byte { 1 } else { 2 },
        ),
        Saida::Psg(v) => (1, u32::from(v), 0, 0),
        Saida::Ym { porta, valor } => (2, u32::from(porta), u32::from(valor), 0),
        Saida::YmZ80 { porta, valor } => (3, u32::from(porta), u32::from(valor), 0),
        Saida::BusReq(b) => (4, 0, u32::from(b), 0),
        Saida::ResetZ80(b) => (4, 1, u32::from(b), 0),
    })
}

/// Roda o oráculo e o núcleo juntos por `quadros` quadros.
pub fn comparar(rom: Vec<u8>, quadros: u32, nomes: &Nomes) -> Result<Relatorio, String> {
    oracle_trace::tempo_ligar();
    // Experimento do nível 2 (7.8): o modelo lê o código na ROM e prevê o custo de três laços.
    let soma = laco_da_soma(&rom, 0x31C);
    let limpeza = laco_da_limpeza(&rom, 0x364);
    let voz = laco_da_voz(&rom, 0x7_1FAC);
    let mut extras: Vec<(&'static str, u32)> = Vec::new();
    for (nome, p, ini) in [
        ("soma_da_rom", soma, 0x31Cu32),
        ("limpeza_da_ram", limpeza, 0x364),
        ("espera_da_voz", voz, 0x7_1FAC),
    ] {
        if let Some(p) = p {
            extras.push((nome, ini));
            extras.push((nome, p.fim));
        }
    }
    let mut extras_mclk: BTreeMap<(&'static str, u32), u64> = BTreeMap::new();
    let mut o = Oraculo::novo(rom.clone(), Entradas::default())?;
    let mut nucleo = Abertura::nova(rom);
    nucleo.mem.registrar_acessos_z80();
    let mut acessos_oraculo: Vec<AcessoZ80> = Vec::new();
    let mut rel = Relatorio::default();
    let mut esperadas: [Vec<Escrita>; 5] = Default::default();
    let mut obtidas: [Vec<Escrita>; 5] = Default::default();
    let mut verificadas = [0usize; 5];
    // Estado inicial do console: BUSREQ solto e Z80 em RESET.
    let mut z80_estado = [0u32, 1u32];
    let mut descarte = Vec::new();
    oracle_trace::drain_tempo(&mut descarte);
    let mut ultimo_pc = u32::MAX;
    let mut pend: Option<Fechamento> = None;
    nucleo.registrar_esperas();
    let mut esperas_o: std::collections::VecDeque<(Vec<u8>, Vec<u8>)> = Default::default();
    let mut esperas_n: std::collections::VecDeque<(Vec<u8>, Vec<u8>)> = Default::default();

    while o.quadro_atual() < quadros {
        let mut marcas: Vec<(u64, usize)> = Vec::new();
        let mut passos = 0u64;
        let (q, mascara_fecha) = loop {
            let q = o.passo()?;
            let regs = o.emulador().oracle_regs68();
            let pc = regs.pc;
            let mascara = (regs.sr >> 8) & 7;
            // Chegada: o PC mudou para o endereço da marca (laço de uma instrução conta uma vez).
            let chegou = pc != ultimo_pc;
            ultimo_pc = pc;
            if chegou && let Some((nome, _)) = extras.iter().find(|(_, a)| *a == pc) {
                extras_mclk
                    .entry((*nome, pc))
                    .or_insert_with(|| o.emulador().oracle_mclk());
            }
            if chegou && pc == WAIT_FOR_VBLANK {
                esperas_o.push_back((o.ram68k(), o.ram_z80()));
                rel.esperas_oraculo += 1;
            }
            if chegou && let Some(i) = MARCAS.iter().position(|m| m.2 == pc) {
                marcas.push((o.emulador().oracle_mclk(), i));
            }
            if let Some(q) = q {
                break (q, mascara);
            }
            passos += 1;
            if passos > LIMITE_PASSOS_QUADRO {
                return Err("o quadro nao terminou".into());
            }
        };
        let mut brutos = Vec::new();
        let mut int6_no_quadro = 0u32;
        oracle_trace::drain_tempo(&mut brutos);
        // Eventos de tempo na ordem do relógio mestre.
        let mut linha: Vec<(u64, EventoQuadro)> = Vec::new();
        for r in &brutos {
            match *r {
                TempoRaw::Leitura {
                    classe,
                    valor,
                    mclk,
                } => {
                    let c = match classe {
                        oracle_trace::leitura::BUSACK => ClasseLeitura::Busack,
                        oracle_trace::leitura::VDP_STATUS => ClasseLeitura::VdpStatus,
                        oracle_trace::leitura::YM_STATUS => ClasseLeitura::YmStatus,
                        _ => continue,
                    };
                    linha.push((mclk, EventoQuadro::Leitura { classe: c, valor }));
                }
                TempoRaw::Interrupcao {
                    nivel,
                    pc_interrompido,
                    mclk,
                    ..
                } => {
                    if nivel == 6 {
                        rel.int6_oraculo += 1;
                        int6_no_quadro += 1;
                    }
                    linha.push((
                        mclk,
                        EventoQuadro::Interrupcao {
                            nivel,
                            pc: pc_interrompido,
                        },
                    ));
                }
                TempoRaw::Z80Ponto {
                    acesso,
                    pc,
                    espera,
                    ativos,
                    banco,
                    endereco,
                    byte,
                    mclk,
                } => {
                    if let Some(acesso) = acesso_de(acesso) {
                        acessos_oraculo.push(acesso);
                        linha.push((
                            mclk,
                            EventoQuadro::PontoZ80(PontoZ80 {
                                acesso,
                                pc,
                                espera,
                                ativos,
                                banco,
                                endereco,
                                byte,
                                mclk,
                            }),
                        ));
                    }
                }
                TempoRaw::EsperaZ80 { .. } => {}
            }
        }
        // Os eventos brutos ficam na ordem em que o oráculo os emitiu (a ordem dos pontos do Z80
        // importa mais que o relógio de cada um). Cada marca entra antes do primeiro evento cujo
        // relógio é maior que o dela.
        for (mclk, i) in &marcas {
            let pos = linha
                .iter()
                .position(|(m, _)| m > mclk)
                .unwrap_or(linha.len());
            linha.insert(pos, (*mclk, EventoQuadro::Marca(MARCAS[*i].0)));
            rel.marcas_vistas
                .entry(MARCAS[*i].1.to_string())
                .or_default()
                .push(q.numero);
        }
        let eventos: Vec<EventoQuadro> = linha.into_iter().map(|(_, e)| e).collect();

        let atomico = nucleo.em_trecho_atomico();
        nucleo.executar_quadro(&eventos);
        if o.quadro_atual() >= quadros {
            nucleo.terminar();
        }
        let atomico = atomico || nucleo.em_trecho_atomico();
        let n = q.numero;
        if let Some(e) = nucleo.erro
            && rel.erro_nucleo.is_none()
        {
            rel.erro_nucleo = Some(format!("quadro {n}: {e:?}"));
        }

        // Estados nas esperas de VBlank: o k-ésimo do oráculo contra o k-ésimo do núcleo.
        for e in nucleo.tirar_esperas() {
            rel.esperas_nucleo += 1;
            esperas_n.push_back(e);
        }
        while !esperas_o.is_empty() && !esperas_n.is_empty() {
            let (Some((ro, zo)), Some((rn, zn))) = (esperas_o.pop_front(), esperas_n.pop_front())
            else {
                break;
            };
            let k = rel.esperas_iguais + rel.esperas_diverge;
            let (ro, rn) = (mascarar_ram(&ro), mascarar_ram(&rn));
            let (zo, zn) = (mascarar_z80(&zo), mascarar_z80(&zn));
            if let Some((p, total)) = primeira_diferenca(&ro, &rn) {
                rel.esperas_diverge += 1;
                let mut vistos: Vec<String> = Vec::new();
                for (i, (x, y)) in ro.iter().zip(&rn).enumerate() {
                    if x != y {
                        let nome = nomes.de(0xFF_0000 + u32::try_from(i).unwrap_or(0));
                        let base = nome.split('+').next().unwrap_or("").to_string();
                        if !vistos.contains(&base) {
                            vistos.push(base);
                        }
                    }
                }
                if rel.primeira_espera.is_none() {
                    rel.primeira_espera = Some(format!(
                        "espera {k} (quadro do oraculo ao fim: {}) primeira_variavel {} bytes_diferentes {total} variaveis {:?}",
                        o.quadro_atual(),
                        nomes.de(0xFF_0000 + u32::try_from(p).unwrap_or(0)),
                        vistos
                    ));
                }
                for v in vistos {
                    *rel.esperas_variaveis.entry(v).or_default() += 1;
                }
            } else {
                rel.esperas_iguais += 1;
            }
            if primeira_diferenca(&zo, &zn).is_some() {
                rel.esperas_z80_diverge += 1;
            }
        }
        // O fechamento do quadro só vale fora do tratador de VBlank e sem VBlank no quadro:
        // o oráculo fecha o quadro na linha 232, no meio do tratador.
        let comparavel = int6_no_quadro == 0 && mascara_fecha != 6;
        if !comparavel {
            rel.fechamento_nao_comparavel += 1;
        }
        // O núcleo roda os eventos com um quadro de atraso: o estado dele agora é o do fechamento
        // do quadro anterior. Compara com o que o oráculo tinha naquele fechamento.
        let atual = (
            o.ram68k(),
            o.ram_z80(),
            q.numero,
            comparavel,
            nucleo.em_trecho_atomico(),
        );
        let anterior = pend.replace(atual);
        'cmp: {
            let Some((ram_ant, z80_ant, n_ant, comparavel_ant, atomico_ant)) = anterior else {
                break 'cmp;
            };
            let atomico = atomico || !comparavel_ant || atomico_ant;
            let n = n_ant;
            // RAM do 68000.
            let ram_o = mascarar_ram(&ram_ant);
            let ram_n = mascarar_ram(nucleo.mem.ram());
            match primeira_diferenca(&ram_o, &ram_n) {
                None => rel.ram_iguais += 1,
                Some((p, total)) => {
                    if !atomico {
                        let mut vistos: Vec<String> = Vec::new();
                        for (i, (x, y)) in ram_o.iter().zip(&ram_n).enumerate() {
                            if x != y {
                                let nome = nomes.de(0xFF_0000 + u32::try_from(i).unwrap_or(0));
                                let base = nome.split('+').next().unwrap_or("").to_string();
                                if !vistos.contains(&base) {
                                    vistos.push(base);
                                }
                            }
                        }
                        for v in vistos {
                            *rel.ram_variaveis.entry(v).or_default() += 1;
                        }
                    }
                    if atomico {
                        rel.ram_diverge_atomico += 1;
                    } else {
                        rel.ram_diverge += 1;
                        if rel.primeira_ram.is_none() {
                            let end = 0xFF_0000 + u32::try_from(p).unwrap_or(0);
                            rel.primeira_ram = Some(format!(
                                "quadro {n} campo ram68k primeira_variavel {} bytes_diferentes {total} hash_oraculo {} hash_nucleo {}",
                                nomes.de(end),
                                &em_hex(&sha1_de_partes(&[&ram_o]))[..8],
                                &em_hex(&sha1_de_partes(&[&ram_n]))[..8]
                            ));
                        }
                    }
                }
            }
            // RAM do Z80.
            let z_o = mascarar_z80(&z80_ant);
            let z_n = mascarar_z80(nucleo.mem.ram_z80());
            match primeira_diferenca(&z_o, &z_n) {
                None => rel.z80_iguais += 1,
                Some((p, total)) => {
                    if atomico {
                        rel.z80_diverge_atomico += 1;
                    } else {
                        rel.z80_diverge += 1;
                        if rel.primeira_z80.is_none() {
                            rel.primeira_z80 = Some(format!(
                                "quadro {n} campo ram_z80 +{p:#x} bytes_diferentes {total}"
                            ));
                        }
                    }
                }
            }
        }
        // Escritas: o núcleo pode estar atrasado ou adiantado dentro de um trecho atômico, nunca
        // diferente. Cada item é conferido assim que os dois lados o têm.
        for e in escritas_do_quadro(&q) {
            esperadas[usize::from(e.0)].push(e);
            rel.escritas_oraculo[usize::from(e.0)] += 1;
            if e.0 == 4 {
                rel.z80_oraculo.push((e.1, e.2));
            }
        }
        for s in &nucleo.mem.tirar_saidas() {
            if let Some(e) = escrita_do_nucleo(s) {
                // O oráculo só registra mudanças no BUSREQ e no RESET do Z80.
                if e.0 == 4 {
                    let estado = &mut z80_estado[usize::try_from(e.1).unwrap_or(0).min(1)];
                    if *estado == e.2 {
                        continue;
                    }
                    *estado = e.2;
                    rel.z80_nucleo.push((e.1, e.2));
                }
                obtidas[usize::from(e.0)].push(e);
                rel.escritas_nucleo[usize::from(e.0)] += 1;
            }
        }
        for t in 0..5 {
            let ate = esperadas[t].len().min(obtidas[t].len());
            while verificadas[t] < ate {
                let i = verificadas[t];
                if esperadas[t][i] != obtidas[t][i] && rel.escritas_diverge.is_none() {
                    rel.escritas_diverge = Some(format!(
                        "quadro {n} sequencia {} indice {i} nucleo {:?} oraculo {:?}",
                        NOMES_SEQ[t], obtidas[t][i], esperadas[t][i]
                    ));
                }
                verificadas[t] += 1;
            }
        }
        let atraso: u64 = (0..5)
            .map(|t| esperadas[t].len().abs_diff(obtidas[t].len()) as u64)
            .sum();
        rel.atraso_maximo = rel.atraso_maximo.max(atraso);
        rel.quadros += 1;
    }
    for (nome, p, ini) in [
        ("soma_da_rom", soma, 0x31Cu32),
        ("limpeza_da_ram", limpeza, 0x364),
        ("espera_da_voz", voz, 0x7_1FAC),
    ] {
        if let Some(p) = p
            && let (Some(a), Some(b)) = (
                extras_mclk.get(&(nome, ini)),
                extras_mclk.get(&(nome, p.fim)),
            )
        {
            rel.nivel2
                .push((nome.to_string(), b.saturating_sub(*a) / 7, p.ciclos));
        }
    }
    rel.int6_nucleo_tratadas = nucleo.vblanks_tratados;
    rel.int6_nucleo_ignoradas = nucleo.interrupcoes_ignoradas;
    rel.ignoradas_em = nucleo.ignoradas_em.clone();
    let log = nucleo.mem.acessos_z80();
    if let Some(i) =
        (0..acessos_oraculo.len().min(log.len())).find(|&i| acessos_oraculo[i] != log[i])
    {
        let de = i.saturating_sub(4);
        rel.z80_contexto = Some(format!(
            "indice {i} de {} (oraculo) e {} (nucleo); oraculo {:?}; nucleo {:?}",
            acessos_oraculo.len(),
            log.len(),
            &acessos_oraculo[de..(i + 8).min(acessos_oraculo.len())],
            &log[de..(i + 8).min(log.len())]
        ));
    }
    let consumidos = nucleo.mem.pontos_consumidos_z80();
    if let Some(i) = (0..log.len().min(consumidos.len())).find(|&i| consumidos[i] != Some(log[i])) {
        let de = i.saturating_sub(4);
        let fim = (i + 8).min(log.len());
        rel.z80_fila = Some(format!(
            "indice {i} de {} acessos; acessos {:?}; pontos consumidos {:?}",
            log.len(),
            &log[de..fim],
            &consumidos[de..fim]
        ));
    }
    rel.divergencias_posicao_z80 = nucleo.mem.divergencias_z80().len();
    for d in nucleo.mem.divergencias_z80() {
        let t = format!("{d:?}");
        let chave = t.split(['{', '(']).next().unwrap_or("").to_string()
            + t.split("acesso: ")
                .nth(1)
                .and_then(|x| x.split(',').next())
                .unwrap_or("");
        *rel.z80_por_acesso.entry(chave).or_default() += 1;
        if rel.z80_primeiras.len() < 6 {
            rel.z80_primeiras.push(t);
        }
    }
    for t in 0..5 {
        if esperadas[t].len() != obtidas[t].len() && rel.escritas_diverge.is_none() {
            rel.escritas_diverge = Some(format!(
                "contagem final sequencia {} oraculo {} nucleo {}",
                NOMES_SEQ[t],
                esperadas[t].len(),
                obtidas[t].len()
            ));
        }
    }
    Ok(rel)
}

/// Imprime o relatório. Só números, nomes e vereditos.
pub fn imprimir(rel: &Relatorio) {
    println!("quadros={}", rel.quadros);
    println!(
        "ram68k: iguais={} diverge={} diverge_em_trecho_atomico={}",
        rel.ram_iguais, rel.ram_diverge, rel.ram_diverge_atomico
    );
    println!(
        "ram_z80: iguais={} diverge={} diverge_em_trecho_atomico={}",
        rel.z80_iguais, rel.z80_diverge, rel.z80_diverge_atomico
    );
    for (i, n) in NOMES_SEQ.iter().enumerate() {
        println!(
            "escritas_{n}: oraculo={} nucleo={}",
            rel.escritas_oraculo[i], rel.escritas_nucleo[i]
        );
    }
    println!("atraso_maximo_de_escritas={}", rel.atraso_maximo);
    println!("posicao_z80_divergencias={}", rel.divergencias_posicao_z80);
    for (m, v) in &rel.marcas_vistas {
        println!("marca {m} quadros={v:?}");
    }
    for (k, v) in &rel.z80_por_acesso {
        println!("z80_divergencia {k} vezes={v}");
    }
    for p in &rel.z80_primeiras {
        println!("z80_divergencia_inicial {p}");
    }
    println!(
        "interrupcoes_nivel6: oraculo={} nucleo_tratadas={} nucleo_ignoradas={}",
        rel.int6_oraculo, rel.int6_nucleo_tratadas, rel.int6_nucleo_ignoradas
    );
    println!(
        "esperas_de_vblank: oraculo={} nucleo={} iguais={} diverge={} z80_diverge={}",
        rel.esperas_oraculo,
        rel.esperas_nucleo,
        rel.esperas_iguais,
        rel.esperas_diverge,
        rel.esperas_z80_diverge
    );
    println!(
        "fechamento_nao_comparavel={}",
        rel.fechamento_nao_comparavel
    );
    if let Some(p) = &rel.primeira_espera {
        println!("primeira_divergencia_em_espera: {p}");
    }
    for (nome, n) in &rel.esperas_variaveis {
        println!("espera_variavel_divergente {nome} esperas={n}");
    }
    for (nome, medido, previsto) in &rel.nivel2 {
        let erro = i128::from(*previsto) - i128::from(*medido);
        let ppm = if *medido == 0 {
            0
        } else {
            erro * 1_000_000 / i128::from(*medido)
        };
        println!(
            "nivel2 {nome} medido_ciclos={medido} previsto_ciclos={previsto} erro_ciclos={erro} erro_ppm={ppm}"
        );
    }
    for (q, m) in &rel.ignoradas_em {
        println!("interrupcao_ignorada quadro={q} modo={m}");
    }
    let mut v: Vec<_> = rel.ram_variaveis.iter().collect();
    v.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    for (nome, n) in v.iter().take(20) {
        println!("ram68k_variavel_divergente {nome} quadros={n}");
    }
    if let Some(c) = &rel.z80_fila {
        println!("z80_fila_desalinhada {c}");
    }
    if let Some(c) = &rel.z80_contexto {
        println!("z80_primeiro_acesso_diferente {c}");
    }
    println!(
        "z80_mudancas: oraculo={} nucleo={}",
        rel.z80_oraculo.len(),
        rel.z80_nucleo.len()
    );
    if let Some(e) = &rel.erro_nucleo {
        println!("erro_do_nucleo: {e}");
    }
    if let Some(p) = &rel.primeira_ram {
        println!("primeira_divergencia: {p}");
    }
    if let Some(p) = &rel.primeira_z80 {
        println!("primeira_divergencia: {p}");
    }
    if let Some(p) = &rel.escritas_diverge {
        println!("primeira_divergencia: {p}");
    }
    println!("veredito: {}", if rel.zero() { "IGUAL" } else { "DIVERGE" });
}
