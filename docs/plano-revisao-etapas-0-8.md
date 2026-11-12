# Plano de revisão das etapas 0 a 8 do WebSonic

Escrito na fase 1 (modo plan, somente leitura). Nada foi executado. "Confirmado" quer dizer que eu conferi no repositório. "Subagente" quer dizer que vem dos relatórios de leitura. Cada item reconfere a evidência antes de mudar qualquer coisa.

## 1. Topo

- Modelo da sessão: claude-sonnet-5-5 (Sonnet 5.5), modo plan, esforço max.
- Total: **51 itens**. São 32 de IA, 5 de PATCH e 14 do MARCELO. Há mais 5 itens de IA, opcionais (bloco O7: R7.06 a R7.10), que só valem com a aprovação do R7.03.
- Itens do MARCELO não vão para subagente. Cada um traz o comando exato, quando existe.
- Contêiner: **NÃO**. Não existe `/.dockerenv`. O repositório está em `/ssd400/Repos/websonic`, não em `/workspaces/websonic`. **A execução precisa ser feita no contêiner.**
- Decisão L1: **NÃO ativa**. O teste 1 falhou. Falta o passo 1 de "Ainda não vale" em `docs/estado.md`, e depois os passos 2, 3 e 4.
- Fase 2: bloqueada. A seção 3 do prompt exige contêiner e L1. A seção 2 abaixo diz o que falta e propõe dois caminhos.
- Árvore limpa: `git status --porcelain` vazio. HEAD `f7aa604`. A `main` está 161 commits à frente de `origin/main` (`05576b5`), confirmado.
- Divisão: 51 itens é menos que 60. Mesmo assim recomendo cinco sessões, uma por bloco (seção 4), porque são cerca de 70 chamadas de subagente.

## 2. O que falta para liberar a fase 2

### 2.1 Testes da seção 0.3 do prompt

| Teste | Resultado | Evidência |
| --- | --- | --- |
| 1. o hook chama o detector | FALHOU | `test -f hooks/detector/Cargo.toml`: não existe. `grep -q 'hooks/detector' hooks/pre-commit`: nada |
| 2. ROM achada e legível | NÃO RODADO | O teste 1 já falhou, então não procurei a ROM |
| 3. controles do detector | NÃO RODADO | O controle positivo copia 64 bytes da ROM. A regra proíbe ler a ROM quando um teste falha. O detector também não existe |

Também confirmado: `.devcontainer/managed-settings.json` ainda nega `Read(//mnt/rom...)` e `Read(//mnt/disasm...)` (linhas 9 a 12) e tem `denyRead` (linhas 82 a 85).

### 2.2 Passos de "Ainda não vale" (docs/estado.md) que faltam

1. Aplicar `l1-hook-detector.patch` e `l1-hook-pre-commit.patch`. Os dois aplicam limpo no HEAD (`git apply --check`), mas não foram aplicados. Depois rodar `sh .devcontainer/testes/hooks.sh` e conferir 14 de 14.
2. Rodar os dois controles do detector com a ROM real, no host (comandos na seção 8).
3. Aplicar `l1-permissoes-rom.patch` (aplica limpo), reconstruir a imagem e confirmar que a IA lê a ROM.
4. Levar a decisão ao chat de planejamento e exportar `docs/requisitos.md` de novo.

### 2.3 Defeitos dos patches da L1 (confirmados lendo os patches, o managed-settings.json e o .cargo/config.toml)

1. **Crítico (R0.02).** O hook roda `cargo run --manifest-path hooks/detector/Cargo.toml` sem `--target-dir`, e o patch do detector não traz `Cargo.lock`. No contêiner `hooks/` é somente leitura (managed-settings.json: regra Edit e `denyWrite` de `hooks`). O cargo não grava `target/` nem o lock. Todo commit da IA seria barrado, com a mensagem "trecho copiado da ROM". O alias `fidelidade` de `.cargo/config.toml` já registra a mesma lição (o comentário diz que a pasta congelada não aceita escrita do compilador). Não há `CARGO_TARGET_DIR` em `devcontainer.json` nem no `Dockerfile`. A verificação feita em 11/11 foi num worktree do host, que é gravável.
2. **Alto (R0.01).** O detector depende de `../../crates/rom` (Cargo.toml do detector, linha 18). A IA edita `crates/rom`. O comentário do patch e o texto de `docs/estado.md` ("Por que o detector fica em hooks/") dizem que a guarda não depende do código vigiado.
3. **Médio (R0.03).** `numero_em` descarta número seguido de letra, então `0x12u8` e `12u16` não contam, e o teste `numero_colado_a_letra_nao_conta` fixa isso. `listas` encerra a lista em `(`, `[`, `{`, `$` e `-`. Hexadecimal sem `0x` e palavras de 32 bits não são lidos. Tabela copiada nesses formatos passa.
4. **Decisão (R0.04).** `hooks/pre-push` não chama o detector. `docs/conhecimento/hooks.md:39` diz que o pre-push é a barreira contra `--no-verify`. Para conteúdo, hoje não existe segunda camada. Commit por plumbing (`git commit-tree`), `merge` sem conflito e `git am` não passam pelo pre-commit.

### 2.4 Dois caminhos

**Caminho A (mínimo, à mão pelo Marcelo).** Depois de aplicar os dois patches do passo 1:

1. Editar `hooks/pre-commit` e trocar a linha do `cargo run` por: `cargo run --quiet --locked --manifest-path "$raiz/hooks/detector/Cargo.toml" --target-dir "$raiz/target/detector" -- --rom "$rom" ||`
2. Rodar `cargo generate-lockfile --manifest-path hooks/detector/Cargo.toml`.
3. Commitar `hooks/pre-commit` e `hooks/detector/Cargo.lock`.

Os itens R0.01 a R0.03 ficam para depois.

**Caminho B (recomendado).** Ao aprovar este plano, o Marcelo autoriza o **bloco P** (R0.01 a R0.03). É uma sessão só de texto, no host, sem ler a ROM e sem executar o resto. Ela cria `docs/pedidos/rev-R0.01.patch` a `rev-R0.03.patch`. Depois o Marcelo aplica os patches da L1, os três `rev`, e segue os passos 1 a 4. Isto exige autorização explícita, porque a seção 3 do prompt pede contêiner e L1 para a fase 2.

Sem um dos dois caminhos, o primeiro commit da IA no contêiner seria barrado pelo hook.

### 2.5 Decisões pedidas na aprovação

Itens que dependem de decisão ficam BLOQUEADOS se o Marcelo não decidir: R6.03 (precisa de R6.01), R7.06 a R7.10 (precisam de R7.03) e R8.02 (precisa de R8.01). Recomendações:

- Caminho: B.
- R0.04: sim, rodar o detector também no pre-push (a IA gera o patch num plano revisado).
- R1.02: opção (a) agora, com o registro do risco, e opção (c) antes da auditoria final (Etapa 14). A opção (b) congelaria `memoria/mod.rs`, que as Etapas 9 a 12 precisam editar.
- R6.01: aceitar a máquina do Z80 como "máquina nativa por programa" e registrar em `som.md`.
- R6.02: depende do que o detector achar depois da L1.
- R7.03: sim, autorizar o bloco O7, mas só depois de R7.01 e R7.02.
- R8.01: decidir a licença antes de qualquer outra coisa do `crates/audio`.
- R8.11: aceitar o PCM no worklet e atualizar o requisito na fonte.

## 3. Resumo

| Etapa | Itens | IA | PATCH | MARCELO | crítica | alta | média | baixa |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 8 | 3 | 3 | 2 | 1 | 2 | 4 | 1 |
| 1 | 2 | 1 | 0 | 1 | 0 | 0 | 2 | 0 |
| 2 | 7 | 4 | 1 | 2 | 0 | 3 | 4 | 0 |
| 3 | 3 | 2 | 0 | 1 | 2 | 0 | 0 | 1 |
| 4 | 3 | 2 | 0 | 1 | 2 | 1 | 0 | 0 |
| 5 | 6 | 5 | 0 | 1 | 0 | 0 | 5 | 1 |
| 6 | 5 | 2 | 1 | 2 | 0 | 1 | 4 | 0 |
| 7 | 5 | 4 | 0 | 1 | 0 | 1 | 4 | 0 |
| 8 | 12 | 9 | 0 | 3 | 2 | 4 | 5 | 1 |
| **Total** | **51** | **32** | **5** | **14** | **7** | **12** | **28** | **4** |

Por tipo: corrigir 15, melhorar 12, implementar 11, documentar 13. Itens "implementar" do MARCELO são rodadas com a ROM ou passos de host. Itens "documentar" do MARCELO são decisões. Mais 5 itens opcionais de IA do tipo melhorar (R7.06 a R7.10).

Prioridade "crítica" quer dizer que uma frase do "Pronto quando" fica falsa hoje, ou que a fase 2 não consegue commitar.

## 4. Ordem de execução em blocos

Dentro de cada bloco os itens seguem esta ordem. Itens do MARCELO aparecem entre colchetes e não bloqueiam a IA, salvo onde a coluna "Depende de" disser.

- **Bloco P (pré-L1, só texto):** R0.01, R0.02, R0.03.
- **Bloco 1 (etapas 0 a 2):** R0.06, R0.05, R0.07, R1.01, R2.01, R2.02, R2.05, R2.04, R2.06. [R0.04, R0.08, R1.02, R2.03, R2.07]
- **Bloco 2 (etapas 3 a 5):** R3.01, R3.03, R4.01, R4.02, R5.01, R5.04, R5.02, R5.05, R5.06. [R3.02, R4.03, R5.03]
- **Bloco 3 (etapas 6 e 7):** R6.03, R6.04, R6.05, R7.01, R7.02, R7.04, R7.05, depois o bloco O7 (R7.06 a R7.10) se aprovado. [R6.01, R6.02, R7.03]
- **Bloco 4 (etapa 8):** R8.03, R8.04, R8.05, R8.07, R8.08, R8.09, R8.10, R8.06, R8.02. [R8.01, R8.11, R8.12]
- **Fechamento:** revisão final (Explore), documento (Haiku), `docs/estado.md`.
- A assinatura única (R2.07) e a rodada final do Marcelo (R8.12) ficam por último, depois de todos os itens que mudam caminho coberto pelo manifesto.

Arquivos compartilhados, que impõem a ordem: `site/app.js` (R0.06, R7.05, R8.05); `crates/console/src/lib.rs` e `crates/web/src/lib.rs` (R7.05, R8.05); `crates/core/src/abertura/objetos.rs` (R7.01, R7.02, R7.06); `scripts/etapa4-host.sh` (R0.07, R4.02); `scripts/etapa8-bloco2.sh` (R0.07, R8.08); `diferencial-sintese/src/bin/sintese.rs` (R8.08, R8.10); `crates/audio-saida/tests/saida.rs` (R8.04, R8.10); `docs/estado.md` e `docs/backlog.md` (R2.06, R5.06, R8.06); `docs/conhecimento/nucleo.md` (R5.02, R5.06).

## 5. Tabela dos itens

