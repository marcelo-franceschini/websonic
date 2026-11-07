# Plano de fechamento da Etapa 2

Escrito em 07/11/2026 pela sessão de revisão da Etapa 2. O Marcelo pediu: "só quero meus objetivos e requisitos atendidos". As decisões abaixo são da IA e seguem `docs/requisitos.md`. Pedir a execução deste plano numa sessão no contêiner vale como aprovação dele.

## 1. O que falta para cumprir o requisito

| # | Requisito | Hoje | Fecho |
| --- | --- | --- | --- |
| R1 | "O segundo oráculo concorda nas demos" | 14 amostras, só até o quadro 5600 (GHZ e MZ) | F8 e passo 5 do script |
| R2 | "TAS conferidos contra a REV01 viram traces" | nenhum TAS | F3 |
| R3 | "Demos ... viram traces" | 2 das 4 demos | F4 |
| R4 | "partidas gravadas ... viram traces" | nenhuma partida no `corpus/` | F5 |
| R5 | "a IA nunca altera o que mede", "por bloqueio técnico" | comando de fidelidade, manifesto, ocultos e `corpus-trace` no `xtask`, que a IA edita; `oracle2/` e `crates/corpus/` sem congelamento | F1, F7, F9 |
| R6 | "Trace: hashes por quadro e estados completos a intervalos" | o `corpus-trace` guarda 3 traces completos por caso; um TAS daria de 14 a 18 GB por trace, e há 38 GB livres | F2 |
| R7 | "testes ocultos, nunca vistos pela IA"; "a chave de assinatura exige senha" | a chave dos ocultos está sem senha no computador onde a IA roda; a senha da chave de assinatura não foi conferida | F6 e passo 2 do script |
| R8 | "Onde a IA roda: dentro do contêiner de trabalho" | as sessões rodam no host desde a Etapa 1 | seção 2 |

## 2. Onde cada parte roda

- Sub-etapas 2.10 e 2.11: sessão do Claude Code no contêiner de trabalho. No host não existe bloqueio técnico nenhum.
- Tudo o que usa a ROM: Marcelo, no host, com um script só (`scripts/etapa2-fechamento.sh`).
- Os dois TAS já foram baixados em 07/11/2026 para `$SONIC1_EXT_DIR/trabalho/tas/` (no contêiner, `/mnt/trabalho/tas/`): `3570M/x/aglar-sonic.gmv` e `1937M/x/aglar-nozips-sonicthehedgehog.gmv`. São só botões, sem dado da ROM.

## 3. Decisões

### D-R1. Segundo oráculo

Base: requisitos ("sem evidência, vale o oráculo principal, e a diferença fica registrada") e plano da Etapa 2, seção 8 (diferença de alinhamento se registra). O D1 vira três linhas em `docs/conhecimento/oraculos.md`:

| id | o que difere | causa | decisão |
| --- | --- | --- | --- |
| D1a | o segundo oráculo fecha o quadro um depois | alinhamento do quadro (hipótese) | comparar o principal no quadro N com o segundo no quadro N+1 |
| D1b | pilha, `FFFCC0..FFFDFF` (de `v_systemstack_end` a `v_systemstack`) | bytes antigos que dependem do ciclo da interrupção (hipótese) | vale o principal; tempo é nível 2 (Etapa 15) |
| D1c | `v_vblank_routine`, `v_jpadhold1`, `v_jpadpress1` | escritas perto do VBlank (hipótese) | vale o principal; Etapa 15 |

"Concorda nas demos" passa a significar: em todas as amostras das quatro demos, nada difere fora de D1b e D1c. Amostras novas: 6100, 6600, 7100 (fim da MZ e título), 7600, 8400 (SYZ), 10200, 11000 (special stage). Limite registrado: só a RAM do 68000 é comparada, porque o núcleo libretro do Genesis Plus GX só expõe essa memória.

A exclusão da pilha na comparação do núcleo contra o oráculo é outra decisão, da Etapa 5. Ela é do Marcelo (requisitos, critério 2).

### D-R2. TAS

