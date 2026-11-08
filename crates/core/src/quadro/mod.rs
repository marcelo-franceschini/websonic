//! Laço de quadro com o tempo gravado e pontos de checagem de interrupção (sub-etapas 5.3 e 5.4).
//!
//! Este módulo é infraestrutura e não tem par no original (requisitos, "Arquitetura"). O núcleo
//! roda os passos do jogo na ordem. Onde o laço principal toca memória que as interrupções
//! também usam, o código chama [`Contexto::checagem`]. O tempo gravado diz em que rotina do
//! original cada interrupção chegou, e a checagem a entrega no ponto que cobre aquele lugar.
//!
//! Nível 1: o núcleo NÃO calcula ciclos nem prevê interrupções. Isso é o nível 2 (Etapa 15).

pub mod tempo;

use crate::memoria::Memoria;
use std::collections::VecDeque;
use tempo::{EventoTempo, TempoGravado, TempoQuadro};

/// O ponto fica antes ou depois do acesso à memória compartilhada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Posicao {
    Antes,
    Depois,
}

/// Ponto de checagem. Só nomes e endereços do original.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ponto {
    pub id: u16,
    /// Rotina do original onde a interrupção pode cair.
    pub rotulo: &'static str,
    /// Faixa de endereços da rotina no original: `inicio <= pc < fim`.
    pub inicio: u32,
    pub fim: u32,
    /// Endereço do acesso à memória compartilhada que o ponto protege.
    pub endereco: u32,
    pub posicao: Posicao,
}

impl Ponto {
    /// Este ponto é o lugar certo para entregar uma interrupção gravada com este PC?
    /// `Antes`: o PC ainda não passou do acesso. `Depois`: já passou.
    pub fn cobre(&self, pc: u32) -> bool {
        if pc < self.inicio || pc >= self.fim {
            return false;
        }
        match self.posicao {
            Posicao::Antes => pc <= self.endereco,
            Posicao::Depois => pc > self.endereco,
        }
    }
}

/// Onde uma interrupção foi entregue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entrega {
    pub quadro: u32,
    pub nivel: u8,
    pub pc_gravado: u32,
    /// `None`: nenhum ponto cobria aquele PC; entregue no fim dos passos.
    pub ponto: Option<u16>,
}

/// Tratador de uma interrupção. Recebe a memória do console.
pub type Tratador = Box<dyn FnMut(&mut Memoria)>;

struct Pendente {
    nivel: u8,
    pc: u32,
}

/// O que um passo vê durante o quadro.
pub struct Contexto<'a> {
    pub mem: &'a mut Memoria,
    quadro: u32,
    pendentes: &'a mut VecDeque<Pendente>,
    entregas: &'a mut Vec<Entrega>,
    tratadores: &'a mut [Option<Tratador>; 8],
}

impl Contexto<'_> {
    /// Cruza um ponto de checagem. Se a primeira interrupção pendente cai neste ponto, o
    /// tratador dela roda agora. Só a primeira pendente pode ser entregue: interrupções não se
    /// reordenam.
    pub fn checagem(&mut self, ponto: &Ponto) {
        let cobre = self.pendentes.front().is_some_and(|p| ponto.cobre(p.pc));
        if cobre && let Some(p) = self.pendentes.pop_front() {
            self.entregar(&p, Some(ponto.id));
        }
    }

    fn entregar(&mut self, p: &Pendente, ponto: Option<u16>) {
        if let Some(Some(t)) = self.tratadores.get_mut(usize::from(p.nivel & 7)) {
            t(self.mem);
        }
        self.entregas.push(Entrega {
            quadro: self.quadro,
            nivel: p.nivel,
            pc_gravado: p.pc,
            ponto,
        });
    }
}

/// Resumo de um quadro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatorioQuadro {
    pub numero: u32,
    /// Quadro de lag: o VBlank gravado chegou com o PC fora da rotina de espera.
    pub lag: bool,
    pub orcamento: u64,
    /// Ciclos mestres declarados pelos passos. Nos testes, os passos de brinquedo declaram.
    pub gasto: u64,
    /// `gasto > orcamento`. O adiamento do resto para o quadro seguinte é do nível 2.
    pub estourou: bool,
    pub entregues_em_ponto: usize,
    pub sem_ponto: usize,
    /// Leituras gravadas que nenhum acesso consumiu.
    pub leituras_sobrando: usize,
}

