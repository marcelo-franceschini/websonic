//! `cargo xtask inventario-som` (Etapa 6, sub-etapa 6.0).
//!
//! Lê os traces do corpus (`*.a.s1t`) e os arquivos de tempo (`*.s1tempo`, versão 2) e escreve
//! `docs/inventario-som.md`: o que o Sonic 1 faz com o YM2612, o PSG e o Z80, por origem (68000 ou
//! Z80). Serve para fechar o desenho do programa do Z80 e do driver com dados. Só nomes, números e
//! endereços de código do Z80: nunca bytes de dados.
//!
//! Uso: `cargo xtask inventario-som --traces DIR --tempos DIR [--saida docs/inventario-som.md]`
use sonic1_core::quadro::tempo::{AcessoZ80, EventoTempo, TempoGravado};
use sonic1_fidelidade::{Leitor, Origem, Quadro, Tipo};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Máximo de linhas das tabelas de posição do Z80 no arquivo (o resto é só contado).
const LIMITE_LINHAS: usize = 80;

#[derive(Default, Debug)]
pub struct Resumo {
    /// YM2612: (origem, porta, registrador) -> vezes. Só escritas de dado.
    pub ym: BTreeMap<(&'static str, u8, u8), u64>,
    /// Escritas na porta de endereço do YM2612 por origem.
    pub ym_endereco: BTreeMap<&'static str, u64>,
    /// PSG: origem -> vezes.
    pub psg: BTreeMap<&'static str, u64>,
    /// Escritas do DAC por quadro (apenas quadros com pelo menos uma): quantidade -> quadros.
    pub dac_por_quadro: BTreeMap<u32, u64>,
    /// Quadros lidos e quadros com DAC.
    pub quadros: u64,
    pub quadros_com_dac: u64,
    /// Desvios do Z80: (pc, destino, classe) -> (tomado, nao_tomado).
    pub desvios_z80: BTreeMap<(u32, u32, u32), (u64, u64)>,
    /// Pontos do Z80 por tipo de acesso: nome -> (vezes, menor ativos, maior ativos, soma).
    pub pontos: BTreeMap<&'static str, (u64, u32, u32, u64)>,
    /// Posição do Z80 vista nos pontos: (acesso, pc, espera) -> vezes.
    pub posicoes: BTreeMap<(&'static str, u16, u16), u64>,
    /// Valores do registrador de banco vistos nos pontos.
    pub bancos: BTreeMap<u16, u64>,
    /// Endereços do espaço do Z80 tocados pelo 68000 na janela (endereço -> vezes), agrupados.
    pub janela: BTreeMap<(&'static str, u16), u64>,
    /// Quadros de tempo lidos e maior número de pontos num quadro.
    pub quadros_de_tempo: u64,
    pub maior_pontos_por_quadro: u64,
}

fn origem_nome(o: Origem) -> &'static str {
    match o {
        Origem::Cpu68 => "68000",
        Origem::Z80 => "Z80",
        Origem::Hardware => "hardware",
    }
}

/// Soma os eventos de um quadro do trace.
pub fn somar_quadro(r: &mut Resumo, q: &Quadro) {
    r.quadros = r.quadros.saturating_add(1);
    let mut dac = 0u32;
    for e in &q.eventos {
        let origem = origem_nome(e.origem);
        match e.tipo {
            Tipo::Ym => {
                // Porta par é endereço, ímpar é dado (e = a % 2). O campo c leva o registrador.
                if e.a % 2 == 1 {
                    let porta = u8::try_from(e.a / 2).unwrap_or(0);
                    let reg = u8::try_from(e.c & 0xFF).unwrap_or(0);
                    *r.ym.entry((origem, porta, reg)).or_default() += 1;
                    if e.d == 1 {
                        dac = dac.saturating_add(1);
                    }
                } else {
                    *r.ym_endereco.entry(origem).or_default() += 1;
                }
            }
            Tipo::Psg => *r.psg.entry(origem).or_default() += 1,
            Tipo::Desvio80 => {
                let item = r.desvios_z80.entry((e.a, e.b, e.c)).or_default();
                if e.d != 0 {
                    item.0 += 1;
                } else {
                    item.1 += 1;
                }
            }
            _ => {}
        }
    }
    if dac > 0 {
        r.quadros_com_dac = r.quadros_com_dac.saturating_add(1);
        *r.dac_por_quadro.entry(dac).or_default() += 1;
    }
}

fn nome_acesso(a: AcessoZ80) -> &'static str {
    match a {
        AcessoZ80::BusReqPedido => "BUSREQ pedido",
        AcessoZ80::BusReqLiberado => "BUSREQ liberado",
        AcessoZ80::ResetLigado => "RESET ligado",
        AcessoZ80::ResetSolto => "RESET solto",
        AcessoZ80::LeituraJanela => "janela, leitura",
        AcessoZ80::EscritaJanela => "janela, escrita",
    }
}

/// Soma os pontos do Z80 de um arquivo de tempo.
pub fn somar_tempo(r: &mut Resumo, t: &TempoGravado) {
    for q in &t.quadros {
        r.quadros_de_tempo = r.quadros_de_tempo.saturating_add(1);
        let mut no_quadro = 0u64;
        for e in &q.eventos {
            if let EventoTempo::PontoZ80 {
                acesso,
                pc,
                espera,
                ativos,
                banco,
                endereco,
                ..
            } = *e
            {
                no_quadro += 1;
                let nome = nome_acesso(acesso);
                let p = r.pontos.entry(nome).or_insert((0, u32::MAX, 0, 0));
                p.0 += 1;
                p.1 = p.1.min(ativos);
                p.2 = p.2.max(ativos);
                p.3 = p.3.saturating_add(u64::from(ativos));
                *r.posicoes.entry((nome, pc, espera)).or_default() += 1;
                *r.bancos.entry(banco).or_default() += 1;
                if matches!(acesso, AcessoZ80::LeituraJanela | AcessoZ80::EscritaJanela) {
                    *r.janela.entry((nome, endereco)).or_default() += 1;
                }
            }
        }
        r.maior_pontos_por_quadro = r.maior_pontos_por_quadro.max(no_quadro);
    }
}

/// Gera o Markdown. Determinístico.
pub fn renderizar(r: &Resumo, casos: &[String], tempos: &[String]) -> String {
    let mut md = String::from(
        "# Inventário do som\n\nGerado por `cargo xtask inventario-som`. Só nomes, números e endereços de código do Z80. Nenhum byte de dados.\n",
    );
    let _ = writeln!(md, "\nTraces lidos ({}): {}", casos.len(), casos.join(", "));
    let _ = writeln!(
        md,
        "Arquivos de tempo lidos ({}): {}\n",
        tempos.len(),
        tempos.join(", ")
    );
    let _ = writeln!(
        md,
        "Quadros dos traces: {}. Quadros com escrita de DAC: {}.\n",
        r.quadros, r.quadros_com_dac
    );

    md.push_str(
        "## YM2612: escritas de endereço por origem\n\n| Origem | Vezes |\n| --- | --- |\n",
    );
    for (o, v) in &r.ym_endereco {
        let _ = writeln!(md, "| {o} | {v} |");
    }
    md.push_str("\n## YM2612: escritas de dado por origem, porta e registrador\n\n| Origem | Porta | Registrador | Vezes |\n| --- | --- | --- | --- |\n");
    for ((o, p, reg), v) in &r.ym {
        let _ = writeln!(md, "| {o} | {p} | {reg:#04x} | {v} |");
    }
    md.push_str("\n## PSG: escritas por origem\n\n| Origem | Vezes |\n| --- | --- |\n");
    for (o, v) in &r.psg {
        let _ = writeln!(md, "| {o} | {v} |");
    }
    md.push_str("\n## DAC: escritas por quadro (só quadros com DAC)\n\n| Escritas no quadro | Quadros |\n| --- | --- |\n");
    for (n, v) in &r.dac_por_quadro {
        let _ = writeln!(md, "| {n} | {v} |");
    }
    md.push_str("\n## Z80: desvios executados (PC, destino, classe)\n\nClasses: 1 JR, 2 JR cc, 3 DJNZ, 4 JP, 5 JP cc, 6 JP (HL), 7 CALL, 8 CALL cc, 9 RET, 10 RET cc, 11 RETI, 12 RST.\n\n| PC | Destino | Classe | Tomado | Não tomado |\n| --- | --- | --- | --- | --- |\n");
    for ((pc, destino, classe), (t, n)) in &r.desvios_z80 {
        let _ = writeln!(md, "| {pc:#06x} | {destino:#06x} | {classe} | {t} | {n} |");
    }
    md.push_str("\n## Z80: pontos do tempo (acessos do 68000 que tocam o Z80)\n\n");
    let _ = writeln!(
        md,
        "Quadros de tempo: {}. Maior número de pontos num quadro: {}.\n",
        r.quadros_de_tempo, r.maior_pontos_por_quadro
    );
    md.push_str("| Acesso | Pontos | Menor ciclos ativos | Maior ciclos ativos | Soma |\n| --- | --- | --- | --- | --- |\n");
    for (n, (v, menor, maior, soma)) in &r.pontos {
        let menor = if *v == 0 { 0 } else { *menor };
        let _ = writeln!(md, "| {n} | {v} | {menor} | {maior} | {soma} |");
    }
    md.push_str("\n## Z80: valores do registrador de banco nos pontos\n\n| Banco | Pontos |\n| --- | --- |\n");
    for (b, v) in &r.bancos {
        let _ = writeln!(md, "| {b:#05x} | {v} |");
    }
    // Endereços de controle (fim da RAM e YM2612) um a um; o resto agrupado por página de 256 bytes.
    md.push_str("\n## Z80: endereços da janela tocados pelo 68000\n\n| Acesso | Endereço ou página | Endereços distintos | Vezes |\n| --- | --- | --- | --- |\n");
    let mut paginas: BTreeMap<(&str, u16), (u64, u64)> = BTreeMap::new();
    for ((n, e), v) in &r.janela {
        if *e >= 0x1FF0 || (0x4000..=0x4003).contains(e) {
            let _ = writeln!(md, "| {n} | {e:#06x} | 1 | {v} |");
        } else {
            let p = paginas.entry((n, *e >> 8)).or_default();
            p.0 += 1;
            p.1 += v;
        }
    }
    for ((n, pg), (d, v)) in &paginas {
        let _ = writeln!(md, "| {n} | página {:#06x} | {d} | {v} |", pg << 8);
    }
    md.push_str("\n## Z80: posição vista em cada ponto (acesso, PC, ciclos que faltam)\n\n");
    let mut por_vezes: Vec<_> = r.posicoes.iter().collect();
    por_vezes.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let _ = writeln!(
        md,
        "Posições distintas: {}. Mostradas as {} mais frequentes.\n",
        por_vezes.len(),
        LIMITE_LINHAS.min(por_vezes.len())
    );
    md.push_str("| Acesso | PC | Faltam | Pontos |\n| --- | --- | --- | --- |\n");
    for ((n, pc, espera), v) in por_vezes.into_iter().take(LIMITE_LINHAS) {
        let _ = writeln!(md, "| {n} | {pc:#06x} | {espera} | {v} |");
    }
    md
}

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn arquivos_com(dir: &str, sufixo: &str) -> Result<Vec<std::path::PathBuf>, String> {
    let mut v: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("não li {dir}: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(sufixo))
        })
        .collect();
    v.sort();
    Ok(v)
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let traces = opcao(args, "--traces").ok_or("falta --traces")?;
    let tempos = opcao(args, "--tempos").ok_or("falta --tempos")?;
    let saida = opcao(args, "--saida").unwrap_or("docs/inventario-som.md");
    let mut r = Resumo::default();
    let mut casos = Vec::new();
    for p in arquivos_com(traces, ".a.s1t")? {
        let nome = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .trim_end_matches(".a.s1t")
            .to_string();
        let f = fs::File::open(&p).map_err(|e| e.to_string())?;
        let mut leitor = Leitor::abrir(std::io::BufReader::new(f)).map_err(|e| e.to_string())?;
        while let Some(q) = leitor.proximo().map_err(|e| e.to_string())? {
            somar_quadro(&mut r, &q);
        }
        casos.push(nome);
    }
    let mut nomes_tempo = Vec::new();
    for p in arquivos_com(tempos, ".s1tempo")? {
        let bytes = fs::read(&p).map_err(|e| e.to_string())?;
        let t = TempoGravado::de_bytes(&bytes).map_err(|e| format!("{}: {e:?}", p.display()))?;
        somar_tempo(&mut r, &t);
        nomes_tempo.push(
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .trim_end_matches(".s1tempo")
                .to_string(),
        );
    }
    let md = renderizar(&r, &casos, &nomes_tempo);
    super::inventario::filtro(&md)?;
    fs::write(Path::new(&super::raiz()).join(saida), md).map_err(|e| e.to_string())?;
    println!(
        "inventario-som: {} traces, {} tempos, {} pontos do Z80, {} posições distintas",
        casos.len(),
        nomes_tempo.len(),
        r.pontos.values().map(|p| p.0).sum::<u64>(),
        r.posicoes.len()
    );
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_core::quadro::tempo::TempoQuadro;
    use sonic1_fidelidade::{Categoria, Evento};

