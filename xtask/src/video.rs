//! `cargo xtask video-estado` e `cargo xtask video-comparar` (sub-etapas 4.2 e 4.8).
//!
//! Repetem as escritas de vídeo de um trace do oráculo no renderizador (`sonic1-video`) e
//! comparam com o que o oráculo produziu. Rodam no computador do Marcelo (usam traces feitos com
//! a ROM). A saída tem só números, hashes e coordenadas: nunca cores nem bytes.
//!
//! `video-estado`: compara, por quadro, os hashes de VRAM, CRAM e VSRAM com os do trace.
//! `video-comparar`: compara o hash da imagem de 320x224 em 9 bits com o `.s1i`.
//!
//! Uso:
//! `cargo xtask video-estado --trace T.s1t --video T.s1v`
//! `cargo xtask video-comparar --trace T.s1t --video T.s1v --imagens T.s1i [--saida R.tsv]
//!    [--rom ROM --oraculo BIN --estados DIR --caso NOME --intervalo K [--entradas ARQ]]`
use sonic1_fidelidade::hash::{em_hex, sha1};
use sonic1_fidelidade::{Categoria, Leitor, Quadro as QuadroDoTrace, Tipo};
use sonic1_formato_video as fv;
use sonic1_video::renderizador::{Escrita, Renderizador};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn abrir_trace(caminho: &str) -> Result<Leitor<BufReader<File>>, String> {
    let f = File::open(caminho).map_err(|e| format!("{caminho}: {e}"))?;
    Leitor::abrir(BufReader::new(f)).map_err(|e| e.to_string())
}

fn ler_dma(caminho: &str) -> Result<BTreeMap<u32, Vec<u16>>, String> {
    let f = File::open(caminho).map_err(|e| format!("{caminho}: {e}"))?;
    let regs = fv::ler_dma(BufReader::new(f)).map_err(|e| e.to_string())?;
    Ok(regs.into_iter().map(|r| (r.quadro, r.palavras)).collect())
}

/// Aplica ao renderizador as escritas de vídeo de um quadro do trace e fecha o quadro.
pub fn aplicar_quadro(r: &mut Renderizador, q: &QuadroDoTrace, dma: &BTreeMap<u32, Vec<u16>>) {
    if let Some(p) = dma.get(&q.numero) {
        r.entregar_dma(p);
    }
    for e in &q.eventos {
        if e.tipo != Tipo::Vdp {
            continue;
        }
        let valor = u16::try_from(e.b & 0xffff).unwrap_or(0);
        let escrita = if e.a == 1 {
            Escrita::Controle(valor)
        } else {
            Escrita::Dados {
                valor,
                tamanho: u8::try_from(e.c).unwrap_or(2),
            }
        };
        r.aplicar(e.linha, u64::from(e.coluna), escrita);
    }
    r.fechar_quadro();
}

