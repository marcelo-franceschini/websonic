//! Chama as rotinas de descompressao do jogo (`KosDec`, `EniDec`, `NemDecToRAM`) de forma
//! isolada, para cada bloco do manifesto, e devolve so tamanho e SHA-1 da saida.
//!
//! O manifesto e o mapa de rotulos dao os enderecos. Nada e impresso alem de nomes, enderecos,
//! tamanhos e hashes. A saida bruta nunca sai deste modulo.
//!
//! Como se mede o tamanho: as rotinas restauram o registrador de destino ao sair, entao o
//! tamanho vem da RAM. A area de destino e preenchida duas vezes (com 00 e com FF); o que a
//! rotina escreveu e o prefixo em que as duas execucoes terminam iguais.

use crate::{ChamadaRotina, Entradas, Oraculo};
use sonic1_fidelidade::hash::{em_hex, sha1};
use sonic1_fidelidade::rotulos::Rotulos;

/// Onde a saida e escrita na RAM do 68000 ($FF0000).
pub const DESTINO: u32 = 0xFF_0000;
/// Tamanho maximo de saida aceito (igual ao do xtask).
pub const LIMITE_SAIDA: u32 = 0xAA00;
/// Endereco de retorno falso. Fica dentro da ROM e nunca e executado.
pub const SENTINELA: u32 = 0x1_FFF0;
/// Pilha de supervisor usada nas chamadas.
pub const PILHA: u32 = 0xFF_FE00;
const MAX_PASSOS: u64 = 400_000_000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Formato {
    Nemesis,
    Enigma,
    Kosinski,
}

impl Formato {
    fn nome(self) -> &'static str {
        match self {
            Self::Nemesis => "nemesis",
            Self::Enigma => "enigma",
            Self::Kosinski => "kosinski",
        }
    }

    /// Rotulo da rotina do jogo que reproduz o formato.
    fn rotina(self) -> &'static str {
        match self {
            Self::Nemesis => "NemDecToRAM",
            Self::Enigma => "EniDec",
            Self::Kosinski => "KosDec",
        }
    }
}

struct Item {
    rotulo: String,
    formato: Formato,
    tile_texto: String,
    tile: u16,
}

fn ler_manifesto(texto: &str) -> Result<Vec<Item>, String> {
    let mut itens = Vec::new();
    for (i, linha) in texto.lines().enumerate() {
        if linha.is_empty() || linha.starts_with('#') {
            continue;
        }
        let c: Vec<&str> = linha.split('\t').collect();
        let [rotulo, formato, tiles] = c.as_slice() else {
            return Err(format!("manifesto, linha {}: esperados 3 campos", i + 1));
        };
        let formato = match *formato {
            "nemesis" => Formato::Nemesis,
            "enigma" => Formato::Enigma,
            "kosinski" => Formato::Kosinski,
            _ => return Err(format!("manifesto, linha {}: formato desconhecido", i + 1)),
        };
        if formato == Formato::Enigma {
            for t in tiles.split(',') {
                let tile = u16::from_str_radix(t, 16)
                    .map_err(|_| format!("manifesto, linha {}: tile invalido", i + 1))?;
                itens.push(Item {
                    rotulo: (*rotulo).to_owned(),
                    formato,
                    tile_texto: t.to_owned(),
                    tile,
                });
            }
        } else {
            itens.push(Item {
                rotulo: (*rotulo).to_owned(),
                formato,
                tile_texto: "-".to_owned(),
                tile: 0,
            });
        }
    }
    Ok(itens)
}

/// Uma chamada com a area de destino preenchida com `enchimento`. Devolve a area depois.
fn chamar(
    o: &mut Oraculo,
    estado: &[u8],
    item: &Item,
    rotina: u32,
    origem: u32,
    enchimento: u8,
    tamanho_area: u32,
) -> Result<Vec<u8>, String> {
    o.carregar_estado(estado)?;
    let mut regs = o.emulador().oracle_regs68();
    regs.d = [0; 8];
    regs.a = [0; 7];
    regs.d[0] = u32::from(item.tile);
    regs.a[0] = origem;
    regs.a[1] = DESTINO;
    regs.a[4] = DESTINO;
    regs.pc = rotina;
    regs.sr = 0x2700;
    regs.ssp = PILHA;
    let pokes = (0..tamanho_area)
        .map(|i| (DESTINO + i, enchimento))
        .collect();
    o.chamar_rotina(&ChamadaRotina {
        regs,
        pokes_ram68: pokes,
        sentinela: SENTINELA,
        mascarar_interrupcoes: true,
        max_passos: MAX_PASSOS,
    })?;
    Ok((0..tamanho_area)
        .map(|i| o.ler_byte_ram68k(DESTINO + i))
        .collect())
}

/// Roda o manifesto inteiro. Cada linha da saida: `rotulo formato tile endereco tamanho sha1`
/// ou `rotulo formato tile ERRO motivo`.
pub fn descomprimir_lote(
    rom: Vec<u8>,
    rotulos_texto: &str,
    manifesto: &str,
) -> Result<String, String> {
    let rotulos = Rotulos::analisar(rotulos_texto).map_err(|e| e.to_string())?;
    let itens = ler_manifesto(manifesto)?;
    let mut o = Oraculo::novo(rom, Entradas::default())?;
    let estado = o.salvar_estado()?;
    // A area de destino nao pode chegar ao buffer que o Nemesis usa como rascunho.
    let area = match rotulos.resolver("v_ngfx_buffer") {
        Some(b) if b > DESTINO && b < DESTINO + LIMITE_SAIDA => b - DESTINO,
        _ => LIMITE_SAIDA,
    };
    let mut saida =
        String::from("# oraculo descomprimir: rotulo formato tile endereco tamanho sha1\n");
    for item in &itens {
        let prefixo = format!(
            "{}\t{}\t{}",
            item.rotulo,
            item.formato.nome(),
            item.tile_texto
        );
        let (Some(origem), Some(rotina)) = (
            rotulos.resolver(&item.rotulo),
            rotulos.resolver(item.formato.rotina()),
        ) else {
            saida.push_str(&format!("{prefixo}\tERRO\trotulo ausente\n"));
            continue;
        };
        let a = chamar(&mut o, &estado, item, rotina, origem, 0x00, area);
        let b = chamar(&mut o, &estado, item, rotina, origem, 0xFF, area);
        match (a, b) {
            (Ok(a), Ok(b)) => {
                let n = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
                if n == a.len() {
                    saida.push_str(&format!("{prefixo}\tERRO\tsaida passa da area\n"));
                } else {
                    saida.push_str(&format!(
                        "{prefixo}\t{origem:x}\t{n}\t{}\n",
                        em_hex(&sha1(&a[..n]))
                    ));
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                saida.push_str(&format!("{prefixo}\tERRO\t{}\n", e.replace('\t', " ")));
            }
        }
    }
    Ok(saida)
}
