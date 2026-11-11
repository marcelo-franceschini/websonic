//! Relatorio de sintese diferencial (Etapa 8).
//!
//! Compara FM e PSG amostra a amostra contra os chips de referencia Nuked-OPN2 e Nuked-PSG.
//! Uso:
//! `sintese [--sintetico] [--traces DIR] [--saida ARQ]`

use sonic1_diferencial_sintese::comparador::{
    ResultadoCaso, Veredito, comparar_fm, comparar_psg,
};
use sonic1_fidelidade::Tipo;
use sonic1_fidelidade::formato::Leitor;
use sonic1_fidelidade::hash::{em_hex, sha1};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

struct Rng(u64);
impl Rng {
    fn proximo(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }
    fn ate(&mut self, n: u64) -> u64 {
        self.proximo().checked_rem(n).unwrap_or(0)
    }
}

/// Bateria sintetica cobrindo os varios modos do FM e PSG.
fn rodar_bateria_sintetica() -> Vec<ResultadoCaso> {
    let mut resultados = Vec::new();

    // 1. FM: Algoritmo 7 (seno simples)
    {
        let mut e = Vec::new();
        e.push((0u64, 0u32, 0xB0));
        e.push((42u64, 1u32, 0x07)); // Algo 7
        e.push((84u64, 0u32, 0xB4));
        e.push((126u64, 1u32, 0xC0)); // Pan L+R
        for o in 0..4u8 {
            e.push((168u64 + u64::from(o) * 168, 0, 0x30 + 4 * o));
            e.push((210u64 + u64::from(o) * 168, 1, 0x01)); // Mul 1
            e.push((252u64 + u64::from(o) * 168, 0, 0x40 + 4 * o));
            e.push((294u64 + u64::from(o) * 168, 1, if o == 0 { 0 } else { 127 })); // TL
            e.push((336u64 + u64::from(o) * 168, 0, 0x50 + 4 * o));
            e.push((378u64 + u64::from(o) * 168, 1, 0x1F)); // AR max
        }
        e.push((1000u64, 0, 0xA4));
        e.push((1042u64, 1, (4 << 3) | 2)); // Bloco 4
        e.push((1084u64, 0, 0xA0));
        e.push((1126u64, 1, 0x69)); // F-Num
        e.push((1168u64, 0, 0x28));
        e.push((1210u64, 1, 0xF0)); // Key on canal 0
        let r = comparar_fm("sintetico_fm_alg7", &e, 4096 * 1008);
        resultados.push(r);
    }

    // 2. FM: Envelopes variados e DAC
    {
        let mut rng = Rng(0x5EED_0001);
        let mut e = Vec::new();
        let mut t = 0u64;
        for _ in 0..200 {
            t = t.wrapping_add(rng.ate(2000).wrapping_add(200));
            let porta = u32::try_from(rng.ate(4)).unwrap();
            let valor = u8::try_from(rng.ate(256)).unwrap();
            e.push((t, porta, valor));
        }
        let r = comparar_fm("sintetico_fm_dinamico_1", &e, 8192 * 1008);
        resultados.push(r);
    }

    // 3. FM: 5 sementes do teste dinamico aleatorio denso
    for semente in 2..=6u64 {
        let mut rng = Rng(semente.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let mut e = Vec::new();
        let mut t = 0u64;
        // Configuracao dos 6 canais
        for ch in 0..6u8 {
            let (p, off) = if ch < 3 { (0u32, ch) } else { (2, ch - 3) };
            e.push((t, p, 0xB0 + off));
            e.push((t + 42, p + 1, (rng.ate(8) as u8) | ((rng.ate(4) as u8) << 3)));
            e.push((t + 84, p, 0xB4 + off));
            e.push((t + 126, p + 1, 0xC0));
            for o in 0..4u8 {
                e.push((t + 168 + u64::from(o) * 168, p, 0x30 + off + 4 * o));
                e.push((t + 210 + u64::from(o) * 168, p + 1, ((rng.ate(8) as u8) << 4) | (rng.ate(16) as u8)));
                e.push((t + 252 + u64::from(o) * 168, p, 0x40 + off + 4 * o));
                e.push((t + 294 + u64::from(o) * 168, p + 1, rng.ate(64) as u8));
                e.push((t + 336 + u64::from(o) * 168, p, 0x50 + off + 4 * o));
                e.push((t + 378 + u64::from(o) * 168, p + 1, rng.ate(32) as u8));
            }
            t += 1000;
        }
        // Eventos aleatorios de teclas e DAC
        for _ in 0..150 {
            t += rng.ate(5000) + 200;
            let op = rng.ate(5);
            match op {
                0 => {
                    let ch = rng.ate(6) as u8;
                    e.push((t, 0, 0x28));
                    e.push((t + 42, 1, 0xF0 | (ch % 3) | ((ch / 3) << 2)));
                }
                1 => {
                    e.push((t, 0, 0x2A));
                    e.push((t + 42, 1, rng.ate(256) as u8));
                }
                2 => {
                    e.push((t, 0, 0x2B));
                    e.push((t + 42, 1, if rng.ate(2) == 0 { 0x80 } else { 0 }));
                }
                _ => {
                    let ch = rng.ate(6) as u8;
                    let (p, off) = if ch < 3 { (0u32, ch) } else { (2, ch - 3) };
                    e.push((t, p, 0xA0 + off));
                    e.push((t + 42, p + 1, rng.ate(256) as u8));
                }
            }
        }
        let nome = format!("sintetico_fm_dinamico_{semente}");
        let r = comparar_fm(&nome, &e, 4096 * 1008);
        resultados.push(r);
    }

    // 4. PSG: Silencio e tons simples
    {
        let e = vec![
            (0u64, 0x90u8), // Vol tom 0 = 0
            (100u64, 0x80u8), // Latch tom 0
            (150u64, 0x01u8), // Periodo baixo
        ];
        let r = comparar_psg("sintetico_psg_tom", &e, 4096 * 240);
        resultados.push(r);
    }

    // 5. PSG: Ruido e tons aleatorios
    {
        let mut rng = Rng(0xCAFE_0001);
        let mut e = Vec::new();
        let mut t = 0u64;
        for _ in 0..200 {
            t = t.wrapping_add(rng.ate(3000).wrapping_add(150));
            let val = u8::try_from(rng.ate(256)).unwrap();
            e.push((t, val));
        }
        let r = comparar_psg("sintetico_psg_dinamico", &e, 8192 * 240);
        resultados.push(r);
    }

    resultados
}

/// Processa um arquivo de trace (.a.s1t) e extrai as escritas de som.
fn extrair_escritas_do_trace(
    caminho: &Path,
) -> Result<(Vec<(u64, u32, u8)>, Vec<(u64, u8)>, u64), String> {
    let f = File::open(caminho).map_err(|e| format!("abrir trace {}: {e}", caminho.display()))?;
    let mut leitor = Leitor::abrir(BufReader::new(f))
        .map_err(|e| format!("formato trace {}: {e:?}", caminho.display()))?;

    let mut escritas_fm = Vec::new();
    let mut escritas_psg = Vec::new();
    let mut mclk_maximo = 0u64;

    while let Some(q) = leitor
        .proximo()
        .map_err(|e| format!("ler quadro {}: {e:?}", caminho.display()))?
    {
        if q.mclk_fim > mclk_maximo {
            mclk_maximo = q.mclk_fim;
        }
        for ev in &q.eventos {
            match ev.tipo {
                Tipo::Ym => {
                    let porta = ev.a;
                    let valor = u8::try_from(ev.b & 0xFF).unwrap_or(0);
                    escritas_fm.push((ev.mclk, porta, valor));
                }
                Tipo::Psg => {
                    let valor = u8::try_from(ev.a & 0xFF).unwrap_or(0);
                    escritas_psg.push((ev.mclk, valor));
                }
                _ => {}
            }
        }
    }

    // Garante ordenacao por mclk
    escritas_fm.sort_by_key(|&(m, _, _)| m);
    escritas_psg.sort_by_key(|&(m, _)| m);

    Ok((escritas_fm, escritas_psg, mclk_maximo))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rodar_sint = args.iter().any(|a| a == "--sintetico") || args.is_empty();
    let dir_traces = opcao(&args, "--traces");
    let arq_saida = opcao(&args, "--saida");

    let mut resultados: Vec<ResultadoCaso> = Vec::new();

    if rodar_sint {
        println!("Executando bateria sintetica contra Nuked-OPN2 e Nuked-PSG...");
        resultados.extend(rodar_bateria_sintetica());
    }

    if let Some(dir) = dir_traces {
        let p_dir = Path::new(dir);
        if p_dir.is_dir() {
            println!("Processando traces do corpus em {}...", p_dir.display());
            let mut caminhos = Vec::new();
            if let Ok(entries) = std::fs::read_dir(p_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "s1t") {
                        caminhos.push(path);
                    }
                }
            }
            caminhos.sort();

            for c in caminhos {
                let nome_caso = c.file_stem().and_then(|s| s.to_str()).unwrap_or("caso");
                // Remove possivel sufixo .a
                let nome_caso = nome_caso.strip_suffix(".a").unwrap_or(nome_caso);
                println!("Extraindo som do trace: {nome_caso}...");

                match extrair_escritas_do_trace(&c) {
                    Ok((fm_esc, psg_esc, mclk_fim)) => {
                        println!(
                            "  {} escritas FM, {} escritas PSG, mclk total: {}",
                            fm_esc.len(),
                            psg_esc.len(),
                            mclk_fim
                        );
                        let r_fm = comparar_fm(nome_caso, &fm_esc, mclk_fim);
                        let r_psg = comparar_psg(nome_caso, &psg_esc, mclk_fim);
                        resultados.push(r_fm);
                        resultados.push(r_psg);
                    }
                    Err(e) => {
                        eprintln!("Erro ao extrair trace {}: {e}", c.display());
                    }
                }
            }
        } else {
            eprintln!("Diretorio de traces nao encontrado: {dir}");
        }
    }

    // Monta o relatorio final
    let mut linhas_relatorio = Vec::new();
    linhas_relatorio.push(String::from("=== RELATORIO DE SINTESE DIFERENCIAL (ETAPA 8) ==="));
    linhas_relatorio.push(String::from("Referencia FM: Nuked-OPN2 (modo YM2612)"));
    linhas_relatorio.push(String::from("Referencia PSG: Nuked-PSG (YM7101)"));
    linhas_relatorio.push(String::from("Comparacao: amostra a amostra em blocos de 4096"));
    linhas_relatorio.push(String::from("--------------------------------------------------"));

    let mut todos_iguais = true;
    for r in &resultados {
        let st = match r.veredito {
            Veredito::Igual => "IGUAL",
            Veredito::Diverge => {
                todos_iguais = false;
                "DIVERGE"
            }
        };
        let l = format!(
            "CASO {:<25} CHIP {:<3}: {:>8} amostras, {:>4} blocos ({:>4} iguais) -> {}",
            r.caso, r.chip, r.n_amostras, r.n_blocos, r.blocos_iguais, st
        );
        println!("{l}");
        linhas_relatorio.push(l);

        if let Some(err) = &r.primeiro_erro {
            let l_err = format!(
                "  DIVERGENCIA na amostra {}, bloco {}, mclk ~{}: camada {}",
                err.amostra, err.bloco, err.mclk_aprox, err.camada
            );
            println!("{l_err}");
            linhas_relatorio.push(l_err);
        }
    }

    linhas_relatorio.push(String::from("--------------------------------------------------"));
    let veredito_geral = if todos_iguais { "IGUAL" } else { "DIVERGE" };
    linhas_relatorio.push(format!("VEREDITO GERAL DA ETAPA: {veredito_geral}"));

    linhas_relatorio.push(String::from(
        "parte analogica (APROXIMADO, nao verificada): filtro passa-baixa e amplificador",
    ));

    let conteudo_relatorio = linhas_relatorio.join("\n") + "\n";
    let hash_final = em_hex(&sha1(conteudo_relatorio.as_bytes()));
    let linha_hash = format!("HASH DO RELATORIO: {hash_final}");
    println!("{linha_hash}");

    if let Some(caminho_saida) = arq_saida {
        let mut f = File::create(caminho_saida)?;
        f.write_all(conteudo_relatorio.as_bytes())?;
        f.write_all(linha_hash.as_bytes())?;
        f.write_all(b"\n")?;
        println!("Relatorio salvo em: {caminho_saida}");
    }

    if todos_iguais {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
