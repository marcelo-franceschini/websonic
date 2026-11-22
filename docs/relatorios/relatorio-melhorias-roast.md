# Relatorio de Auditoria e Melhorias: WebSonic (Roast & Rebuild)

> **Registro historico (arquivado):** esta auditoria foi gerada em 17/11/2026 e foi superada por alteracoes posteriores no repositorio (incluindo suporte a cabecalho e desintercalacao SMD, `#![forbid(unsafe_code)]` em `sonic1-web`, pipeline WebGL2, AudioWorklet em Web Worker e otimizacoes de VDP/FM). Mantida apenas para historico.

**Auditor:** AntiGravity Technical Reviewer  
**Metodologia:** Roast & Rebuild (`janderswag/roast-skill`)  
**Data:** 17 de Novembro de 2026  
**Classificacao do Projeto (Phase 0.5):** Motor Nativo Recompilado em Rust + Frontend WebAssembly, WebGL2 e Web Audio.  
**Nota Geral do Sistema:** **8.1 / 10** (Engenharia de nucleo e fidelidade excepcionais, com tetos de escala na integracao de tempo e perifericos web).

---

## Modulo 0: The Roast (O Choque de Realidade)

> *"Voce passou centenas de horas traduzindo meticulosamente 30 mil linhas de assembly 68000 para Rust estrito sem um unico float, eliminou diferencas de clock com um oraculo pixel a pixel, montou um sintetizador FM bit a bit identico ao silicio da Yamaha... e ai colocou um loop JavaScript que ficava copiando quase um megabyte de buffers de textura a 60 quadros por segundo, fazendo o coletor de lixo do V8 engasgar enquanto o AudioWorklet cortava o som a cada milissegundo de oscilacao de frame rate. E como se voce tivesse construido um motor de Formula 1 e o acoplasse as rodas usando elastico de escritorio. O nucleo em Rust e uma obra de arte de determinismo e rigor de tipos, mas a cola de frontend parecia um projeto de feira de ciencias feito na pressa antes da meia-noite. A boa noticia: a base em Rust e tao solida que, assim que tiramos o peso do lixo ciclico do navegador e calibramos os buffers, o jogo finalmente roda como o Sonic que conhecemos."*

---

## Modulo 1: Seguranca, Integridade da ROM e Fronteira Web

**Pontuacao do Modulo:** **9.2 / 10**

### O que esta excelente
- **Barreira de Ingestao Local e Anti-Vazamento:** Conforme auditado em `crates/rom/src/lib.rs:58-87`, a verificacao de integridade e estritamente local no navegador. A ROM nao e transmitida, armazenada em servidor nem propagada via rede. O veredito por SHA-1 (`TAMANHO_REV01` em 524.288 bytes e hash `69e102855d4389c3fd1a8f3dc7d193f8eee5fe5b`) bloqueia versoes modificadas ou corrompidas de contaminar o estado do motor.
- **Politica de Hooks e CI Anti-ROM:** Em `xtask/src/anti_rom.rs:13-31` e `hooks/lib.sh`, ha varredura preventiva de historico (`git rev-list --objects --all`) impedindo que binarios protegidos por direitos autorais ou traces proprietarios entrem no git.
- **Politica de Seguranca de Conteudo (CSP):** Em `site/index.html:7-8`, a declaracao `default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; connect-src 'self'` isola a pagina contra qualquer injecao externa de scripts ou exfiltracao de dados.

### Pontos Criticos e Vulnerabilidades de Usabilidade
1. **Rejeicao Inflexivel de Cabecalhos de Copiador (Copier Headers):**
   - *Citacao:* `crates/rom/src/lib.rs:72-76`
   - *Evidencia:* Se o arquivo fornecido pelo usuario for um dump legitimo extraido de copiador SMD / Super Magic Drive contendo o cabecalho tradicional de 512 bytes (524.800 bytes em vez de 524.288), o validador falha imediatamente com `Motivo::TamanhoDiferente`.
   - *Impacto:* Rejeicao de dumps legitimos de cartuchos sem orientacao ao usuario.
   - *Acao Recomendada:* Detectar se o arquivo possui exatamente 524.800 bytes e os primeiros 512 bytes correspondem a cabecalho de copiador, ou se esta em formato intercalado SMD (pares/impares), descartando o cabecalho automaticamente antes de submeter ao SHA-1.