| ID | Tipo | Prior. | Ação | Arquivos | Depende de | Risco |
| --- | --- | --- | --- | --- | --- | --- |
| R0.01 | corrigir | alta | PATCH | docs/pedidos/rev-R0.01.patch | nenhuma | baixo |
| R0.02 | corrigir | crítica | PATCH | docs/pedidos/rev-R0.02.patch | R0.01 | médio |
| R0.03 | melhorar | média | PATCH | docs/pedidos/rev-R0.03.patch | R0.01 | médio |
| R0.04 | documentar | média | MARCELO | decisão sobre hooks/pre-push | R0.03 | médio |
| R0.05 | documentar | média | IA | docs/conhecimento/{hooks,ci,esqueleto,README}.md, README.md | R0.06 | baixo |
| R0.06 | corrigir | baixa | IA | site/app.js, xtask/src/site.rs | nenhuma | baixo |
| R0.07 | melhorar | média | IA | scripts/{etapa4-host,etapa6-bloco2,etapa7-bloco2,etapa7-bloco5,etapa8-bloco2}.sh | nenhuma | baixo |
| R0.08 | implementar | alta | MARCELO | push, CI, Pages, imagem | nenhuma | médio |
| R1.01 | documentar | média | IA | docs/conhecimento/oraculo.md, xtask/src/rotulos.rs | nenhuma | baixo |
| R1.02 | melhorar | média | MARCELO | decisão sobre o manifesto | nenhuma | médio |
| R2.01 | corrigir | média | IA | xtask/src/inventario.rs, scripts/etapa2-host.sh | nenhuma | baixo |
| R2.02 | implementar | alta | IA | scripts/etapa2-inventarios.sh (novo) | R0.07, R2.01 | baixo |
| R2.03 | implementar | alta | MARCELO | docs/inventario-hardware.md, docs/inventario-memoria.md | R2.02 | baixo |
| R2.04 | corrigir | média | PATCH | docs/pedidos/rev-R2.04.patch | nenhuma | médio |
| R2.05 | corrigir | média | IA | scripts/etapa5-assinar.sh, scripts/lib-assinar.sh, scripts/testes/lib-assinar.sh | nenhuma | médio |
| R2.06 | documentar | média | IA | docs/estado.md, docs/backlog.md, docs/conhecimento/{corpus,oraculos}.md | R1.01 | baixo |
| R2.07 | implementar | alta | MARCELO | patches, assinatura, auditoria | R2.04, R2.05, R3.01, R5.02, R6.05, R7.04, R8.03, R8.08, R8.09, R8.10 | médio |
| R3.01 | corrigir | crítica | IA | xtask/src/descompressao.rs, descompressao/manifesto.txt | nenhuma | baixo |
| R3.02 | implementar | crítica | MARCELO | prova da descompressão | R3.01 | baixo |
| R3.03 | documentar | baixa | IA | docs/conhecimento/descompressao.md | nenhuma | baixo |
| R4.01 | corrigir | alta | IA | xtask/src/video.rs | nenhuma | baixo |
| R4.02 | melhorar | crítica | IA | scripts/etapa4-host.sh, docs/conhecimento/video.md | R0.07, R4.01 | baixo |
| R4.03 | implementar | crítica | MARCELO | rodar etapa4-host.sh nos 4 casos | R4.02 | médio |
| R5.01 | implementar | média | IA | crates/core/src/descompressao/{nemesis,enigma}.rs, crates/teste-descompressao/tests/cobertura.rs | nenhuma | médio |
| R5.02 | melhorar | média | IA | crates/teste-nucleo/rotinas/{KosDec,EniDec,NemDecToRAM}.txt, docs/conhecimento/nucleo.md | nenhuma | médio |
| R5.03 | implementar | média | MARCELO | bloco B2 e exclusões | R5.01, R5.02 | médio |
| R5.04 | implementar | média | IA | crates/teste-descompressao/tests/rotinas_vdp.rs (novo) | nenhuma | baixo |
| R5.05 | corrigir | baixa | IA | crates/core/src/memoria/mod.rs | nenhuma | baixo |
| R5.06 | documentar | média | IA | docs/conhecimento/nucleo.md, docs/estado.md, docs/backlog.md | R2.06, R5.02 | baixo |
| R6.01 | documentar | média | MARCELO | decisão sobre z80.rs | nenhuma | alto |
| R6.02 | documentar | alta | MARCELO | decisão sobre bytes em testes | L1 ativa | médio |
| R6.03 | melhorar | média | IA | crates/core/src/som/z80.rs | R6.01 | baixo |
| R6.04 | melhorar | média | IA | crates/core/src/som/driver.rs, crates/teste-som/tests/driver.rs | nenhuma | médio |
| R6.05 | melhorar | média | PATCH | docs/pedidos/rev-R6.05.patch | nenhuma | baixo |
| R7.01 | corrigir | alta | IA | crates/core/src/abertura/{objetos,fase,animacao}.rs, crates/teste-som/tests/tabelas_da_rom.rs (novo) | nenhuma | médio |
| R7.02 | corrigir | média | IA | crates/core/src/abertura/objetos.rs, crates/teste-som/tests/objetos.rs | R7.01 | baixo |
| R7.03 | documentar | média | MARCELO | autorizar o bloco O7 | nenhuma | médio |
| R7.04 | melhorar | média | IA | diferencial-abertura/src/lib.rs, diferencial-abertura/tests/fumaca.rs | nenhuma | médio |
| R7.05 | melhorar | média | IA | crates/console/src/lib.rs, crates/web/src/lib.rs, site/app.js, crates/console/tests/ponte.rs | R0.06 | baixo |
| R8.01 | documentar | alta | MARCELO | decisão de licença e origem | nenhuma | alto |
| R8.02 | documentar | média | IA | crates/audio/Cargo.toml, crates/audio/src/{fm,psg}.rs (cabeçalhos), docs/conhecimento/sintese.md | R8.01 | baixo |
| R8.03 | corrigir | alta | IA | docs/pedidos/etapa8-ci.patch, docs/pedidos/etapa8-relatorio.patch | nenhuma | baixo |
| R8.04 | corrigir | crítica | IA | crates/audio-saida/src/misturador.rs, crates/audio-saida/tests/saida.rs | nenhuma | médio |
| R8.05 | corrigir | crítica | IA | crates/console/src/lib.rs, crates/web/src/lib.rs, site/app.js, site/audio-worklet.js, crates/console/tests/audio.rs | R8.04, R7.05 | alto |
| R8.06 | documentar | média | IA | docs/estado.md, docs/backlog.md, site/index.html | R5.06 | baixo |
| R8.07 | implementar | alta | IA | crates/audio/tests/propriedades.rs, crates/audio/tests/regressao.rs (novos) | nenhuma | baixo |
| R8.08 | corrigir | média | IA | diferencial-sintese/src/bin/sintese.rs, scripts/etapa8-bloco2.sh | R0.07 | baixo |
| R8.09 | melhorar | média | IA | diferencial-sintese/tests/producao.rs (novo) | nenhuma | baixo |
| R8.10 | melhorar | baixa | IA | crates/audio-saida/src/analogico.rs, crates/audio-saida/tests/saida.rs, diferencial-sintese/src/bin/sintese.rs | R8.04, R8.08 | baixo |
| R8.11 | documentar | média | MARCELO | decisão: PCM no worklet | nenhuma | médio |
| R8.12 | implementar | alta | MARCELO | escuta, wasm, patches, assinatura | R8.03, R8.05 | médio |
| R7.06 a R7.10 | melhorar | média | IA (opcional) | ver seção 6, etapa 7 | R7.03, R7.01, R7.02 | médio |

## 6. Itens, um por um

Em todo item de IA o teste vem antes da implementação, quando o comportamento muda. Comandos `cargo` rodam no contêiner. Em item PATCH os comandos rodam num worktree do scratchpad, depois de aplicar o patch.

### Etapa 0 (esqueleto, ambiente e L1)

**R0.01** | corrigir | alta | PATCH
- Arquivos: `docs/pedidos/rev-R0.01.patch`. Aplica depois de `l1-hook-detector.patch`.
- Problema: o detector depende de `../../crates/rom` (confirmado, linha 18 do Cargo.toml do patch). A IA edita esse crate.
- Mudança: o patch traz o SHA-1 e a conversão hexadecimal para `hooks/detector/src/` e remove a dependência.
- Teste: o teste de vetor conhecido passa a usar o módulo local. Acrescentar vetores de 55, 56 e 64 bytes (fronteiras de bloco).
- Aceite: `hooks/detector/Cargo.toml` sem dependência de caminho. `cargo tree` do detector lista só `detector-rom`. Testes do detector passam.
- Verificação (no worktree, com os patches L1 e este aplicados): `cargo test --manifest-path hooks/detector/Cargo.toml --target-dir alvo`; `cargo clippy --manifest-path hooks/detector/Cargo.toml --target-dir alvo --all-targets -- -D warnings`; `cargo fmt --check --manifest-path hooks/detector/Cargo.toml`; `cargo tree --manifest-path hooks/detector/Cargo.toml`.
- Dependências: nenhuma. Risco: baixo.

**R0.02** | corrigir | crítica | PATCH
- Arquivos: `docs/pedidos/rev-R0.02.patch`. Aplica depois de R0.01.
- Problema: seção 2.3, defeito 1 (confirmado). O hook não consegue compilar o detector no contêiner.
- Mudança: o patch traz `hooks/detector/Cargo.lock`. O `hooks/pre-commit` compila com `cargo build --quiet --locked ... --target-dir "$raiz/target/detector"` e depois roda o binário. Erro de compilação e achado têm mensagens diferentes. A ROM é procurada em `/mnt/rom` antes de `SONIC1_EXT_DIR` (hoje a ordem vem do `ls`, que ordena os dois caminhos juntos).
- Teste: `sh .devcontainer/testes/hooks.sh` continua com 14 de 14. No worktree, com `chmod -R a-w hooks`, o build do detector com `--locked --target-dir` termina com código 0.
- Aceite: os dois testes acima passam. Sem ROM o hook ainda bloqueia o commit.
- Verificação (worktree): `chmod -R a-w hooks && cargo build --locked --manifest-path hooks/detector/Cargo.toml --target-dir target/detector`; `sh .devcontainer/testes/hooks.sh`. Depende de você: um commit real, com a ROM, dentro do contêiner.
- Dependências: R0.01. Risco: médio. O hook protege todo commit.

**R0.03** | melhorar | média | PATCH
- Arquivos: `docs/pedidos/rev-R0.03.patch`. Aplica depois de R0.01.
- Problema: seção 2.3, defeito 3 (confirmado em `numero_em`, linhas 247 a 280 do patch, e no teste das linhas 559 a 564).
- Mudança: ler inteiros com sufixo de tipo, `$`, `0x` e negativos. Tratar `(`, `[`, `{` e `}` como separadores. Montar fluxos u8, u16be e u32be. Acrescentar um fluxo de pares hexadecimais ("4E 71 4E 75") e de hexadecimal corrido de 64 dígitos ou mais.
- Teste (antes): ROM falsa e um caso por formato (sufixo, tupla, aninhado, `$`, par hexadecimal, u32be), mais dois negativos (texto comum e números fora de ordem). Varrer `docs/*.md` com a ROM falsa: zero achados.
- Aceite: todos os casos novos são achados. Os 11 testes antigos passam, sem enfraquecer nenhum.
- Verificação: igual à do R0.01.
- Dependências: R0.01. Risco: médio. Mais cobertura pode dar falso positivo, e o hook bloqueia o commit.

**R0.04** | documentar | média | MARCELO (decisão)
- Problema: seção 2.3, defeito 4.
- Mudança pedida: decidir se o `pre-push` roda o detector nos blobs novos, no host do Marcelo, que tem a ROM. Se sim, a IA gera o patch num plano revisado.
- Aceite: decisão escrita em `docs/conhecimento/hooks.md`.
- Dependências: R0.03. Risco: médio.

**R0.05** | documentar | média | IA
- Arquivos: `docs/conhecimento/hooks.md`, `docs/conhecimento/ci.md`, `docs/conhecimento/esqueleto.md`, `README.md`, `docs/conhecimento/README.md`.
- Problema: `hooks.md:11` não cita `s1t s1e s1i s1p s1v` (a lista de `hooks/lib.sh:30` cita; confirmado). `esqueleto.md:9` diz que o núcleo é "só `aritmetica`" e `:12` diz que o `xtask` tem dois comandos (confirmado; são 16). `ci.md:7` descreve um fluxo que o `ci.yml` mudou (confirmado em parte). O índice não lista `sintese.md` (confirmado). `README.md:15-21` lista as pastas externas sem `privado`, `ocultos` e `tas` (subagente).
- Mudança: reescrever só as frases defasadas e acrescentar a linha do índice. Manter a evidência histórica. Depois do R0.06, trocar a frase "só com `textContent`" por algo verdadeiro.
- Teste: nenhum comportamento muda.
- Aceite: o laço abaixo não imprime nada, e `hooks.md` cita `s1t`.
- Verificação: `for f in docs/conhecimento/*.md; do b=${f##*/}; [ "$b" = README.md ] || grep -q "($b)" docs/conhecimento/README.md || echo "FALTA $b"; done`; `grep -c s1t docs/conhecimento/hooks.md`; `grep -rIl -P '[\x{2013}\x{2014}]' docs README.md` (saída vazia).
- Dependências: R0.06. Risco: baixo.