/// Erros do laço.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroQuadro {
    /// O tempo gravado acabou.
    SemTempo,
}

/// O núcleo no nível do console: memória, tempo gravado e tratadores.
pub struct Nucleo {
    pub mem: Memoria,
    tempo: TempoGravado,
    proximo: usize,
    tratadores: [Option<Tratador>; 8],
    entregas: Vec<Entrega>,
    /// Faixa `[inicio, fim)` da rotina que espera o VBlank no original.
    espera_vblank: (u32, u32),
    botoes: u8,
}

impl Nucleo {
    pub fn novo(mem: Memoria, tempo: TempoGravado, espera_vblank: (u32, u32)) -> Self {
        Self {
            mem,
            tempo,
            proximo: 0,
            tratadores: Default::default(),
            entregas: Vec::new(),
            espera_vblank,
            botoes: 0,
        }
    }

    pub fn definir_tratador(&mut self, nivel: u8, t: Tratador) {
        if let Some(slot) = self.tratadores.get_mut(usize::from(nivel & 7)) {
            *slot = Some(t);
        }
    }

    pub fn entregas(&self) -> &[Entrega] {
        &self.entregas
    }

    pub fn tempo(&self) -> &TempoGravado {
        &self.tempo
    }

    /// Botões que o jogo está vendo (a última linha de entrada aplicada).
    pub fn botoes(&self) -> u8 {
        self.botoes
    }

    fn e_lag(&self, q: &TempoQuadro) -> bool {
        let (ini, fim) = self.espera_vblank;
        q.eventos.iter().any(|e| {
            matches!(e, EventoTempo::Interrupcao { nivel: 6, pc_interrompido, .. }
                if *pc_interrompido < ini || *pc_interrompido >= fim)
        })
    }

    /// Roda um quadro com a linha de entrada `botoes` e os `passos` na ordem.
    /// Cada passo devolve os ciclos mestres que declara gastar (zero se não declara).
    pub fn executar_quadro(
        &mut self,
        botoes: u8,
        passos: &mut [&mut dyn FnMut(&mut Contexto) -> u64],
    ) -> Result<RelatorioQuadro, ErroQuadro> {
        let q = self
            .tempo
            .quadros
            .get(self.proximo)
            .ok_or(ErroQuadro::SemTempo)?
            .clone();
        self.proximo = self.proximo.saturating_add(1);
        let lag = self.e_lag(&q);
        // Num quadro de lag o jogo não lê o controle: a linha de entrada é ignorada.
        if !lag {
            self.botoes = botoes;
            self.mem.definir_botoes(botoes);
        }
        let mut pendentes = VecDeque::new();
        for e in &q.eventos {
            match *e {
                EventoTempo::Interrupcao {
                    nivel,
                    pc_interrompido,
                    ..
                } => {
                    pendentes.push_back(Pendente {
                        nivel,
                        pc: pc_interrompido,
                    });
                }
                EventoTempo::Leitura { classe, valor, .. } => {
                    self.mem.enfileirar_leitura(classe, valor)
                }
                EventoTempo::EsperaZ80 { .. } => {}
            }
        }
        let antes = self.entregas.len();
        let mut gasto = 0u64;
        {
            let mut ctx = Contexto {
                mem: &mut self.mem,
                quadro: q.numero,
                pendentes: &mut pendentes,
                entregas: &mut self.entregas,
                tratadores: &mut self.tratadores,
            };
            for passo in passos.iter_mut() {
                gasto = gasto.saturating_add(passo(&mut ctx));
            }
            // O que sobrou não tinha ponto: entrega no fim dos passos.
            while let Some(p) = ctx.pendentes.pop_front() {
                ctx.entregar(&p, None);
            }
        }
        let novas = self.entregas.get(antes..).unwrap_or_default();
        let sem_ponto = novas.iter().filter(|e| e.ponto.is_none()).count();
        let orcamento = q.orcamento();
        Ok(RelatorioQuadro {
            numero: q.numero,
            lag,
            orcamento,
            gasto,
            estourou: gasto > orcamento,
            entregues_em_ponto: novas.len().saturating_sub(sem_ponto),
            sem_ponto,
            leituras_sobrando: self.mem.leituras_pendentes(),
        })
    }
}

