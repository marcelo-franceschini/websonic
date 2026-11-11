//! Comparador diferencial amostra a amostra contra os chips de referencia Nuked-OPN2 e Nuked-PSG.
//!
//! Regras (docs/plano-etapa-8.md secao 2):
//! - Amostra FM: 24 pares (mol, mor) de 9 bits com sinal, serializados em 96 bytes little-endian.
//! - Amostra PSG: saida digital de 16 bits (4 bits por canal), serializada em 2 bytes little-endian.
//! - Bloco: 4096 amostras. SHA-1 por bloco comparado entre referencia e nucleo.
//! - Relatorio: contagens, hashes, veredito IGUAL ou DIVERGE, sem expor dados da ROM.

use crate::{RefOpn2, RefPsg};
use sonic1_audio::fm::Fm;
use sonic1_audio::psg::Psg;
use sonic1_fidelidade::hash::{em_hex, sha1};

pub const AMOSTRAS_POR_BLOCO: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Veredito {
    Igual,
    Diverge,
}

#[derive(Clone, Debug)]
pub struct DetalheDivergencia {
    pub amostra: usize,
    pub bloco: usize,
    pub mclk_aprox: u64,
    pub camada: &'static str,
    pub hash_ref: String,
    pub hash_nuc: String,
}

#[derive(Clone, Debug)]
pub struct ResultadoCaso {
    pub caso: String,
    pub chip: &'static str,
    pub n_amostras: usize,
    pub n_blocos: usize,
    pub blocos_iguais: usize,
    pub primeiro_erro: Option<DetalheDivergencia>,
    pub veredito: Veredito,
    pub hash_relatorio_caso: String,
}