**R0.06** | corrigir | baixa | IA
- Arquivos: `site/app.js`, `xtask/src/site.rs`.
- Problema: sete atribuições a `innerHTML` em `site/app.js` (linhas 69, 74, 78, 82, 128, 169 e 182; confirmado). `esqueleto.md:26` afirma que a página escreve "só com `textContent`". Duas linhas interpolam texto (`${st}` e `${e.message}`).
- Mudança: trocar por `textContent` e por uma classe CSS para a cor.
- Teste (antes): um `#[test]` em `site.rs` com `include_str!("../../site/app.js")` que exige zero ocorrências de `innerHTML`. Falha hoje.
- Aceite: o teste passa. O painel de diagnóstico continua mostrando estado e pipeline.
- Verificação: `cargo test -p xtask site`; `cargo xtask site`. Depende de você: abrir o painel no navegador.
- Dependências: nenhuma. Risco: baixo.

**R0.07** | melhorar | média | IA
- Arquivos: `scripts/etapa4-host.sh`, `scripts/etapa6-bloco2.sh`, `scripts/etapa7-bloco2.sh`, `scripts/etapa7-bloco5.sh`, `scripts/etapa8-bloco2.sh`.
- Problema: a "Pendência fora da L1" de `docs/estado.md`. Os scripts gravam em `$ext/privado` (confirmado em `etapa4-host.sh` e `etapa8-bloco2.sh`), pasta que o contêiner não monta. A ROM é fixa em `etapa6-bloco2.sh:11` (subagente). Por isso o gate de cada etapa fica PENDENTE no contêiner.
- Mudança: `saida=${SONIC1_SAIDA:-$ext/privado}` e ROM por argumento, com o padrão atual. Sem mudar nenhum resultado por padrão. Em `etapa7-bloco5.sh`, quando `SONIC1_SEM_DOCKER=1`, usar `wasmtime` direto (hoje chama `docker`).
- Teste: `sh -n` em cada script e `grep` sem sobras de `$ext/privado`.
- Aceite: com `SONIC1_EXT_DIR=/mnt` e `SONIC1_SAIDA=/mnt/trabalho/saida` os caminhos apontam só para pastas montadas.
- Verificação: `for s in etapa4-host etapa6-bloco2 etapa7-bloco2 etapa7-bloco5 etapa8-bloco2; do sh -n scripts/$s.sh; done`; `grep -n 'ext/privado' scripts/etapa4-host.sh scripts/etapa6-bloco2.sh scripts/etapa7-bloco2.sh scripts/etapa7-bloco5.sh scripts/etapa8-bloco2.sh` (só a definição de `saida`). A execução depende de você (ROM e oráculo).
- Dependências: nenhuma. Risco: baixo.

**R0.08** | implementar | alta | MARCELO
- Problema: a `main` tem 161 commits que nunca foram enviados (confirmado). O CI e o Pages só foram provados até `05576b5`. Depois mudaram `ci.yml`, o relatório e o `managed-settings.json`.
- Mudança pedida: enviar, conferir o CI e o site, reconstruir a imagem e repetir a rede e o roteiro das permissões.
- Verificação (depende de você): `git status --short`; `git push origin main` (o pre-push roda no host); conferir os jobs `verificar` e `publicar` na aba Actions; abrir a página (endereço em `docs/conhecimento/ci.md:40`) e escolher a ROM REV01; `devcontainer build --workspace-folder .`; `devcontainer up --workspace-folder . --remove-existing-container`; `devcontainer exec --workspace-folder . sh .devcontainer/testes/rede.sh`; roteiro `.devcontainer/testes/permissoes.md` (testes 23 a 36).
- Aceite: os dois jobs verdes, página com "ROM REV01 válida", `rede.sh` sem falha.
- Dependências: nenhuma. Risco: médio (primeiro envio de 161 commits; o pre-push varre o histórico novo).

### Etapa 1 (oráculo)

**R1.01** | documentar | média | IA
- Arquivos: `docs/conhecimento/oraculo.md`, `xtask/src/rotulos.rs` (só comentário).
- Problema: `oraculo.md:62` ainda diz "mapa com DEFEITO a corrigir" (confirmado). O defeito (base 10) foi corrigido na 2.0 e na 3.0. A lista de revisão R1 a R13 não mostra o estado de cada linha. O comentário de `rotulos.rs:8-12` diz "formato ainda não conferido" (subagente).
- Mudança: atualizar o título e o texto do mapa. Acrescentar uma tabela com o estado de R1 a R13, cada linha com a evidência (commit, teste ou "sem evidência"). Corrigir o comentário.
- Teste: nenhum comportamento muda.
- Aceite: `grep -n DEFEITO docs/conhecimento/oraculo.md` não acha o título antigo, e a tabela tem 13 linhas.
- Verificação: `grep -n 'DEFEITO' docs/conhecimento/oraculo.md`; `cargo test -p xtask rotulos`.
- Dependências: nenhuma. Risco: baixo.

**R1.02** | melhorar | média | MARCELO (decisão)
- Problema: o `runner` congelado do oráculo importa `sonic1_core::quadro::tempo` e `sonic1_core::memoria::ClasseLeitura` (confirmado: `oracle/runner/Cargo.toml:16`, `bin/oraculo.rs:146` e `428-431`). Quem edita o núcleo muda esses tipos sem tocar em arquivo congelado nem coberto pelo manifesto. Os hashes dos traces `.s1t` assinados detectam parte do efeito, mas o manifesto não cobre o arquivo de tempo gravado (`.s1tempo`).
- Mudança pedida: escolher (a) aceitar o risco e registrar; (b) pôr `crates/core/src/quadro/tempo.rs` e `crates/core/src/memoria/mod.rs` em `REFERENCIAS` (cada mudança futura exige assinatura nova); (c) mover os tipos compartilhados para um crate pequeno e congelado, por exemplo `crates/fidelidade`.
- Aceite: decisão escrita em `docs/conhecimento/oraculo.md`.
- Dependências: nenhuma. Risco: médio.

### Etapa 2 (corpus e integridade)

**R2.01** | corrigir | média | IA
- Arquivos: `xtask/src/inventario.rs`, `scripts/etapa2-host.sh`.
- Problema: `cargo xtask inventario` não recusa uma pasta sem `*.a.s1t` e sobrescreve `docs/inventario-hardware.md` com "Nenhum item observado" (confirmado: `executar`, linhas 275 a 314, não checa `arquivos.is_empty()`). Desde a 2.10 o `corpus-trace` apaga os `.a.s1t` (subagente).
- Mudança: devolver `Err` antes de escrever se não houver nenhum `.a.s1t`. O `etapa2-host.sh` passa `--manter-trace` ao `corpus-trace` (subagente: as linhas 29 a 33 rodam o `corpus-trace` e depois o inventário).
- Teste (antes): `sem_traces_nao_escreve_nada`: pasta vazia, `--saida` em diretório temporário, espera `Err` e arquivo inexistente.
- Aceite: o teste falha antes e passa depois.
- Verificação: `cargo test -p xtask inventario`; `cargo clippy -p xtask --all-targets -- -D warnings`; `sh -n scripts/etapa2-host.sh`.
- Dependências: nenhuma. Risco: baixo.

**R2.02** | implementar | alta | IA
- Arquivos: `scripts/etapa2-inventarios.sh` (novo).
- Problema: os dois inventários foram gerados só com `boot-demos` e `lz-agua` (confirmado: "Casos lidos (2)"). O corpus tem quatro casos (`boot-demos`, `demos-quatro`, `lz-agua`, `partida-ghz1`). A special stage (fim de `demos-quatro`) não entrou. O critério da Etapa 2 pede inventários "prontos"; o renderizador e o núcleo só implementam o que o inventário lista.
- Mudança: um script de host que (1) confere o SHA-1 da ROM; (2) usa os traces `.a.s1t` já existentes em `${SONIC1_SAIDA:-$ext/privado}/traces` (variável do R0.07) ou os gera com `cargo fidelidade corpus-trace --manter-trace`; (3) roda `cargo xtask inventario`; (4) roda o oráculo com `--acessos` nos quatro casos (modelo: `scripts/etapa2-final.sh:30-33`) e `cargo xtask inventario-memoria`; (5) apaga os `.a.s1t` no fim, salvo `MANTER=1`.
- Teste: `sh -n`. Só usa comandos de `xtask/src/main.rs`.
- Aceite: o script existe, é executável e termina com falha se qualquer passo falhar.
- Verificação: `sh -n scripts/etapa2-inventarios.sh`. A execução é o R2.03.
- Dependências: R0.07, R2.01. Risco: baixo. Espaço em disco: cerca de 8 GB de traces temporários.

**R2.03** | implementar | alta | MARCELO
- Problema: R2.02.
- Mudança pedida: rodar o script e commitar os dois inventários depois de ver o `git diff --stat`.
- Verificação (depende de você): `sh scripts/etapa2-inventarios.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"` (confirmar o nome e os argumentos depois do R2.02).
- Aceite: a linha de casos dos dois arquivos lista os quatro casos. O filtro do `xtask` recusa 16 ou mais hexadecimais seguidos.
- Dependências: R2.02. Risco: baixo.

**R2.04** | corrigir | média | PATCH
- Arquivos: `docs/pedidos/rev-R2.04.patch` (altera `crates/fidelidade/relatorio/src/ocultos.rs` e `.../manifesto.rs` no worktree).
- Problema: o manifesto grava as linhas `zona` e `oculto` (confirmado: `manifesto.rs:279` e `290`), mas nada as confere. `conferir_arquivos` só lê linhas `arquivo` (`:91`). `ocultos auditar` imprime `passou (0 casos)` para zona sem caso, ignora zona ausente e aceita `.age` a mais (confirmado em `ocultos.rs`, linhas 112 a 146).
- Mudança: `ocultos auditar` ganha `--manifesto` (padrão `refs/manifesto.txt`). Termina com erro se o conjunto de zonas, os arquivos `.age` ou seus SHA-256 diferirem das linhas `zona` e `oculto`. Zona sem caso vira `falhou`. Os scripts atuais não mudam.
- Teste (antes): `auditoria_recusa_oculto_extra`, `auditoria_recusa_oculto_trocado`, `auditoria_recusa_zona_faltando`, `zona_vazia_nao_passa`. Nenhum teste existente muda.
- Aceite: quatro testes novos verdes. Os 45 testes do pacote seguem verdes.
- Verificação (worktree): `cargo test --locked --manifest-path crates/fidelidade/relatorio/Cargo.toml --target-dir target/relatorio`; `cargo clippy --locked --manifest-path crates/fidelidade/relatorio/Cargo.toml --target-dir target/relatorio --all-targets -- -D warnings`; `cargo fmt --check --manifest-path crates/fidelidade/relatorio/Cargo.toml`.
- Dependências: nenhuma. Risco: médio. Muda a CLI usada em `etapa5-assinar.sh` e exige nova assinatura.

