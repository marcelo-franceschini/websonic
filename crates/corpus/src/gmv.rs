//! Conversor de filme GMV (Gens Movie) para mascaras do projeto.
//!
//! Fonte do formato: pagina GMV do TASVideos (https://tasvideos.org/EmulatorResources/Gens/GMV),
//! lida em 06/11/2026. Cabecalho de 64 bytes ("Gens Movie TEST" na assinatura), depois 3 bytes por
//! quadro: controle 1, controle 2, byte extra. Bits ativos em zero:
//! 01 Cima, 02 Baixo, 04 Esquerda, 08 Direita, 10 A, 20 B, 40 C, 80 Start.
//!
//! Controle de 6 botoes ('6'): o byte extra guarda X, Y, Z e Mode. O Sonic 1 le o controle como de
//! 3 botoes, entao o conversor descarta o byte extra e a porta 2 e conta em quantos quadros houve
//! botao descartado (o `.meta` registra). Que o descarte nao muda a partida e uma hipotese: quem
//! confirma e a conferencia contra a REV01 (`cargo xtask tas conferir`).

use crate::{Erro, MAX_QUADROS};

const CABECALHO: usize = 64;
const ASSINATURA: &[u8] = b"Gens Movie TEST";
/// Byte 0x16 do cabecalho: bit 7 = 50 Hz (PAL), bit 6 = o filme comeca de um estado salvo.
const FLAGS: usize = 0x16;
const FLAG_PAL: u8 = 0x80;
const FLAG_ESTADO_SALVO: u8 = 0x40;

/// Resultado de uma conversao.
#[derive(Debug, PartialEq, Eq)]
pub struct Conversao {
    /// Mascaras do projeto, uma por quadro (ver `lib.rs`).
    pub mascaras: Vec<u8>,
    /// Quadros em que o byte extra (X, Y, Z, Mode) tinha botao apertado e foi descartado.
    pub quadros_xyz_mode: usize,
    /// Quadros em que a porta 2 tinha botao apertado e foi descartada.
    pub quadros_porta2: usize,
}

/// Converte um GMV. Recusa assinatura errada, controle 1 que nao seja de 3 ou 6 botoes, filme de
/// 50 Hz e filme que comeca de um estado salvo (nao comeca no ligar do console).
pub fn converter(dados: &[u8]) -> Result<Conversao, Erro> {
    if dados.len() < CABECALHO || !dados.starts_with(ASSINATURA) {
        return Err(Erro::Formato("assinatura GMV"));
    }
    let seis_botoes = match dados[0x14] {
        b'3' => false,
        b'6' => true,
        _ => return Err(Erro::Formato("controle 1 nao e de 3 nem de 6 botoes")),
    };
    if !matches!(dados[0x15], b'3' | b'6') {
        return Err(Erro::Formato("controle 2 desconhecido"));
    }
    if dados[FLAGS] & FLAG_PAL != 0 {
        return Err(Erro::Formato("filme de 50 Hz (PAL fora do escopo)"));
    }
    if dados[FLAGS] & FLAG_ESTADO_SALVO != 0 {
        return Err(Erro::Formato("filme comeca de um estado salvo"));
    }
    let corpo = &dados[CABECALHO..];
    if !corpo.len().is_multiple_of(3) {
        return Err(Erro::Formato("tamanho do corpo nao e multiplo de 3"));
    }
    let n = corpo.len() / 3;
    if n > MAX_QUADROS {
        return Err(Erro::Grande);
    }
    let quadros = corpo.as_chunks::<3>().0;
    Ok(Conversao {
        mascaras: quadros.iter().map(|q| mascara(q[0])).collect(),
        // Sem botao, o byte vale ff (ativo em zero). Controle de 3 botoes nao usa o byte extra.
        quadros_xyz_mode: if seis_botoes {
            quadros.iter().filter(|q| q[2] != 0xff).count()
        } else {
            0
        },
        quadros_porta2: quadros.iter().filter(|q| q[1] != 0xff).count(),
    })
}