    fn ev(tipo: Tipo, origem: Origem, a: u32, c: u32, d: u32) -> Evento {
        Evento {
            mclk: 0,
            seq: 0,
            tipo,
            origem,
            linha: 0,
            coluna: 0,
            a,
            b: 0,
            c,
            d,
        }
    }

    fn quadro(eventos: Vec<Evento>) -> Quadro {
        Quadro {
            numero: 0,
            mclk_fim: 0,
            eventos,
            hashes: [[0; 20]; Categoria::TOTAL],
        }
    }

    #[test]
    fn separa_origem_dac_e_desvios() {
        let mut r = Resumo::default();
        let q = quadro(vec![
            ev(Tipo::Ym, Origem::Cpu68, 0, 0, 0),
            ev(Tipo::Ym, Origem::Cpu68, 1, 0x28, 0),
            ev(Tipo::Ym, Origem::Z80, 1, 0x2A, 1),
            ev(Tipo::Ym, Origem::Z80, 1, 0x2A, 1),
            ev(Tipo::Psg, Origem::Cpu68, 0x9F, 0, 0),
            ev(Tipo::Desvio80, Origem::Z80, 0x80, 0x7F, 0),
        ]);
        somar_quadro(&mut r, &q);
        assert_eq!(r.ym[&("68000", 0, 0x28)], 1);
        assert_eq!(r.ym[&("Z80", 0, 0x2A)], 2);
        assert_eq!(r.ym_endereco["68000"], 1);
        assert_eq!(r.psg["68000"], 1);
        assert_eq!(r.dac_por_quadro[&2], 1);
        assert_eq!(r.quadros_com_dac, 1);
        assert_eq!(r.desvios_z80[&(0x80, 0, 0x7F)], (0, 1));
    }

