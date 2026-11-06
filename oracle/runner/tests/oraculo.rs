//! Testes do oraculo com ROMs sinteticas escritas por nos. Nenhum teste usa a ROM real.

use sonic1_fidelidade::{Categoria, Divergencia, Evento, Leitor, Onde, Tipo, comparar_leitores};
use websonic_oracle::oraculo::ChamadaRotina;
use websonic_oracle::rom_sintetica::{
    RomSintetica, SSP, programa_entrada, programa_vblank, programa_z80,
};
use websonic_oracle::{Entradas, MotivoParada, Oraculo, Parada};

/// Roda `n` quadros e devolve o trace serializado e seu hash.
fn rodar(rom: Vec<u8>, entradas: Entradas, n: u32) -> (Vec<u8>, [u8; 20]) {
    let mut oraculo = Oraculo::novo(rom, entradas).unwrap();
    let cabecalho = oraculo.cabecalho().clone();
    let mut bytes = Vec::new();
    let mut escritor = sonic1_fidelidade::Escritor::novo(&mut bytes, &cabecalho).unwrap();
    for _ in 0..n {
        escritor
            .escrever(&oraculo.executar_quadro().unwrap())
            .unwrap();
    }
    let hash = escritor.finalizar(&cabecalho).unwrap();
    (bytes, hash)
}

fn todos_os_eventos(rom: Vec<u8>, n: u32) -> Vec<Evento> {
    let mut oraculo = Oraculo::novo(rom, Entradas::default()).unwrap();
    let mut eventos = Vec::new();
    for _ in 0..n {
        eventos.extend(oraculo.executar_quadro().unwrap().eventos);
    }
    eventos
}

#[test]
fn entrada_injetada_chega_a_ram() {
    // Quadros 0 a 2 sem botao; do quadro 3 em diante, Cima apertado.
    let entradas = Entradas::nova(vec![0, 0, 0, 0x01, 0x01, 0x01]);
    let mut oraculo = Oraculo::novo(programa_entrada(), entradas).unwrap();
    for _ in 0..3 {
        oraculo.executar_quadro().unwrap();
    }
    // Botao de Cima solto: o bit 0 le 1 (nivel alto).
    assert_eq!(oraculo.ler_byte_ram68k(0xFF_0000) & 0x01, 0x01);
    for _ in 0..2 {
        oraculo.executar_quadro().unwrap();
    }
    // Botao de Cima apertado: o bit 0 le 0.
    assert_eq!(oraculo.ler_byte_ram68k(0xFF_0000) & 0x01, 0x00);
}

#[test]
fn duas_execucoes_geram_o_mesmo_trace() {
    for rom in [programa_entrada(), programa_vblank(), programa_z80()] {
        let entradas = Entradas::nova(vec![0, 0x01, 0x80, 0]);
        let (a, ha) = rodar(rom.clone(), entradas.clone(), 6);
        let (b, hb) = rodar(rom, entradas, 6);
        assert_eq!(a, b, "os bytes do trace diferem entre duas execucoes");
        assert_eq!(ha, hb);
    }
}

#[test]
fn comparador_acusa_entrada_diferente_no_quadro_certo() {
    let a = rodar(programa_entrada(), Entradas::nova(vec![0; 8]), 6).0;
    let b = rodar(
        programa_entrada(),
        Entradas::nova(vec![0, 0, 0, 0x01, 0, 0, 0, 0]),
        6,
    )
    .0;
    let mut la = Leitor::abrir(a.as_slice()).unwrap();
    let mut lb = Leitor::abrir(b.as_slice()).unwrap();
    // Os cabecalhos diferem (hash das entradas), entao o comparador para ali.
    let d = comparar_leitores(&mut la, &mut lb).unwrap().unwrap();
    assert_eq!(
        d,
        Divergencia {
            quadro: 0,
            onde: Onde::Cabecalho
        }
    );
    // Com o cabecalho igualado, a primeira divergencia e a RAM do 68000 no quadro 3.
    let mut la = Leitor::abrir(a.as_slice()).unwrap();
    let mut lb = Leitor::abrir(b.as_slice()).unwrap();
    lb.cabecalho = la.cabecalho.clone();
    let d = comparar_leitores(&mut la, &mut lb).unwrap().unwrap();
    assert_eq!(
        d,
        Divergencia {
            quadro: 3,
            onde: Onde::Categoria {
                categoria: Categoria::Ram68k,
                evento: None
            }
        }
    );
}