/// Troca de ativo-em-zero para ativo-em-um e reordena A, B, C para o mapa do projeto.
fn mascara(gmv: u8) -> u8 {
    let p = !gmv;
    let mut m = p & 0x0f; // direcoes: mesmos bits
    if p & 0x10 != 0 {
        m |= 0x40; // A
    }
    if p & 0x20 != 0 {
        m |= 0x10; // B
    }
    if p & 0x40 != 0 {
        m |= 0x20; // C
    }
    if p & 0x80 != 0 {
        m |= 0x80; // Start
    }
    m
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Filme inventado por nos (nao e de nenhum jogo).
    fn filme(quadros: &[[u8; 3]], p1: u8, p2: u8, flags: u8) -> Vec<u8> {
        let mut v = vec![0u8; CABECALHO];
        v[..ASSINATURA.len()].copy_from_slice(ASSINATURA);
        v[0x0f] = b'A';
        v[0x14] = p1;
        v[0x15] = p2;
        v[FLAGS] = flags;
        for q in quadros {
            v.extend_from_slice(q);
        }
        v
    }

    #[test]
    fn converte_botoes() {
        // sem botao = ff; so Direita = f7; A = ef; B = df; C = bf; Start = 7f.
        let f = filme(
            &[
                [0xff; 3],
                [0xf7, 0xff, 0xff],
                [0xef, 0xff, 0xff],
                [0xdf, 0xff, 0xff],
                [0xbf, 0xff, 0xff],
                [0x7f, 0xff, 0xff],
            ],
            b'3',
            b'3',
            0,
        );
        let c = converter(&f).unwrap();
        assert_eq!(c.mascaras, vec![0x00, 0x08, 0x40, 0x10, 0x20, 0x80]);
        assert_eq!((c.quadros_xyz_mode, c.quadros_porta2), (0, 0));
    }

    /// Controle de 6 botoes: aceito, com a contagem dos botoes descartados.
    #[test]
    fn seis_botoes_descarta_xyz_mode_e_porta2_e_conta() {
        let f = filme(
            &[
                [0xf7, 0xff, 0xff], // Direita, nada descartado
                [0xff, 0xff, 0xfe], // so botao do byte extra
                [0xef, 0xff, 0xf7], // A (vale) e botao do byte extra (descartado)
                [0xff, 0xdf, 0xff], // so porta 2
                [0x7f, 0xef, 0xfb], // Start (vale), porta 2 e byte extra
                [0xff, 0xff, 0xff],
            ],
            b'6',
            b'6',
            0,
        );
        let c = converter(&f).unwrap();
        // Os botoes que valem (controle 1) passam; o resto nao muda a mascara.
        assert_eq!(c.mascaras, vec![0x08, 0x00, 0x40, 0x00, 0x80, 0x00]);
        assert_eq!(c.quadros_xyz_mode, 3);
        assert_eq!(c.quadros_porta2, 2);
    }

    #[test]
    fn tres_botoes_ignora_o_byte_extra() {
        let f = filme(&[[0xff, 0xff, 0x00]], b'3', b'3', 0);
        let c = converter(&f).unwrap();
        assert_eq!(c.quadros_xyz_mode, 0);
    }

    #[test]
    fn recusa_malformados() {
        assert!(converter(b"curto").is_err());
        // Tipo de controle desconhecido nas duas portas.
        assert!(converter(&filme(&[], b'x', b'3', 0)).is_err());
        assert!(converter(&filme(&[], b'6', b'x', 0)).is_err());
        let mut f = filme(&[[0xff; 3]], b'3', b'3', 0);
        f.push(0);
        assert!(converter(&f).is_err());
        let mut f = filme(&[], b'3', b'3', 0);
        f[0] = b'X';
        assert!(converter(&f).is_err());
    }

    /// 50 Hz e estado salvo continuam recusados, com o controle de 3 ou de 6 botoes.
    #[test]
    fn recusa_50_hz_e_estado_salvo() {
        for p1 in *b"36" {
            assert_eq!(
                converter(&filme(&[], p1, b'3', 0x80)),
                Err(Erro::Formato("filme de 50 Hz (PAL fora do escopo)"))
            );
            assert_eq!(
                converter(&filme(&[], p1, b'3', 0x40)),
                Err(Erro::Formato("filme comeca de um estado salvo"))
            );
            assert!(converter(&filme(&[], p1, b'3', 0xc0)).is_err());
            // Os outros bits do byte de flags nao bloqueiam.
            assert!(converter(&filme(&[], p1, b'3', 0x3f)).is_ok());
        }
    }

    #[test]
    fn vazio_e_valido() {
        let c = converter(&filme(&[], b'3', b'3', 0)).unwrap();
        assert_eq!(c.mascaras, Vec::<u8>::new());
    }
}