    #[test]
    fn pontos_do_z80_somam_ciclos_e_posicoes() {
        let ponto = |acesso, pc, espera, ativos, banco| EventoTempo::PontoZ80 {
            acesso,
            pc,
            espera,
            ativos,
            banco,
            endereco: 0x1FFD,
            byte: 0,
            mclk: 0,
        };
        let t = TempoGravado {
            quadros: vec![TempoQuadro {
                numero: 0,
                mclk_inicio: 0,
                mclk_fim: 1,
                eventos: vec![
                    ponto(AcessoZ80::BusReqPedido, 0x30, 2, 100, 0),
                    ponto(AcessoZ80::LeituraJanela, 0x30, 2, 5, 0x1FF),
                    ponto(AcessoZ80::BusReqPedido, 0x31, 0, 300, 0),
                ],
            }],
        };
        let mut r = Resumo::default();
        somar_tempo(&mut r, &t);
        assert_eq!(r.pontos["BUSREQ pedido"], (2, 100, 300, 400));
        assert_eq!(r.posicoes[&("BUSREQ pedido", 0x30, 2)], 1);
        assert_eq!(r.bancos[&0x1FF], 1);
        assert_eq!(r.janela[&("janela, leitura", 0x1FFD)], 1);
        assert_eq!(r.maior_pontos_por_quadro, 3);
        let md = renderizar(&r, &["c".into()], &["t".into()]);
        assert!(md.contains("| BUSREQ pedido | 2 | 100 | 300 | 400 |"));
        assert!(md.contains("| janela, leitura | 0x1ffd | 1 | 1 |"));
        assert!(super::super::inventario::filtro(&md).is_ok());
    }
}
