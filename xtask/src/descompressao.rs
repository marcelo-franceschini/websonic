//! `cargo xtask descompressao-relatorio`: compara os descompressores do núcleo com o que o
//! oráculo produz para cada bloco comprimido da ROM.
//!
//! Roda no computador do Marcelo, que tem a ROM. A saída tem só rótulos, endereços, tamanhos,
//! hashes e vereditos. Nunca bytes.
//!
//! Entradas:
//! - `--rom`: a ROM REV01 (lida aqui só para descomprimir; nada dela é impresso).
//! - `--rotulos`: `refs/rotulos.txt`.
//! - `--manifesto`: `descompressao/manifesto.txt` (rótulo, formato, tiles iniciais do Enigma).
//! - `--oraculo`: a saída do subcomando `oraculo descomprimir`, uma linha por bloco.
//!
//! Linha do oráculo (campos separados por TAB):
//! `rotulo formato tile endereco_hex tamanho sha1` ou `rotulo formato tile ERRO motivo`.
//!
//! Uso: `cargo xtask descompressao-relatorio --rom ROM --rotulos refs/rotulos.txt
//! --manifesto descompressao/manifesto.txt --oraculo SAIDA.tsv [--saida RELATORIO.tsv]`
use sonic1_core::descompressao::{enigma, kosinski, nemesis};
use sonic1_fidelidade::rotulos::Rotulos;
use sonic1_rom::{em_hex, sha1};
use std::collections::{BTreeMap, BTreeSet};

/// Saída máxima aceita por bloco. O oráculo usa o mesmo limite (área de RAM de destino).
pub const LIMITE_SAIDA: usize = 0xAA00;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Formato {
    Nemesis,
    Enigma,
    Kosinski,
}

impl Formato {
    fn de_texto(t: &str) -> Option<Self> {
        match t {
            "nemesis" => Some(Self::Nemesis),
            "enigma" => Some(Self::Enigma),
            "kosinski" => Some(Self::Kosinski),
            _ => None,
        }
    }
}

/// Um item a testar: bloco, formato e tile inicial (só Enigma usa; os outros ficam com 0).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub rotulo: String,
    pub formato: Formato,
    pub tile: u16,
    /// Texto do tile como aparece no relatório (`-` fora do Enigma).
    pub tile_texto: String,
}

/// Lê o manifesto: uma linha por bloco, expandida em um item por tile inicial.
pub fn ler_manifesto(texto: &str) -> Result<Vec<Item>, String> {
    let mut itens = Vec::new();
    let mut vistos = BTreeSet::new();
    for (i, linha) in texto.lines().enumerate() {
        if linha.is_empty() || linha.starts_with('#') {
            continue;
        }
        let campos: Vec<&str> = linha.split('\t').collect();
        let [rotulo, formato, tiles] = campos.as_slice() else {
            return Err(format!("manifesto, linha {}: esperados 3 campos", i + 1));
        };
        let formato = Formato::de_texto(formato)
            .ok_or_else(|| format!("manifesto, linha {}: formato desconhecido", i + 1))?;
        if !rotulo
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.'))
            || rotulo.is_empty()
        {
            return Err(format!("manifesto, linha {}: rotulo invalido", i + 1));
        }
        if !vistos.insert((*rotulo).to_owned()) {
            return Err(format!("manifesto, linha {}: rotulo repetido", i + 1));
        }
        if formato == Formato::Enigma {
            for t in tiles.split(',') {
                let tile = u16::from_str_radix(t, 16)
                    .map_err(|_| format!("manifesto, linha {}: tile invalido", i + 1))?;
                itens.push(Item {
                    rotulo: (*rotulo).to_owned(),
                    formato,
                    tile,
                    tile_texto: t.to_owned(),
                });
            }
        } else {
            itens.push(Item {
                rotulo: (*rotulo).to_owned(),
                formato,
                tile: 0,
                tile_texto: "-".to_owned(),
            });
        }
    }
    Ok(itens)
}

/// Resposta do oráculo para um item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RespostaOraculo {
    Ok { tamanho: usize, sha1: String },
    Erro(String),
}

type ChaveItem = (String, String, String);