#[test]
fn estado_salvo_e_recarregado_gera_o_mesmo_trace() {
    let entradas = Entradas::nova(vec![0, 0, 0x01, 0x01, 0, 0, 0, 0]);
    let mut a = Oraculo::novo(programa_vblank(), entradas.clone()).unwrap();
    for _ in 0..3 {
        a.executar_quadro().unwrap();
    }
    let estado = a.salvar_estado().unwrap();
    let continuar =
        |o: &mut Oraculo| -> Vec<_> { (0..3).map(|_| o.executar_quadro().unwrap()).collect() };
    let originais = continuar(&mut a);

    // No mesmo oraculo: volta ao estado salvo e roda de novo.
    a.carregar_estado(&estado).unwrap();
    assert_eq!(continuar(&mut a), originais);

    // Em outro oraculo, criado do zero com a mesma ROM.
    let mut b = Oraculo::novo(programa_vblank(), entradas).unwrap();
    b.carregar_estado(&estado).unwrap();
    assert_eq!(b.quadro_atual(), 3);
    assert_eq!(continuar(&mut b), originais);
}

#[test]
fn estado_salvo_no_meio_do_quadro_tambem_volta() {
    let mut a = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    a.executar_quadro().unwrap();
    let mut fechados = Vec::new();
    let motivo = a
        .executar_ate(&[Parada::Pc68(0x400)], 5_000_000, &mut |q| fechados.push(q))
        .unwrap();
    assert_eq!(motivo, MotivoParada::Parada(Parada::Pc68(0x400)));
    let estado = a.salvar_estado().unwrap();
    let originais: Vec<_> = (0..2).map(|_| a.executar_quadro().unwrap()).collect();
    a.carregar_estado(&estado).unwrap();
    let repetidos: Vec<_> = (0..2).map(|_| a.executar_quadro().unwrap()).collect();
    assert_eq!(originais, repetidos);
}

#[test]
fn para_em_endereco_antes_de_executar_a_instrucao() {
    let mut o = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    let motivo = o
        .executar_ate(&[Parada::Pc68(0x210)], 1000, &mut |_| ())
        .unwrap();
    assert_eq!(motivo, MotivoParada::Parada(Parada::Pc68(0x210)));
    assert_eq!(o.emulador().oracle_pc68(), 0x210);
    // O contador de interrupcoes ainda esta em zero: nenhuma interrupcao chegou.
    assert_eq!(o.ler_palavra_ram68k(0xFF_0102), 0);
}

#[test]
fn para_no_inicio_de_um_quadro_e_em_um_relogio() {
    let mut o = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    let motivo = o
        .executar_ate(&[Parada::InicioDoQuadro(2)], 5_000_000, &mut |_| ())
        .unwrap();
    assert_eq!(motivo, MotivoParada::Parada(Parada::InicioDoQuadro(2)));
    assert_eq!(o.quadro_atual(), 2);
    let alvo = o.emulador().oracle_mclk() + 10_000;
    let motivo = o
        .executar_ate(&[Parada::Mclk(alvo)], 5_000_000, &mut |_| ())
        .unwrap();
    assert_eq!(motivo, MotivoParada::Parada(Parada::Mclk(alvo)));
    assert!(o.emulador().oracle_mclk() >= alvo);
}

#[test]
fn interrupcao_de_vblank_e_registrada_com_endereco_e_ciclo() {
    let eventos = todos_os_eventos(programa_vblank(), 4);
    let ints: Vec<_> = eventos.iter().filter(|e| e.tipo == Tipo::Int68).collect();
    assert!(
        ints.len() >= 3,
        "esperava uma interrupcao por quadro, achei {}",
        ints.len()
    );
    for e in &ints {
        assert_eq!(e.a, 6, "nivel");
        assert_eq!(e.b, 0x210, "o laco bra.s * e o ponto interrompido");
        assert_eq!(e.c, 0x400, "tratador");
        assert!(e.mclk > 0);
    }
    // Interrupcoes em instantes diferentes, em ordem.
    assert!(ints.windows(2).all(|w| w[0].mclk < w[1].mclk));
    // O contador do tratador conta as mesmas interrupcoes.
    let mut o = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    let mut contadas = 0usize;
    for _ in 0..4 {
        contadas += o
            .executar_quadro()
            .unwrap()
            .eventos
            .iter()
            .filter(|e| e.tipo == Tipo::Int68)
            .count();
    }
    let contador = u32::from(o.ler_palavra_ram68k(0xFF_0100)) << 16
        | u32::from(o.ler_palavra_ram68k(0xFF_0102));
    assert_eq!(contador as usize, contadas);
}