**R2.05** | corrigir | média | IA
- Arquivos: `scripts/etapa5-assinar.sh`, `scripts/lib-assinar.sh` (novo), `scripts/testes/lib-assinar.sh` (novo).
- Problema: no fluxo de assinatura do Marcelo (confirmado): `registrar` devolve o status do último `tee`, então `cargo fidelidade ...` pode falhar e o script segue (`etapa5-assinar.sh:28`); `refs/assinante.pub` é trocado sem confirmação (`:41-44`); o script não confere se a chave de assinatura tem senha, e o `etapa2-fechamento.sh:93-104` conferia.
- Mudança: funções `exige_senha` (falha se `ssh-keygen -y -P ''` abrir a chave), `confirma_troca` (mostra as duas impressões digitais por `ssh-keygen -lf` e exige digitar `trocar`) e `registrar` que preserva o status do comando. O script mostra `git diff --stat -- refs/manifesto.txt` antes de assinar.
- Teste (antes): `scripts/testes/lib-assinar.sh`, com `ssh-keygen` falso no `PATH`. Chave sem senha falha. Troca sem "trocar" falha. `registrar false` devolve diferente de 0.
- Aceite: o teste passa. `sh -n scripts/etapa5-assinar.sh` limpo.
- Verificação: `sh scripts/testes/lib-assinar.sh`; `sh -n scripts/etapa5-assinar.sh`. A execução real exige terminal e a chave: depende de você.
- Dependências: nenhuma. Risco: médio. O script faz parte do fluxo de assinatura. A IA não assina. O Marcelo lê o diff antes do próximo uso.

**R2.06** | documentar | média | IA
- Arquivos: `docs/estado.md`, `docs/backlog.md`, `docs/conhecimento/corpus.md`, `docs/conhecimento/oraculos.md`.
- Problema: `estado.md:112-124` ainda traz "Etapa atual: 1 (Oráculo)" e o "defeito no rotulos" (confirmado). `estado.md:89` diz que a Etapa 2 "fecha quando reconstruir a imagem" (subagente). `backlog.md:43` diz que o CI não cobre o `relatorio`, e `ci.yml:68-72` cobre (confirmado). `corpus.md:8, 28, 110, 112` e `oraculos.md` (21 amostras, só RAM do 68000, causas em hipótese, critério redefinido e aceito) estão defasados (subagente).
- Mudança: marcar a seção "Etapa atual" como histórico (sem apagar). Corrigir as outras frases com a evidência do repositório.
- Teste: nenhum comportamento muda.
- Aceite: `grep -n "fecha quando" docs/estado.md` vazio. `grep -n "continua sem senha" docs/conhecimento/corpus.md` vazio.
- Verificação: os dois `grep` acima; `grep -rIl -P '[\x{2013}\x{2014}]' docs` vazio.
- Dependências: R1.01. Risco: baixo.

**R2.07** | implementar | alta | MARCELO
- Problema: depois dos itens de IA e PATCH acima, o manifesto assinado deixa de bater. Também falta conferir a senha da chave nova (a prova B1 vale para a chave antiga).
- Mudança pedida: aplicar `docs/pedidos/rev-R2.04.patch` e os patches da etapa 7 e 8 (R8.03), reassinar, conferir a chave e rodar relatório e auditoria.
- Verificação (depende de você): `git apply --check docs/pedidos/rev-R2.04.patch && git apply docs/pedidos/rev-R2.04.patch`; `sh scripts/etapa5-assinar.sh` (pede senhas; os comandos de relatório e auditoria estão em `scripts/etapa5-assinar.sh:65-75`); `ssh-keygen -y -P '' -f ~/.ssh/websonic_assinatura_nova` (deve falhar); `ssh-keygen -lf refs/assinante.pub`.
- Aceite: `assinatura: OK`, `veredito: PASSOU`, auditoria `passou` nas seis zonas, `ssh-keygen -y -P ''` falha.
- Dependências: R2.04, R2.05, R3.01, R5.02, R6.05, R7.04, R8.03, R8.08, R8.09, R8.10 (todos os itens que mudam caminho coberto pelo manifesto). A assinatura acontece uma vez, por último. Risco: médio.

### Etapa 3 (descompressores)

**R3.01** | corrigir | crítica | IA
- Arquivos: `xtask/src/descompressao.rs`, `descompressao/manifesto.txt`.
- Problema: o critério pede "a arte, os blocos e os mapas" idênticos. Dos seis `Blk256_*` do mapa só quatro estão no manifesto de descompressão (faltam `Blk256_MZ` e `Blk256_SBZ`). `Nem_GhzUnkBlock` e `Nem_LzSpikeBall` também faltam. Tudo confirmado contra `refs/rotulos.txt` e `descompressao/manifesto.txt` (174 linhas). O teste de `xtask/src/descompressao.rs:446` só exige mais de 100 itens (subagente).
- Mudança: acrescentar as quatro linhas, com o formato das vizinhas (`-` no terceiro campo, quando for o caso).
- Teste (antes): `o_manifesto_cobre_os_rotulos_de_dados_do_mapa`. Lê `refs/rotulos.txt` com `include_str!` e exige que todo rótulo `Nem_`, `Blk16_`, `Blk256_`, `Eni_`, `Kos_` e `SS_1` a `SS_6` esteja no manifesto ou numa lista de exclusão do teste, cada uma com motivo.
- Aceite: o teste falha sem as quatro linhas e passa com elas.
- Verificação: `cargo test -p xtask descompressao`; `cargo fmt --all --check`; `cargo clippy -p xtask --all-targets -- -D warnings`.
- Dependências: nenhuma. Risco: baixo. Um bloco novo pode dar ERRO se não for do formato suposto; a prova do R3.02 decide, e então corrige-se o jogo, nunca o teste. `descompressao/` está no manifesto: o item exige nova assinatura.

**R3.02** | implementar | crítica | MARCELO
- Problema: o manifesto de descompressão mudou e `descompressao/` é referência assinada.
- Verificação (depende de você): `sh descompressao/prova.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`; depois `sh scripts/etapa5-assinar.sh`.
- Aceite: 189 IGUAL (os 185 de antes mais os 4 novos), 0 falhas e 6 AUSENTE (ramos da REV00), no formato que `descompressao/prova.sh` imprime. Se um bloco novo der DIFERE ou ERRO, corrige-se o jogo.
- Dependências: R3.01. Risco: baixo.

**R3.03** | documentar | baixa | IA
- Arquivos: `docs/conhecimento/descompressao.md`.
- Problema (subagente): diz "11 passam" e hoje são 13. O destino do `NemDecToRAM` é `a4` no texto e `a1` em `crates/teste-nucleo/rotinas/NemDecToRAM.txt:4`. Falta registrar a prova indireta do `NemDec` (escritas de vídeo da Etapa 7).
- Mudança: atualizar a contagem, a convenção de registradores e o registro da prova indireta.
- Aceite: os números batem com `cargo test -p sonic1-teste-descompressao`.
- Verificação: `cargo test -p sonic1-teste-descompressao`; `grep -rIl -P '[\x{2013}\x{2014}]' docs/conhecimento/descompressao.md` vazio.
- Dependências: nenhuma. Risco: baixo.

### Etapa 4 (renderizador)

**R4.01** | corrigir | alta | IA
- Arquivos: `xtask/src/video.rs`.
- Problema: `video-estado` e `video-comparar` terminam com `Ok(())` mesmo com DIFERE ou com avisos fora do inventário (confirmado: `video.rs:117-121` e `255-265`). Os vereditos só existem como texto. O requisito diz que o uso fora do inventário "falha nos testes".
- Mudança: extrair `conferir(primeira_diferenca, &Avisos) -> Result<(), String>`. Devolve erro quando há diferença ou `avisos.total() > 0` (`dma_sem_dados` não conta). Imprimir tudo antes de devolver o erro.
- Teste (antes): três testes unitários: igual e sem aviso dá `Ok`; diferença dá `Err`; aviso de janela dá `Err` e cita o campo.
- Aceite: os três testes passam, e o código de saída do comando acompanha o veredito.
- Verificação: `cargo test -p xtask video`; `cargo clippy -p xtask --all-targets -- -D warnings`.
- Dependências: nenhuma. Risco: baixo. Pode expor avisos nos casos novos, que é o objetivo.

**R4.02** | melhorar | crítica | IA
- Arquivos: `scripts/etapa4-host.sh`, `docs/conhecimento/video.md`.
- Problema: o critério pede "todo quadro do corpus" idêntico. O script percorre só `boot-demos` e `lz-agua` (confirmado). `demos-quatro` (11700 quadros, com a special stage) e `partida-ghz1` (8309) nunca passaram por `video-comparar`.
- Mudança: percorrer os quatro casos, passar `--entradas corpus/$caso.s1in` quando o arquivo existir, juntar os códigos de saída e terminar com falha se algum caso falhar (depende do R4.01). Atualizar `video.md` com o procedimento.
- Teste: sem ROM só dá para checar a sintaxe. O teste real é o R4.03.
- Aceite: o script cobre os quatro casos e falha se um falhar.
- Verificação: `sh -n scripts/etapa4-host.sh`; `grep -n 'for caso' scripts/etapa4-host.sh`.
- Dependências: R0.07, R4.01. Risco: baixo. De 0,6 a 3,3 GB de disco por caso.

**R4.03** | implementar | crítica | MARCELO
- Verificação (depende de você): `sh scripts/etapa4-host.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`.
- Aceite: `video: IGUAL` nos quatro casos (`11700` e `8309` quadros nos casos novos), `avisos_fora_do_inventario=0`, VRAM, CRAM e VSRAM iguais. Divergência é defeito do renderizador, corrigido no jogo.
- Dependências: R4.02. Risco: médio. A special stage pode usar recurso fora do inventário.

### Etapa 5 (núcleo base)

**R5.01** | implementar | média | IA
- Arquivos: `crates/core/src/descompressao/nemesis.rs`, `crates/core/src/descompressao/enigma.rs`, `crates/teste-descompressao/tests/cobertura.rs`.
- Problema: o Nemesis inteiro e as flags do Enigma não passam por `decisao!` (subagente: `nemesis.rs:54, 67, 70, 169, 182, 185, 201, 215`, `enigma.rs:47`). O backlog (linha 46, alvo Etapa 6) pede e não foi feito (confirmado). O requisito diz que toda decisão do núcleo tem par no original.
- Mudança: anotar só os ramos que têm par no original. Os de validação ficam de fora.
- Teste: estender as entradas de `cobertura.rs` até todo id novo ir nos dois caminhos. Os testes de formatos ficam iguais.
- Aceite: o teste de cobertura passa. Sem as features, o comportamento é o mesmo.
- Verificação: `cargo test -p sonic1-teste-descompressao`; `cargo test --workspace`; `sh .github/lints-core.sh`; `sh .github/checa-core.sh`.
- Dependências: nenhuma. Risco: médio. Mutantes novos precisam ser detectados pela suíte atual.

**R5.02** | melhorar | média | IA
- Arquivos: `crates/teste-nucleo/rotinas/KosDec.txt`, `crates/teste-nucleo/rotinas/EniDec.txt`, `crates/teste-nucleo/rotinas/NemDecToRAM.txt`, `docs/conhecimento/nucleo.md`.
- Problema: `usados_depois:` está vazio nas três rotinas (confirmado). O requisito diz que o teste por rotina compara "os registradores que o chamador usa depois". A evidência é um `grep` em três arquivos (`nucleo.md:63`).
- Mudança: ler os chamadores no `.asm` de texto (`/mnt/disasm-texto`) e preencher os registradores que cada chamador lê depois. Registrar o resultado em `nucleo.md`, com o nome do rótulo de cada chamador e sem copiar código.
- Teste: `cargo test -p sonic1-teste-nucleo` segue verde. A prova nova vem do R5.03.
- Aceite: cada arquivo tem a lista ou a frase "nenhum chamador usa registrador depois", com a evidência.
- Verificação: `cargo test -p sonic1-teste-nucleo`; `git diff --stat`. Depende de você: `cargo test --release --manifest-path diferencial/Cargo.toml` com o oráculo preparado.
- Dependências: nenhuma. Risco: médio. Fortalece o teste e pode acusar divergência real; então corrige-se o jogo. `crates/teste-nucleo` está no manifesto: exige nova assinatura.

