//! Comparação do vídeo da abertura (Etapa 7, sub-etapa 7.7): o renderizador (`sonic1-video`) é
//! alimentado só pelas escritas do núcleo, na ordem em que o núcleo as fez, com a posição (linha e
//! coluna) que o oráculo deu à escrita de mesmo índice, e com as palavras de DMA lidas da RAM do
//! núcleo na hora do evento. Em cada quadro do oráculo, quando todas as escritas dele já foram
//! aplicadas, o quadro fecha e o relatório compara o hash da imagem (9 bits), da VRAM, da CRAM
//! e da VSRAM com os do oráculo. Só imprime contagens e nomes.

use sonic1_console::{Capturada, Ponte};
use sonic1_core::memoria::{Memoria, Saida};
use sonic1_fidelidade::hash::sha1;
use sonic1_fidelidade::{Categoria, Quadro, Tipo};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// O que o oráculo diz de um quadro, guardado até o renderizador chegar nele.
struct QuadroDoOraculo {
    numero: u32,
    /// Total acumulado de escritas de vídeo do oráculo até o fim deste quadro.
    fim: u64,
    coluna_fim: u64,
    vram: [u8; 20],
    cram: [u8; 20],
    vsram: [u8; 20],
    imagem: [u8; 20],
    /// Para o diagnóstico: a CRAM e a imagem do oráculo (só os primeiros divergentes se usam).
    cram_palavras: Vec<u16>,
    pontos: Vec<u16>,
}

/// Contagens do vídeo, para o relatório.
#[derive(Clone, Debug, Default)]
pub struct ResultadoVideo {
    pub quadros: u32,
    pub imagem_iguais: u32,
    pub imagem_diverge: u32,
    pub vram_iguais: u32,
    pub vram_diverge: u32,
    pub cram_iguais: u32,
    pub cram_diverge: u32,
    pub vsram_iguais: u32,
    pub vsram_diverge: u32,
    /// Quadros do oráculo que não chegaram a fechar (o núcleo parou antes das escritas deles).
    pub pendentes: u32,
    /// O primeiro quadro que diverge e em que campos.
    pub primeiro: Option<String>,
    pub avisos: String,
    /// Diagnóstico dos primeiros quadros divergentes: índices e contagens, nunca valores.
    pub diagnostico: Vec<String>,
}

impl ResultadoVideo {
    pub fn zero(&self) -> bool {
        self.imagem_diverge == 0
            && self.vram_diverge == 0
            && self.cram_diverge == 0
            && self.vsram_diverge == 0
    }
}

