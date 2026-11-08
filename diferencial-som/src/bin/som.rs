//! `som`: relatório do som, um ID por linha. Uso (pelo Marcelo, com a ROM):
//! `som --rom ARQ --rotulos refs/rotulos.txt --ids crates/teste-som/ids.txt --saida ARQ
//!      [--quadros-base N] [--quadros-musica N] [--trechos-efeito N] [--so ID,ID]`
//! Imprime só números, hashes e vereditos.

use sonic1_diferencial_som::harness::{
    Config, Falha, em_thread_nova, hex, preparar_base, rodar_id,
};
use std::io::Write;
use std::sync::Arc;

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn endereco(rotulos: &str, nome: &str) -> Result<u32, String> {
    rotulos
        .lines()
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| {
            let mut c = l.split('\t');
            (c.next() == Some(nome)).then(|| c.nth(1)).flatten()
        })
        .and_then(|v| u32::from_str_radix(v, 16).ok())
        .ok_or_else(|| format!("rotulo {nome} nao achado"))
}

fn principal() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rom =
        std::fs::read(opcao(&args, "--rom").ok_or("falta --rom")?).map_err(|e| e.to_string())?;
    let rotulos = std::fs::read_to_string(opcao(&args, "--rotulos").ok_or("falta --rotulos")?)
        .map_err(|e| e.to_string())?;
    let ids = std::fs::read_to_string(opcao(&args, "--ids").ok_or("falta --ids")?)
        .map_err(|e| e.to_string())?;
    let saida = opcao(&args, "--saida").ok_or("falta --saida")?;
    let num = |n: &str, d: u32| opcao(&args, n).and_then(|v| v.parse().ok()).unwrap_or(d);
    let so: Option<Vec<u8>> = opcao(&args, "--so").map(|v| {
        v.split(',')
            .filter_map(|x| u8::from_str_radix(x.trim(), 16).ok())
            .collect()
    });
    let cfg = Arc::new(Config {
        rom,
        update_music: endereco(&rotulos, "UpdateMusic")?,
        quadros_base: num("--quadros-base", 700),
        quadros_musica: num("--quadros-musica", 2400),
        trechos_maximo_efeito: num("--trechos-efeito", 600),
    });
    let base = {
        let cfg = Arc::clone(&cfg);
        Arc::new(em_thread_nova(move || preparar_base(&cfg))?)
    };
    let mut f = std::fs::File::create(saida).map_err(|e| e.to_string())?;
    let (mut ok, mut ruim) = (0u32, 0u32);
    // IDs fora da faixa que o jogo trata como "nada" ou comando desconhecido.
    let mut lista: Vec<(u8, String)> = vec![
        (0x80, "nada".into()),
        (0x00, "zero".into()),
        (0x7F, "positivo".into()),
        (0xE5, "fora_dos_comandos".into()),
        (0xFF, "fora_dos_comandos".into()),
    ];
    for l in ids.lines().filter(|l| !l.starts_with('#')) {
        let mut c = l.split('\t');
        if let (Some(id), Some(_tipo), Some(nome)) = (c.next(), c.next(), c.next()) {
            if let Ok(id) = u8::from_str_radix(id, 16) {
                lista.push((id, nome.to_string()));
            }
        }
    }
    for (id, nome) in lista {
        if so.as_ref().is_some_and(|s| !s.contains(&id)) {
            continue;
        }
        let r = {
            let (cfg, base, nome) = (Arc::clone(&cfg), Arc::clone(&base), nome.clone());
            em_thread_nova(move || rodar_id(&cfg, &base, id, &nome))?
        };
        let veredito = if r.igual() { "IGUAL" } else { "DIVERGE" };
        let falha = r.falha.as_ref().map_or("-".to_string(), |(k, x)| match x {
            Falha::Escritas { indice } => format!("trecho {k} escritas indice {indice}"),
            Falha::RegiaoDoDriver { deslocamento } => {
                format!("trecho {k} regiao_do_driver +{deslocamento:#x}")
            }
            Falha::RamDoZ80 { deslocamento } => format!("trecho {k} ram_z80 +{deslocamento:#x}"),
            Falha::PosicaoDoZ80 { descricao } => format!("trecho {k} posicao_z80 {descricao}"),
            Falha::TempoNaoConsumido { pontos, leituras } => {
                format!("trecho {k} tempo_sobrando pontos={pontos} leituras={leituras}")
            }
            Falha::EsperaEstourada => format!("trecho {k} espera_estourada"),
        });
        let linha = format!(
            "{id:02X}\t{nome}\tquadros={}\tesperadas={}\tobtidas={}\texterna={}\tesperado={}\tobtido={}\tprimeira_falha={falha}\t{veredito}",
            r.quadros,
            r.esperadas,
            r.obtidas,
            r.externas,
            hex(&r.hash_esperado),
            hex(&r.hash_obtido)
        );
        writeln!(f, "{linha}").map_err(|e| e.to_string())?;
        println!("{linha}");
        if r.igual() { ok += 1 } else { ruim += 1 }
    }
    println!("resumo: {ok} IGUAL, {ruim} DIVERGE");
    Ok(ruim == 0)
}

fn main() -> std::process::ExitCode {
    match principal() {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::from(1),
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::from(2)
        }
    }
}