/// Lê a saída do oráculo. Linhas com outro formato são recusadas.
pub fn ler_oraculo(texto: &str) -> Result<BTreeMap<ChaveItem, RespostaOraculo>, String> {
    let mut mapa = BTreeMap::new();
    for (i, linha) in texto.lines().enumerate() {
        if linha.is_empty() || linha.starts_with('#') {
            continue;
        }
        let c: Vec<&str> = linha.split('\t').collect();
        let chave = match c.as_slice() {
            [r, f, t, ..] => ((*r).to_owned(), (*f).to_owned(), (*t).to_owned()),
            _ => return Err(format!("oraculo, linha {}: poucos campos", i + 1)),
        };
        let resposta = match c.as_slice() {
            [_, _, _, "ERRO", motivo] => RespostaOraculo::Erro((*motivo).to_owned()),
            [_, _, _, _endereco, tamanho, hash] => {
                let tamanho = tamanho
                    .parse()
                    .map_err(|_| format!("oraculo, linha {}: tamanho invalido", i + 1))?;
                if hash.len() != 40 || !hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
                    return Err(format!("oraculo, linha {}: sha1 invalido", i + 1));
                }
                RespostaOraculo::Ok {
                    tamanho,
                    sha1: (*hash).to_owned(),
                }
            }
            _ => return Err(format!("oraculo, linha {}: formato inesperado", i + 1)),
        };
        mapa.insert(chave, resposta);
    }
    Ok(mapa)
}

/// Resultado do relatório.
pub struct Relatorio {
    pub texto: String,
    pub iguais: usize,
    pub falhas: usize,
    pub ausentes: usize,
}

/// Descomprime um item com o núcleo. Devolve (bytes, consumidos).
fn descomprimir(item: &Item, entrada: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let r = match item.formato {
        Formato::Kosinski => kosinski::descomprimir(entrada, LIMITE_SAIDA),
        Formato::Enigma => enigma::descomprimir(entrada, item.tile, LIMITE_SAIDA),
        Formato::Nemesis => nemesis::descomprimir(entrada, LIMITE_SAIDA),
    }
    .map_err(|e| e.to_string())?;
    Ok((r.saida, r.consumidos))
}