fn palavras_em_bytes(p: &[u16]) -> Vec<u8> {
    p.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// Hash da imagem: cada ponto em 2 bytes, little-endian (o mesmo do oráculo).
pub fn hash_da_imagem(r: &Renderizador) -> [u8; 20] {
    sha1(&r.quadro().para_bytes())
}

pub fn estado(args: &[String]) -> Result<(), String> {
    let trace = opcao(args, "--trace").ok_or("falta --trace")?;
    let video = opcao(args, "--video").ok_or("falta --video")?;
    let dma = ler_dma(video)?;
    let mut leitor = abrir_trace(trace)?;
    let mut r = Renderizador::novo();
    let (mut total, mut ok_vram, mut ok_cram, mut ok_vsram) = (0u32, 0u32, 0u32, 0u32);
    let mut primeira: Option<(u32, &'static str)> = None;
    while let Some(q) = leitor.proximo().map_err(|e| e.to_string())? {
        aplicar_quadro(&mut r, &q, &dma);
        total = total.saturating_add(1);
        let v = r.vdp();
        let iguais = [
            (
                "vram",
                sha1(v.vram()) == q.hashes[Categoria::Vram as usize],
                &mut ok_vram,
            ),
            (
                "cram",
                sha1(&palavras_em_bytes(v.cram())) == q.hashes[Categoria::Cram as usize],
                &mut ok_cram,
            ),
            (
                "vsram",
                sha1(&palavras_em_bytes(v.vsram())) == q.hashes[Categoria::Vsram as usize],
                &mut ok_vsram,
            ),
        ];
        for (nome, igual, contador) in iguais {
            if igual {
                *contador = contador.saturating_add(1);
            } else if primeira.is_none() {
                primeira = Some((q.numero, nome));
            }
        }
    }
    println!("quadros={total} vram_iguais={ok_vram} cram_iguais={ok_cram} vsram_iguais={ok_vsram}");
    println!("avisos_fora_do_inventario={}", r.vdp().avisos.total());
    match primeira {
        None => println!("video-estado: IGUAL em {total} quadros"),
        Some((q, c)) => println!("video-estado: DIFERE no quadro {q}, categoria {c}"),
    }
    Ok(())
}

/// Primeiro ponto diferente entre duas imagens de mesmo tamanho: (x, y, quantidade de pontos).
pub fn primeiro_ponto_diferente(
    a: &[u16],
    b: &[u16],
    largura: usize,
) -> Option<(usize, usize, usize)> {
    let mut primeiro = None;
    let mut n = 0usize;
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        if x != y {
            n = n.saturating_add(1);
            if primeiro.is_none() && largura > 0 {
                primeiro = Some((i % largura, i / largura));
            }
        }
    }
    primeiro.map(|(x, y)| (x, y, n))
}

pub fn comparar(args: &[String]) -> Result<(), String> {
    let trace = opcao(args, "--trace").ok_or("falta --trace")?;
    let video = opcao(args, "--video").ok_or("falta --video")?;
    let imagens = opcao(args, "--imagens").ok_or("falta --imagens")?;
    let dma = ler_dma(video)?;
    let esperados = fv::ler_imagens(BufReader::new(
        File::open(imagens).map_err(|e| e.to_string())?,
    ))
    .map_err(|e| e.to_string())?;
    let mut leitor = abrir_trace(trace)?;
    let mut r = Renderizador::novo();
    let mut relatorio = String::from("quadro\thash_esperado\thash_obtido\tveredito\n");
    let (mut total, mut iguais) = (0u32, 0u32);
    let mut primeiro: Option<u32> = None;
    let mut i = 0usize;
    while let Some(q) = leitor.proximo().map_err(|e| e.to_string())? {
        aplicar_quadro(&mut r, &q, &dma);
        let esperado = esperados
            .get(i)
            .ok_or("o .s1i tem menos quadros que o trace")?;
        i = i.saturating_add(1);
        let obtido = hash_da_imagem(&r);
        if std::env::var_os("DEPURAR_VIDEO").is_some() && q.numero < 4 {
            let mut m = BTreeMap::new();
            for p in r.quadro().pontos() {
                *m.entry(*p).or_insert(0u32) += 1;
            }
            eprintln!("q{} {:?} pos={:?} mclk_fim={} linhas={}", q.numero, m, r.posicao(), q.mclk_fim, q.mclk_fim / 3420);
            let negros: Vec<(usize, usize)> = (0..224).flat_map(|y| (0..320).map(move |x| (x, y))).filter(|(x, y)| r.quadro().ponto(*x, *y) == Some(0)).take(12).collect();
            eprintln!("negros {negros:?}");
            if let Some(e) = q.eventos.last() {
                eprintln!("  ULTIMO mclk={} linha={} col={} tipo={:?}", e.mclk, e.linha, e.coluna, e.tipo);
            }
            for e in q.eventos.iter().filter(|e| e.tipo == Tipo::Desvio68).step_by(40000).take(8) {
                eprintln!("  DESVIO mclk={} linha={} col={}", e.mclk, e.linha, e.coluna);
            }
            for e in q.eventos.iter().filter(|e| e.tipo != Tipo::Desvio68 && e.linha >= 10000).take(30) {
                eprintln!("  OUTRO tipo={:?} linha={} col={} a={} b={}", e.tipo, e.linha, e.coluna, e.a, e.b);
            }
            for e in q.eventos.iter().filter(|e| e.tipo == Tipo::Vdp).take(30) {
                eprintln!("  ev mclk={} linha={} col={} a={} b={:#06x} c={}", e.mclk, e.linha, e.coluna, e.a, e.b, e.c);
            }
        }
        let tamanho_ok = esperado.largura == 320 && esperado.altura == 224;
        let igual = tamanho_ok && esperado.sha1 == obtido;
        let veredito = if igual {
            "IGUAL"
        } else if tamanho_ok {
            "DIFERE"
        } else {
            "TAMANHO"
        };
        total = total.saturating_add(1);
        if igual {
            iguais = iguais.saturating_add(1);
        } else if primeiro.is_none() {
            primeiro = Some(q.numero);
        }
        relatorio.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            q.numero,
            em_hex(&esperado.sha1),
            em_hex(&obtido),
            veredito
        ));
        if primeiro == Some(q.numero) && !igual {
            // O detalhe do primeiro quadro que difere é pedido abaixo, com o quadro já desenhado.
            detalhar(args, q.numero, &r)?;
        }
    }
    println!("avisos_fora_do_inventario={}", r.vdp().avisos.total());
    let resumo = match primeiro {
        None => format!("video: IGUAL em {total} quadros"),
        Some(q) => format!("video: DIFERE no quadro {q} ({iguais} de {total} quadros iguais)"),
    };
    println!("{resumo}");
    if let Some(saida) = opcao(args, "--saida") {
        std::fs::write(saida, format!("{relatorio}{resumo}\n")).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Para o primeiro quadro que difere: regenera a imagem do oráculo e imprime a linha e a coluna
/// do primeiro ponto diferente. Nunca imprime cores.
fn detalhar(args: &[String], quadro: u32, r: &Renderizador) -> Result<(), String> {
    let (Some(rom), Some(bin), Some(estados), Some(caso)) = (
        opcao(args, "--rom"),
        opcao(args, "--oraculo"),
        opcao(args, "--estados"),
        opcao(args, "--caso"),
    ) else {
        println!("detalhe: faltam --rom, --oraculo, --estados e --caso para achar o ponto");
        return Ok(());
    };
    let k: u32 = opcao(args, "--intervalo")
        .unwrap_or("600")
        .parse()
        .map_err(|_| "--intervalo")?;
    // Último estado salvo em quadro menor ou igual a `quadro` (ou nenhum: roda do zero).
    let inicio = (quadro / k).saturating_mul(k);
    let estado = Path::new(estados).join(format!("{caso}.a.{inicio}.s1e"));
    let saida = Path::new(estados).join(format!("{caso}.diferenca.s1p"));
    let mut cmd = Command::new(bin);
    cmd.args([
        "imagem",
        "--rom",
        rom,
        "--quadro",
        &quadro.to_string(),
        "--saida",
    ])
    .arg(&saida);
    if inicio > 0 && estado.exists() {
        cmd.arg("--estado").arg(&estado);
    }
    if let Some(e) = opcao(args, "--entradas") {
        cmd.args(["--entradas", e]);
    }
    let s = cmd.output().map_err(|e| format!("oraculo: {e}"))?;
    if !s.status.success() {
        return Err(format!(
            "oraculo imagem falhou: {}",
            String::from_utf8_lossy(&s.stderr).trim()
        ));
    }
    let f = File::open(&saida).map_err(|e| e.to_string())?;
    let esperado = fv::ler_quadro_inteiro(BufReader::new(f)).map_err(|e| e.to_string())?;
    if std::env::var_os("DEPURAR_VIDEO").is_some() {
        let conta = |v: &[u16]| v.iter().filter(|p| **p == 448).count();
        eprintln!("azuis esperado={} obtido={} tam={}x{} n={}", conta(&esperado.pontos), conta(r.quadro().pontos()), esperado.largura, esperado.altura, esperado.pontos.len());
    }
    match primeiro_ponto_diferente(
        &esperado.pontos,
        r.quadro().pontos(),
        usize::from(esperado.largura),
    ) {
        None => println!(
            "detalhe: o quadro regenerado do oraculo tem pontos iguais aos nossos (hash do .s1i difere do regenerado)"
        ),
        Some((x, y, n)) => println!(
            "detalhe: quadro {quadro}, primeiro ponto diferente em x={x} linha={y}, {n} pontos diferentes"
        ),
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn primeiro_ponto_e_contagem() {
        let a = vec![1u16, 2, 3, 4, 5, 6];
        let mut b = a.clone();
        assert_eq!(primeiro_ponto_diferente(&a, &b, 3), None);
        b[4] = 0;
        b[5] = 0;
        assert_eq!(primeiro_ponto_diferente(&a, &b, 3), Some((1, 1, 2)));
        b[0] = 9;
        assert_eq!(primeiro_ponto_diferente(&a, &b, 3), Some((0, 0, 3)));
    }
}