**R5.03** | implementar | média | MARCELO
- Mudança pedida: rodar o bloco B2, registrar o conjunto de registradores ignorados mexidos por rotina e o mapa de decisões, e decidir as duas exclusões de `cobertura/exclusoes-propostas.txt` (recomendação do plano: aprovar a pilha).
- Verificação (depende de você): `sh scripts/etapa5-bloco-b2.sh 2>&1 | tee /tmp/etapa5-bloco-b2.txt`; `cargo test --release --manifest-path diferencial/Cargo.toml`.
- Aceite: três conjuntos registrados (ignorados mexidos, mapa, decisão das exclusões).
- Dependências: R5.01, R5.02. Risco: médio.

**R5.04** | implementar | média | IA
- Arquivos: `crates/teste-descompressao/tests/rotinas_vdp.rs` (novo).
- Problema: o backlog pede "provar `NemDec` (porta da VDP) além de `NemDecToRAM`" (linha 13, Etapa 5; confirmado). `rotinas::nem_dec` repete quase todo `nem_dec_to_ram` (`rotinas/mod.rs:63-116`, subagente) e não há teste que compare as duas.
- Mudança: com os compressores do crate, de 1 a 300 tiles, com e sem XOR, comparar as palavras escritas na porta de dados da VDP com os bytes gravados na RAM e a tabela em `v_ngfx_buffer`.
- Aceite: passa em nativo e wasm. Alterar o agrupamento em longas em `nem_dec` faz o teste falhar.
- Verificação: `cargo test -p sonic1-teste-descompressao`; `cargo test --target wasm32-wasip1 -p sonic1-teste-descompressao`.
- Dependências: nenhuma. Risco: baixo.

**R5.05** | corrigir | baixa | IA
- Arquivos: `crates/core/src/memoria/mod.rs`.
- Problema (subagente): `Aviso::Z80SemBarramento` nunca é emitido (`mod.rs:45`; o acesso cai em `LeituraSemDispositivo`, `:423, :457-460`). Entrada e saída não modelada devolve 0 ou ignora em silêncio (`:493, :629`). O requisito pede aviso para uso fora do inventário.
- Mudança: emitir `Z80SemBarramento` no acesso à janela sem BUSACK e criar `Aviso::IoNaoModelado` nos braços `_` de `ler_io` e `escrever_io`.
- Teste (antes): `janela_do_z80_sem_barramento_conta_aviso` e `io_nao_modelado_conta_aviso`.
- Aceite: contagens específicas. Os testes de `diferencial/tests/memoria.rs` seguem iguais.
- Verificação: `cargo test -p sonic1-core`; `cargo test --workspace`; `sh .github/lints-core.sh`.
- Dependências: nenhuma. Risco: baixo.

**R5.06** | documentar | média | IA
- Arquivos: `docs/conhecimento/nucleo.md`, `docs/estado.md`, `docs/backlog.md`.
- Problema (subagente): `quadro::Nucleo`, `Contexto::checagem`, `Ponto` e `conferir_entregas` só são usados pelos próprios testes. A abertura usa `EventoQuadro`. O plano da Etapa 7 previa o contrário e a D7.9 mudou sem dizer. `estado.md` mantém como pendentes passos já feitos das Etapas 3 e 5. `nucleo.md` diz "315 itens em 455 arquivos" (são 87 arquivos com itens) e tem a tabela de provas defasada. A coluna `estado` de `bugs-conhecidos.md` é fixa em "pendente" (`bugs_conhecidos.rs:108`).
- Mudança: registrar o estado real do laço de quadro e dos pontos (só testes de brinquedo). Remover de `estado.md` os passos já feitos (sem apagar histórico). Triar o backlog: fundir as linhas 38 e 48, reclassificar as linhas 16 e 28, fechar a linha 12. Corrigir os números de `nucleo.md`.
- Aceite: nenhum passo já feito aparece como pendente em `estado.md`.
- Verificação: `git diff --stat`; `grep -n 'Falta (Marcelo' docs/estado.md`; `grep -rIl -P '[\x{2013}\x{2014}]' docs` vazio.
- Dependências: R2.06, R5.02. Risco: baixo.

### Etapa 6 (driver de som)

**R6.01** | documentar | média | MARCELO (decisão)
- Problema: `crates/core/src/som/z80.rs` é a máquina do Z80. Não é interpretador: não busca nem decodifica opcode (subagente: o despacho é por endereço e reconhece dois programas por três bytes, `:313-319`). Mas guarda `pc: u16` e 14 variáveis que fazem o papel de registradores (confirmado, `:130-157`, nomeadas pelo papel no programa), e tem 77 comentários com mnemônicos do Z80 (confirmado). A regra dura do projeto proíbe registradores do Z80 como estado do núcleo. O requisito de 11/11 pede o programa do Z80 "reproduzido no núcleo como o código do 68000".
- Mudança pedida: escolher (a) aceitar como "máquina nativa por programa" e registrar em `docs/conhecimento/som.md`; (b) pedir uma refatoração (rotinas nomeadas, sem `pc`, sem comentários de mnemônico).
- Aceite: decisão escrita.
- Dependências: nenhuma. Risco: alto. A refatoração, se pedida, é grande.

**R6.02** | documentar | alta | MARCELO (decisão)
- Problema: `diferencial-som/tests/z80.rs:8-13` tem 38 bytes contíguos de programa do Z80, e o cabeçalho (`:3-4`) diz que nenhum byte vem da ROM. A sequência parece o código de partida padrão do Z80 (subagente; sem prova). Outras sequências longas do mesmo tipo estão em `crates/teste-som/tests/{nivel2,cenario,driver}.rs`. Se algum trecho de 32 bytes ou mais for igual à ROM, o detector da L1 barra qualquer commit que toque o arquivo.
- Mudança pedida: depois da L1, ver o que o detector acha. Se achar, escolher entre trocar o programa de teste por um escrito de outro jeito ou abrir exceção (o detector é congelado).
- Verificação (depende de você): os controles do passo 2 da L1 e um `git add` seguido de `git commit` de teste em `diferencial-som/tests/z80.rs`.
- Dependências: L1 ativa. Risco: médio.

**R6.03** | melhorar | média | IA
- Arquivos: `crates/core/src/som/z80.rs`.
- Problema: zero `decisao!` em `z80.rs` (confirmado), e `plano-etapa-6.md:132` manda marcar as decisões do Z80 com `decisao!("Z80:nome", ...)`. Oito desvios sem anotação (subagente: `:378, 515, 537, 558, 743, 768, 796, 858`).
- Mudança: anotar os oito pontos sem mudar o comportamento.
- Teste: com `--features cobertura`, rodar os dois programas numa RAM sintética e afirmar que os oito ids foram registrados, quatro deles nos dois caminhos.
- Aceite: ids registrados. Os testes atuais do Z80 ficam intactos.
- Verificação: `cargo test -p sonic1-core --features cobertura`; `cargo test -p sonic1-core -p sonic1-teste-som`; `sh .github/lints-core.sh`.
- Dependências: R6.01. Risco: baixo.

**R6.04** | melhorar | média | IA
- Arquivos: `crates/core/src/som/driver.rs`, `crates/teste-som/tests/driver.rs`.
- Problema (subagente): cerca de oito desvios do driver sem `decisao!` (`driver.rs:279, 488, 959, 1315, 1320, 1874, 1879, 2154`). Em `StopSFX` (`:1873`) falta o teste de "especial tocando" que `cfStopTrack` faz (`:1313`). Os bugs BK-0282 a 0284 não têm roteiro.
- Mudança: conferir cada ponto contra o `.asm` de texto (só lógica e rótulo) e anotar. Se `StopSFX` diferir do original, corrigir no jogo.
- Teste: um por desvio e um para `StopSFX` com especial tocando. Nenhum teste existente muda.
- Aceite: ids registrados; testes novos verdes.
- Verificação: `cargo test -p sonic1-core -p sonic1-teste-som`; `cargo clippy -p sonic1-core -p sonic1-teste-som --all-targets -- -D warnings`. Depende de você se `StopSFX` mudar: `sh scripts/etapa6-bloco2.sh` (esperado 78 IGUAL, 0 DIVERGE).
- Dependências: nenhuma. Risco: médio.

**R6.05** | melhorar | média | PATCH
- Arquivos: `docs/pedidos/rev-R6.05.patch` (altera `cobertura/exclusoes-propostas.txt` no worktree).
- Problema (subagente): o harness do som ignora `1FFD` e `1FFF` da RAM do Z80 e compara essa RAM só fora de trechos longos (`harness.rs:28, 351, 473-477`). O requisito diz que área ignorada na comparação segue a mesma aprovação das exclusões de código, e o arquivo só propõe dois grupos (pilha e despacho por tabela).
- Mudança: acrescentar dois grupos propostos, `z80-estado-do-dac` e `z80-trecho-longo`, com o motivo de `som.md`. É proposta; o Marcelo decide.
- Aceite: `git apply --check` limpo no HEAD.
- Verificação: `git apply --check docs/pedidos/rev-R6.05.patch`.
- Dependências: nenhuma. Risco: baixo. `cobertura/` está no manifesto: exige nova assinatura.

### Etapa 7 (telas de abertura)

**R7.01** | corrigir | alta | IA
- Arquivos: `crates/core/src/abertura/objetos.rs`, `crates/core/src/abertura/fase.rs`, `crates/core/src/abertura/animacao.rs`, `crates/teste-som/tests/tabelas_da_rom.rs` (novo).
- Problema: três tabelas do jogo estão escritas no código. `RING_POS_DATA` (`objetos.rs:177-195`, 16 pares) e `BASE` de `oscillate_num_init` (`fase.rs:143-146`, 33 palavras) estão confirmadas. A terceira é uma sequência de 4 bytes em `animacao.rs:14` (subagente). A regra é: toda tabela vem da ROM na execução. São dados de rótulos do disassembly. O detector da L1 provavelmente acha a lista plana de `fase.rs` e não acha a de `objetos.rs`, escrita em pares entre parênteses (ver R0.03).
- Mudança: ler cada tabela da ROM pelo endereço do rótulo (`Ring_PosData`, `OscillateNumInit.baselines` e a sequência das flores) e apagar as constantes. Os endereços vêm de `refs/rotulos.txt`, como nos outros acessos do módulo. Ler `.asm` de texto só para saber o tamanho. Não copiar valor nenhum.
- Teste (antes): ROM sintética com valores inventados nesses endereços. Conferir a posição do segundo anel, a base da oscilação e a sequência das flores. Incluir um teste de layout contra `refs/rotulos.txt`, no padrão de `cenario_layout.rs`. Se `fase_cartoes.rs` depender da tabela antiga, ajustar o preparo da ROM sintética, não a asserção.
- Aceite: nenhum `const ...: [` com dado de jogo em `abertura/`. Testes verdes.
- Verificação: `cargo test -p sonic1-core -p sonic1-teste-som`; `sh .github/lints-core.sh`. Depende de você: `sh scripts/etapa7-bloco2.sh "$ROM" 1170` deve seguir IGUAL.
- Dependências: nenhuma. Risco: médio.

**R7.02** | corrigir | média | IA
- Arquivos: `crates/core/src/abertura/objetos.rs`, `crates/teste-som/tests/objetos.rs`.
- Problema: `build_sprites` gira para sempre se a contagem de uma fila for ímpar, como no original (backlog linha 70, confirmado; subagente: `objetos.rs:679-690`). O limite do comparador só vale para o oráculo, então o navegador trava.
- Mudança: limitar as voltas por fila a `0x80` e devolver `ErroAbertura`, para o comparador acusar em vez de travar.
- Teste (antes): fila com contagem ímpar retorna `Err`.
- Aceite: o teste falha antes e passa depois. A abertura normal não muda.
- Verificação: `cargo test -p sonic1-core -p sonic1-teste-som`.
- Dependências: R7.01 (mesmo arquivo). Risco: baixo.