/// Monta o relatório. Pura: recebe os textos e os bytes da ROM.
pub fn relatorio(
    rom: &[u8],
    rotulos_texto: &str,
    manifesto: &str,
    oraculo_texto: &str,
) -> Result<Relatorio, String> {
    let rotulos = Rotulos::analisar(rotulos_texto).map_err(|e| e.to_string())?;
    let itens = ler_manifesto(manifesto)?;
    let oraculo = ler_oraculo(oraculo_texto)?;
    // Limite de leitura de um bloco: o próximo bloco do manifesto. Rótulos de rotina em geral
    // não servem: o mapa tem símbolos que caem no meio de dados (ex.: uma constante de som).
    let enderecos: BTreeSet<u32> = itens
        .iter()
        .filter_map(|i| rotulos.resolver(&i.rotulo))
        .collect();

    let mut texto = String::from(
        "bloco\tformato\ttile\tendereco_origem\tconsumidos\ttamanho_saida\tsha1_oraculo\tsha1_obtido\tveredito\n",
    );
    let (mut iguais, mut falhas, mut ausentes) = (0usize, 0usize, 0usize);
    let nome_formato = |f: Formato| match f {
        Formato::Nemesis => "nemesis",
        Formato::Enigma => "enigma",
        Formato::Kosinski => "kosinski",
    };
    for item in &itens {
        let formato = nome_formato(item.formato);
        let Some(endereco) = rotulos.resolver(&item.rotulo) else {
            texto.push_str(&format!(
                "{}\t{formato}\t{}\t-\t-\t-\t-\t-\tAUSENTE\n",
                item.rotulo, item.tile_texto
            ));
            ausentes += 1;
            continue;
        };
        let prefixo = format!(
            "{}\t{formato}\t{}\t{endereco:x}",
            item.rotulo, item.tile_texto
        );
        let chave = (
            item.rotulo.clone(),
            formato.to_owned(),
            item.tile_texto.clone(),
        );
        let origem = usize::try_from(endereco).unwrap_or(usize::MAX);
        let entrada = rom.get(origem..).unwrap_or(&[]);
        let obtido = descomprimir(item, entrada);
        let (veredito, linha) = match (&oraculo.get(&chave), &obtido) {
            (None, _) => ("SEM_ORACULO", format!("{prefixo}\t-\t-\t-\t-\tSEM_ORACULO")),
            (Some(RespostaOraculo::Erro(m)), _) => (
                "ERRO",
                format!("{prefixo}\t-\t-\t-\t-\tERRO (oraculo: {m})"),
            ),
            (Some(_), Err(m)) => ("ERRO", format!("{prefixo}\t-\t-\t-\t-\tERRO ({m})")),
            (
                Some(RespostaOraculo::Ok {
                    tamanho,
                    sha1: esperado,
                }),
                Ok((bytes, consumidos)),
            ) => {
                let hash = em_hex(&sha1(bytes));
                // O bloco não pode passar do próximo rótulo de rotina ou dado.
                let proximo = enderecos
                    .range(endereco.saturating_add(1)..)
                    .next()
                    .copied();
                let passa = proximo.is_some_and(|p| {
                    u64::from(endereco).saturating_add(*consumidos as u64) > u64::from(p)
                });
                let v = if passa {
                    "ERRO (le alem do proximo rotulo)"
                } else if &hash == esperado && bytes.len() == *tamanho {
                    "IGUAL"
                } else {
                    "DIFERE"
                };
                (
                    if v == "IGUAL" { "IGUAL" } else { "FALHA" },
                    format!(
                        "{prefixo}\t{consumidos}\t{}\t{esperado}\t{hash}\t{v}{}",
                        bytes.len(),
                        if v == "DIFERE" {
                            format!(" (tamanho_oraculo={tamanho})")
                        } else {
                            String::new()
                        }
                    ),
                )
            }
        };
        match veredito {
            "IGUAL" => iguais += 1,
            _ => falhas += 1,
        }
        texto.push_str(&linha);
        texto.push('\n');
    }
    texto.push_str(&format!(
        "# resumo: iguais={iguais} falhas={falhas} ausentes={ausentes} total={}\n",
        itens.len()
    ));
    texto.push_str(&format!(
        "# sha1_do_relatorio={}\n",
        em_hex(&sha1(texto.as_bytes()))
    ));
    Ok(Relatorio {
        texto,
        iguais,
        falhas,
        ausentes,
    })
}

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let ler = |nome: &str| -> Result<String, String> {
        let caminho = opcao(args, nome).ok_or(format!("falta {nome}"))?;
        std::fs::read_to_string(caminho).map_err(|e| format!("{caminho}: {e}"))
    };
    let rom = std::fs::read(opcao(args, "--rom").ok_or("falta --rom")?)
        .map_err(|e| format!("nao li a ROM: {e}"))?;
    let rotulos = ler("--rotulos")?;
    // Não repete a conferência do SHA-1 aqui: o script de prova já confere a ROM.
    let r = relatorio(&rom, &rotulos, &ler("--manifesto")?, &ler("--oraculo")?)?;
    if let Some(saida) = opcao(args, "--saida") {
        std::fs::write(saida, &r.texto).map_err(|e| e.to_string())?;
    }
    print!("{}", r.texto);
    // Rótulo ausente é esperado (ramos REV00). Falha é qualquer outro veredito, ou nenhum IGUAL.
    eprintln!("ausentes (rotulo fora da REV01): {}", r.ausentes);
    if r.falhas > 0 || r.iguais == 0 {
        return Err(format!(
            "descompressao: {} falhas, {} iguais",
            r.falhas, r.iguais
        ));
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_teste_descompressao::{enigma as ce, kosinski as ck, nemesis as cn};

    /// ROM sintética: três blocos em endereços conhecidos, com folga entre eles.
    fn cenario() -> (Vec<u8>, String, String, String) {
        let kos_dados: Vec<u8> = (0..400u32).map(|i| (i % 7) as u8).collect();
        let palavras: Vec<u16> = (0..200u16).map(|i| i % 9).collect();
        let p = ce::parametros_para(&palavras, 0x0100, 0);
        let tiles: Vec<u8> = (0..64u32).map(|i| (i % 5) as u8).collect();
        let blocos = [
            ck::comprimir(&kos_dados),
            ce::comprimir(&palavras, &p),
            cn::comprimir(&tiles, false),
        ];
        let mut rom = vec![0u8; 0x100];
        let mut enderecos = Vec::new();
        for b in &blocos {
            enderecos.push(rom.len());
            rom.extend_from_slice(b);
            rom.resize(rom.len() + 16, 0xEE);
        }
        let rotulos = format!(
            "BlocoK\trotina\t{:x}\nBlocoE\trotina\t{:x}\nBlocoN\trotina\t{:x}\nFim\trotina\t{:x}\n",
            enderecos[0],
            enderecos[1],
            enderecos[2],
            rom.len()
        );
        let manifesto = "BlocoK\tkosinski\t-\nBlocoE\tenigma\t0000,2345\nBlocoN\tnemesis\t-\nFora\tnemesis\t-\n"
            .to_owned();
        // Respostas "do oráculo" geradas pelos próprios descompressores sobre dados sintéticos.
        let linha = |r: &str, f: &str, t: &str, saida: &[u8]| {
            format!(
                "{r}\t{f}\t{t}\t0\t{}\t{}\n",
                saida.len(),
                em_hex(&sha1(saida))
            )
        };
        let mut oraculo = String::new();
        oraculo += &linha("BlocoK", "kosinski", "-", &kos_dados);
        for t in [0u16, 0x2345] {
            let bytes: Vec<u8> = palavras
                .iter()
                .flat_map(|w| w.wrapping_add(t).to_be_bytes())
                .collect();
            oraculo += &linha("BlocoE", "enigma", &format!("{t:04x}"), &bytes);
        }
        oraculo += &linha("BlocoN", "nemesis", "-", &tiles);
        (rom, rotulos, manifesto, oraculo)
    }

    #[test]
    fn tudo_igual_e_rotulo_ausente_nao_falha() {
        let (rom, rotulos, manifesto, oraculo) = cenario();
        // O tile do manifesto é "0000"; ajusta o oráculo para a mesma grafia.
        let r = relatorio(&rom, &rotulos, &manifesto, &oraculo).unwrap();
        assert_eq!((r.iguais, r.falhas, r.ausentes), (4, 0, 1), "{}", r.texto);
        assert!(r.texto.contains("Fora\tnemesis\t-\t-"));
        assert!(r.texto.contains("AUSENTE"));
    }

    #[test]
    fn hash_adulterado_difere_e_nao_vaza_bytes() {
        let (rom, rotulos, manifesto, oraculo) = cenario();
        let ruim = oraculo.replacen("kosinski\t-\t0\t400\t", "kosinski\t-\t0\t400\t0000", 1);
        // O hash inválido (44 chars) é recusado na leitura.
        assert!(relatorio(&rom, &rotulos, &manifesto, &ruim).is_err());
        let primeira = oraculo.lines().next().unwrap().to_owned();
        let hash = primeira.rsplit('\t').next().unwrap().to_owned();
        let trocado = oraculo.replacen(&hash, &"0".repeat(40), 1);
        let r = relatorio(&rom, &rotulos, &manifesto, &trocado).unwrap();
        assert_eq!(r.falhas, 1);
        assert!(r.texto.contains("DIFERE"));
        // Nenhuma linha tem mais campos que o formato.
        assert!(
            r.texto
                .lines()
                .filter(|l| !l.starts_with('#'))
                .all(|l| l.split('\t').count() <= 9)
        );
    }

    #[test]
    fn erro_do_oraculo_e_do_descompressor_viram_erro() {
        let (rom, rotulos, manifesto, oraculo) = cenario();
        let com_erro = oraculo.replacen(
            "BlocoK\tkosinski\t-\t0\t400",
            "BlocoK\tkosinski\t-\tERRO\tpassos",
            1,
        );
        let hash = oraculo
            .lines()
            .next()
            .unwrap()
            .rsplit('\t')
            .next()
            .unwrap()
            .to_owned();
        let com_erro = com_erro.replacen(&format!("\t{hash}"), "", 1);
        let r = relatorio(&rom, &rotulos, &manifesto, &com_erro).unwrap();
        assert!(r.texto.contains("ERRO (oraculo: passos)"), "{}", r.texto);
        // ROM zerada: o Kosinski do bloco acaba truncado ou invalido.
        let zerada = vec![0u8; rom.len()];
        let r = relatorio(&zerada, &rotulos, &manifesto, &oraculo).unwrap();
        assert!(r.falhas >= 1);
    }

    #[test]
    fn bloco_que_passa_do_proximo_rotulo_e_erro() {
        let (rom, rotulos, manifesto, oraculo) = cenario();
        // Aproxima o rótulo seguinte do BlocoK: o bloco consome mais do que cabe.
        let linhas: Vec<&str> = rotulos.lines().collect();
        let k: u32 = u32::from_str_radix(linhas[0].rsplit('\t').next().unwrap(), 16).unwrap();
        let apertado = rotulos.replacen(linhas[1], &format!("BlocoE\trotina\t{:x}", k + 4), 1);
        let r = relatorio(&rom, &apertado, &manifesto, &oraculo).unwrap();
        assert!(r.texto.contains("le alem do proximo rotulo"), "{}", r.texto);
    }

    #[test]
    fn manifesto_ruim_e_recusado() {
        for ruim in [
            "a\tnemesis",
            "a\toutro\t-",
            "a b\tnemesis\t-",
            "a\tnemesis\t-\na\tnemesis\t-",
            "a\tenigma\tzz",
        ] {
            assert!(ler_manifesto(ruim).is_err(), "{ruim:?}");
        }
    }

    #[test]
    fn o_manifesto_do_repositorio_e_valido() {
        let texto = include_str!("../../descompressao/manifesto.txt");
        let itens = ler_manifesto(texto).unwrap();
        assert!(itens.len() > 100);
        assert!(itens.iter().any(|i| i.formato == Formato::Kosinski));
        assert!(
            itens
                .iter()
                .any(|i| i.formato == Formato::Enigma && i.tile == 0x2345)
        );
    }
}
