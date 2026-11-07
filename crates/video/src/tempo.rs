//! Conversões de tempo da VDP em 40 colunas (NTSC) e agenda de eventos de cada linha.
//!
//! Nomes e valores seguem o oráculo (jgenesis `386db855`, `vdp.rs`: `scanline_mclk_to_pixel_h40`,
//! `pixel_to_internal_h_h40`, `vdp_event_times`). O relógio é o mestre: 3420 ciclos por linha.

/// Ciclos do relógio mestre em uma linha.
pub const MCLK_POR_LINHA: u64 = 3420;
/// Linhas por quadro (NTSC, sem entrelaçamento).
pub const LINHAS_POR_QUADRO: u16 = 262;
/// Pontos por linha contando o HBlank, em 40 colunas.
pub const PONTOS_POR_LINHA: u16 = 420;
/// Primeiro ponto da área ativa (H interno) em 40 colunas.
pub const INICIO_ATIVO: u16 = 0x1a;
/// Fim da área ativa (H interno) em 40 colunas.
pub const FIM_ATIVO: u16 = 0x15a;
/// Linha em que o oráculo fecha o quadro (224 linhas ativas mais a borda inferior).
pub const LINHA_DO_FIM_DO_QUADRO: u16 = 232;

const SALTO: u64 = 0x1c9 - 0x16d;

/// Ponto de vídeo em que está um instante da linha, dado em ciclos mestres desde o começo dela.
pub fn mclk_para_ponto(mclk: u64) -> u16 {
    let a = 0x1cc_u64.saturating_sub(SALTO);
    // O relógio de ponto é mclk/8 até H = 0x1CB.
    if mclk < a.saturating_mul(8) {
        return u16::try_from(mclk / 8).unwrap_or(u16::MAX);
    }
    // De H = 0x1CC a 0x1ED o padrão se repete duas vezes: 1 de mclk/8, 7 de mclk/10,
    // 2 de mclk/9, 7 de mclk/10.
    let inicio_hsync = a.saturating_mul(8);
    let ciclo: u64 = 166;
    let fim_hsync = inicio_hsync.saturating_add(ciclo.saturating_mul(2));
    if mclk >= inicio_hsync && mclk < fim_hsync {
        let h = mclk.saturating_sub(inicio_hsync);
        let resto = h % ciclo;
        let no_padrao = match resto {
            0..=7 => 0,
            8..=77 => 1u64.saturating_add(resto.saturating_sub(8) / 10),
            78..=95 => 8u64.saturating_add(resto.saturating_sub(78) / 9),
            _ => 10u64.saturating_add(resto.saturating_sub(96) / 10),
        };
        let base = if h < ciclo { a } else { a.saturating_add(17) };
        return u16::try_from(base.saturating_add(no_padrao)).unwrap_or(u16::MAX);
    }
    // De H = 0x1EE a 0x1FF volta a mclk/8.
    let depois = mclk.saturating_sub(fim_hsync);
    u16::try_from(a.saturating_add(34).saturating_add(depois / 8)).unwrap_or(u16::MAX)
}

/// H interno de um ponto de vídeo.
pub fn ponto_para_h(ponto: u16) -> u16 {
    if ponto <= 0x16c {
        ponto
    } else {
        ponto.saturating_add(0x1c9 - 0x16d)
    }
}

/// Eventos de cada linha, na ordem em que a agenda os dispara.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evento {
    InterrupcaoDeVBlank,
    LerRolagemH,
    RenderizarLinha,
    BuscarAtributosDeSprites,
    InicioDoHBlank,
    InterrupcaoDeLinha,
    TravarRegistradores,
}

/// Evento com o H interno em que dispara.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventoNoTempo {
    pub h: u16,
    pub evento: Evento,
}

/// Agenda de 40 colunas, já ordenada (a ordenação é estável, como no oráculo).
pub const AGENDA: [EventoNoTempo; 7] = [
    EventoNoTempo {
        h: 0x002,
        evento: Evento::InterrupcaoDeVBlank,
    },
    EventoNoTempo {
        h: INICIO_ATIVO - 24,
        evento: Evento::LerRolagemH,
    },
    EventoNoTempo {
        h: INICIO_ATIVO,
        evento: Evento::RenderizarLinha,
    },
    EventoNoTempo {
        h: FIM_ATIVO - 16 - 8,
        evento: Evento::BuscarAtributosDeSprites,
    },
    EventoNoTempo {
        h: FIM_ATIVO - 16,
        evento: Evento::InicioDoHBlank,
    },
    EventoNoTempo {
        h: 0x14a,
        evento: Evento::InterrupcaoDeLinha,
    },
    EventoNoTempo {
        h: 0x169,
        evento: Evento::TravarRegistradores,
    },
];

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ate_o_hsync_o_ponto_e_mclk_sobre_8() {
        assert_eq!(mclk_para_ponto(0), 0);
        assert_eq!(mclk_para_ponto(7), 0);
        assert_eq!(mclk_para_ponto(8), 1);
        assert_eq!(mclk_para_ponto(26 * 8), 26);
        assert_eq!(mclk_para_ponto(2943), 367);
    }

    #[test]
    fn dentro_do_hsync_o_ponto_cresce_sem_voltar() {
        let mut anterior = 0;
        for m in 0..MCLK_POR_LINHA {
            let p = mclk_para_ponto(m);
            assert!(p >= anterior, "mclk {m}");
            anterior = p;
        }
        // O fim da linha cai perto de 420 pontos (as bordas somadas).
        assert!((410..=430).contains(&anterior), "{anterior}");
    }

    #[test]
    fn h_interno_pula_depois_de_0x16c() {
        assert_eq!(ponto_para_h(0x16c), 0x16c);
        assert_eq!(ponto_para_h(0x16d), 0x1c9);
    }

    #[test]
    fn agenda_esta_ordenada() {
        for par in AGENDA.windows(2) {
            assert!(par[0].h <= par[1].h);
        }
        assert_eq!(AGENDA[2].evento, Evento::RenderizarLinha);
    }
}
