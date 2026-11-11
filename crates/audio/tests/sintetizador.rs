use sonic1_audio::{
    Chip, EscritaSom, Sintetizador, MCLK_POR_AMOSTRA_FM, MCLK_POR_AMOSTRA_PSG,
    MCLK_POR_QUADRO_NTSC,
};

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

#[test]
fn contagem_de_amostras_por_quadro() {
    let mut sint = Sintetizador::novo();
    let mut fm_amostras = Vec::new();
    let mut psg_amostras = Vec::new();

    let mut total_fm = 0usize;
    let mut total_psg = 0usize;

    for quadro in 1..=60u64 {
        let mclk_alvo = quadro.wrapping_mul(MCLK_POR_QUADRO_NTSC);
        fm_amostras.clear();
        psg_amostras.clear();

        sint.avancar_ate(mclk_alvo, &mut fm_amostras, &mut psg_amostras);

        let n_fm = fm_amostras.len();
        let n_psg = psg_amostras.len();

        assert!(
            n_fm == 888 || n_fm == 889,
            "Quadro {quadro}: contagem de FM inesperada: {n_fm} (esperado 888 ou 889)"
        );
        assert!(
            n_psg == 3733 || n_psg == 3734,
            "Quadro {quadro}: contagem de PSG inesperada: {n_psg} (esperado 3733 ou 3734)"
        );

        total_fm = total_fm.wrapping_add(n_fm);
        total_psg = total_psg.wrapping_add(n_psg);

        let esperado_acum_fm = usize::try_from(mclk_alvo / MCLK_POR_AMOSTRA_FM).unwrap();
        let esperado_acum_psg = usize::try_from(mclk_alvo / MCLK_POR_AMOSTRA_PSG).unwrap();

        assert_eq!(
            total_fm, esperado_acum_fm,
            "Quadro {quadro}: acumulado de FM incorreto ({total_fm} != {esperado_acum_fm})"
        );
        assert_eq!(
            total_psg, esperado_acum_psg,
            "Quadro {quadro}: acumulado de PSG incorreto ({total_psg} != {esperado_acum_psg})"
        );
    }
}

#[test]
fn invariancia_a_fatiacao_do_tempo() {
    let mut rng = Rng(0xCAFE_BABE_1234_5678);

    // Gera um cronograma aleatorio de escritas ao longo de 5 quadros.
    let mclk_maximo = 5u64.wrapping_mul(MCLK_POR_QUADRO_NTSC);
    let mut escritas = Vec::new();
    let mut mclk_cur = 0u64;

    while mclk_cur < mclk_maximo {
        mclk_cur = mclk_cur.wrapping_add(rng.ate(5000).wrapping_add(100));
        if mclk_cur >= mclk_maximo {
            break;
        }
        let chip = if rng.ate(2) == 0 {
            Chip::Fm
        } else {
            Chip::Psg
        };
        let porta = u32::try_from(rng.ate(4)).unwrap();
        let valor = u8::try_from(rng.ate(256)).unwrap();
        escritas.push(EscritaSom {
            mclk: mclk_cur,
            chip,
            porta,
            valor,
        });
    }

    // Execucao 1: avancar em uma unica chamada grande.
    let mut sint1 = Sintetizador::novo();
    sint1.agendar_lote(&escritas);
    let mut fm1 = Vec::new();
    let mut psg1 = Vec::new();
    sint1.avancar_ate(mclk_maximo, &mut fm1, &mut psg1);

    // Execucao 2: avancar em passos fixos curtos (1000 mclk).
    let mut sint2 = Sintetizador::novo();
    sint2.agendar_lote(&escritas);
    let mut fm2 = Vec::new();
    let mut psg2 = Vec::new();
    let mut t = 0u64;
    while t < mclk_maximo {
        t = (t.wrapping_add(1000)).min(mclk_maximo);
        sint2.avancar_ate(t, &mut fm2, &mut psg2);
    }

    // Execucao 3: avancar em fatias aleatorias com escritas agendadas dinamicamente.
    let mut sint3 = Sintetizador::novo();
    let mut fm3 = Vec::new();
    let mut psg3 = Vec::new();
    let mut t3 = 0u64;
    let mut esc_idx = 0usize;

    while t3 < mclk_maximo {
        let passo = rng.ate(4000).wrapping_add(1);
        t3 = (t3.wrapping_add(passo)).min(mclk_maximo);

        // Adiciona escritas que caem nesta fatia ou no futuro proximo
        while esc_idx < escritas.len() && escritas[esc_idx].mclk <= t3.wrapping_add(2000) {
            let e = &escritas[esc_idx];
            sint3.agendar(e.mclk, e.chip, e.porta, e.valor);
            esc_idx = esc_idx.wrapping_add(1);
        }

        sint3.avancar_ate(t3, &mut fm3, &mut psg3);
    }

    // Verificacao: todas as execucoes devem produzir exatamente as mesmas amostras!
    assert_eq!(
        fm1.len(),
        fm2.len(),
        "Contagens de FM diferem entre passo grande e passos fixos"
    );
    assert_eq!(
        fm1.len(),
        fm3.len(),
        "Contagens de FM diferem entre passo grande e passos aleatorios"
    );
    assert_eq!(
        psg1.len(),
        psg2.len(),
        "Contagens de PSG diferem entre passo grande e passos fixos"
    );
    assert_eq!(
        psg1.len(),
        psg3.len(),
        "Contagens de PSG diferem entre passo grande e passos aleatorios"
    );

    for (i, (a, b)) in fm1.iter().zip(fm2.iter()).enumerate() {
        assert_eq!(
            a, b,
            "Amostra de FM {i} diverge entre passo grande e passos fixos"
        );
    }
    for (i, (a, b)) in fm1.iter().zip(fm3.iter()).enumerate() {
        assert_eq!(
            a, b,
            "Amostra de FM {i} diverge entre passo grande e passos aleatorios"
        );
    }

    for (i, (a, b)) in psg1.iter().zip(psg2.iter()).enumerate() {
        assert_eq!(
            a, b,
            "Amostra de PSG {i} diverge entre passo grande e passos fixos"
        );
    }
    for (i, (a, b)) in psg1.iter().zip(psg3.iter()).enumerate() {
        assert_eq!(
            a, b,
            "Amostra de PSG {i} diverge entre passo grande e passos aleatorios"
        );
    }
}

