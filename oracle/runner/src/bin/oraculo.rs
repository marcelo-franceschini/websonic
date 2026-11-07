//! Linha de comando do oraculo.
//!
//! ```text
//! oraculo rodar --rom ARQ --quadros N --saida ARQ [--entradas ARQ] [--rotulos ARQ --parar-em NOME]
//!               [--salvar-estado ARQ] [--estado-a-cada N] [--carregar-estado ARQ]
//!               [--perfil-a-cada N] [--video PREFIXO]   (com --rotulos: imprime modo, zona, ato e demo)
//! oraculo comparar TRACE_A TRACE_B
//! oraculo imagem --rom ARQ --estado ARQ --quadro N --saida ARQ.s1p [--entradas ARQ]
//! oraculo descomprimir --rom ARQ --rotulos ARQ --manifesto ARQ [--saida ARQ]
//! ```
//!
//! A saida no terminal tem so hashes, contagens e nomes. Nunca imprime memoria, nem eventos.

use sonic1_fidelidade::hash::em_hex;
use sonic1_fidelidade::rotulos::Rotulos;
use sonic1_fidelidade::{Escritor, Leitor, Onde, Tipo, comparar_leitores};
use sonic1_formato_video as fv;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process::ExitCode;
use websonic_oracle::{Entradas, MotivoParada, Oraculo, Parada};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        Some("rodar") => rodar(&args[1..]),
        Some("comparar") => comparar(&args[1..]),
        Some("imagem") => imagem(&args[1..]),
        Some("descomprimir") => descomprimir(&args[1..]),
        Some("rom-sintetica") => rom_sintetica(&args[1..]),
        _ => Err(
            "uso: oraculo rodar --rom ARQ --quadros N --saida ARQ [...] | oraculo comparar A B"
                .into(),
        ),
    };
    match resultado {
        Ok(codigo) => codigo,
        Err(mensagem) => {
            eprintln!("erro: {mensagem}");
            ExitCode::from(2)
        }
    }
}