fn palavras_em_bytes(p: &[u16]) -> Vec<u8> {
    p.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// O renderizador e as filas que o ligam ao núcleo e ao oráculo.
pub struct Video {
    ponte: Rc<RefCell<Ponte>>,
    capturadas: Rc<RefCell<VecDeque<Capturada>>>,
    saidas: Rc<RefCell<Vec<Saida>>>,
    posicoes: VecDeque<(u16, u64)>,
    quadros: VecDeque<QuadroDoOraculo>,
    aplicadas: u64,
    total_oraculo: u64,
    pub resultado: ResultadoVideo,
}

impl Default for Video {
    fn default() -> Self {
        Self::novo()
    }
}

impl Video {
    pub fn novo() -> Self {
        Self {
            ponte: Rc::new(RefCell::new(Ponte::nova())),
            capturadas: Rc::new(RefCell::new(VecDeque::new())),
            saidas: Rc::new(RefCell::new(Vec::new())),
            posicoes: VecDeque::new(),
            quadros: VecDeque::new(),
            aplicadas: 0,
            total_oraculo: 0,
            resultado: ResultadoVideo::default(),
        }
    }

    /// O observador que o núcleo chama depois de cada evento: recolhe as saídas, acompanha a
    /// VDP (e lê o DMA) e guarda tudo na ordem.
    pub fn observador(&self) -> Box<dyn FnMut(&mut Memoria)> {
        let ponte = Rc::clone(&self.ponte);
        let capturadas = Rc::clone(&self.capturadas);
        let saidas = Rc::clone(&self.saidas);
        Box::new(move |mem: &mut Memoria| {
            for s in mem.tirar_saidas() {
                if let Some(c) = ponte.borrow_mut().capturar(mem, &s) {
                    capturadas.borrow_mut().push_back(c);
                }
                saidas.borrow_mut().push(s);
            }
        })
    }

    /// Entrega e esvazia as saídas recolhidas desde a última chamada.
    pub fn tirar_saidas(&self) -> Vec<Saida> {
        std::mem::take(&mut *self.saidas.borrow_mut())
    }

    /// Registra um quadro do oráculo: as posições das escritas de vídeo e os hashes de fechamento.
    pub fn quadro_do_oraculo(&mut self, q: &Quadro, imagem: &[u16], cram: &[u16]) {
        let mut novas = 0u64;
        for e in &q.eventos {
            if e.tipo == Tipo::Vdp {
                self.posicoes.push_back((e.linha, u64::from(e.coluna)));
                novas = novas.saturating_add(1);
            }
        }
        self.total_oraculo = self.total_oraculo.saturating_add(novas);
        let fim = self.total_oraculo;
        let inicio_da_linha = (232u64 + 262 * u64::from(q.numero)) * 3420;
        let bytes: Vec<u8> = imagem.iter().flat_map(|p| p.to_le_bytes()).collect();
        self.quadros.push_back(QuadroDoOraculo {
            numero: q.numero,
            fim,
            coluna_fim: q.mclk_fim.saturating_sub(inicio_da_linha),
            vram: q.hashes[Categoria::Vram as usize],
            cram: q.hashes[Categoria::Cram as usize],
            vsram: q.hashes[Categoria::Vsram as usize],
            imagem: sha1(&bytes),
            cram_palavras: cram.to_vec(),
            pontos: imagem.to_vec(),
        });
    }

    /// Aplica as escritas que já têm posição e fecha os quadros que ficaram completos.
    pub fn processar(&mut self) {
        loop {
            let Some(f) = self.quadros.front() else {
                return;
            };
            // Aplica as escritas deste quadro enquanto o núcleo tiver dado.
            while self.aplicadas < f.fim {
                let (Some(c), Some(&(linha, coluna))) = (
                    self.capturadas.borrow_mut().pop_front(),
                    self.posicoes.front(),
                ) else {
                    return;
                };
                self.posicoes.pop_front();
                self.ponte.borrow_mut().aplicar_capturada(&c, linha, coluna);
                self.aplicadas = self.aplicadas.saturating_add(1);
            }
            let Some(f) = self.quadros.pop_front() else {
                return;
            };
            self.ponte.borrow_mut().fechar_quadro_em(f.coluna_fim);
            self.comparar(&f);
        }
    }

    fn comparar(&mut self, f: &QuadroDoOraculo) {
        let ponte = self.ponte.borrow();
        let r = ponte.renderizador();
        let v = r.vdp();
        let imagem = sha1(&r.quadro().para_bytes());
        let vram = sha1(v.vram());
        let cram = sha1(&palavras_em_bytes(v.cram()));
        let vsram = sha1(&palavras_em_bytes(v.vsram()));
        let res = &mut self.resultado;
        res.quadros = res.quadros.saturating_add(1);
        let mut diferentes: Vec<&str> = Vec::new();
        for (nome, igual, iguais, diverge) in [
            (
                "imagem",
                imagem == f.imagem,
                &mut res.imagem_iguais,
                &mut res.imagem_diverge,
            ),
            (
                "vram",
                vram == f.vram,
                &mut res.vram_iguais,
                &mut res.vram_diverge,
            ),
            (
                "cram",
                cram == f.cram,
                &mut res.cram_iguais,
                &mut res.cram_diverge,
            ),
            (
                "vsram",
                vsram == f.vsram,
                &mut res.vsram_iguais,
                &mut res.vsram_diverge,
            ),
        ] {
            if igual {
                *iguais = iguais.saturating_add(1);
            } else {
                *diverge = diverge.saturating_add(1);
                diferentes.push(nome);
            }
        }
        if !diferentes.is_empty() && res.diagnostico.len() < 6 {
            let cram_dif: Vec<usize> = v
                .cram()
                .iter()
                .zip(&f.cram_palavras)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, _)| i)
                .collect();
            let pontos = r.quadro().pontos();
            let dif_pontos: Vec<usize> = pontos
                .iter()
                .zip(&f.pontos)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, _)| i)
                .collect();
            let linhas = (
                dif_pontos.first().map(|i| i / 320),
                dif_pontos.last().map(|i| i / 320),
            );
            res.diagnostico.push(format!(
                "quadro {} campos {:?} cram_indices={:?} pontos_diferentes={} linhas={:?}",
                f.numero,
                diferentes,
                cram_dif,
                dif_pontos.len(),
                linhas
            ));
        }
        if !diferentes.is_empty() && res.primeiro.is_none() {
            res.primeiro = Some(format!("quadro {} campos {:?}", f.numero, diferentes));
        }
        res.avisos = format!("{:?}", v.avisos);
    }

    /// Fecha a conta no fim da rodada: quantos quadros do oráculo ficaram sem fechar.
    pub fn terminar(&mut self) {
        self.resultado.pendentes = u32::try_from(self.quadros.len()).unwrap_or(u32::MAX);
    }
}