#[test]
fn determinismo_e_reset() {
    let mut sint = Sintetizador::novo();
    let escritas = [
        EscritaSom {
            mclk: 100,
            chip: Chip::Fm,
            porta: 0,
            valor: 0x22,
        },
        EscritaSom {
            mclk: 150,
            chip: Chip::Fm,
            porta: 1,
            valor: 0x00,
        },
        EscritaSom {
            mclk: 200,
            chip: Chip::Psg,
            porta: 0,
            valor: 0x80,
        },
        EscritaSom {
            mclk: 250,
            chip: Chip::Psg,
            porta: 0,
            valor: 0x01,
        },
    ];

    sint.agendar_lote(&escritas);
    let mut fm_a = Vec::new();
    let mut psg_a = Vec::new();
    sint.avancar_ate(100_000, &mut fm_a, &mut psg_a);

    sint.reset();
    assert_eq!(sint.mclk(), 0);

    sint.agendar_lote(&escritas);
    let mut fm_b = Vec::new();
    let mut psg_b = Vec::new();
    sint.avancar_ate(100_000, &mut fm_b, &mut psg_b);

    assert_eq!(fm_a, fm_b);
    assert_eq!(psg_a, psg_b);
}

#[test]
fn escritas_fora_de_ordem_sao_ordenadas_pelo_mclk() {
    let mut sint1 = Sintetizador::novo();
    let mut sint2 = Sintetizador::novo();

    // sint1 recebe em ordem cronologica
    sint1.agendar(1000, Chip::Fm, 0, 0x28);
    sint1.agendar(2000, Chip::Fm, 1, 0xF0);

    // sint2 recebe fora de ordem
    sint2.agendar(2000, Chip::Fm, 1, 0xF0);
    sint2.agendar(1000, Chip::Fm, 0, 0x28);

    let mut fm1 = Vec::new();
    let mut psg1 = Vec::new();
    sint1.avancar_ate(10_000, &mut fm1, &mut psg1);

    let mut fm2 = Vec::new();
    let mut psg2 = Vec::new();
    sint2.avancar_ate(10_000, &mut fm2, &mut psg2);

    assert_eq!(fm1, fm2);
    assert_eq!(psg1, psg2);
}