**R7.03** | documentar | média | MARCELO (decisão)
- Problema: a abertura tem zero `decisao!` (confirmado) e cerca de 360 linhas com `if` ou `match` (contagem do subagente). O plano da Etapa 7 (linha 32) e o requisito de arquitetura mandam anotar. A Etapa 14 exige cobertura fechada.
- Mudança pedida: decidir se a anotação entra agora (recomendado: sim, por arquivo, antes da Etapa 9) e autorizar o bloco O7. Se não, vira backlog.
- Dependências: nenhuma. Risco: médio.

**R7.04** | melhorar | média | IA
- Arquivos: `diferencial-abertura/src/lib.rs`, `diferencial-abertura/tests/fumaca.rs`.
- Problema: `Relatorio::zero()` ignora `ram_diverge_atomico` e `z80_diverge_atomico` e só reprova o aviso `EsperaDoSomEstourada` (confirmado: `lib.rs:103-116`). Os números finais de trechos atômicos e de "fechamento não comparável" não aparecem no registro do critério (subagente: `abertura.md:294-301`). O uso fora do inventário também não reprova o veredito.
- Mudança: imprimir `ram_comparada: esperas=A fechamentos=B atomicos=C nao_comparaveis=D de Q` e uma linha `veredito_sem_trecho_atomico`. Acrescentar a falha por avisos fora do inventário do núcleo e do vídeo.
- Teste (antes): a fumaça confere as linhas, com `atomicos=0` para o programa parado. Um aviso sintético reprova.
- Aceite: testes verdes. Com a ROM, se houver aviso, o veredito muda, e isso é o objetivo.
- Verificação: `cargo test --release --manifest-path diferencial-abertura/Cargo.toml` (precisa do oráculo preparado em `/mnt/trabalho/oracle`; se não houver, depende de você). Depende de você: `sh scripts/etapa7-bloco2.sh "$ROM" 1170` e registrar os números em `abertura.md`.
- Dependências: nenhuma. Risco: médio.

**R7.05** | melhorar | média | IA
- Arquivos: `crates/console/src/lib.rs`, `crates/web/src/lib.rs`, `site/app.js`, `crates/console/tests/ponte.rs`.
- Problema: `requisitos.md:111` diz que, no site, o uso fora do inventário "só registra um aviso". `Console::diagnostico` existe (confirmado, `console/lib.rs:357-368`), mas `web/src/lib.rs` e `app.js` não o expõem (subagente).
- Mudança: `Jogo::avisos()` devolve as contagens do núcleo e da VDP. `app.js` chama `console.warn` quando o valor muda.
- Teste (antes): ROM sintética com escrita na ROM gera aviso não vazio, em `crates/console/tests/ponte.rs`.
- Aceite: o teste passa em nativo e wasm.
- Verificação: `cargo test -p sonic1-web -p sonic1-console`; `cargo test --target wasm32-wasip1 -p sonic1-console`.
- Dependências: R0.06. Risco: baixo.

**Bloco O7 (R7.06 a R7.10), opcional, só com a aprovação do R7.03.** Mesmo molde: `if decisao!("Rotina:desvio", cond)` em cada `if` ou `match` que espelha desvio do original, com o rótulo no id, sem mudar o comportamento. Conferir contra o `.asm` de texto. Teste: com `--features cobertura`, rodar o cenário sintético já existente e afirmar os ids registrados, com o caminho não tomado onde o teste atual permitir. Verificação: `cargo test -p sonic1-core -p sonic1-teste-som --features sonic1-core/cobertura`; `sh .github/lints-core.sh`. Depende de você: `sh scripts/etapa7-bloco2.sh "$ROM" 1170` segue IGUAL. Risco: médio. Um erro mecânico mudaria uma prova já feita, então cada item exige a rodada do Marcelo. Dependem de R7.01 e R7.02.

| ID | Arquivos (cada um mais um teste novo em `crates/teste-som/tests/`) | Desvios |
| --- | --- | --- |
| R7.06 | `abertura/objetos.rs`, `animacao.rs`, `paleta.rs`, `hud.rs` | cerca de 91 |
| R7.07 | `abertura/fisica.rs` | 88 |
| R7.08 | `abertura/cenario.rs`, `desenho.rs`, `fase.rs`, `objpos.rs` | 88 |
| R7.09 | `abertura/mod.rs`, `titulo.rs`, `vblank.rs` | cerca de 60 |
| R7.10 | `abertura/plc.rs`, `sega.rs`, `boot.rs` | cerca de 37 |

### Etapa 8 (sintetizador)

**R8.01** | documentar | alta | MARCELO (decisão)
- Problema: `crates/audio/src/fm.rs:15-27` tem tabelas com os mesmos nomes (em maiúsculas), as mesmas dimensões e os mesmos valores das tabelas estáticas de `ym3438.c` do Nuked-OPN2 (LGPL 2.1 ou superior): `fm_algorithm[4][6][8]`, `eg_stephi[4][4]`, `pg_detune[8]`, `fn_note[16]`, `op_offset[12]`, `ch_offset[6]`. Confirmado por `grep` no fonte público do Nuked. Esses nomes não estão em `ym3438.h`. `docs/conhecimento/sintese.md` (D8.2) diz que a IA "não abriu o corpo do `ym3438.c`" para o FM, e `crates/audio/Cargo.toml:2-3` diz "nenhum código de emulador de referência entra aqui". O PSG é admitido como "derivado do comportamento do Nuked-PSG" (GPL 2 ou superior), mas `psg.rs` não diz isso. O crate herda a licença MIT do workspace e vai para o WebAssembly do site. Não há prova de cópia de código, só de semelhança estrutural.
- Mudança pedida: escolher (a) tratar `fm.rs` e `psg.rs` como derivados e ajustar licença e aviso; (b) reescrever limpo, por outra sessão, usando o R8.07 como contrato; (c) aceitar e registrar o risco.
- Aceite: decisão escrita em `docs/conhecimento/sintese.md`.
- Dependências: nenhuma. Risco: alto. Afeta a distribuição.

**R8.02** | documentar | média | IA
- Arquivos: `crates/audio/Cargo.toml`, `crates/audio/src/fm.rs` (só o cabeçalho), `crates/audio/src/psg.rs` (só o cabeçalho), `docs/conhecimento/sintese.md`.
- Problema: as frases sobre a origem do código se contradizem (R8.01).
- Mudança: corrigir as frases de acordo com a decisão do R8.01. Não criar `LICENSE` nem `NOTICE`: isso é do Marcelo.
- Aceite: nenhuma frase do repositório afirma o que a decisão desmente.
- Verificação: `cargo fmt --all --check`; `cargo clippy -p sonic1-audio --all-targets -- -D warnings`; `git grep -n 'Nenhum código de emulador' -- crates/audio`.
- Dependências: R8.01. Risco: baixo.

**R8.03** | corrigir | alta | IA
- Arquivos: `docs/pedidos/etapa8-ci.patch`, `docs/pedidos/etapa8-relatorio.patch`.
- Problema: os dois patches estão corrompidos (confirmado: `git apply --check` dá "patch corrompido em docs/pedidos/etapa8-ci.patch:7" e "...etapa8-relatorio.patch:12"; os cabeçalhos dos blocos declaram mais linhas do que o corpo tem). Mesmo corrigidos, mexem nas mesmas linhas dos patches da etapa 7 e não aplicariam em sequência (confirmado: os dois pares partem da mesma base). O `estado.md` manda o Marcelo aplicá-los.
- Mudança: recriar os dois para aplicarem depois dos da etapa 7. O CI passa a testar em wasm: `-p sonic1-core -p sonic1-rom -p sonic1-video -p sonic1-console -p sonic1-audio -p sonic1-audio-saida -p sonic1-teste-nucleo -p sonic1-teste-descompressao -p sonic1-teste-som`. O manifesto passa a listar `diferencial-sintese` (os de `diferencial-som` e `diferencial-abertura` vêm do patch da etapa 7).
- Teste: num worktree limpo, aplicar `etapa7-ci`, `etapa7-relatorio`, e depois `etapa8-ci` e `etapa8-relatorio`, todos com `git apply --check` e depois `git apply`.
- Aceite: os quatro patches aplicam em sequência. O comando de wasm abaixo passa no contêiner, senão o item registra o que falha.
- Verificação: `git apply --check` nos quatro, em sequência, num worktree; `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-rom -p sonic1-video -p sonic1-console -p sonic1-audio -p sonic1-audio-saida -p sonic1-teste-nucleo -p sonic1-teste-descompressao -p sonic1-teste-som`.
- Dependências: nenhuma. Risco: baixo. Os testes de `video`, `console`, `audio` e `teste-som` nunca rodaram em wasm no CI.

**R8.04** | corrigir | crítica | IA
- Arquivos: `crates/audio-saida/src/misturador.rs`, `crates/audio-saida/tests/saida.rs`.
- Problema: o controlador de taxa tem o sinal invertido e mede a quantidade errada (confirmado). Em `atualizar_controlador_taxa` (`misturador.rs:144-155`) o ajuste vale `-ajuste`, e `reamostrador.rs:146` faz `passo = base * (1 + ajuste_fino)`. Passo maior gera menos amostras, então buffer cheio gera mais amostras: realimentação positiva, contra o comentário do código. Além disso, `console/lib.rs:335-341` esvazia o misturador a cada quadro, então `ocupacao` fica perto de 800, nunca perto do alvo de 2400. O erro fica em torno de -1600 e o ajuste satura em cerca de +0,32%, uma queda constante da taxa de saída (cerca de 150 amostras por segundo a menos). O worklet consome 48000 por segundo.
- Mudança: corrigir o sinal e aceitar a ocupação medida fora, na fila do worklet: `definir_ocupacao_externa(pares)`. Sem valor externo, não ajustar. Limitar a fila interna.
- Teste (antes): produtor a 60 Hz exatos (888 ou 889 amostras de FM e 3733 ou 3734 de PSG por quadro) contra consumidor de 48000 por segundo, durante 36000 quadros, com a ocupação da fila informada ao controlador. A ocupação deve convergir ao alvo (mais ou menos 10%), sem falha. Repetir com o relógio de áudio 0,1% mais rápido e com 44100 Hz. Com o código atual o teste falha.
- Aceite: os testes passam em release. Nenhum teste existente enfraquece.
- Verificação: `cargo test -p sonic1-audio-saida --release`; `cargo clippy -p sonic1-audio-saida --all-targets -- -D warnings`.
- Dependências: nenhuma. Risco: médio. Muda o áudio do navegador, não a prova amostra a amostra. Inferência por leitura: só a escuta do Marcelo confirma (R8.12).

**R8.05** | corrigir | crítica | IA
- Arquivos: `crates/console/src/lib.rs`, `crates/web/src/lib.rs`, `site/app.js`, `site/audio-worklet.js`, `crates/console/tests/audio.rs`.
- Problema: o critério pede música "reamostrada sem cortes". O console fixa 48000 Hz (`console/lib.rs:246`, confirmado). A página só mostra a fila do worklet na tela (`app.js:118-122`, confirmado) e não devolve o valor ao núcleo. O worklet começa a tocar com a fila vazia, sem pré-carga (`audio-worklet.js:35-55`, confirmado), e descarta em silêncio o que passa do tamanho (`:25-30`). O plano pedia pré-carga de três quadros (subagente: `plano-etapa-8.md:66`).
- Mudança: `Jogo::criar(dados, taxa_hz)` com a taxa real do `AudioContext`. A página repassa ao núcleo a ocupação da fila do worklet (`Jogo::fila_do_audio(n)`). O worklet só consome depois de 2400 pares, volta a esperar a pré-carga depois de um esvaziamento e conta descartes no status.
- Teste (antes): em `crates/console/tests/audio.rs`, alimentar `fila_do_audio` com valores simulados e conferir o ajuste. Teste com 44100 Hz.
- Aceite: testes verdes em nativo e wasm.
- Verificação: `cargo test -p sonic1-console -p sonic1-web`; `cargo test --target wasm32-wasip1 -p sonic1-console`. Depende de você: abrir o site com a ROM por cinco minutos e ver "Falhas: 0" e a fila estável.
- Dependências: R8.04, R7.05. Risco: alto. O JavaScript não tem teste automático.