/// Divergência entre o tempo gravado e as entregas do núcleo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DivergenciaEntrega {
    /// Faltou entregar uma interrupção gravada.
    Perdida { quadro: u32, nivel: u8, pc: u32 },
    /// O núcleo entregou algo que não foi gravado.
    Extra { quadro: u32, nivel: u8, pc: u32 },
    /// A ordem difere da gravada.
    Ordem {
        quadro: u32,
        esperado_pc: u32,
        obtido_pc: u32,
    },
    /// O ponto entregue não cobre o PC gravado.
    PontoErrado { quadro: u32, pc: u32, ponto: u16 },
}

/// Confere as entregas do núcleo com o tempo gravado, quadro a quadro. Para na primeira
/// divergência de cada quadro. `pontos` é a tabela de pontos do núcleo.
pub fn conferir_entregas(
    tempo: &TempoGravado,
    entregas: &[Entrega],
    pontos: &[Ponto],
) -> Vec<DivergenciaEntrega> {
    let mut ruins = Vec::new();
    for q in &tempo.quadros {
        let gravadas: Vec<(u8, u32)> = q
            .eventos
            .iter()
            .filter_map(|e| match *e {
                EventoTempo::Interrupcao {
                    nivel,
                    pc_interrompido,
                    ..
                } => Some((nivel, pc_interrompido)),
                _ => None,
            })
            .collect();
        let obtidas: Vec<&Entrega> = entregas.iter().filter(|e| e.quadro == q.numero).collect();
        for (i, (nivel, pc)) in gravadas.iter().enumerate() {
            match obtidas.get(i) {
                None => {
                    ruins.push(DivergenciaEntrega::Perdida {
                        quadro: q.numero,
                        nivel: *nivel,
                        pc: *pc,
                    });
                    break;
                }
                Some(o) if o.pc_gravado != *pc || o.nivel != *nivel => {
                    ruins.push(DivergenciaEntrega::Ordem {
                        quadro: q.numero,
                        esperado_pc: *pc,
                        obtido_pc: o.pc_gravado,
                    });
                    break;
                }
                Some(o) => {
                    if let Some(id) = o.ponto {
                        let cobre = pontos
                            .iter()
                            .find(|p| p.id == id)
                            .is_some_and(|p| p.cobre(*pc));
                        if !cobre {
                            ruins.push(DivergenciaEntrega::PontoErrado {
                                quadro: q.numero,
                                pc: *pc,
                                ponto: id,
                            });
                            break;
                        }
                    }
                }
            }
        }
        if let Some(extra) = obtidas.get(gravadas.len()) {
            ruins.push(DivergenciaEntrega::Extra {
                quadro: q.numero,
                nivel: extra.nivel,
                pc: extra.pc_gravado,
            });
        }
    }
    ruins
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::memoria::{ClasseLeitura, Tamanho};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Rotina de brinquedo do "original": endereços $1000 a $1100, acesso à variável
    /// compartilhada em $1040.
    const ROTINA: (u32, u32) = (0x1000, 0x1100);
    const ANTES: Ponto = Ponto {
        id: 1,
        rotulo: "Brinquedo",
        inicio: ROTINA.0,
        fim: ROTINA.1,
        endereco: 0x1040,
        posicao: Posicao::Antes,
    };
    const DEPOIS: Ponto = Ponto {
        id: 2,
        rotulo: "Brinquedo",
        inicio: ROTINA.0,
        fim: ROTINA.1,
        endereco: 0x1040,
        posicao: Posicao::Depois,
    };
    const ESPERA: (u32, u32) = (0x0300, 0x0320);

    fn int(nivel: u8, pc: u32) -> EventoTempo {
        EventoTempo::Interrupcao {
            nivel,
            pc_interrompido: pc,
            pc_tratador: 0,
            mclk: 0,
            linha: 224,
            coluna: 0,
        }
    }

    fn tempo(eventos_por_quadro: Vec<Vec<EventoTempo>>) -> TempoGravado {
        TempoGravado {
            quadros: eventos_por_quadro
                .into_iter()
                .enumerate()
                .map(|(i, eventos)| {
                    let n = u64::try_from(i).unwrap();
                    TempoQuadro {
                        numero: u32::try_from(i).unwrap(),
                        mclk_inicio: n.saturating_mul(1000),
                        mclk_fim: n.saturating_add(1).saturating_mul(1000),
                        eventos,
                    }
                })
                .collect(),
        }
    }

    /// Brinquedo: o passo escreve a variável $FF0000 em duas etapas, com um ponto antes e outro
    /// depois da escrita. O tratador de VBlank lê a variável e acumula o que viu.
    fn rodar(
        eventos: Vec<Vec<EventoTempo>>,
    ) -> (Nucleo, Rc<RefCell<Vec<u32>>>, Vec<RelatorioQuadro>) {
        let mut n = Nucleo::novo(Memoria::nova(vec![0; 16]), tempo(eventos), ESPERA);
        let vistos = Rc::new(RefCell::new(Vec::new()));
        let v2 = Rc::clone(&vistos);
        n.definir_tratador(
            6,
            Box::new(move |m| {
                v2.borrow_mut()
                    .push(m.ler(0xFF_0000, Tamanho::Long).unwrap())
            }),
        );
        let mut rel = Vec::new();
        for _ in 0..n.tempo().quadros.len() {
            let mut passo = |c: &mut Contexto| {
                c.checagem(&ANTES);
                c.mem
                    .escrever(0xFF_0000, Tamanho::Long, 0xAAAA_BBBB)
                    .unwrap();
                c.checagem(&DEPOIS);
                10
            };
            rel.push(n.executar_quadro(0, &mut [&mut passo]).unwrap());
        }
        (n, vistos, rel)
    }

    #[test]
    fn interrupcao_antes_e_depois_do_acesso_ve_valores_diferentes() {
        // Quadro 0: PC antes do acesso ($1010). Quadro 1: depois ($1080). Quadro 2: sem interrupcao.
        let (n, vistos, rel) = rodar(vec![vec![int(6, 0x1010)], vec![int(6, 0x1080)], vec![]]);
        // No quadro 0 a escrita ainda nao aconteceu (0). No quadro 1 ela ja aconteceu.
        assert_eq!(*vistos.borrow(), vec![0, 0xAAAA_BBBB]);
        assert_eq!(n.entregas()[0].ponto, Some(1));
        assert_eq!(n.entregas()[1].ponto, Some(2));
        assert_eq!(rel[0].entregues_em_ponto, 1);
        assert_eq!(rel[2].entregues_em_ponto + rel[2].sem_ponto, 0);
        assert!(conferir_entregas(n.tempo(), n.entregas(), &[ANTES, DEPOIS]).is_empty());
    }

    #[test]
    fn interrupcao_em_rotina_sem_ponto_e_entregue_no_fim() {
        let (n, vistos, rel) = rodar(vec![vec![int(6, 0x2000)]]);
        assert_eq!(*vistos.borrow(), vec![0xAAAA_BBBB]);
        assert_eq!(n.entregas()[0].ponto, None);
        assert_eq!(rel[0].sem_ponto, 1);
    }

    #[test]
    fn duas_interrupcoes_no_mesmo_quadro_nao_se_reordenam() {
        // A primeira cai depois do acesso, a segunda antes: a segunda espera a primeira.
        let (n, _, _) = rodar(vec![vec![int(6, 0x1080), int(6, 0x1010)]]);
        let pcs: Vec<u32> = n.entregas().iter().map(|e| e.pc_gravado).collect();
        assert_eq!(pcs, vec![0x1080, 0x1010]);
        // A segunda foi entregue no fim (nenhum ponto restante a cobria).
        assert_eq!(n.entregas()[1].ponto, None);
    }

    #[test]
    fn quadro_de_lag_ignora_a_linha_de_entrada() {
        let t = tempo(vec![
            vec![int(6, 0x0310)],
            vec![int(6, 0x2222)],
            vec![int(6, 0x0310)],
        ]);
        let mut n = Nucleo::novo(Memoria::nova(vec![0; 16]), t, ESPERA);
        // Quadro 0: VBlank dentro da espera. Sem lag; a entrada 0x01 vale.
        let r = n.executar_quadro(0x01, &mut []).unwrap();
        assert!(!r.lag);
        assert_eq!(n.botoes(), 0x01);
        // Quadro 1: VBlank no meio da logica. Lag; a entrada 0x80 e ignorada.
        let r = n.executar_quadro(0x80, &mut []).unwrap();
        assert!(r.lag);
        assert_eq!(n.botoes(), 0x01);
        // Quadro 2: normal de novo.
        let r = n.executar_quadro(0x40, &mut []).unwrap();
        assert!(!r.lag);
        assert_eq!(n.botoes(), 0x40);
        assert_eq!(n.executar_quadro(0, &mut []), Err(ErroQuadro::SemTempo));
    }

    #[test]
    fn leituras_gravadas_chegam_na_ordem_e_o_orcamento_vem_do_arquivo() {
        let t = tempo(vec![vec![
            EventoTempo::Leitura {
                classe: ClasseLeitura::VdpStatus,
                valor: 0x3401,
                mclk: 1,
            },
            EventoTempo::Leitura {
                classe: ClasseLeitura::VdpStatus,
                valor: 0x3400,
                mclk: 2,
            },
        ]]);
        let mut n = Nucleo::novo(Memoria::nova(vec![0; 16]), t, ESPERA);
        let mut lidos = Vec::new();
        let mut passo = |c: &mut Contexto| {
            lidos.push(c.mem.ler(0xC0_0004, Tamanho::Word).unwrap());
            lidos.push(c.mem.ler(0xC0_0004, Tamanho::Word).unwrap());
            5000
        };
        let r = n.executar_quadro(0, &mut [&mut passo]).unwrap();
        assert_eq!(lidos, vec![0x3401, 0x3400]);
        assert_eq!(r.orcamento, 1000);
        assert!(r.estourou);
        assert_eq!(r.leituras_sobrando, 0);
    }

    #[test]
    fn conferencia_acusa_perdida_extra_ordem_e_ponto_errado() {
        let t = tempo(vec![vec![int(6, 0x1010)]]);
        let pontos = [ANTES, DEPOIS];
        let ok = Entrega {
            quadro: 0,
            nivel: 6,
            pc_gravado: 0x1010,
            ponto: Some(1),
        };
        assert!(conferir_entregas(&t, std::slice::from_ref(&ok), &pontos).is_empty());
        assert_eq!(
            conferir_entregas(&t, &[], &pontos),
            vec![DivergenciaEntrega::Perdida {
                quadro: 0,
                nivel: 6,
                pc: 0x1010
            }]
        );
        let extra = Entrega {
            pc_gravado: 0x1234,
            ..ok.clone()
        };
        assert_eq!(
            conferir_entregas(&t, &[ok.clone(), extra], &pontos),
            vec![DivergenciaEntrega::Extra {
                quadro: 0,
                nivel: 6,
                pc: 0x1234
            }]
        );
        let outra = Entrega {
            pc_gravado: 0x1020,
            ..ok.clone()
        };
        assert!(matches!(
            conferir_entregas(&t, &[outra], &pontos)[0],
            DivergenciaEntrega::Ordem { .. }
        ));
        // O ponto "Depois" nao cobre um PC anterior ao acesso.
        let errada = Entrega {
            ponto: Some(2),
            ..ok
        };
        assert_eq!(
            conferir_entregas(&t, &[errada], &pontos),
            vec![DivergenciaEntrega::PontoErrado {
                quadro: 0,
                pc: 0x1010,
                ponto: 2
            }]
        );
    }
}