/// Compara a sintese de FM entre Nuked-OPN2 e nosso Fm.
pub fn comparar_fm(
    caso: &str,
    escritas: &[(u64, u32, u8)], // (mclk, porta, valor)
    mclk_fim: u64,
) -> ResultadoCaso {
    let mut ref_chip = RefOpn2::novo();
    let mut nuc_chip = Fm::novo();

    let total_ciclos = usize::try_from(mclk_fim / 42).unwrap_or(0);
    let mut esc_idx = 0usize;

    let mut buf_ref_bloco = Vec::with_capacity(AMOSTRAS_POR_BLOCO * 96);
    let mut buf_nuc_bloco = Vec::with_capacity(AMOSTRAS_POR_BLOCO * 96);

    let mut n_amostras = 0usize;
    let mut n_blocos = 0usize;
    let mut blocos_iguais = 0usize;
    let mut primeiro_erro = None;

    let mut subciclo = 0usize;

    for c in 0..total_ciclos {
        // Aplica escritas cujo mclk / 42 <= c
        while esc_idx < escritas.len() {
            let (m, p, v) = escritas[esc_idx];
            let c_escrita = usize::try_from(m / 42).unwrap_or(0);
            if c_escrita <= c {
                ref_chip.escreve(p, v);
                nuc_chip.escreve(p, v);
                esc_idx = esc_idx.wrapping_add(1);
            } else {
                break;
            }
        }

        let ref_lr = ref_chip.ciclo();
        let nuc_lr = nuc_chip.ciclo();

        buf_ref_bloco.extend_from_slice(&ref_lr.0.to_le_bytes());
        buf_ref_bloco.extend_from_slice(&ref_lr.1.to_le_bytes());
        buf_nuc_bloco.extend_from_slice(&nuc_lr.0.to_le_bytes());
        buf_nuc_bloco.extend_from_slice(&nuc_lr.1.to_le_bytes());

        subciclo = subciclo.wrapping_add(1);
        if subciclo == 24 {
            subciclo = 0;
            n_amostras = n_amostras.wrapping_add(1);

            if n_amostras % AMOSTRAS_POR_BLOCO == 0 {
                let h_ref = sha1(&buf_ref_bloco);
                let h_nuc = sha1(&buf_nuc_bloco);
                n_blocos = n_blocos.wrapping_add(1);

                if h_ref == h_nuc {
                    blocos_iguais = blocos_iguais.wrapping_add(1);
                } else if primeiro_erro.is_none() {
                    let camada = diagnosticar_camada_fm(&ref_chip, &nuc_chip);
                    primeiro_erro = Some(DetalheDivergencia {
                        amostra: n_amostras.wrapping_sub(AMOSTRAS_POR_BLOCO),
                        bloco: n_blocos.wrapping_sub(1),
                        mclk_aprox: (n_amostras as u64).wrapping_mul(1008),
                        camada,
                        hash_ref: em_hex(&h_ref),
                        hash_nuc: em_hex(&h_nuc),
                    });
                }
                buf_ref_bloco.clear();
                buf_nuc_bloco.clear();
            }
        }
    }

    // Bloco final incompleto, se houver
    if !buf_ref_bloco.is_empty() {
        let h_ref = sha1(&buf_ref_bloco);
        let h_nuc = sha1(&buf_nuc_bloco);
        n_blocos = n_blocos.wrapping_add(1);

        if h_ref == h_nuc {
            blocos_iguais = blocos_iguais.wrapping_add(1);
        } else if primeiro_erro.is_none() {
            let camada = diagnosticar_camada_fm(&ref_chip, &nuc_chip);
            primeiro_erro = Some(DetalheDivergencia {
                amostra: n_amostras,
                bloco: n_blocos.wrapping_sub(1),
                mclk_aprox: (n_amostras as u64).wrapping_mul(1008),
                camada,
                hash_ref: em_hex(&h_ref),
                hash_nuc: em_hex(&h_nuc),
            });
        }
    }

    let veredito = if primeiro_erro.is_none() {
        Veredito::Igual
    } else {
        Veredito::Diverge
    };

    let resumo = format!(
        "{}:{}:{}:{}:{}:{:?}",
        caso, "FM", n_amostras, n_blocos, blocos_iguais, veredito
    );
    let hash_relatorio_caso = em_hex(&sha1(resumo.as_bytes()));

    ResultadoCaso {
        caso: caso.to_string(),
        chip: "FM",
        n_amostras,
        n_blocos,
        blocos_iguais,
        primeiro_erro,
        veredito,
        hash_relatorio_caso,
    }
}

/// Diagnostica qual camada interna do FM difere quando uma amostra diverge.
fn diagnosticar_camada_fm(ref_chip: &RefOpn2, nuc_chip: &Fm) -> &'static str {
    let ref_fase = ref_chip.estado(0);
    let nuc_fase = nuc_chip.diag_fase();
    for i in 0..24 {
        if ref_fase[i] != nuc_fase[i] as i32 {
            return "fase";
        }
    }

    let ref_eg = ref_chip.estado(1);
    let nuc_eg = nuc_chip.diag_eg();
    for i in 0..24 {
        if ref_eg[i] != nuc_eg[i] as i32 {
            return "envelope";
        }
    }

    let ref_fm = ref_chip.estado(3);
    let nuc_fm = nuc_chip.diag_saida_op();
    for i in 0..24 {
        if ref_fm[i] != nuc_fm[i] {
            return "operador";
        }
    }

    let ref_ch = ref_chip.estado(4);
    let nuc_ch = nuc_chip.diag_saida_canal();
    for i in 0..6 {
        if ref_ch[i] != nuc_ch[i] {
            return "canal";
        }
    }

    "pino_multiplexado"
}