#[test]
fn desvios_do_68000_tomados_e_nao_tomados() {
    let eventos = todos_os_eventos(programa_vblank(), 2);
    let desvios: Vec<_> = eventos
        .iter()
        .filter(|e| e.tipo == Tipo::Desvio68)
        .collect();
    // bra.s * : de $210 para $210, classe BRA (1), tomado.
    assert!(
        desvios
            .iter()
            .any(|e| e.a == 0x210 && e.b == 0x210 && e.c == 1 && e.d == 1)
    );
    // rte do tratador: a partir de $406, classe RTE (9).
    assert!(
        desvios
            .iter()
            .any(|e| e.a == 0x406 && e.c == 9 && e.d == 1 && e.b == 0x210)
    );
    // Para o nao tomado, uso a rotina isolada abaixo (beq com Z = 0).
}

#[test]
fn escrita_na_vdp_e_registrada_com_a_linha() {
    let mut o = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    let q = o.executar_quadro().unwrap();
    let vdp: Vec<_> = q.eventos.iter().filter(|e| e.tipo == Tipo::Vdp).collect();
    assert_eq!(vdp.len(), 1);
    let e = vdp[0];
    assert_eq!(
        (e.a, e.b, e.c),
        (1, 0x8164, 2),
        "porta de controle, valor, tamanho"
    );
    assert_eq!(e.origem, sonic1_fidelidade::Origem::Cpu68);
    assert!(e.linha < 300 && e.coluna < 3500);
}

#[test]
fn z80_barramento_ym_dac_e_esperas() {
    let eventos = todos_os_eventos(programa_z80(), 6);
    let quantos = |t: Tipo| eventos.iter().filter(|e| e.tipo == t).count();

    // O 68000 pede o barramento, espera o BUSACK e o devolve.
    assert!(
        eventos
            .iter()
            .any(|e| e.tipo == Tipo::BarramentoZ80 && e.a == 0 && e.b == 1)
    );
    assert!(
        eventos
            .iter()
            .any(|e| e.tipo == Tipo::BarramentoZ80 && e.a == 0 && e.b == 0)
    );
    assert!(quantos(Tipo::LeituraBusAck) >= 1);

    // Cada acesso do 68000 ao Z80 gera espera de 1 ciclo (causa 1).
    let esperas_68_z80 = eventos
        .iter()
        .filter(|e| e.tipo == Tipo::EsperaAcumulada && e.a == 1)
        .count();
    assert!(esperas_68_z80 >= 18 + 2, "achei {esperas_68_z80}");
    assert!(quantos(Tipo::EsperaConsumida) >= 1);

    // YM: duas escritas do 68000 e quatro do Z80.
    let ym_68k: Vec<_> = eventos
        .iter()
        .filter(|e| e.tipo == Tipo::Ym && e.origem == sonic1_fidelidade::Origem::Cpu68)
        .collect();
    assert_eq!(ym_68k.len(), 2);
    let ym_z80: Vec<_> = eventos
        .iter()
        .filter(|e| e.tipo == Tipo::Ym && e.origem == sonic1_fidelidade::Origem::Z80)
        .collect();
    assert_eq!(
        ym_z80.iter().map(|e| (e.a, e.b)).collect::<Vec<_>>(),
        vec![(0, 0x22), (1, 0x0F), (0, 0x2A), (1, 0x80)]
    );
    // A escrita de dado seguinte a $2A e o DAC.
    assert_eq!((ym_z80[1].c, ym_z80[1].d), (0x22, 0));
    assert_eq!((ym_z80[3].c, ym_z80[3].d), (0x2A, 1));

    // Desvio do Z80: o jr $ final (classe JR = 1), de $0010 para $0010.
    assert!(eventos.iter().any(|e| e.tipo == Tipo::Desvio80
        && e.a == 0x10
        && e.b == 0x10
        && e.c == 1
        && e.d == 1));
}