2. **Politica Estrita de Unsafe no Wasm:**
   - *Citacao:* `crates/web/src/lib.rs:1` e `Cargo.toml:9-11`
   - *Evidencia:* O crate `sonic1-web` utiliza `#![forbid(unsafe_code)]` na raiz dos crates auxiliares. A integridade de memoria do Wasm e garantida pelo runtime, sem ponteiros soltos ou risco de corrupcao fora da tabela linear exportada.

---

## Modulo 2: Arquitetura e Tetos de Escala (Scale-Ceilings)

**Pontuacao do Modulo:** **7.8 / 10**

### 1. Risco de "Espiral da Morte" no Acumulador de Tempo NTSC
- **Citacao:** `site/relogio.js:21-28`
- **Evidencia:**
  ```javascript
  avancar(agora) {
    this.acumulado += Math.max(0, agora - this.ultimo);
    this.ultimo = agora;
    const passos = Math.min(4, Math.floor(this.acumulado / PERIODO_QUADRO));
    this.acumulado -= passos * PERIODO_QUADRO;
    this.alpha = Math.max(0, Math.min(1, this.acumulado / PERIODO_QUADRO));
    return passos;
  }
  ```
- **Problema:** Quando a aba do navegador sofre uma lentidao momentanea (e.g. aba oculta ou gargalo de CPU do sistema), `this.acumulado` cresce. Como os passos sao limitados a 4 (`Math.min(4, ...)`), se o atraso for superior a 66 ms (4 quadros), o tempo residual continua retido em `this.acumulado`. No quadro seguinte, o loop sera forcado a executar 4 quadros pesados novamente, criando uma espiral de lentidao continua da qual a maquina nunca se recupera.
- **Acao Recomendada:** Introduzir um teto rigido de descarte (`clamp`) para `this.acumulado` em no maximo 4 quadros (`this.acumulado = Math.min(this.acumulado, PERIODO_QUADRO * 4)`), descartando tempo irrecuperavel para priorizar a responsividade em tempo real.

### 2. Alocacao Ciclica Residual no Buffer de Audio do Console
- **Citacao:** `crates/console/src/lib.rs:438-444`
- **Evidencia:**
  ```rust
  let disp = self.misturador.ocupacao_buffer();
  if disp > 0 {
      let mut bloco = vec![0.0f32; disp.wrapping_mul(2)];
      self.misturador.extrair_bloco_worklet(&mut bloco);
      self.buffer_audio_quadro = bloco;
  }
  ```
- **Problema:** A cada quadro da simulacao (60 vezes por segundo), `vec![0.0f32; disp * 2]` aloca um novo vetor no heap do Rust para extrair as amostras do misturador.
- **Acao Recomendada:** Manter a capacidade de `self.buffer_audio_quadro` viva entre chamadas com `self.buffer_audio_quadro.resize(disp * 2, 0.0)` em vez de instanciar um novo `Vec` por quadro.

### 3. Modelo Nominal de Esperas Calibrado por Tabela Estatica
- **Citacao:** `crates/console/src/lib.rs:22-46`
- **Evidencia:** O atraso de carregamento de zonas (`Marca::LevelDelay => 25` quadros de lag, `Marca::LevelTtlCardLoop => 2`, `Marca::ChecksumOk => 49`) foi empiricamente calibrado contra o oraculo.
- **Consequencia:** Esse modelo e perfeito e deterministico para o Sonic 1 REV01 padrao, mas cria um teto rigido caso o projeto no futuro suporte hacks, selecao dinamica de fases ou fases personalizadas onde o tempo de descompressao Kosinski/Enigma varie.

---

## Modulo 3: Fluxo do Jogador e Ergonomia da Interface

**Pontuacao do Modulo:** **8.5 / 10**

### O que foi corrigido e elevado
- **Retencao de Foco e Mapeamento de Teclas:** O `<canvas id="tela">` agora possui `tabindex="0"` e chamada automatica de `tela.focus()` ao clicar. A utilizacao de `e.code` (`site/app.js:295-340`) resolveu as falhas em teclados ABNT2/internacionais, CapsLock e padronizou as combinacoes gamer (WASD, Setas, Z/X/C, J/K/L).
- **Interface Arcade Retro-Moderna:** A interface em `site/index.html` evoluiu de uma pagina crua para um layout escuro, elegante, com zona interativa de drag-and-drop, keycaps com iluminacao em tempo real, controle de volume mestre com botao de mudo e filtro opcional de scanlines CRT.