- Publicações 3570M ("Baseline", 14:13.87) e 1937M ("no zips", 17:36.58), autor Aglar, Gens 11b, formato GMV, jogo "Sonic The Hedgehog (W) (REV01) [!].gen" (páginas lidas em 07/11/2026).
- Licença: https://tasvideos.org/SiteLicense diz "All content on the site is under the Creative Commons Attribution 2.0 license." A atribuição fica no `.meta` (fonte = página da publicação, autor).
- Cabeçalho, conferido em 07/11/2026 (só os 64 bytes do cabeçalho e as contagens de botões): 60 Hz, sem estado salvo (começa no ligar), 51166 e 63395 quadros, controle '6' nas duas portas. A porta 2 nunca tem botão. O byte X/Y/Z/Mode tem botão em poucos quadros: 3570M com Mode do jogador 1 em 145 quadros e Z do jogador 2 em 30; 1937M com 12 e 51.
- Conversor: aceita controle '6', descarta X, Y, Z, Mode e a porta 2, e grava no `.meta` quantos quadros tinham esses botões. Base: o Sonic 1 lê o controle como de 3 botões. Isso é hipótese; a conferência confirma ou descarta. Continua recusando estado salvo (bit 6 do byte 0x16, verificação nova) e 50 Hz.
- Conferência: o oráculo roda o TAS e mais 3600 quadros sem botão, com `--perfil-a-cada 60`. O TAS é aceito se o modo de jogo chega a `id_Ending` ou `id_Credits` (valores de `refs/rotulos.txt`). A 2.10 confere no `.asm` se o jogo liga o bit 7 do modo durante a carga; se ligar, a conferência ignora esse bit. Se não chegar, o TAS é descartado: sai do `corpus/` e ganha uma linha em `corpus/descartados.txt` (nome, fonte, motivo). O requisito aceita o descarte (tabela de riscos).
- Segundo oráculo nos TAS: não, porque o `oracle2` não aceita entradas. Fica no backlog.

### D-R3. Quatro demos

Caso novo `demos-quatro`, 11700 quadros, em `corpus/demos.txt`. O `boot-demos` não muda, porque é teste congelado (requisitos: a IA cria testes novos e nunca altera os congelados).

### D-R4. Partida gravada

`scripts/etapa2-gravar.sh` ganha a ZONA `corpus`. Depois do `REPRODUZ`, grava `corpus/partida-ghz1.s1in` e o `.meta` (fonte: partida gravada pelo Marcelo; licença MIT; autor Marcelo). O Marcelo joga de 1 a 2 minutos da Green Hill.

### D-R5. Congelar o que mede

- O comando de fidelidade, o manifesto, os ocultos e o `corpus-trace` saem do `xtask` para `crates/fidelidade/relatorio/`. Base: requisitos, "Comparador: fica em crates/fidelidade/, junto com o formato do trace e o relatório". A pasta já é congelada, então o código entra por patch do Marcelo.
- Workspace próprio, como o `oracle/runner`. Binário `relatorio` com os subcomandos `fidelidade`, `manifesto`, `ocultos` e `corpus-trace`.
- Atalho `cargo fidelidade` em `.cargo/config.toml` (congelado, mesmo patch), com `--locked` e `--target-dir target/relatorio`, porque a pasta congelada não aceita escrita do compilador.
- O `xtask` perde esses comandos. Não fica cópia.
- O comando `fidelidade` roda também os testes do pacote `relatorio`. Ele falha se algum teste falhar ou faltar.
- Congelam também `oracle2/` e `crates/corpus/` (regra Edit e `denyWrite`).
- O manifesto passa a cobrir `.cargo/config.toml`, o `oracle/` inteiro, o `oracle2/`, o `crates/fidelidade/` inteiro (com o `relatorio`) e o `crates/corpus/`, sempre sem pastas `target`.

### D-R6. Trace sem cópias

O `corpus-trace` compara a segunda execução e a retomada por hash e apaga as duas logo depois. Por caso ficam: o hash do trace, `<nome>.quadros.tsv` (hashes por quadro) e os estados a cada 600 quadros. É o que o requisito chama de trace. O `.s1t` completo só fica com `--manter-trace`, para os casos que o `video-comparar` usa. A saída de vídeo (`--video`) não muda.

### D-R7. Chaves

- A identidade `age` dos ocultos passa a ser cifrada com senha (scrypt do próprio crate `age`). O subcomando novo `ocultos proteger` cifra a identidade atual, e o script apaga a cópia sem senha. A `auditar` lê a senha pela entrada padrão. O script desliga o eco do terminal.
- Chave de assinatura: o script confere se ela tem senha (`ssh-keygen -y -P ''`). Se não tiver, pede uma (`ssh-keygen -p`). A assinatura usa o `ssh-agent` só durante o script, que mata o agente no fim.
- Limite registrado: até hoje a chave dos ocultos estava legível pelas sessões no host. Não há evidência de leitura. Os ocultos não são regravados.

## 4. Sub-etapas

### 2.10 (IA, no contêiner, só dados sintéticos)