#[test]
fn rotina_isolada_a_partir_de_um_estado() {
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(0x200, &[0x60FE]); // bra.s *
    rom.palavras(
        0x500,
        &[
            0x7005, // moveq #5,d0
            0xD041, // add.w d1,d0
            0x6700, 0x0004, // beq.w +4   (nao tomado: Z = 0, pois o resultado nao e zero)
            0x23C0, 0x00FF, 0x0200, // move.l d0,$FF0200
            0x4E75, // rts
        ],
    );
    let rom = rom.finalizar();

    // Estado de partida: um estado salvo, carregado num oraculo novo.
    let mut origem = Oraculo::novo(rom.clone(), Entradas::default()).unwrap();
    origem.executar_quadro().unwrap();
    let estado = origem.salvar_estado().unwrap();
    let mut o = Oraculo::novo(rom, Entradas::default()).unwrap();
    o.carregar_estado(&estado).unwrap();

    let mut regs = o.emulador().oracle_regs68();
    regs.d = [0; 8];
    regs.d[1] = 0x10;
    regs.pc = 0x500;
    regs.sr = 0x2000;
    let ssp_antes = regs.ssp;
    let chamada = ChamadaRotina {
        regs,
        pokes_ram68: vec![(0xFF_0300, 0xAB)],
        sentinela: 0x600,
        mascarar_interrupcoes: true,
        max_passos: 1000,
    };
    let r = o.chamar_rotina(&chamada).unwrap();
    assert_eq!(r.regs.d[0], 0x15);
    assert_eq!(r.regs.pc, 0x600);
    assert_eq!(
        r.regs.ssp, ssp_antes,
        "o rts desempilhou o endereco de retorno"
    );
    assert!(r.mudancas_ram68.contains(&(0xFF_0203, 0, 0x15)));
    // Os pokes fazem parte do estado de partida: nao contam como mudanca da rotina.
    assert_eq!(o.ler_byte_ram68k(0xFF_0300), 0xAB);
    assert!(r.mudancas_ram68.iter().all(|m| m.0 != 0xFF_0300));
    assert!(r.mclk_gasto > 0 && r.passos >= 5);
    // O beq.w nao tomado aparece como desvio nao tomado (classe BCC = 2).
    assert!(
        r.eventos
            .iter()
            .any(|e| e.tipo == Tipo::Desvio68 && e.c == 2 && e.d == 0 && e.a == 0x504)
    );
    // O rts e um desvio tomado de $50C para a sentinela (classe RTS = 7).
    assert!(
        r.eventos
            .iter()
            .any(|e| e.tipo == Tipo::Desvio68 && e.c == 7 && e.b == 0x600)
    );

    // Duas chamadas iguais, a partir do mesmo estado, dao o mesmo resultado.
    o.carregar_estado(&estado).unwrap();
    let r2 = o.chamar_rotina(&chamada).unwrap();
    assert_eq!(r2.regs, r.regs);
    assert_eq!(r2.mudancas_ram68, r.mudancas_ram68);
    assert_eq!(r2.mclk_gasto, r.mclk_gasto);
    assert_eq!(r2.eventos.len(), r.eventos.len());
}

#[test]
fn para_em_rotulo_resolvido_pelo_mapa() {
    use sonic1_fidelidade::rotulos::Rotulos;
    let rotulos =
        Rotulos::analisar("tratador_vblank\trotina\t400\nlaco_principal\trotina\t210\n").unwrap();
    let mut o = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    let alvo = rotulos.resolver("tratador_vblank").unwrap();
    let motivo = o
        .executar_ate(&[Parada::Pc68(alvo)], 5_000_000, &mut |_| ())
        .unwrap();
    assert_eq!(motivo, MotivoParada::Parada(Parada::Pc68(0x400)));
    assert_eq!(o.emulador().oracle_pc68(), 0x400);
}