### Ponto de Melhoria no Monitoramento de Telemetria
- **Calculo de Pico de Volume em Segundo Plano:**
  - *Citacao:* `site/app.js:460-475`
  - *Evidencia:* O laco `for (let i = 0; i < audio.length; i++) { const abs = Math.abs(audio[i]); ... }` percorre ~1600 floats a cada quadro em JavaScript para computar o VU meter.
  - *Problema:* Quando o painel de telemetria esta recolhido (`painelDiag.hidden === true`), esse calculo e um desperdicio desnecessario de ciclos de CPU no thread principal.
  - *Acao Recomendada:* Envolver o calculo de `picoQuadro` e `picoHistorico` em uma condicional simples: `if (painelDiag && !painelDiag.hidden)`.

---

## Modulo 4: Prontidao de Performance e Compatibilidade de Hardware

**Pontuacao do Modulo:** **8.7 / 10**

### 1. Renderizacao WebGL2 e Shaders
- **Citacao:** `site/gpu.js:13-63` (Fragment Shader)
- **Analise:** O shader fragmentario executa uma busca de prioridade em 20 slots de sprites com `for (int i = 0; i < 20; i++) { if (i >= quantidadeSprites) break; ... }`.
- **Comportamento em GPUs:** Em computadores desktop, 95.000 pixels (426x224) sao processados em menos de 0.1 ms. Contudo, em dispositivos moveis (smartphones com GPUs Mali ou Adreno), lacos com desvios dinamicos (`break` dependente de dados) dentro do fragment shader podem desativar a execucao paralela em bloco (warp divergente).
- **Otimizacao Aplicada:** A substituicao de `gl.texImage2D` por `gl.texSubImage2D` em `site/gpu.js:100-110` eliminou a destruicao de texturas na VRAM, resolvendo a causa-raiz dos travamentos graficos relatados.

### 2. AudioWorklet e Sincronismo de Baixa Latencia
- **Citacao:** `site/audio-worklet.js:87-95`
- **Analise:** O envio de telemetria a cada 6 chamadas (~16.6 ms) coincide exatamente com o ritmo de 60 Hz do motor. A fila circular de 16.384 amostras (~340 ms) oferece margem contra cortes, enquanto o alvo do misturador (`crates/audio-saida/src/misturador.rs:52`) opera em 3 quadros nominais (~50 ms), garantindo resposta agil aos efeitos sonoros de pulo e molas.

---

## Modulo 5: Founder & Lead Maintainer Briefing (Prioridades Executivas)

### Top 4 Acoes Prioritarias Recomendadas

| # | Prioridade | Arquivo / Componente | Descricao e Impacto | Complexidade |
|---|------------|----------------------|---------------------|:------------:|
| **1** | **Trava Anti-Espiral de Tempo** | `site/relogio.js:21-28` | Limitar `this.acumulado` a 4 quadros para evitar a espiral da morte quando o navegador sofrer congelamento momentaneo de sistema. | Minima (2 linhas) |
| **2** | **Otimizacao do VU Meter Oculto** | `site/app.js:460-475` | Condicionar o calculo de `Math.abs` nas amostras de audio apenas quando o painel de diagnostico estiver visivel. | Minima (1 linha) |
| **3** | **Zero-Allocation no Buffer de Audio Rust** | `crates/console/src/lib.rs:441` | Reutilizar a capacidade do `Vec<f32>` em `buffer_audio_quadro` via `resize`, eliminando a ultima alocacao repetitiva por frame no Rust. | Baixa (3 linhas) |
| **4** | **Suporte a Dumps com Cabecalho SMD** | `crates/rom/src/lib.rs:70-80` | Ignorar cabecalhos de 512 bytes automaticamente para aceitar dumps de cartucho populares sem erros de tamanho. | Baixa (10 linhas) |

---

## Conclusao da Auditoria

O projeto WebSonic possui uma fundacao de software extraordinariamente robusta. O trabalho de recompilacao estrita em Rust sem ponto flutuante e a comprovacao de determinismo via oraculo colocam o nucleo entre as implementacoes mais fieis e elegantes existentes.

Com as otimizacoes recentes de zero-copy e `texSubImage2D`, os gargalos criticos de frontend foram solucionados. A implementacao das quatro vitorias rapidas descritas no Modulo 5 colocara a experiencia de execucao em nivel de emulador nativo de console em qualquer navegador moderno.
