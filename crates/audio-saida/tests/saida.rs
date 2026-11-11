use sonic1_audio::{AmostraFm, AmostraPsg, MCLK_POR_QUADRO_NTSC, Sintetizador};
use sonic1_audio_saida::{FiltroAnalogico, MisturadorSaida};

#[test]
fn reamostragem_sem_cortes_600_quadros() {
    let mut sint = Sintetizador::novo();
    let mut misturador = MisturadorSaida::novo(48_000.0);

    let mut fm_buf = Vec::new();
    let mut psg_buf = Vec::new();

    let mut total_fm_in = 0usize;
    let mut total_psg_in = 0usize;

    let mut worklet_buf = [0.0f32; 256]; // 128 pares estereo por chamada do AudioWorklet

    // Pre-carrega 3 quadros de buffer (~50 ms), conforme o requisito da sub-etapa 8.7
    for q in 1..=3u64 {
        fm_buf.clear();
        psg_buf.clear();
        sint.avancar_ate(q * MCLK_POR_QUADRO_NTSC, &mut fm_buf, &mut psg_buf);
        total_fm_in += fm_buf.len();
        total_psg_in += psg_buf.len();
        misturador.alimentar_quadro(&fm_buf, &psg_buf);
    }

    // Simula 600 quadros (~10 segundos a 60 Hz)
    for q in 4..=600u64 {
        let mclk_alvo = q * MCLK_POR_QUADRO_NTSC;
        fm_buf.clear();
        psg_buf.clear();

        sint.avancar_ate(mclk_alvo, &mut fm_buf, &mut psg_buf);

        let n_fm = fm_buf.len();
        let n_psg = psg_buf.len();

        // (1) Amostras de entrada por quadro estritamente 888 ou 889 (FM) e 3733 ou 3734 (PSG)
        assert!(n_fm == 888 || n_fm == 889, "Quadro {q}: FM {n_fm}");
        assert!(n_psg == 3733 || n_psg == 3734, "Quadro {q}: PSG {n_psg}");

        total_fm_in += n_fm;
        total_psg_in += n_psg;

        // Acumulado fracionario exato
        assert_eq!(total_fm_in, (mclk_alvo / 1008) as usize);
        assert_eq!(total_psg_in, (mclk_alvo / 240) as usize);

        // Alimenta o misturador
        misturador.alimentar_quadro(&fm_buf, &psg_buf);

        // Simula consumo do AudioWorklet: a cada quadro de 16.67ms sao ~800 amostras (6 a 7 blocos de 128)
        for _ in 0..6 {
            misturador.extrair_bloco_worklet(&mut worklet_buf);
        }
        if q % 4 == 0 {
            misturador.extrair_bloco_worklet(&mut worklet_buf);
        }

        // Buffer nunca deve esvaziar
        assert!(
            misturador.ocupacao_buffer() > 0,
            "Quadro {q}: buffer de audio esvaziou!"
        );
    }

    assert_eq!(
        misturador.falhas(),
        0,
        "Nao deve ocorrer nenhuma falha (underrun) de buffer"
    );
}

#[test]
fn simulacao_resiliencia_jitter_longo() {
    let mut sint = Sintetizador::novo();
    let mut misturador = MisturadorSaida::novo(48_000.0);

    let mut fm_buf = Vec::new();
    let mut psg_buf = Vec::new();
    let mut worklet_buf = [0.0f32; 256];

    // Simula 600 quadros (~10 segundos) com jitter variavel no consumo
    let mut rng_seed = 0x1234_5678u64;
    let mut rng = move || {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng_seed
    };

    // Pre-carrega o buffer com 3 quadros iniciais (comportamento padrao do web)
    for q in 1..=3u64 {
        fm_buf.clear();
        psg_buf.clear();
        sint.avancar_ate(q * MCLK_POR_QUADRO_NTSC, &mut fm_buf, &mut psg_buf);
        misturador.alimentar_quadro(&fm_buf, &psg_buf);
    }

    for q in 4..=600u64 {
        fm_buf.clear();
        psg_buf.clear();
        sint.avancar_ate(q * MCLK_POR_QUADRO_NTSC, &mut fm_buf, &mut psg_buf);
        misturador.alimentar_quadro(&fm_buf, &psg_buf);

        // Consumo do AudioWorklet com jitter de entrega de ate 3 ms (±1 bloco de 128)
        let delta_blocos = (rng() % 3) as i32 - 1; // -1, 0, +1
        let blocos_consumo = (6 + delta_blocos).max(4) as usize;

        for _ in 0..blocos_consumo {
            misturador.extrair_bloco_worklet(&mut worklet_buf);
        }

        assert!(
            misturador.ocupacao_buffer() > 0,
            "Quadro {q}: buffer esvaziou sob jitter!"
        );
    }

    assert_eq!(misturador.falhas(), 0, "Zero falhas sob jitter de 3ms");
}

#[test]
fn continuidade_de_seno_na_juncao_de_quadros() {
    let mut misturador = MisturadorSaida::novo(48_000.0);

    // Gera uma senoide pura de 440 Hz na taxa do FM (~53 267 Hz)
    let taxa_fm = 53_693_175.0 / 1008.0;
    let mut fase = 0.0f32;
    let passo_fase = 2.0 * std::f32::consts::PI * 440.0 / (taxa_fm as f32);

    let mut saida_amostras = Vec::new();
    let mut worklet_buf = [0.0f32; 256];

    for _ in 0..10 {
        let mut fm_lote = Vec::new();
        for _ in 0..889 {
            let s = (fase.sin() * 1000.0) as i16;
            fm_lote.push(AmostraFm {
                misturado: (s, s),
                picos_pino: (s, s),
                ciclos: [(s, s); 24],
            });
            fase += passo_fase;
        }
        let psg_vazio: [AmostraPsg; 0] = [];
        misturador.alimentar_quadro(&fm_lote, &psg_vazio);

        while misturador.ocupacao_buffer() >= 128 {
            misturador.extrair_bloco_worklet(&mut worklet_buf);
            for i in 0..128 {
                saida_amostras.push(worklet_buf[i * 2]);
            }
        }
    }

    // Verifica que nao ha nenhum degrau/descontinuidade brusca entre amostras consecutivas
    // Para 440 Hz a 48 kHz, a inclinacao maxima de sin(w*t) e w = 2*pi*440/48000 ~ 0.058
    for i in 1..saida_amostras.len() {
        let delta = (saida_amostras[i] - saida_amostras[i - 1]).abs();
        assert!(
            delta < 0.15,
            "Descontinuidade brusca detectada na amostra {i}: delta={delta}"
        );
    }
}

#[test]
fn filtro_analogico_aproximado_resposta_frequencia() {
    let filtro = FiltroAnalogico::novo(48_000.0);
    let resp = filtro.medir_resposta_db();

    // 100 Hz deve ter perda quase zero (< 0.2 dB)
    assert!(
        resp[0].1 > -0.2,
        "100 Hz deve passar limpo: {} dB",
        resp[0].1
    );

    // Na frequencia de corte (~3386 Hz), a atenuacao deve ser ~ -3 dB
    let corte_resp = resp.iter().find(|&&(f, _)| f == 3386).unwrap().1;
    assert!(
        (corte_resp - (-3.01)).abs() < 0.2,
        "No corte deve estar a -3 dB: {corte_resp} dB"
    );

    // Em 10 kHz, deve estar atenuado significativamente (< -9 dB)
    let r10k = resp.iter().find(|&&(f, _)| f == 10000).unwrap().1;
    assert!(r10k < -9.0, "10 kHz deve estar atenuado: {r10k} dB");
}