/// Pega o valor da opcao `--nome`.
fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn rodar(args: &[String]) -> Result<ExitCode, String> {
    let caminho_rom = opcao(args, "--rom").ok_or("falta --rom")?;
    let quadros: u32 = opcao(args, "--quadros")
        .ok_or("falta --quadros")?
        .parse()
        .map_err(|_| "--quadros deve ser um numero")?;
    let saida = opcao(args, "--saida").ok_or("falta --saida")?;

    let rom = std::fs::read(caminho_rom).map_err(|e| format!("nao li a ROM: {e}"))?;
    let entradas = match opcao(args, "--entradas") {
        Some(c) => Entradas::de_texto(&std::fs::read_to_string(c).map_err(|e| e.to_string())?)?,
        None => Entradas::default(),
    };
    let parada = match (opcao(args, "--rotulos"), opcao(args, "--parar-em")) {
        (Some(arq), Some(nome)) => {
            let texto = std::fs::read_to_string(arq).map_err(|e| e.to_string())?;
            let rotulos = Rotulos::analisar(&texto).map_err(|e| e.to_string())?;
            let endereco = rotulos.resolver(nome).ok_or("rotulo desconhecido")?;
            Some(Parada::Pc68(endereco))
        }
        (_, None) => None,
        _ => return Err("--rotulos e --parar-em andam juntos".into()),
    };

    let perfil_a_cada: u32 = match opcao(args, "--perfil-a-cada") {
        Some(n) => n
            .parse()
            .map_err(|_| "--perfil-a-cada deve ser um numero")?,
        None => 0,
    };
    // Enderecos das variaveis do perfil (numeros de modo, zona, ato e demo; sao metadados).
    let perfil_enderecos = if perfil_a_cada > 0 {
        let arq = opcao(args, "--rotulos").ok_or("--perfil-a-cada exige --rotulos")?;
        let rotulos = Rotulos::analisar(&std::fs::read_to_string(arq).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let mut v = Vec::new();
        for nome in ["v_gamemode", "v_zone", "v_act", "v_demonum"] {
            v.push(
                rotulos
                    .resolver(nome)
                    .ok_or("variavel do perfil ausente no mapa")?,
            );
        }
        v
    } else {
        Vec::new()
    };
    let estado_a_cada: u32 = match opcao(args, "--estado-a-cada") {
        Some(n) => n
            .parse()
            .map_err(|_| "--estado-a-cada deve ser um numero")?,
        None => 0,
    };
    // Video (pedidos P6 e P7): `--video PREFIXO` grava PREFIXO.s1i e PREFIXO.s1v.
    let mut video = match opcao(args, "--video") {
        Some(prefixo) => {
            if opcao(args, "--parar-em").is_some() {
                return Err("--video nao combina com --parar-em".into());
            }
            let mut i =
                BufWriter::new(File::create(format!("{prefixo}.s1i")).map_err(|e| e.to_string())?);
            let mut v =
                BufWriter::new(File::create(format!("{prefixo}.s1v")).map_err(|e| e.to_string())?);
            fv::escrever_cabecalho_imagem(&mut i).map_err(|e| e.to_string())?;
            fv::escrever_cabecalho_video(&mut v).map_err(|e| e.to_string())?;
            Some((i, v))
        }
        None => None,
    };
    let mut oraculo = Oraculo::novo(rom, entradas)?;
    if let Some(arq) = opcao(args, "--carregar-estado") {
        let bytes = std::fs::read(arq).map_err(|e| format!("nao li o estado: {e}"))?;
        oraculo.carregar_estado(&bytes)?;
    }
    let cabecalho = oraculo.cabecalho().clone();
    let arquivo = BufWriter::new(File::create(saida).map_err(|e| e.to_string())?);
    let mut escritor = Escritor::novo(arquivo, &cabecalho).map_err(|e| e.to_string())?;
    let mut contagem: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut fechados = 0u32;
    let mut mclk_final = 0u64;
    let mut parou_em_endereco = false;

    let mut registrar = |q: sonic1_fidelidade::Quadro,
                         escritor: &mut Escritor<BufWriter<File>>|
     -> Result<(), String> {
        for e in &q.eventos {
            *contagem.entry(e.tipo.nome()).or_default() += 1;
        }
        mclk_final = q.mclk_fim;
        escritor.escrever(&q).map_err(|e| e.to_string())
    };

    if let Some(parada) = parada {
        let mut pendentes = Vec::new();
        let motivo = oraculo.executar_ate(
            &[parada, Parada::InicioDoQuadro(quadros)],
            u64::from(quadros).saturating_mul(5_000_000),
            &mut |q| pendentes.push(q),
        )?;
        for q in pendentes {
            registrar(q, &mut escritor)?;
            fechados += 1;
        }
        parou_em_endereco = motivo == MotivoParada::Parada(parada);
    } else {
        while oraculo.quadro_atual() < quadros {
            let q = oraculo.executar_quadro()?;
            let numero = q.numero;
            registrar(q, &mut escritor)?;
            fechados += 1;
            if let Some((i, v)) = video.as_mut() {
                registrar_video(&oraculo, numero, i, v)?;
            }
            let atual = oraculo.quadro_atual();
            if estado_a_cada > 0 && atual % estado_a_cada == 0 && atual < quadros {
                let base = std::path::Path::new(saida).with_extension("");
                let nome = format!("{}.{atual}.s1e", base.display());
                std::fs::write(&nome, oraculo.salvar_estado()?).map_err(|e| e.to_string())?;
            }
            if perfil_a_cada > 0 && atual % perfil_a_cada == 0 {
                let v: Vec<u8> = perfil_enderecos
                    .iter()
                    .map(|e| oraculo.ler_byte_ram68k(*e))
                    .collect();
                println!(
                    "perfil quadro={atual} modo={} zona={} ato={} demo={}",
                    v[0], v[1], v[2], v[3]
                );
            }
        }
    }
    if let Some((mut i, mut v)) = video {
        use std::io::Write;
        i.flush().map_err(|e| e.to_string())?;
        v.flush().map_err(|e| e.to_string())?;
    }
    if let Some(arq) = opcao(args, "--salvar-estado") {
        std::fs::write(arq, oraculo.salvar_estado()?).map_err(|e| e.to_string())?;
    }
    let hash = escritor.finalizar(&cabecalho).map_err(|e| e.to_string())?;

    println!("oraculo={}", cabecalho.oraculo);
    println!("rom_sha1={}", em_hex(&cabecalho.rom_sha1));
    println!("quadros={fechados}");
    println!("mclk_final={mclk_final}");
    println!("parou_no_endereco={parou_em_endereco}");
    for tipo in Tipo::TODOS {
        println!(
            "eventos_{}={}",
            tipo.nome(),
            contagem.get(tipo.nome()).copied().unwrap_or(0)
        );
    }
    println!("hash_do_trace={}", em_hex(&hash));
    Ok(ExitCode::SUCCESS)
}

/// Bytes da imagem usados no hash: cada ponto de 9 bits em 2 bytes, little-endian.
fn bytes_da_imagem(pontos: &[u16]) -> Vec<u8> {
    pontos.iter().flat_map(|p| p.to_le_bytes()).collect()
}

/// Grava o registro de imagem e, se houve DMA, o de palavras do quadro recem-fechado.
fn registrar_video(
    oraculo: &Oraculo,
    quadro: u32,
    i: &mut BufWriter<File>,
    v: &mut BufWriter<File>,
) -> Result<(), String> {
    let tela = oraculo.imagem();
    if let Some(e) = &tela.erro {
        return Err(format!("quadro {quadro}: {e}"));
    }
    let registro = fv::RegistroImagem {
        quadro,
        largura: u16::try_from(tela.largura).map_err(|_| "largura")?,
        altura: u16::try_from(tela.altura).map_err(|_| "altura")?,
        sha1: sonic1_fidelidade::hash::sha1(&bytes_da_imagem(&tela.pontos)),
    };
    fv::escrever_imagem(i, &registro).map_err(|e| e.to_string())?;
    let palavras = oraculo.dma_do_quadro();
    if !palavras.is_empty() {
        let r = fv::RegistroDma {
            quadro,
            palavras: palavras.to_vec(),
        };
        fv::escrever_dma(v, &r).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Regenera um quadro inteiro a partir de um estado salvo (pedido P6). Imprime so o hash.
fn imagem(args: &[String]) -> Result<ExitCode, String> {
    let rom = std::fs::read(opcao(args, "--rom").ok_or("falta --rom")?)
        .map_err(|e| format!("nao li a ROM: {e}"))?;
    let alvo: u32 = opcao(args, "--quadro")
        .ok_or("falta --quadro")?
        .parse()
        .map_err(|_| "--quadro deve ser um numero")?;
    let saida = opcao(args, "--saida").ok_or("falta --saida")?;
    let entradas = match opcao(args, "--entradas") {
        Some(c) => Entradas::de_texto(&std::fs::read_to_string(c).map_err(|e| e.to_string())?)?,
        None => Entradas::default(),
    };
    let mut oraculo = Oraculo::novo(rom, entradas)?;
    if let Some(arq) = opcao(args, "--estado") {
        let bytes = std::fs::read(arq).map_err(|e| format!("nao li o estado: {e}"))?;
        oraculo.carregar_estado(&bytes)?;
    }
    if oraculo.quadro_atual() > alvo {
        return Err("o estado ja passou do quadro pedido".into());
    }
    while oraculo.quadro_atual() <= alvo {
        oraculo.executar_quadro()?;
    }
    let tela = oraculo.imagem();
    if let Some(e) = &tela.erro {
        return Err(e.clone());
    }
    let q = fv::QuadroInteiro {
        quadro: alvo,
        largura: u16::try_from(tela.largura).map_err(|_| "largura")?,
        altura: u16::try_from(tela.altura).map_err(|_| "altura")?,
        pontos: tela.pontos.clone(),
    };
    let mut f = BufWriter::new(File::create(saida).map_err(|e| e.to_string())?);
    fv::escrever_quadro_inteiro(&mut f, &q).map_err(|e| e.to_string())?;
    println!("quadro={alvo}");
    println!("tamanho={}x{}", q.largura, q.altura);
    println!(
        "sha1_da_imagem={}",
        em_hex(&sonic1_fidelidade::hash::sha1(&bytes_da_imagem(&q.pontos)))
    );
    Ok(ExitCode::SUCCESS)
}

fn comparar(args: &[String]) -> Result<ExitCode, String> {
    let [a, b] = args else {
        return Err("uso: oraculo comparar A B".into());
    };
    let abrir = |c: &String| -> Result<Leitor<BufReader<File>>, String> {
        Leitor::abrir(BufReader::new(File::open(c).map_err(|e| e.to_string())?))
            .map_err(|e| e.to_string())
    };
    let (mut la, mut lb) = (abrir(a)?, abrir(b)?);
    match comparar_leitores(&mut la, &mut lb).map_err(|e| e.to_string())? {
        None => {
            println!("IGUAIS");
            Ok(ExitCode::SUCCESS)
        }
        Some(d) => {
            let onde = match d.onde {
                Onde::Cabecalho => "cabecalho".to_owned(),
                Onde::Tamanho { esquerda } => format!(
                    "tamanho (acabou primeiro: {})",
                    if esquerda { "A" } else { "B" }
                ),
                Onde::NumeroDoQuadro => "numero do quadro".to_owned(),
                Onde::RelogioFinal => "relogio final do quadro".to_owned(),
                Onde::Categoria { categoria, evento } => match evento {
                    Some(i) => format!("{} (primeiro evento diferente: {i})", categoria.nome()),
                    None => categoria.nome().to_owned(),
                },
            };
            println!("DIVERGE quadro={} onde={onde}", d.quadro);
            Ok(ExitCode::from(1))
        }
    }
}

/// Grava uma ROM sintetica (escrita por nos, sem dado da ROM real) para testar os scripts.
fn rom_sintetica(args: &[String]) -> Result<ExitCode, String> {
    let [tipo, saida] = args else {
        return Err("uso: oraculo rom-sintetica entrada|vblank|z80 ARQ".into());
    };
    let rom = match tipo.as_str() {
        "entrada" => websonic_oracle::rom_sintetica::programa_entrada(),
        "vblank" => websonic_oracle::rom_sintetica::programa_vblank(),
        "z80" => websonic_oracle::rom_sintetica::programa_z80(),
        "video" => websonic_oracle::rom_sintetica::programa_video(),
        _ => return Err("tipo desconhecido".into()),
    };
    std::fs::write(saida, rom).map_err(|e| e.to_string())?;
    Ok(ExitCode::SUCCESS)
}

/// Chama `KosDec`, `EniDec` e `NemDecToRAM` para cada bloco do manifesto e imprime so
/// rotulo, formato, tile, endereco, tamanho e SHA-1 da saida.
fn descomprimir(args: &[String]) -> Result<ExitCode, String> {
    let rom = std::fs::read(opcao(args, "--rom").ok_or("falta --rom")?)
        .map_err(|e| format!("nao li a ROM: {e}"))?;
    let ler = |nome: &str| -> Result<String, String> {
        let c = opcao(args, nome).ok_or(format!("falta {nome}"))?;
        std::fs::read_to_string(c).map_err(|e| format!("{c}: {e}"))
    };
    let texto = websonic_oracle::descompressao::descomprimir_lote(
        rom,
        &ler("--rotulos")?,
        &ler("--manifesto")?,
    )?;
    if let Some(saida) = opcao(args, "--saida") {
        std::fs::write(saida, &texto).map_err(|e| e.to_string())?;
    }
    print!("{texto}");
    Ok(ExitCode::SUCCESS)
}