**R8.06** | documentar | média | IA
- Arquivos: `docs/estado.md`, `docs/backlog.md`, `site/index.html`.
- Problema (subagente): no navegador não há bateria nem voz SEGA, porque o Z80 só anda com pontos gravados e o tempo nominal não os gera (`memoria/mod.rs:282-314`, `console/lib.rs:255-275`). `index.html:71` só cita a voz. `estado.md` dá "8.0 a 8.10 feitas" sem a escuta (8.7) nem a medida em wasm (8.9).
- Mudança: registrar o limite nos três arquivos e marcar 8.7 e 8.9 como parciais.
- Aceite: o limite está escrito nos três lugares.
- Verificação: `grep -n 'bateria' docs/estado.md docs/backlog.md site/index.html`.
- Dependências: R5.06. Risco: baixo.

**R8.07** | implementar | alta | IA
- Arquivos: `crates/audio/tests/propriedades.rs` (novo), `crates/audio/tests/regressao.rs` (novo).
- Problema: `crates/audio/src` não tem teste unitário (confirmado: zero `cfg(test)`). A prova contra o Nuked só roda no host, com os fontes C. O CI não prova nada do FM nem do PSG.
- Mudança: (1) escritas aleatórias em todas as portas e instantes por 2 milhões de ciclos: sem pânico, FM em 9 bits, PSG de 0 a 15, determinismo e reset. (2) Regressão: hash dos fluxos de cinco cenários (algoritmo 7, seis canais, envelope, DAC, PSG com ruído), fixado a partir do código atual, que o Marcelo provou IGUAL ao Nuked. Vale como guarda, não como prova.
- Aceite: testes verdes em nativo e wasm.
- Verificação: `cargo test -p sonic1-audio`; `cargo test --target wasm32-wasip1 -p sonic1-audio`.
- Dependências: nenhuma. Risco: baixo.

**R8.08** | corrigir | média | IA
- Arquivos: `diferencial-sintese/src/bin/sintese.rs`, `scripts/etapa8-bloco2.sh`.
- Problema (subagente, `sintese.rs:215-257`): pasta ausente ou trace ilegível só gera `eprintln`, e o relatório pode sair com "VEREDITO GERAL: IGUAL" e código 0 com resultados vazios. O `etapa8-bloco2.sh:28, 31-32` mascara a falha do `cargo test` com `grep | head`. O fim do veredito está na linha 295 (confirmado).
- Mudança: opção `--esperar caso1,caso2,...`. Caso ausente ou erro de leitura vira DIVERGE (código 1). O script passa os casos do corpus e usa o código de saída do `cargo test`.
- Teste (antes): pasta vazia com `--esperar` deve dar código 1.
- Aceite: o teste falha antes e passa depois.
- Verificação: `sh -n scripts/etapa8-bloco2.sh`. Depende de você (fontes do Nuked e `cc`): `sh scripts/etapa8-bloco2.sh "$ROM" 600`.
- Dependências: R0.07. Risco: baixo.

**R8.09** | melhorar | média | IA
- Arquivos: `diferencial-sintese/tests/producao.rs` (novo).
- Problema (subagente): o comparador usa `Fm` e `Psg` com agendador próprio (`comparador.rs:10-11`). O navegador usa `Sintetizador` (`sintetizador.rs:203-286`) e `misturado`. O critério fala em "escritas do oráculo nos mesmos instantes", e esse caminho não é comparado ao Nuked.
- Mudança: alimentar `Sintetizador` e os chips de referência com a mesma bateria sintética e comparar ciclos e canais.
- Aceite: IGUAL na bateria. Documentar se o navegador usa `misturado` ou `picos_pino`.
- Verificação: `NUKED_DIR=/mnt/trabalho/nuked cargo test --release --manifest-path diferencial-sintese/Cargo.toml --test producao` se o clone do Nuked estiver no contêiner. Senão, depende de você.
- Dependências: nenhuma. Risco: baixo.

**R8.10** | melhorar | baixa | IA
- Arquivos: `crates/audio-saida/src/analogico.rs`, `crates/audio-saida/tests/saida.rs`, `diferencial-sintese/src/bin/sintese.rs`.
- Problema (subagente): a parte analógica é "medida" só pela fórmula (`analogico.rs:84-100`). O relatório só traz a linha fixa APROXIMADO. O corte de 3386 Hz não cita fonte.
- Mudança: passar senoides de 100 Hz a 20 kHz (oito frequências) pelo filtro e imprimir os ganhos medidos no relatório, com a fonte do corte ou "SEM FONTE".
- Teste: o ganho medido a 3386 Hz fica em -3 dB, mais ou menos 0,5.
- Aceite: teste verde e as oito linhas no relatório.
- Verificação: `cargo test -p sonic1-audio-saida`.
- Dependências: R8.04, R8.08. Risco: baixo.

**R8.11** | documentar | média | MARCELO (decisão)
- Problema: `requisitos.md:145` diz que o áudio usa um AudioWorklet "alimentado por eventos de escrita nos chips". O worklet recebe PCM já reamostrado (confirmado, `audio-worklet.js:20-31`).
- Mudança pedida: aceitar o desvio e atualizar o requisito na fonte, ou pedir a síntese dentro do worklet (item grande, depois da Etapa 8).
- Dependências: nenhuma. Risco: médio.

**R8.12** | implementar | alta | MARCELO
- Problema: a escuta no navegador e a medida de desempenho em wasm nunca foram registradas. Os patches da etapa 7 e 8 não foram aplicados.
- Verificação (depende de você): aplicar os patches (R2.07); `cargo xtask site` e servir `dist/` por HTTP local (`docs/conhecimento/esqueleto.md:39`); tocar a música por cinco minutos; `sh scripts/etapa6-bloco2.sh`; `sh scripts/etapa7-bloco2.sh "$ROM" 1170`; `sh scripts/etapa7-bloco5.sh "$ROM" 1200`; `sh scripts/etapa8-bloco1.sh`; `sh scripts/etapa8-bloco2.sh "$ROM" 600`.
- Aceite: "Falhas: 0" estável no painel. `VEREDITO GERAL DA ETAPA: IGUAL`. Nativo igual a wasm nos 1200 quadros.
- Dependências: R8.03, R8.05. Risco: médio.

## 7. Verificação

### 7.1 Base (fase 2, no contêiner; comandos que existem)

```
git status --porcelain
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
sh .github/checa-core.sh
sh .github/lints-core.sh
cargo xtask anti-rom
cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-rom -p sonic1-teste-nucleo -p sonic1-teste-descompressao
cargo fmt --check --manifest-path crates/fidelidade/relatorio/Cargo.toml
cargo clippy --locked --manifest-path crates/fidelidade/relatorio/Cargo.toml --target-dir target/relatorio --all-targets -- -D warnings
cargo test --locked --manifest-path crates/fidelidade/relatorio/Cargo.toml --target-dir target/relatorio
cargo xtask site
sh .devcontainer/testes/hooks.sh
```

Esperado: árvore vazia, tudo verde, `hooks.sh` com 14 de 14. Gravo a saída resumida no registro como referência dos gates. Falha que já existia na base é registrada e não conta como regressão.

Aviso sobre o manifesto: o manifesto assinado cobre `.cargo/config.toml`, `refs`, `oracle`, `oracle2`, `crates/fidelidade`, `crates/corpus`, `corpus`, `descompressao`, `cobertura`, `diferencial` e `crates/teste-nucleo` (confirmado em `manifesto.rs:29-41`). Alterar um arquivo existente nesses caminhos invalida a assinatura. Arquivo novo não é acusado. Teste assinado que sumir ou ficar ignorado reprova. `manifesto verificar` precisa de `ssh-keygen`, que a imagem não tem. Por isso o gate de manifesto é sempre PENDENTE (reassinatura do Marcelo), nunca regressão. Itens afetados: R3.01, R5.02, R6.05.

### 7.2 Por etapa

| Etapa | A IA roda no contêiner | Depende de você |
| --- | --- | --- |
| 0 | `sh .devcontainer/testes/hooks.sh`; `sh .devcontainer/testes/anti-rom.sh`; `sh .devcontainer/verifica-config.sh`; `cargo test -p sonic1-rom -p xtask`; `cargo xtask anti-rom`; `cargo xtask site`; `sh .github/checa-core.sh`; `sh .github/lints-core.sh` | R0.08; `claude doctor` fora do Claude |
| 1 | `cargo test -p sonic1-fidelidade`; `cargo test -p xtask rotulos` | `sh oracle/scripts/monta-disasm.sh montar s1built.bin sonic.lst 16` (esperado `ROM montada: IDENTICA`); `sh oracle/scripts/roda-oraculo.sh "$ROM" 600` (esperado `Prova de determinismo: PASSOU`) |
| 2 | `cargo test -p sonic1-corpus -p xtask`; os três comandos do `relatorio` da seção 7.1; `cargo test --manifest-path oracle2/Cargo.toml --target-dir target/oracle2` | R2.03, R2.07 |
| 3 | `cargo test -p sonic1-core -p sonic1-teste-descompressao -p xtask`; `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-descompressao`; `sh .github/lints-core.sh` | R3.02 |
| 4 | `cargo test -p sonic1-video -p sonic1-formato-video -p xtask`; `cargo test --target wasm32-wasip1 -p sonic1-video` | R4.03; medida wasm; último run do CI |
| 5 | `cargo test --workspace`; `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-nucleo -p sonic1-teste-descompressao`; comandos do `relatorio` | R5.03; `cargo fidelidade mutantes --orcamento 600 --limite 120` |
| 6 | `cargo test -p sonic1-core -p sonic1-teste-som`; `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-som`; `sh .github/lints-core.sh` | `sh scripts/etapa6-bloco2.sh` (esperado 78 IGUAL, 0 DIVERGE) |
| 7 | `cargo test -p sonic1-core -p sonic1-teste-som -p sonic1-console -p sonic1-web`; `cargo xtask site` | `sh scripts/etapa7-bloco2.sh "$ROM" 1170`; `sh scripts/etapa7-bloco5.sh "$ROM" 1200`; ver o site |
| 8 | `cargo test -p sonic1-audio -p sonic1-audio-saida -p sonic1-console`; `cargo test --target wasm32-wasip1 -p sonic1-audio -p sonic1-audio-saida` | `sh scripts/etapa8-bloco1.sh`; `sh scripts/etapa8-bloco2.sh "$ROM" 600`; escuta; medida em wasm |

Gate de etapa que depende de script que grava em `privado` fica PENDENTE com o comando exato. O R0.07 reduz esses casos.

## 8. Pendências do Marcelo (comando exato quando existe)

Prioridade crítica:

1. Abrir a sessão no contêiner (`devcontainer up --workspace-folder .`). Sem isso a fase 2 não começa.
2. Ativar a L1, passos 1 a 4 da seção 2.2:
   - `git apply --check docs/pedidos/l1-hook-detector.patch docs/pedidos/l1-hook-pre-commit.patch && git apply docs/pedidos/l1-hook-detector.patch docs/pedidos/l1-hook-pre-commit.patch`, depois o ajuste do hook (caminho A, seção 2.4) ou os `rev-R0.01` a `rev-R0.03` (caminho B), depois `sh .devcontainer/testes/hooks.sh` (14 de 14).
   - Controles, com `ROM` apontando para o arquivo: `dd if="$ROM" of=/tmp/controle-positivo.bin bs=1 skip=1000 count=64`; `cargo run --quiet --manifest-path hooks/detector/Cargo.toml -- --rom "$ROM" --arquivo /tmp/controle-positivo.bin` (esperado: código 1); `printf 'texto comum\n' > /tmp/controle-negativo.txt`; `cargo run --quiet --manifest-path hooks/detector/Cargo.toml -- --rom "$ROM" --arquivo /tmp/controle-negativo.txt` (esperado: código 0).
   - `git apply docs/pedidos/l1-permissoes-rom.patch`, `devcontainer build --workspace-folder .`, `devcontainer up --workspace-folder . --remove-existing-container`, e confirmar que a IA lê a ROM.
   - Levar a decisão ao chat de planejamento e exportar `docs/requisitos.md` de novo. A cópia atual traz a coluna Status das etapas 0 a 8 como "Não iniciada" e cita `cargo xtask` onde hoje é `cargo fidelidade` (subagente).