/// Compara a sintese de PSG entre Nuked-PSG e nosso Psg.
pub fn comparar_psg(
    caso: &str,
    escritas: &[(u64, u8)], // (mclk, valor)
    mclk_fim: u64,
) -> ResultadoCaso {
    let mut ref_chip = RefPsg::novo();
    let mut nuc_chip = Psg::novo();

    let total_ciclos = usize::try_from(mclk_fim / 15).unwrap_or(0);
    let mut esc_idx = 0usize;

    let mut buf_ref_bloco = Vec::with_capacity(AMOSTRAS_POR_BLOCO * 2);
    let mut buf_nuc_bloco = Vec::with_capacity(AMOSTRAS_POR_BLOCO * 2);

    let mut n_amostras = 0usize;
    let mut n_blocos = 0usize;
    let mut blocos_iguais = 0usize;
    let mut primeiro_erro = None;

    let mut subciclo = 0usize;

    for c in 0..total_ciclos {
        // Aplica escritas cujo mclk / 15 <= c
        while esc_idx < escritas.len() {
            let (m, v) = escritas[esc_idx];
            let c_escrita = usize::try_from(m / 15).unwrap_or(0);
            if c_escrita <= c {
                ref_chip.escreve(v);
                nuc_chip.escreve(v);
                esc_idx = esc_idx.wrapping_add(1);
            } else {
                break;
            }
        }

        ref_chip.ciclo();
        nuc_chip.ciclo();

        subciclo = subciclo.wrapping_add(1);
        if subciclo == 16 {
            subciclo = 0;
            n_amostras = n_amostras.wrapping_add(1);

            let ref_val = ref_chip.le();
            let nuc_val = nuc_chip.saida_empacotada();

            buf_ref_bloco.extend_from_slice(&ref_val.to_le_bytes());
            buf_nuc_bloco.extend_from_slice(&nuc_val.to_le_bytes());

            if n_amostras % AMOSTRAS_POR_BLOCO == 0 {
                let h_ref = sha1(&buf_ref_bloco);
                let h_nuc = sha1(&buf_nuc_bloco);
                n_blocos = n_blocos.wrapping_add(1);

                if h_ref == h_nuc {
                    blocos_iguais = blocos_iguais.wrapping_add(1);
                } else if primeiro_erro.is_none() {
                    primeiro_erro = Some(DetalheDivergencia {
                        amostra: n_amostras.wrapping_sub(AMOSTRAS_POR_BLOCO),
                        bloco: n_blocos.wrapping_sub(1),
                        mclk_aprox: (n_amostras as u64).wrapping_mul(240),
                        camada: "saida_digital_psg",
                        hash_ref: em_hex(&h_ref),
                        hash_nuc: em_hex(&h_nuc),
                    });
                }
                buf_ref_bloco.clear();
                buf_nuc_bloco.clear();
            }
        }
    }

    // Bloco final incompleto
    if !buf_ref_bloco.is_empty() {
        let h_ref = sha1(&buf_ref_bloco);
        let h_nuc = sha1(&buf_nuc_bloco);
        n_blocos = n_blocos.wrapping_add(1);

        if h_ref == h_nuc {
            blocos_iguais = blocos_iguais.wrapping_add(1);
        } else if primeiro_erro.is_none() {
            primeiro_erro = Some(DetalheDivergencia {
                amostra: n_amostras,
                bloco: n_blocos.wrapping_sub(1),
                mclk_aprox: (n_amostras as u64).wrapping_mul(240),
                camada: "saida_digital_psg",
                hash_ref: em_hex(&h_ref),
                hash_nuc: em_hex(&h_nuc),
            });
        }
    }

    let veredito = if primeiro_erro.is_none() {
        Veredito::Igual
    } else {
        Veredito::Diverge
    };

    let resumo = format!(
        "{}:{}:{}:{}:{}:{:?}",
        caso, "PSG", n_amostras, n_blocos, blocos_iguais, veredito
    );
    let hash_relatorio_caso = em_hex(&sha1(resumo.as_bytes()));

    ResultadoCaso {
        caso: caso.to_string(),
        chip: "PSG",
        n_amostras,
        n_blocos,
        blocos_iguais,
        primeiro_erro,
        veredito,
        hash_relatorio_caso,
    }
}
