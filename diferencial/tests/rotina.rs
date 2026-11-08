//! O harness de teste por rotina contra uma rotina sintética de 68000 escrita por nós.
//! Prova que ele acerta quando o núcleo está certo e acusa os erros plantados.

use genesis_core::api::oracle::Regs68;
use sonic1_core::decisao;
use sonic1_core::memoria::{Memoria, Tamanho};
use sonic1_diferencial::rotina::{
    Adaptador, Caso, executar_caso, inferir_mapa, ler_mapa, verificar_mapa,
};
use sonic1_diferencial::{SENTINELA, rom_com_trecho};
use sonic1_teste_nucleo::harness::{Metadado, Reg};
use std::collections::BTreeMap;
use websonic_oracle::{Entradas, Oraculo};

/// Rotina sintética "Limita": D2 = 5; se D0 (byte) >= $80 grava $FF em $FF0200, senão grava D0.
/// moveq #5,d2 ; cmpi.b #$80,d0 ; bcs.s +10 ; move.b #$FF,$FF0200 ; bra.s +6 ; move.b d0,$FF0200
const TRECHO: [u16; 12] = [
    0x7405, 0x0C00, 0x0080, 0x650A, 0x13FC, 0x00FF, 0x00FF, 0x0200, 0x6006, 0x13C0, 0x00FF, 0x0200,
];
/// Endereço do `bcs` na ROM sintética.
const BCS: u32 = 0x406;

const META: &str = "rotulo: Limita\nentrada: d0 = valor\nusados_depois: d2\nignorados: d0 d1 d3 d4 d5 d6 d7 a0 a1 a2 a3 a4 a5 a6 ccr\n";

#[derive(Clone, Copy)]
enum Defeito {
    Nenhum,
    GravaErrado,
    DevolveRegistradorErrado,
    PolaridadeTrocada,
}

struct Limita(Defeito);

impl Adaptador for Limita {
    fn executar(
        &self,
        mem: &mut Memoria,
        entrada: &BTreeMap<Reg, u32>,
    ) -> Result<BTreeMap<Reg, u32>, String> {
        let v = entrada[&Reg::D(0)] & 0xFF;
        let alto = if matches!(self.0, Defeito::PolaridadeTrocada) {
            v < 0x80
        } else {
            v >= 0x80
        };
        let gravado = if decisao!("Limita:alto", alto) {
            0xFF
        } else {
            v
        };
        let gravado = if matches!(self.0, Defeito::GravaErrado) && v == 0x10 {
            gravado ^ 1
        } else {
            gravado
        };
        mem.escrever(0xFF_0200, Tamanho::Byte, gravado)
            .map_err(|e| format!("{e:?}"))?;
        let d2 = if matches!(self.0, Defeito::DevolveRegistradorErrado) {
            6
        } else {
            5
        };
        Ok(BTreeMap::from([(Reg::D(2), d2)]))
    }
}

fn rodar(defeito: Defeito, valores: &[u32]) -> Vec<sonic1_diferencial::rotina::ResultadoCaso> {
    let rom = rom_com_trecho(&TRECHO);
    let mut o = Oraculo::novo(rom.clone(), Entradas::default()).unwrap();
    let estado = o.salvar_estado().unwrap();
    let meta = Metadado::analisar(META).unwrap();
    let adaptador = Limita(defeito);
    valores
        .iter()
        .map(|v| {
            let mut regs: Regs68 = o.emulador().oracle_regs68();
            regs.d = [0; 8];
            regs.a = [0; 7];
            regs.d[0] = *v;
            regs.pc = 0x400;
            regs.sr = 0x2700;
            let caso = Caso {
                rom: &rom,
                estado: &estado,
                meta: &meta,
                adaptador: &adaptador,
                regs,
                sentinela: SENTINELA,
                exclusoes: &[(0x7E00, 0x8000)],
            };
            executar_caso(&mut o, &caso).unwrap()
        })
        .collect()
}

const VALORES: [u32; 6] = [0x00, 0x10, 0x7F, 0x80, 0xFF, 0x1234_5681];

#[test]
fn nucleo_certo_passa_e_o_mapa_de_decisao_e_inferido() {
    let casos = rodar(Defeito::Nenhum, &VALORES);
    for c in &casos {
        assert!(c.relatorio.passou(), "{:?}", c.relatorio);
    }
    // O original mexeu em d0? Nao. Mexeu em d1? Nao. O ccr foi mexido pelo cmpi (ignorado).
    assert!(casos[0].relatorio.ignorados_mexidos.contains(&Reg::Ccr));
    let mapa = inferir_mapa(&casos);
    assert_eq!(mapa.len(), 1);
    // O bcs desvia quando D0 < $80; a decisao do nucleo e "D0 >= $80": polaridade inversa.
    assert_eq!(mapa[0].id, "Limita:alto");
    assert_eq!(mapa[0].candidatos, vec![(BCS, true)]);
    // O mapa declarado confere em todos os casos.
    let declarado = ler_mapa("Limita:alto\t406\tinverso\n").unwrap();
    for c in &casos {
        assert!(verificar_mapa(&declarado, c).is_empty());
    }
}

#[test]
fn byte_de_ram_errado_e_acusado() {
    let casos = rodar(Defeito::GravaErrado, &VALORES);
    let ruim: Vec<_> = casos.iter().filter(|c| !c.relatorio.passou()).collect();
    assert_eq!(ruim.len(), 1);
    assert_eq!(ruim[0].relatorio.ram_diferente, vec![0x200]);
}

#[test]
fn registrador_usado_errado_e_acusado() {
    let casos = rodar(Defeito::DevolveRegistradorErrado, &VALORES);
    assert!(
        casos
            .iter()
            .all(|c| c.relatorio.usados_diferentes == vec![(Reg::D(2), 5, 6)])
    );
}

#[test]
fn polaridade_trocada_so_aparece_no_mapa_declarado() {
    // A RAM e os registradores continuam certos (a decisao trocada tambem troca o valor), mas a
    // sequencia de decisoes nao bate com o mapa declarado.
    let casos = rodar(Defeito::PolaridadeTrocada, &VALORES);
    let declarado = ler_mapa("Limita:alto\t406\tinverso\n").unwrap();
    assert!(
        casos
            .iter()
            .any(|c| !verificar_mapa(&declarado, c).is_empty())
    );
}

#[test]
fn diferenca_dentro_da_area_excluida_nao_reprova() {
    // A chamada empilha o endereco de retorno em $FF7FFC: fica dentro da area excluida.
    let casos = rodar(Defeito::Nenhum, &[0x00]);
    assert!(casos[0].relatorio.excluidos_diferentes > 0);
    assert!(casos[0].relatorio.passou());
}