3. R3.02: `sh descompressao/prova.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`.
4. R4.03: `sh scripts/etapa4-host.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`.
5. R8.12: escutar a música no navegador e rodar os blocos da etapa 8.

Prioridade alta:

6. R0.08: `git push origin main`, CI, Pages, imagem (comandos no item).
7. R2.03: `sh scripts/etapa2-inventarios.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`.
8. R2.07: patches, `sh scripts/etapa5-assinar.sh`, conferir a senha da chave (`ssh-keygen -y -P '' -f ~/.ssh/websonic_assinatura_nova` deve falhar), auditoria.
9. Patches da etapa 7 e 8, em sequência, depois do R8.03: `git apply --check docs/pedidos/etapa7-ci.patch docs/pedidos/etapa7-relatorio.patch && git apply docs/pedidos/etapa7-ci.patch docs/pedidos/etapa7-relatorio.patch`, e depois `etapa8-ci.patch` e `etapa8-relatorio.patch`.
10. Sessões de revisão, só do que cada etapa mudou: etapas 1, 3, 4, 5, 6, 7 e 8 (`docs/estado.md`).

Prioridade média:

11. Decisões: R0.04, R1.02, R6.01, R6.02, R7.03, R8.01, R8.11 (e as exclusões do R5.03).
12. R5.03: `sh scripts/etapa5-bloco-b2.sh 2>&1 | tee /tmp/etapa5-bloco-b2.txt`.
13. Testes wasm na imagem: `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-som` (backlog, etapas 6 e 7).

Prioridade baixa (sugestões, não são itens): acrescentar `ram` e `s1tempo` às três listas de extensões bloqueadas (`hooks/lib.sh`, `.gitignore`, `xtask/src/anti_rom.rs`; o teste de igualdade exige as três juntas); rodar `hooks.sh`, `anti-rom.sh` e `verifica-config.sh` no CI; proteção de branch na `main`; checksums do Dockerfile (N8).

## 9. Descartados e backlog mantido

### 9.1 Descartados (não viram item)

- Estilo: comentário do `.cargo/config.toml` ("<site|anti-rom>"), campo `variante` não usado em `fm.rs`, bloco `DEPURAR_VIDEO`, tamanho de `app.js`, `diag_*` sem `reason`, `&& false` morto em `diferencial-sintese`.
- Os 23 `#[ignore]` de `diferencial-sintese/tests/fm.rs`: 22 são diagnósticos sem `assert` e `diag_ciclo_a_ciclo` tem o mesmo pino comparado em `dinamico_aleatorio`. Nenhum desliga verificação exigida (subagente, conferido pelos nomes).
- `diferencial/src/bin/diferencial.rs:293` (sucesso com zero casos) e o contador morto de `A1`: ferramenta congelada, risco baixo, a saída mostra a contagem de casos.
- Escopo padrão de `cargo fidelidade mutantes` incluir ids do som: a suíte padrão não os exercita. Já está no backlog (linha 61, Etapa 14). Passe `--pacotes` na hora de rodar.
- Linhas `pasta` no manifesto para acusar arquivo novo: já está no backlog (linha 39). As pastas congeladas são protegidas por permissão.
- Medida do tempo de boot inteiro e erro de 1,54% da voz SEGA: Etapa 15 (o experimento da Etapa 7 não trava a etapa).
- DAC no navegador pelo Z80 em tempo nominal: Etapa 15. O R8.06 documenta o limite.
- Aviso do FM para registradores fora do inventário: depois do R7.05 abre-se item na Etapa 13.
- Bench com pior caso de sprites e asserção de 3 ms: o critério não pede. Fica no backlog junto com a medida em wasm (Marcelo).
- Licença do crate `video` (nomes e constantes parecidos com o jgenesis, GPLv3): sem evidência de cópia literal. Decisão do Marcelo, junto com o R8.01.
- `V_NGFX_BUFFER` duplicada e `Blk256` vs rótulos: sem efeito hoje.
- Regra de leitura da ROM em `verifica-config.sh` (a L1 inverte a regra e o patch não toca no script): tratar na hora da L1.

### 9.2 Backlog mantido (itens de docs/backlog.md fora da faixa), com motivo

- Linhas 5, 6, 7, 8 (N8, N9, S1b, proteção de branch): endurecimento, dono Marcelo, não exigido pelo critério.
- Linhas 11, 32, 34, 35, 36 (segundo oráculo por quadro, eventos exatos, VRAM do segundo oráculo, entradas): Etapas 14 e 15.
- Linhas 17, 37, 40 (TAS, BK2): o Marcelo aceitou o corpus sem TAS em 08/11 (Etapa 14).
- Linhas 18, 22 (desempenho do renderizador, medida em wasm): não exigido pelo critério da Etapa 4.
- Linhas 19, 20, 21, 23, 25: ampliação do corpus e recursos da VDP, Etapas 9 a 14.
- Linhas 24, 26, 27: lista estática de memória compartilhada, tamanhos no mapa e E/S do 68000 (o `inventario-barramento.md` cobre por região). Etapa 5 já fechada pelo Marcelo; o R5.06 reclassifica.
- Linha 28 (sistema sob teste real dos ocultos): só existe quando o jogo existir (Etapas 9 e 10). O R5.06 reclassifica.
- Linhas 29, 39, 42, 43, 44 (teste de assinatura sem `ssh-keygen`, arquivo novo no manifesto, mover `tas` para o `relatorio`, CI do `relatorio`, `registrar` sem status): a 43 já foi atendida (`ci.yml:68-72`) e entra no R2.06; a 44 entra no R2.05; as outras ficam.
- Contêiner de auditoria, exclusão da pilha, universo de desvios, volta do pino TH, orçamento de quadro, barramento aberto, testes do `diferencial` fora do manifesto e rodada completa dos mutantes (linhas 33, 38, 45, 47 a 53): Etapas 9 a 15 ou auditoria. A decisão sobre a pilha é do R5.03.
- Bugs conhecidos do som, IDs de som sem comportamento, roteiros BK pendentes, cobertura e mutantes do som e registradores com lixo (linhas 54 a 57, 59 e 61): Etapa 14.
- Linha 46 (`decisao!` em Nemesis): entra no R5.01.
- Linha 13 (`NemDec` pela VDP): entra no R5.04.
- Linha 12 (fonte primária dos formatos): obsoleta, 185 de 185 deu IGUAL. O R5.06 fecha.
- Linhas 70 e 71: a 70 entra no R7.02; a 71 só se a comparação acusar.
- Etapas 6 a 8, o que sobra: os patches do relatório e do CI (linhas 60, 64 e 77) e os testes wasm na imagem (58 e 65) entram no R8.03 e no R8.12. O experimento do nível 2 e a previsão do tempo (63 e 72) ficam para a Etapa 15. `NaoImplementado` e o quadro 1166 (68, 73 e 74) ficam para as Etapas 9 e 10. A fila de arte só provada com fluxos sintéticos (69) foi fechada pelo veredito IGUAL da Etapa 7. O modo leve do sintetizador e a calibração analógica (75 e 76) ficam para a Etapa 15. Os quadros com VBlank fora das esperas, sem comparação de RAM (66), ficam visíveis pelo R7.04. O manifesto da Etapa 5 que não vale mais (62) é do R2.07.

## 10. Limites (o que a IA não consegue verificar)

- Nada foi executado nesta fase: nem `cargo`, nem testes, nem scripts. As contagens vêm de leitura.
- O que eu li inteiro: `docs/requisitos.md`, `CLAUDE.md`, `docs/estado.md`, `docs/backlog.md`, `docs/conhecimento/README.md`, `docs/plano-fechamento-etapa-2.md`, as 120 primeiras linhas de `docs/bugs-conhecidos.md`, os patches da L1 e dos pedidos pendentes, os hooks e o `ci.yml`. Os planos `docs/plano-etapa-2.md` a `docs/plano-etapa-8.md` e os tópicos de `docs/conhecimento/` foram lidos pelos três subagentes (um por grupo de etapas). Eu conferi no código as alegações que mais pesam; as outras estão marcadas "subagente".
- Não rodei os controles do detector nem procurei a ROM: o teste 1 da seção 0.3 já falhou.
- Nada com a ROM: provas IGUAL, hashes de relatório, o que o detector da L1 acha, comparação de bytes dos testes com a ROM e as tabelas do R7.01. O Marcelo reproduz.
- O estado do GitHub (runs, Pages, proteção de branch), a imagem do contêiner em uso e as senhas das chaves.
- A escuta do áudio no navegador. A conta do R8.04 e do R8.05 é inferência por leitura de código.
- A leitura dos subagentes foi parcial em alguns arquivos grandes (`driver.rs` cerca de 700 de 2196 linhas, `fisica.rs`, `cenario.rs`, `desenho.rs`, `titulo.rs`, `vblank.rs`, `abertura/mod.rs`). Os 360 desvios do R7.03 vêm de contagem, não de leitura.
- Não li o `.asm` de texto. A conferência dos `decisao!` (R6.04, bloco O7) e dos chamadores (R5.02) fica para a execução, no contêiner.
- Itens marcados "subagente" foram lidos por um agente e não por mim. Cada item reconfere antes de mudar.
- Os testes ocultos, as chaves e a auditoria são do Marcelo. A IA não os vê.
- O corpus visível mede progresso. A prova é o conjunto oculto e a auditoria. Nenhuma etapa fecha só porque o corpus visível passou.

## 11. Subagentes previstos

- Dois por item de IA ou PATCH: um Haiku (implementação) e um Sonnet (revisão com aplicação). São 37 itens (32 de IA e 5 de PATCH), ou 74 chamadas. Com o bloco O7 são 42 itens, ou 84 chamadas.
- Mais dois: a revisão final (Explore, Sonnet, esforço max) e o documento `docs/revisao-etapas-0-8.md` (Haiku, esforço max).
- Total: 76 chamadas sem o bloco O7, 86 com ele. Itens do MARCELO (14) não usam subagente.
- Itens PATCH e o R8.03: o Haiku trabalha num worktree do scratchpad (`git worktree add --detach`), valida com `git apply --check` num worktree limpo e remove o worktree no fim. O gate de arquivos aceita só `docs/pedidos/rev-<ID>.patch` (ou, no R8.03, os dois arquivos `etapa8-*.patch`).
- Gate de arquivos: cada caminho alterado precisa estar em "Arquivos" do item. Caminho congelado alterado reprova o item e para a execução.
- Antes do primeiro item da fase 2: gravar este plano em `docs/plano-revisao-etapas-0-8.md` (commit local), as regras fixas em `regras-fixas.md` e o `registro.md` no scratchpad, e rodar a verificação de base.

## 12. Registro da aprovação (12/11/2026)

- O Marcelo aprovou o plano acima.
- Pergunta feita na aprovação: executar o bloco P (R0.01 a R0.03) agora, no host, mesmo sem contêiner e sem L1 ativa. Resposta: sim, bloco P completo (caminho B da seção 2.4).
- Escopo autorizado: só texto, sem ler a ROM, três patches em `docs/pedidos/`, commits locais. O resto do plano (R0.04 em diante) espera o contêiner e a L1 ativa.
- Estado das decisões da seção 2.5: só o caminho foi decidido. As outras decisões seguem abertas, e os itens que dependem delas ficam BLOQUEADOS até a decisão.
- Registro da execução: `docs/registro-revisao-etapas-0-8.md`. Em outra sessão, ler este plano e esse registro e continuar no primeiro item sem status final.
