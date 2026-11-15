use sonic1_audio::{MCLK_POR_QUADRO_NTSC, Sintetizador};
use sonic1_audio_saida::MisturadorSaida;

#[test]
fn fila_do_dispositivo_permanece_estavel_no_ritmo_ntsc() {
    for taxa in [44_100.0, 48_000.0] {
        let mut sintetizador = Sintetizador::novo();
        let mut misturador = MisturadorSaida::novo(taxa);
        let mut fm = Vec::new();
        let mut psg = Vec::new();
        let mut fila = 0usize;
        let mut iniciada = false;
        let mut consumo_fracionario = 0.0;
        let por_quadro = taxa * MCLK_POR_QUADRO_NTSC as f64 / 53_693_175.0;
        let alvo = (por_quadro * 3.0).ceil() as usize;
        for quadro in 1..=600 {
            if quadro % 5 == 1 {
                misturador.definir_ocupacao_externa(fila);
            }
            fm.clear();
            psg.clear();
            sintetizador.avancar_ate(quadro * MCLK_POR_QUADRO_NTSC, &mut fm, &mut psg);
            misturador.alimentar_quadro(&fm, &psg);
            let produzidos = misturador.ocupacao_buffer();
            let mut saida = vec![0.0; produzidos * 2];
            misturador.extrair_bloco_worklet(&mut saida);
            fila += produzidos;
            iniciada |= fila >= alvo;
            if iniciada {
                consumo_fracionario += por_quadro;
                let consumo = consumo_fracionario.floor() as usize;
                consumo_fracionario -= consumo as f64;
                assert!(fila >= consumo, "falta no quadro {quadro}, taxa {taxa}");
                fila -= consumo;
            }
            assert!(fila < 16_384, "fila cheia no quadro {quadro}, taxa {taxa}");
        }
        assert!(iniciada);
    }
}
