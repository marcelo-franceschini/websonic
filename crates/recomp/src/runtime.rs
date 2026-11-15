//! Estado e barramento usados pelo código compilado antecipadamente.
use sonic1_core::memoria::ErroAcesso;
pub use sonic1_core::memoria::Memoria;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tamanho {
    Byte,
    Word,
    Long,
}

impl Tamanho {
    fn n(self, reg: u8) -> u32 {
        match self {
            Self::Byte if reg == 7 => 2,
            Self::Byte => 1,
            Self::Word => 2,
            Self::Long => 4,
        }
    }
    fn mascara(self) -> u32 {
        match self {
            Self::Byte => 0xff,
            Self::Word => 0xffff,
            Self::Long => u32::MAX,
        }
    }
    fn sinal(self) -> u32 {
        match self {
            Self::Byte => 0x80,
            Self::Word => 0x8000,
            Self::Long => 0x8000_0000,
        }
    }
    fn nativo(self) -> sonic1_core::memoria::Tamanho {
        match self {
            Self::Byte => sonic1_core::memoria::Tamanho::Byte,
            Self::Word => sonic1_core::memoria::Tamanho::Word,
            Self::Long => sonic1_core::memoria::Tamanho::Long,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operando {
    Dados(u8),
    Endereco(u8),
    Indireto(u8),
    PosIncremento(u8),
    PreDecremento(u8),
    Deslocamento(u8, i16),
    Absoluto(u32),
    Imediato(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperacaoImediata {
    Ou,
    E,
    Sub,
    Add,
    Xor,
    Cmp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperacaoRegistrador {
    Ou,
    E,
    Sub,
    Add,
    Cmp,
    Xor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoDeslocamento {
    Asl,
    Asr,
    Lsl,
    Lsr,
    Rol,
    Ror,
    Roxl,
    Roxr,
}

impl TipoDeslocamento {
    fn nativo(self) -> sonic1_core::ops68k::Deslocamento {
        match self {
            Self::Asl => sonic1_core::ops68k::Deslocamento::AslEsquerda,
            Self::Asr => sonic1_core::ops68k::Deslocamento::AsrDireita,
            Self::Lsl => sonic1_core::ops68k::Deslocamento::LslEsquerda,
            Self::Lsr => sonic1_core::ops68k::Deslocamento::LsrDireita,
            Self::Rol => sonic1_core::ops68k::Deslocamento::RolEsquerda,
            Self::Ror => sonic1_core::ops68k::Deslocamento::RorDireita,
            Self::Roxl => sonic1_core::ops68k::Deslocamento::RoxlEsquerda,
            Self::Roxr => sonic1_core::ops68k::Deslocamento::RoxrDireita,
        }
    }
}

pub trait Barramento {
    fn ler(&mut self, endereco: u32, tamanho: Tamanho) -> Result<u32, ErroAcesso>;
    fn escrever(&mut self, endereco: u32, tamanho: Tamanho, valor: u32) -> Result<(), ErroAcesso>;
}

impl Barramento for Memoria {
    fn ler(&mut self, endereco: u32, tamanho: Tamanho) -> Result<u32, ErroAcesso> {
        Memoria::ler(self, endereco, tamanho.nativo())
    }
    fn escrever(&mut self, endereco: u32, tamanho: Tamanho, valor: u32) -> Result<(), ErroAcesso> {
        Memoria::escrever(self, endereco, tamanho.nativo(), valor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroExecucao {
    Acesso(ErroAcesso),
    OperandoInvalido,
    Privilegio,
    NivelInterrupcaoInvalido,
}
impl From<ErroAcesso> for ErroExecucao {
    fn from(e: ErroAcesso) -> Self {
        Self::Acesso(e)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Estado {
    pub d: [u32; 8],
    pub a: [u32; 8],
    pub pc: u32,
    pub ccr: u8,
    pub sr_alto: u8,
    pub usp: u32,
    pub ssp: u32,
    /// Ciclos básicos acumulados do 68000, sem DMA, refresh ou esperas do barramento.
    /// `None` indica que um custo ainda não foi modelado ou que houve estouro.
    pub ciclos_base: Option<u64>,
    /// Custo básico da última instrução; `None` significa desconhecido.
    pub ultimo_custo_base: Option<u32>,
}

impl Default for Estado {
    fn default() -> Self {
        Self {
            d: [0; 8],
            a: [0; 8],
            pc: 0,
            ccr: 0,
            sr_alto: 0x27,
            usp: 0,
            ssp: 0,
            ciclos_base: Some(0),
            ultimo_custo_base: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parada {
    Continua,
    Retorno,
    RetornoInterrupcao,
    EnderecoNaoGerado(u32),
}

impl Estado {
    pub fn registrar_ciclos_base(&mut self, custo: Option<u32>) {
        self.ultimo_custo_base = custo;
        self.ciclos_base = self
            .ciclos_base
            .and_then(|soma| custo.and_then(|c| soma.checked_add(u64::from(c))));
    }
    pub fn moveq(&mut self, reg: u8, valor: i8) {
        let v = i32::from(valor) as u32;
        self.d[usize::from(reg)] = v;
        self.flags_move(Tamanho::Long, v);
    }
    fn flags_move(&mut self, t: Tamanho, v: u32) {
        let v = v & t.mascara();
        self.ccr = (self.ccr & !0x0f)
            | if v == 0 { 4 } else { 0 }
            | if v & t.sinal() != 0 { 8 } else { 0 };
    }
    fn endereco(&mut self, op: Operando, t: Tamanho) -> Result<(u32, Option<u8>), ErroExecucao> {
        let a = |r: u8| usize::from(r);
        match op {
            Operando::Indireto(r) => Ok((self.a[a(r)], None)),
            Operando::PosIncremento(r) => Ok((self.a[a(r)], Some(r))),
            Operando::PreDecremento(r) => {
                self.a[a(r)] = self.a[a(r)].wrapping_sub(t.n(r));
                Ok((self.a[a(r)], None))
            }
            Operando::Deslocamento(r, d) => {
                Ok((self.a[a(r)].wrapping_add_signed(i32::from(d)), None))
            }
            Operando::Absoluto(p) => Ok((p, None)),
            _ => Err(ErroExecucao::OperandoInvalido),
        }
    }
    fn ler_op(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        op: Operando,
    ) -> Result<u32, ErroExecucao> {
        let valor = match op {
            Operando::Dados(r) => self.d[usize::from(r)],
            Operando::Endereco(r) => self.a[usize::from(r)],
            Operando::Imediato(v) => v,
            _ => {
                let (p, inc) = self.endereco(op, t)?;
                let v = mem.ler(p, t)?;
                if let Some(r) = inc {
                    self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
                }
                v
            }
        };
        Ok(valor & t.mascara())
    }
    fn escrever_op(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        op: Operando,
        v: u32,
    ) -> Result<(), ErroExecucao> {
        match op {
            Operando::Dados(r) => {
                let d = &mut self.d[usize::from(r)];
                *d = (*d & !t.mascara()) | (v & t.mascara());
            }
            Operando::Endereco(r) => {
                self.a[usize::from(r)] = if t == Tamanho::Word {
                    i32::from(v as u16 as i16) as u32
                } else {
                    v
                };
            }
            Operando::Imediato(_) => return Err(ErroExecucao::OperandoInvalido),
            _ => {
                let (p, inc) = self.endereco(op, t)?;
                mem.escrever(p, t, v)?;
                if let Some(r) = inc {
                    self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
                }
            }
        }
        Ok(())
    }
    pub fn mov(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        origem: Operando,
        destino: Operando,
    ) -> Result<(), ErroExecucao> {
        let v = self.ler_op(mem, t, origem)?;
        self.escrever_op(mem, t, destino, v)?;
        if !matches!(destino, Operando::Endereco(_)) {
            self.flags_move(t, v);
        }
        Ok(())
    }

    /// MOVEM transfere registradores na ordem definida pelo 68000, sem alterar CCR.
    pub fn movem(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        carregar: bool,
        op: Operando,
        mascara: u16,
    ) -> Result<(), ErroExecucao> {
        if t == Tamanho::Byte {
            return Err(ErroExecucao::OperandoInvalido);
        }
        let orig_d = self.d;
        let orig_a = self.a;
        let pre = matches!(op, Operando::PreDecremento(_));
        // A pré-decrementação do MOVEM usa máscara invertida. O endereço inicial não
        // deve ser decrementado pela função genérica de operandos, mas por registrador.
        let mut cursor = if let Operando::PreDecremento(r) = op {
            orig_a[usize::from(r)]
        } else {
            self.endereco(op, t)?.0
        };
        for bit in 0..16usize {
            if mascara & (1 << bit) == 0 {
                continue;
            }
            let indice = if pre { 15 - bit } else { bit };
            if pre {
                cursor = cursor.wrapping_sub(t.n(0));
                if let Operando::PreDecremento(r) = op {
                    self.a[usize::from(r)] = cursor;
                }
            }
            if carregar {
                let raw = mem.ler(cursor, t)?;
                let valor = if t == Tamanho::Word {
                    i32::from(raw as u16 as i16) as u32
                } else {
                    raw
                };
                if indice < 8 {
                    self.d[indice] = valor;
                } else {
                    self.a[indice - 8] = valor;
                }
            } else {
                let valor = if indice < 8 {
                    orig_d[indice]
                } else {
                    orig_a[indice - 8]
                };
                mem.escrever(cursor, t, valor)?;
            }
            if !pre {
                cursor = cursor.wrapping_add(t.n(0));
            }
        }
        if let Operando::PosIncremento(r) = op {
            self.a[usize::from(r)] = cursor;
        }
        Ok(())
    }

    /// Operações imediatas preservam o endereço efetivo durante a leitura e escrita.
    pub fn imediata(
        &mut self,
        mem: &mut impl Barramento,
        op: OperacaoImediata,
        t: Tamanho,
        imediato: u32,
        destino: Operando,
    ) -> Result<(), ErroExecucao> {
        let (antigo, endereco) = match destino {
            Operando::Dados(r) => (self.d[usize::from(r)] & t.mascara(), None),
            Operando::Endereco(_) | Operando::Imediato(_) => {
                return Err(ErroExecucao::OperandoInvalido);
            }
            _ => {
                let (p, inc) = self.endereco(destino, t)?;
                (mem.ler(p, t)? & t.mascara(), Some((p, inc)))
            }
        };
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        if op == OperacaoImediata::Cmp {
            self.ccr =
                sonic1_core::ops68k::cmp(t.nativo(), imediato, antigo, ccr).para_bits() as u8;
        } else {
            use sonic1_core::ops68k;
            let resultado = match op {
                OperacaoImediata::Ou => ops68k::or(t.nativo(), imediato, antigo, ccr),
                OperacaoImediata::E => ops68k::and(t.nativo(), imediato, antigo, ccr),
                OperacaoImediata::Sub => ops68k::sub(t.nativo(), imediato, antigo, ccr),
                OperacaoImediata::Add => ops68k::add(t.nativo(), imediato, antigo, ccr),
                OperacaoImediata::Xor => ops68k::eor(t.nativo(), imediato, antigo, ccr),
                OperacaoImediata::Cmp => unreachable!(),
            };
            if let Some((p, _)) = endereco {
                mem.escrever(p, t, resultado.valor)?;
            } else if let Operando::Dados(r) = destino {
                let d = &mut self.d[usize::from(r)];
                *d = (*d & !t.mascara()) | (resultado.valor & t.mascara());
            }
            self.ccr = resultado.ccr.para_bits() as u8;
        }
        if let Some((_, Some(r))) = endereco {
            self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
        }
        Ok(())
    }

    pub fn mover_usp(&mut self, reg: u8, para_usp: bool) -> Result<(), ErroExecucao> {
        if self.sr_alto & 0x20 == 0 {
            return Err(ErroExecucao::Privilegio);
        }
        if para_usp {
            self.usp = self.a[usize::from(reg)];
        } else {
            self.a[usize::from(reg)] = self.usp;
        }
        Ok(())
    }

    /// MOVE para SR é privilegiado; ao trocar S alterna as pilhas de usuário e supervisor.
    pub fn mover_para_sr(
        &mut self,
        mem: &mut impl Barramento,
        origem: Operando,
    ) -> Result<(), ErroExecucao> {
        if self.sr_alto & 0x20 == 0 {
            return Err(ErroExecucao::Privilegio);
        }
        let valor = self.ler_op(mem, Tamanho::Word, origem)? as u16;
        self.definir_sr(valor);
        Ok(())
    }

    fn definir_sr(&mut self, valor: u16) {
        let anterior_supervisor = self.sr_alto & 0x20 != 0;
        let alto = ((valor >> 8) as u8) & 0xa7;
        let novo_supervisor = alto & 0x20 != 0;
        if anterior_supervisor && !novo_supervisor {
            self.ssp = self.a[7];
            self.a[7] = self.usp;
        } else if !anterior_supervisor && novo_supervisor {
            self.usp = self.a[7];
            self.a[7] = self.ssp;
        }
        self.sr_alto = alto;
        self.ccr = (valor as u8) & 0x1f;
    }

    /// Aceita um evento de interrupção no limite entre instruções. Nível 7 deve ser
    /// informado pelo chamador somente na borda de subida, pois não é mascarável.
    /// A temporização do barramento e do vetor ainda não está modelada.
    pub fn interrupcao_autovetor(
        &mut self,
        mem: &mut impl Barramento,
        nivel: u8,
    ) -> Result<bool, ErroExecucao> {
        if !(1..=7).contains(&nivel) {
            return Err(ErroExecucao::NivelInterrupcaoInvalido);
        }
        if nivel != 7 && nivel <= (self.sr_alto & 7) {
            return Ok(false);
        }
        let sr_anterior = (u16::from(self.sr_alto) << 8) | u16::from(self.ccr);
        let pc_anterior = self.pc;
        if self.sr_alto & 0x20 == 0 {
            self.usp = self.a[7];
            self.a[7] = self.ssp;
        }
        self.sr_alto = (self.sr_alto & !0x87) | 0x20 | nivel;
        self.a[7] = self.a[7].wrapping_sub(4);
        mem.escrever(self.a[7], Tamanho::Long, pc_anterior)?;
        self.a[7] = self.a[7].wrapping_sub(2);
        mem.escrever(self.a[7], Tamanho::Word, u32::from(sr_anterior))?;
        self.pc = mem.ler(0x60 + u32::from(nivel) * 4, Tamanho::Long)?;
        self.registrar_ciclos_base(None);
        Ok(true)
    }

    /// RTE restaura o quadro de exceção de seis bytes do 68000.
    pub fn retornar_interrupcao(&mut self, mem: &mut impl Barramento) -> Result<(), ErroExecucao> {
        if self.sr_alto & 0x20 == 0 {
            return Err(ErroExecucao::Privilegio);
        }
        let sr = mem.ler(self.a[7], Tamanho::Word)? as u16;
        let pc = mem.ler(self.a[7].wrapping_add(2), Tamanho::Long)?;
        self.a[7] = self.a[7].wrapping_add(6);
        self.definir_sr(sr);
        self.pc = pc;
        Ok(())
    }

    pub fn registrador(
        &mut self,
        mem: &mut impl Barramento,
        op: OperacaoRegistrador,
        t: Tamanho,
        origem: Operando,
        reg: u8,
    ) -> Result<(), ErroExecucao> {
        let valor = self.ler_op(mem, t, origem)?;
        let antigo = self.d[usize::from(reg)];
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        use sonic1_core::ops68k;
        if op == OperacaoRegistrador::Cmp {
            self.ccr = ops68k::cmp(t.nativo(), valor, antigo, ccr).para_bits() as u8;
            return Ok(());
        }
        let res = match op {
            OperacaoRegistrador::Ou => ops68k::or(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::E => ops68k::and(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Sub => ops68k::sub(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Add => ops68k::add(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Xor => ops68k::eor(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Cmp => unreachable!(),
        };
        self.d[usize::from(reg)] = (antigo & !t.mascara()) | (res.valor & t.mascara());
        self.ccr = res.ccr.para_bits() as u8;
        Ok(())
    }

    pub fn operacao_para_memoria(
        &mut self,
        mem: &mut impl Barramento,
        op: OperacaoRegistrador,
        t: Tamanho,
        reg_origem: u8,
        destino: Operando,
    ) -> Result<(), ErroExecucao> {
        let valor = self.d[usize::from(reg_origem)] & t.mascara();
        let (antigo, endereco) = match destino {
            Operando::Dados(r) => (self.d[usize::from(r)] & t.mascara(), None),
            Operando::Endereco(_) | Operando::Imediato(_) => {
                return Err(ErroExecucao::OperandoInvalido);
            }
            _ => {
                let (p, inc) = self.endereco(destino, t)?;
                (mem.ler(p, t)? & t.mascara(), Some((p, inc)))
            }
        };
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        use sonic1_core::ops68k;
        let res = match op {
            OperacaoRegistrador::Ou => ops68k::or(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::E => ops68k::and(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Sub => ops68k::sub(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Add => ops68k::add(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Xor => ops68k::eor(t.nativo(), valor, antigo, ccr),
            OperacaoRegistrador::Cmp => {
                self.ccr = ops68k::cmp(t.nativo(), valor, antigo, ccr).para_bits() as u8;
                if let Some((_, Some(r))) = endereco {
                    self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
                }
                return Ok(());
            }
        };
        if let Some((p, _)) = endereco {
            mem.escrever(p, t, res.valor)?;
        } else if let Operando::Dados(r) = destino {
            let d = &mut self.d[usize::from(r)];
            *d = (*d & !t.mascara()) | (res.valor & t.mascara());
        }
        self.ccr = res.ccr.para_bits() as u8;
        if let Some((_, Some(r))) = endereco {
            self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
        }
        Ok(())
    }

    pub fn adda(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        origem: Operando,
        reg: u8,
    ) -> Result<(), ErroExecucao> {
        let valor = self.ler_op(mem, t, origem)?;
        let r = usize::from(reg);
        self.a[r] = sonic1_core::ops68k::adda(t.nativo(), valor, self.a[r]);
        Ok(())
    }

    pub fn suba(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        origem: Operando,
        reg: u8,
    ) -> Result<(), ErroExecucao> {
        let valor = self.ler_op(mem, t, origem)?;
        let r = usize::from(reg);
        self.a[r] = sonic1_core::ops68k::suba(t.nativo(), valor, self.a[r]);
        Ok(())
    }

    pub fn cmpa(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        origem: Operando,
        reg: u8,
    ) -> Result<(), ErroExecucao> {
        let valor = self.ler_op(mem, t, origem)?;
        let r = usize::from(reg);
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        self.ccr = sonic1_core::ops68k::cmpa(t.nativo(), valor, self.a[r], ccr).para_bits() as u8;
        Ok(())
    }

    pub fn clr(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        destino: Operando,
    ) -> Result<(), ErroExecucao> {
        self.escrever_op(mem, t, destino, 0)?;
        self.ccr = (self.ccr & 0x10) | 0x04;
        Ok(())
    }

    pub fn neg(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        destino: Operando,
    ) -> Result<(), ErroExecucao> {
        let (antigo, endereco) = match destino {
            Operando::Dados(r) => (self.d[usize::from(r)] & t.mascara(), None),
            Operando::Endereco(_) | Operando::Imediato(_) => {
                return Err(ErroExecucao::OperandoInvalido);
            }
            _ => {
                let (p, inc) = self.endereco(destino, t)?;
                (mem.ler(p, t)? & t.mascara(), Some((p, inc)))
            }
        };
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        let res = sonic1_core::ops68k::neg(t.nativo(), antigo, ccr);
        if let Some((p, _)) = endereco {
            mem.escrever(p, t, res.valor)?;
        } else if let Operando::Dados(r) = destino {
            let d = &mut self.d[usize::from(r)];
            *d = (*d & !t.mascara()) | (res.valor & t.mascara());
        }
        self.ccr = res.ccr.para_bits() as u8;
        if let Some((_, Some(r))) = endereco {
            self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
        }
        Ok(())
    }

    pub fn ext(&mut self, t: Tamanho, reg: u8) {
        let r = usize::from(reg);
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        let res = sonic1_core::ops68k::ext(t.nativo(), self.d[r], ccr);
        self.d[r] = res.valor;
        self.ccr = res.ccr.para_bits() as u8;
    }

    pub fn deslocar_registrador(
        &mut self,
        tipo: TipoDeslocamento,
        t: Tamanho,
        reg: u8,
        contagem_reg: Option<u8>,
        contagem_imediata: u8,
    ) {
        let contagem = match contagem_reg {
            Some(cr) => self.d[usize::from(cr)] % 64,
            None => u32::from(contagem_imediata),
        };
        let r = usize::from(reg);
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        let res =
            sonic1_core::ops68k::deslocar(tipo.nativo(), t.nativo(), contagem, self.d[r], ccr);
        self.d[r] = (self.d[r] & !t.mascara()) | (res.valor & t.mascara());
        self.ccr = res.ccr.para_bits() as u8;
    }

    pub fn deslocar_memoria(
        &mut self,
        mem: &mut impl Barramento,
        tipo: TipoDeslocamento,
        destino: Operando,
    ) -> Result<(), ErroExecucao> {
        let t = Tamanho::Word;
        let (antigo, endereco) = {
            let (p, inc) = self.endereco(destino, t)?;
            (mem.ler(p, t)? & t.mascara(), (p, inc))
        };
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        let res = sonic1_core::ops68k::deslocar(tipo.nativo(), t.nativo(), 1, antigo, ccr);
        mem.escrever(endereco.0, t, res.valor)?;
        self.ccr = res.ccr.para_bits() as u8;
        if let Some(r) = endereco.1 {
            self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
        }
        Ok(())
    }

    pub fn pea(&mut self, mem: &mut impl Barramento, op: Operando) -> Result<(), ErroExecucao> {
        let (p, _) = self.endereco(op, Tamanho::Long)?;
        self.a[7] = self.a[7].wrapping_sub(4);
        mem.escrever(self.a[7], Tamanho::Long, p)?;
        Ok(())
    }

    pub fn chamar_indireto(
        &mut self,
        mem: &mut impl Barramento,
        op: Operando,
        retorno: u32,
    ) -> Result<(), ErroExecucao> {
        let (alvo, _) = self.endereco(op, Tamanho::Long)?;
        self.chamar(mem, retorno)?;
        self.pc = alvo;
        Ok(())
    }

    pub fn testar(
        &mut self,
        mem: &mut impl Barramento,
        t: Tamanho,
        op: Operando,
    ) -> Result<(), ErroExecucao> {
        let valor = self.ler_op(mem, t, op)?;
        self.flags_move(t, valor);
        Ok(())
    }

    /// BTST só altera a flag Z. Registradores usam 32 bits; memória usa um byte.
    pub fn testar_bit(
        &mut self,
        mem: &mut impl Barramento,
        origem: Operando,
        bit_reg: Option<u8>,
        bit_imediato: u8,
    ) -> Result<(), ErroExecucao> {
        let bit = bit_reg
            .map(|r| self.d[usize::from(r)])
            .unwrap_or(u32::from(bit_imediato));
        let registro = matches!(origem, Operando::Dados(_));
        let t = if registro {
            Tamanho::Long
        } else {
            Tamanho::Byte
        };
        let valor = self.ler_op(mem, t, origem)?;
        let indice = bit % if registro { 32 } else { 8 };
        if valor & (1u32 << indice) == 0 {
            self.ccr |= 4;
        } else {
            self.ccr &= !4;
        }
        Ok(())
    }

    pub fn lea(&mut self, reg: u8, op: Operando) -> Result<(), ErroExecucao> {
        let (p, _) = self.endereco(op, Tamanho::Long)?;
        self.a[usize::from(reg)] = p;
        Ok(())
    }

    pub fn decremento_desvio(&mut self, cond: u8, reg: u8) -> bool {
        if self.condicao(cond) {
            return false;
        }
        let d = &mut self.d[usize::from(reg)];
        let baixo = (*d as u16).wrapping_sub(1);
        *d = (*d & 0xffff_0000) | u32::from(baixo);
        baixo != u16::MAX
    }
    pub fn retornar(&mut self, mem: &mut impl Barramento) -> Result<(), ErroExecucao> {
        self.pc = mem.ler(self.a[7], Tamanho::Long)?;
        self.a[7] = self.a[7].wrapping_add(4);
        Ok(())
    }

    /// Empilha o endereço de retorno antes de transferir o controle.
    pub fn chamar(&mut self, mem: &mut impl Barramento, retorno: u32) -> Result<(), ErroExecucao> {
        self.a[7] = self.a[7].wrapping_sub(4);
        mem.escrever(self.a[7], Tamanho::Long, retorno)?;
        Ok(())
    }

    /// ADDQ/SUBQ nos registradores; em registradores de endereço as flags não mudam.
    pub fn aritmetica_rapida(
        &mut self,
        subtrair: bool,
        t: Tamanho,
        reg: u8,
        endereco: bool,
        valor: u8,
    ) {
        let r = usize::from(reg);
        if endereco {
            self.a[r] = if subtrair {
                self.a[r].wrapping_sub(u32::from(valor))
            } else {
                self.a[r].wrapping_add(u32::from(valor))
            };
            return;
        }
        let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
        let res = if subtrair {
            sonic1_core::ops68k::sub(t.nativo(), u32::from(valor), self.d[r], ccr)
        } else {
            sonic1_core::ops68k::add(t.nativo(), u32::from(valor), self.d[r], ccr)
        };
        self.d[r] = (self.d[r] & !t.mascara()) | (res.valor & t.mascara());
        self.ccr = res.ccr.para_bits() as u8;
    }

    pub fn aritmetica_rapida_op(
        &mut self,
        mem: &mut impl Barramento,
        subtrair: bool,
        t: Tamanho,
        destino: Operando,
        valor: u8,
    ) -> Result<(), ErroExecucao> {
        match destino {
            Operando::Endereco(r) => {
                let r = usize::from(r);
                self.a[r] = if subtrair {
                    self.a[r].wrapping_sub(u32::from(valor))
                } else {
                    self.a[r].wrapping_add(u32::from(valor))
                };
                Ok(())
            }
            Operando::Dados(r) => {
                self.aritmetica_rapida(subtrair, t, r, false, valor);
                Ok(())
            }
            _ => {
                let (antigo, endereco) = {
                    let (p, inc) = self.endereco(destino, t)?;
                    (mem.ler(p, t)? & t.mascara(), (p, inc))
                };
                let ccr = sonic1_core::ops68k::Ccr::do_sr(u16::from(self.ccr));
                let res = if subtrair {
                    sonic1_core::ops68k::sub(t.nativo(), u32::from(valor), antigo, ccr)
                } else {
                    sonic1_core::ops68k::add(t.nativo(), u32::from(valor), antigo, ccr)
                };
                mem.escrever(endereco.0, t, res.valor)?;
                self.ccr = res.ccr.para_bits() as u8;
                if let Some(r) = endereco.1 {
                    self.a[usize::from(r)] = self.a[usize::from(r)].wrapping_add(t.n(r));
                }
                Ok(())
            }
        }
    }
    pub fn condicao(&self, cond: u8) -> bool {
        let c = self.ccr & 1 != 0;
        let v = self.ccr & 2 != 0;
        let z = self.ccr & 4 != 0;
        let n = self.ccr & 8 != 0;
        match cond {
            0 => true,
            2 => !c && !z,
            3 => c || z,
            4 => !c,
            5 => c,
            6 => !z,
            7 => z,
            8 => !v,
            9 => v,
            10 => !n,
            11 => n,
            12 => n == v,
            13 => n != v,
            14 => !z && n == v,
            15 => z || n != v,
            _ => false,
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn move_memoria_e_ajuste_de_pilha_em_byte() {
        let mut m = Memoria::nova(vec![0; 32]);
        let mut e = Estado {
            ccr: 0x10,
            ..Estado::default()
        };
        e.a[7] = 0xff0000;
        e.d[0] = 0x12340080;
        e.mov(
            &mut m,
            Tamanho::Byte,
            Operando::Dados(0),
            Operando::PosIncremento(7),
        )
        .unwrap();
        assert_eq!(
            Barramento::ler(&mut m, 0xff0000, Tamanho::Byte).unwrap(),
            0x80
        );
        assert_eq!(e.a[7], 0xff0002);
        assert_eq!(e.ccr, 0x18);
        e.d[1] = 0x12345678;
        e.mov(
            &mut m,
            Tamanho::Word,
            Operando::PreDecremento(7),
            Operando::Dados(1),
        )
        .unwrap();
        assert_eq!(e.a[7], 0xff0000);
        assert_eq!(e.d[1], 0x12348000);
        assert_eq!(e.ccr, 0x18);
    }

    #[test]
    fn movea_word_estende_sinal_e_preserva_ccr() {
        let mut m = Memoria::nova(vec![0; 32]);
        let mut e = Estado {
            ccr: 0x1f,
            ..Estado::default()
        };
        e.mov(
            &mut m,
            Tamanho::Word,
            Operando::Imediato(0xff80),
            Operando::Endereco(2),
        )
        .unwrap();
        assert_eq!(e.a[2], 0xffff_ff80);
        assert_eq!(e.ccr, 0x1f);
    }

    #[test]
    fn erro_de_alinhamento_interrompe_move() {
        let mut m = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.a[0] = 0xff0001;
        assert_eq!(
            e.mov(
                &mut m,
                Tamanho::Word,
                Operando::Indireto(0),
                Operando::Dados(0)
            ),
            Err(ErroExecucao::Acesso(ErroAcesso::EnderecoImpar(0xff0001)))
        );
    }

    #[test]
    fn movem_word_extende_sinal_e_pos_incrementa() {
        let mut m = Memoria::nova(vec![0; 32]);
        for (p, v) in [(0xff0000, 0x8000), (0xff0002, 0x1234), (0xff0004, 0xffff)] {
            Barramento::escrever(&mut m, p, Tamanho::Word, v).unwrap();
        }
        let mut e = Estado {
            ccr: 0x1f,
            ..Estado::default()
        };
        e.a[5] = 0xff0000;
        e.movem(
            &mut m,
            Tamanho::Word,
            true,
            Operando::PosIncremento(5),
            0x00e0,
        )
        .unwrap();
        assert_eq!(&e.d[5..8], &[0xffff8000, 0x1234, 0xffff_ffff]);
        assert_eq!(e.a[5], 0xff0006);
        assert_eq!(e.ccr, 0x1f);
    }

    #[test]
    fn movem_predecrementa_na_ordem_do_68000() {
        let mut m = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.a[7] = 0xff0100;
        e.d[0] = 0x11223344;
        e.movem(
            &mut m,
            Tamanho::Long,
            false,
            Operando::PreDecremento(7),
            0x8001,
        )
        .unwrap();
        assert_eq!(e.a[7], 0xff00f8);
        assert_eq!(
            Barramento::ler(&mut m, 0xff00fc, Tamanho::Long).unwrap(),
            0xff0100
        );
        assert_eq!(
            Barramento::ler(&mut m, 0xff00f8, Tamanho::Long).unwrap(),
            0x11223344
        );
    }

    #[test]
    fn dbf_preserva_palavra_alta_e_ccr() {
        let mut e = Estado {
            ccr: 0x1f,
            ..Estado::default()
        };
        e.d[1] = 0x1234_0001;
        assert!(e.decremento_desvio(1, 1));
        assert_eq!(e.d[1], 0x1234_0000);
        assert!(!e.decremento_desvio(1, 1));
        assert_eq!(e.d[1], 0x1234_ffff);
        assert_eq!(e.ccr, 0x1f);
    }

    #[test]
    fn interrupcao_e_rte_preservam_pc_sr_e_as_duas_pilhas() {
        let mut rom = vec![0; 0x100];
        rom[0x78..0x7c].copy_from_slice(&0x0000_1234u32.to_be_bytes());
        let mut mem = Memoria::nova(rom);
        let mut e = Estado {
            pc: 0x3456,
            ccr: 0x15,
            sr_alto: 0,
            ssp: 0xff2000,
            ..Estado::default()
        };
        e.a[7] = 0xff1000;
        assert!(e.interrupcao_autovetor(&mut mem, 6).unwrap());
        assert_eq!(e.pc, 0x1234);
        assert_eq!(e.usp, 0xff1000);
        assert_eq!(e.a[7], 0xff1ffa);
        assert_eq!(e.sr_alto & 0x27, 0x26);
        assert_eq!(
            Barramento::ler(&mut mem, 0xff1ffa, Tamanho::Word).unwrap(),
            0x15
        );
        assert_eq!(
            Barramento::ler(&mut mem, 0xff1ffc, Tamanho::Long).unwrap(),
            0x3456
        );
        e.retornar_interrupcao(&mut mem).unwrap();
        assert_eq!(e.pc, 0x3456);
        assert_eq!(e.a[7], 0xff1000);
        assert_eq!(e.ssp, 0xff2000);
        assert_eq!(e.ccr, 0x15);
        assert_eq!(e.sr_alto, 0);
    }

    #[test]
    fn interrupcao_mascarada_nao_muda_estado() {
        let mut mem = Memoria::nova(vec![0; 256]);
        let mut e = Estado::default();
        let antes = e.clone();
        assert!(!e.interrupcao_autovetor(&mut mem, 6).unwrap());
        assert_eq!(e, antes);
        assert_eq!(
            e.interrupcao_autovetor(&mut mem, 0),
            Err(ErroExecucao::NivelInterrupcaoInvalido)
        );
    }

    #[test]
    fn clr_zera_e_ajusta_ccr() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado {
            ccr: 0x1f,
            ..Estado::default()
        };
        e.d[2] = 0x12345678;
        e.clr(&mut mem, Tamanho::Word, Operando::Dados(2)).unwrap();
        assert_eq!(e.d[2], 0x12340000);
        assert_eq!(e.ccr, 0x14);

        e.a[0] = 0xff0004;
        Barramento::escrever(&mut mem, 0xff0004, Tamanho::Long, 0xdeadbeef).unwrap();
        e.clr(&mut mem, Tamanho::Long, Operando::Indireto(0))
            .unwrap();
        assert_eq!(
            Barramento::ler(&mut mem, 0xff0004, Tamanho::Long).unwrap(),
            0
        );
        assert_eq!(e.ccr, 0x14);
    }

    #[test]
    fn neg_inverte_sinal_e_ajusta_ccr() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.d[0] = 5;
        e.neg(&mut mem, Tamanho::Word, Operando::Dados(0)).unwrap();
        assert_eq!(e.d[0] & 0xffff, 0xfffb);
        assert_ne!(e.ccr & 8, 0);

        e.a[1] = 0xff0010;
        Barramento::escrever(&mut mem, 0xff0010, Tamanho::Byte, 0x01).unwrap();
        e.neg(&mut mem, Tamanho::Byte, Operando::Indireto(1))
            .unwrap();
        assert_eq!(
            Barramento::ler(&mut mem, 0xff0010, Tamanho::Byte).unwrap(),
            0xff
        );
    }

    #[test]
    fn ext_estende_com_sinal() {
        let mut e = Estado::default();
        e.d[0] = 0x1234_0080;
        e.ext(Tamanho::Word, 0);
        assert_eq!(e.d[0], 0x1234_ff80);
        assert_ne!(e.ccr & 8, 0);

        e.d[1] = 0x0000_8000;
        e.ext(Tamanho::Long, 1);
        assert_eq!(e.d[1], 0xffff_8000);
        assert_ne!(e.ccr & 8, 0);

        e.d[2] = 0x0000_007f;
        e.ext(Tamanho::Word, 2);
        assert_eq!(e.d[2], 0x007f);
        assert_eq!(e.ccr & 8, 0);
    }

    #[test]
    fn operacoes_para_memoria_e_xor() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.d[0] = 0x10;
        e.a[2] = 0xff0000;
        Barramento::escrever(&mut mem, 0xff0000, Tamanho::Byte, 0x20).unwrap();
        e.operacao_para_memoria(
            &mut mem,
            OperacaoRegistrador::Add,
            Tamanho::Byte,
            0,
            Operando::PosIncremento(2),
        )
        .unwrap();
        assert_eq!(
            Barramento::ler(&mut mem, 0xff0000, Tamanho::Byte).unwrap(),
            0x30
        );
        assert_eq!(e.a[2], 0xff0001);

        e.d[1] = 0x0f;
        e.operacao_para_memoria(
            &mut mem,
            OperacaoRegistrador::Xor,
            Tamanho::Byte,
            1,
            Operando::Dados(0),
        )
        .unwrap();
        assert_eq!(e.d[0], 0x1f);
    }

    #[test]
    fn adda_suba_cmpa_enderecamento() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado {
            ccr: 0x15,
            ..Estado::default()
        };
        e.a[0] = 0x1000;
        e.d[0] = 0xffff_ff80;
        e.adda(&mut mem, Tamanho::Word, Operando::Dados(0), 0)
            .unwrap();
        assert_eq!(e.a[0], 0x1000u32.wrapping_add(0xffff_ff80));
        assert_eq!(e.ccr, 0x15);

        e.suba(&mut mem, Tamanho::Long, Operando::Imediato(0x50), 0)
            .unwrap();
        assert_eq!(
            e.a[0],
            0x1000u32.wrapping_add(0xffff_ff80).wrapping_sub(0x50)
        );
        assert_eq!(e.ccr, 0x15);

        e.cmpa(&mut mem, Tamanho::Long, Operando::Imediato(e.a[0]), 0)
            .unwrap();
        assert_ne!(e.ccr & 4, 0);
    }

    #[test]
    fn aritmetica_rapida_op_memoria() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.a[3] = 0xff0008;
        Barramento::escrever(&mut mem, 0xff0008, Tamanho::Word, 0x100).unwrap();
        e.aritmetica_rapida_op(&mut mem, false, Tamanho::Word, Operando::Indireto(3), 2)
            .unwrap();
        assert_eq!(
            Barramento::ler(&mut mem, 0xff0008, Tamanho::Word).unwrap(),
            0x102
        );

        e.aritmetica_rapida_op(&mut mem, true, Tamanho::Word, Operando::Indireto(3), 5)
            .unwrap();
        assert_eq!(
            Barramento::ler(&mut mem, 0xff0008, Tamanho::Word).unwrap(),
            0x0fd
        );
    }

    #[test]
    fn deslocamentos_e_rotacoes() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.d[0] = 0x01;
        e.deslocar_registrador(TipoDeslocamento::Lsl, Tamanho::Byte, 0, None, 3);
        assert_eq!(e.d[0] & 0xff, 0x08);

        e.d[1] = 0x80;
        e.deslocar_registrador(TipoDeslocamento::Asr, Tamanho::Byte, 1, None, 2);
        assert_eq!(e.d[1] & 0xff, 0xe0);

        e.d[2] = 0x8000_0001;
        e.deslocar_registrador(TipoDeslocamento::Rol, Tamanho::Long, 2, None, 1);
        assert_eq!(e.d[2], 0x0000_0003);

        e.a[4] = 0xff0010;
        Barramento::escrever(&mut mem, 0xff0010, Tamanho::Word, 0x0004).unwrap();
        e.deslocar_memoria(&mut mem, TipoDeslocamento::Asl, Operando::Indireto(4))
            .unwrap();
        assert_eq!(
            Barramento::ler(&mut mem, 0xff0010, Tamanho::Word).unwrap(),
            0x0008
        );
    }

    #[test]
    fn pea_empilha_endereco() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.a[7] = 0xff001c;
        e.a[0] = 0xff0000;
        e.pea(&mut mem, Operando::Deslocamento(0, 8)).unwrap();
        assert_eq!(e.a[7], 0xff0018);
        assert_eq!(
            Barramento::ler(&mut mem, 0xff0018, Tamanho::Long).unwrap(),
            0xff0008
        );
    }

    #[test]
    fn chamar_indireto_empilha_e_salta() {
        let mut mem = Memoria::nova(vec![0; 32]);
        let mut e = Estado::default();
        e.a[7] = 0xff0010;
        e.a[1] = 0x1234;
        e.chamar_indireto(&mut mem, Operando::Indireto(1), 0x5678)
            .unwrap();
        assert_eq!(e.pc, 0x1234);
        assert_eq!(e.a[7], 0xff000c);
        assert_eq!(
            Barramento::ler(&mut mem, 0xff000c, Tamanho::Long).unwrap(),
            0x5678
        );
    }
}