- F1. Pacote `relatorio`, montado numa cópia fora da pasta congelada (`/mnt/trabalho/preparo-relatorio/`, feita com `git archive`). Entrega `docs/pedidos/etapa2-relatorio.patch`, que cria `crates/fidelidade/relatorio/` e o atalho. Prova: `git apply --check` numa cópia limpa; na cópia, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test` do pacote passam. Revisar linha a linha que nenhuma verificação do `xtask` ficou mais fraca na mudança.
- F2. `corpus-trace` sem cópias (no F1). Teste com ROM sintética: não sobra arquivo da segunda execução nem da retomada; `quadros.tsv` tem uma linha por quadro; o hash é igual ao do trace completo.
- F3. `cargo xtask tas converter` e `cargo xtask tas conferir`, com o conversor GMV de `crates/corpus`. Testes com GMV inventado: controle '6' aceito, com a contagem dos botões descartados; estado salvo e 50 Hz recusados; conferência com saída de perfil sintética, um caso aceito e um descartado. Converte os dois TAS para `corpus/tas-3570m.s1in` e `corpus/tas-1937m.s1in`, com `.meta`. Os dois ficam abaixo do limite de 256 KB do hook.
- F4. Linha `demos-quatro<TAB>11700` em `corpus/demos.txt`.
- F5. ZONA `corpus` no `scripts/etapa2-gravar.sh`.
- F6. `ocultos proteger` e senha na auditoria (no F1). Testes com chave descartável: a identidade cifrada abre com a senha certa e falha com a errada; a auditoria funciona com a identidade cifrada.
- F7. Cobertura nova do manifesto (no F1). Teste: mudar um arquivo de `oracle2/` ou de `crates/corpus/` acusa a referência.
- F8. D1a, D1b e D1c em `oraculos.md`.
- F9. `docs/pedidos/etapa2-congelar.patch`: `oracle2` e `crates/corpus` no `.devcontainer/managed-settings.json` (regra Edit e `denyWrite`) e no `.devcontainer/verifica-config.sh`.
- F10. `scripts/etapa2-fechamento.sh` (seção 5).
- Fim da 2.10: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` e `cargo xtask anti-rom` passam. Commit local e parar.

### 2.11 (IA, no contêiner, depois do script do host)

Registra o resumo do script em `docs/conhecimento/corpus.md` e `oraculos.md`. Atualiza `docs/estado.md` e `docs/backlog.md`. Faz commit local, incluindo o `corpus/` como o script deixou. Para.

Depois: uma sessão nova revisa só o que a 2.10 e a 2.11 mudaram (seção 7).

## 5. Script único do host

Uso: `sh scripts/etapa2-fechamento.sh ROM`. As partes com o Marcelo ficam no começo (uns 5 minutos). O resto roda sozinho, por horas.

1. Confere que o repositório está limpo. Aplica os dois patches, com `git apply --check` antes.
2. Chaves: senha da assinatura (cria se faltar) e carga no `ssh-agent`; proteção da chave dos ocultos (só na primeira vez) e leitura da senha dela para a auditoria.
3. Grava a partida do corpus. O Marcelo joga de 1 a 2 minutos. Tem de dar `REPRODUZ`.
4. Conferência dos dois TAS. O descartado sai do `corpus/` e vai para `corpus/descartados.txt`.
5. D1 nas amostras novas (`scripts/etapa2-d1c.sh`).
6. `corpus-trace` de todos os casos, manifesto com os ocultos, assinatura e verificação.
7. Relatório completo (`cargo fidelidade fidelidade`).
8. Auditoria dos ocultos.
9. Mata o `ssh-agent`. Grava o resumo em `/tmp/etapa2-fechamento.txt` (só vereditos, contagens, hashes e nomes).

Depois do script, o Marcelo reconstrói a imagem do contêiner, porque o congelamento só vale na imagem nova. Em seguida, cola o resumo na sessão do contêiner (2.11).

Espaço: o maior caso (1937M) usa cerca de 18 GB no pico com o F2. Em 07/11/2026 havia 38 GB livres em `/home`.

## 6. Fora deste plano (vai para o backlog)

- Contêiner de auditoria (`.devcontainer/auditoria/devcontainer.json`, pedido P3 do plano da Etapa 2, nunca entregue). Etapa-alvo: 9.
- Na auditoria, o "passou" tem de vir do comparador congelado, não do sistema sob teste. Etapa-alvo: 5.
- Comparação por quadro com o segundo oráculo, com a máscara de D1b e D1c. Pede patch no oráculo principal para despejar a RAM a cada quadro. Etapa-alvo: 14.
- VRAM, CRAM, VSRAM e RAM do Z80 sem comparação com o segundo oráculo. Etapa-alvo: 14.
- Segundo oráculo com entradas (TAS e partidas). Etapa-alvo: 14.
- TAS 5121M ("100%", formato BK2), junto com o conversor de BK2. Etapa-alvo: 14.
- Exclusão da pilha na comparação do núcleo contra o oráculo: decisão do Marcelo. Recomendação da IA: aprovar. Etapa-alvo: 5.

## 7. Prompt da revisão do fechamento

"Revise só o que as sub-etapas 2.10 e 2.11 mudaram (git log desde o commit que criou docs/plano-fechamento-etapa-2.md), contra docs/requisitos.md e esse plano. Mesmas classes e regras da revisão da Etapa 2 (BLOQUEIA, BACKLOG, DESCARTADO). Confira em especial: o patch do relatorio não enfraquece nenhuma verificação que existia no xtask; o manifesto cobre o que a seção 3 diz; cada TAS aceito ou descartado tem a saída da conferência registrada; a partida do corpus deu REPRODUZ; nada da ROM entrou no git."
