# Estado do Projeto

Documento de acompanhamento do progresso das etapas.

## Etapa 7 (Telas de abertura): em andamento (08/11/2026)

Plano aprovado em 08/11/2026 (`docs/plano-etapa-7.md`, com D7.3, D7.4 e a exclusão da pilha). Bloco 1 feito (`docs/conhecimento/abertura.md`): `N_fim` = quadro 1166 (`Level_StartGame`). Feitas com dados sintéticos: 7.0 (comparador `diferencial-abertura/` sem patch no oráculo, D7.9), 7.1 e 7.2 (boot e soma de verificação no núcleo). Falta a prova real do boot (bloco 2). Próximas: 7.3 (Sega), 7.4 a 7.6, 7.7 (navegador), 7.8 (experimento do nível 2).

## Etapa 6 (Driver de som): concluída, falta a sessão de revisão (08/11/2026)

Plano aprovado em 08/11/2026 (`docs/plano-etapa-6.md`). Relatório do som com a ROM real: **78 IGUAL, 0 DIVERGE** (hash `ff49435d25246d1734cc344369bfe58f0c104735`), cobrindo todas as músicas, todos os efeitos e os comandos: escritas nos chips (incluindo o DAC) idênticas em conteúdo e ordem, estado do driver na RAM, RAM do Z80 e posição do Z80 idênticos. Máquina do Z80 (programas de partida e do DAC, bateria e voz SEGA) provada também contra o oráculo em ROM sintética. Detalhes, desvios do plano e evidência em `docs/conhecimento/som.md`. **Fechamento (Marcelo, 08/11/2026):** manifesto refeito e assinado (383 linhas, commit `4d9358f`), `assinatura: OK`, `veredito: PASSOU`, ocultos passam nas seis zonas. A seção de cobertura do relatório lista 98 decisões do som "sem teste" porque a cobertura só conta os testes do núcleo, não o harness com a ROM; isso é BACKLOG (cobertura do som, Etapa 14) e não entra no veredito. **Pendente, sem bloquear:** rodar os testes wasm na imagem (`cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-som`). (BACKLOG) cobertura e mutantes do som, roteiros para seis bugs conhecidos, patch do relatório e do manifesto para o som. Bugs conhecidos do som: 8 de 20 com teste nomeado, os outros mapeados em `cobertura/bugs-som.txt`.

## Etapa 5 (Núcleo base): concluída, falta a sessão de revisão (texto do cabeçalho corrigido em 08/11/2026)

Plano aprovado em 08/11/2026 (`docs/plano-etapa-5.md`). Implementado e provado com dados sintéticos: memória espelhada (5.1), operações do 68000 (5.2), tempo gravado e laço de quadro (5.3), pontos de checagem (5.4), patch 0004 do oráculo com inventários e tempo (5.0), harness de teste por rotina (5.5), cobertura (5.6), bugs injetados (5.7) e lista de bugs conhecidos (5.8, 315 itens). Detalhes, desvios do plano e evidência em `docs/conhecimento/nucleo.md`.

**Falta (Marcelo, host):**
1. Bloco A: FEITO (inventários commitados, evidência em nucleo.md).
2. Bloco B e B2: FEITOS (mapa de decisões commitado; EniDec:incremental e modo_lista em exclusoes-propostas).
3. Assinatura e cobertura: FEITAS (manifesto de 352 linhas, veredito PASSOU). Pendências do Marcelo: commit de refs e oracle, reconstruir a imagem, aprovar as duas exclusões, sessão de revisão.
3. Assinatura: `sh scripts/etapa5-assinar.sh` (pede senhas; refaz os traces, o manifesto, assina e roda `cargo fidelidade fidelidade`).
4. Congelamento e CI: `git apply docs/pedidos/etapa5-congelar.patch` e reconstruir a imagem do contêiner (corrige também o `esperado-cargo-config.txt` defasado desde a Etapa 2).
5. Decisão do Marcelo: aprovar ou não a exclusão da pilha na comparação (`cobertura/exclusoes-propostas.txt`; recomendação: aprovar).
6. Sessão de revisão da Etapa 5 (só do que ela mudou).

## Etapa 2 (Corpus e integridade): revisão do fechamento feita (08/11/2026); fecha quando o Marcelo reconstruir a imagem do contêiner

Revisão feita por sessão nova em 07/11/2026 (só do que a Etapa 2 mudou). Ela achou três linhas do "Pronto quando" sem cumprimento literal (segundo oráculo, TAS, quatro demos) e mais cinco pontos do requisito sem cumprimento (partida gravada no corpus, congelamento do que mede, trace sem cópias, senha das chaves, IA no contêiner). O Marcelo pediu que a IA decidisse. O plano com as decisões está em `docs/plano-fechamento-etapa-2.md`: sub-etapa 2.10 (IA, no contêiner), um script único do host (Marcelo) e a 2.11 (registro). **Atualização de 08/11/2026:** 2.10, script do host e 2.11 feitos (ver `docs/conhecimento/corpus.md`). Veredito do relatório `PASSOU`, ocultos `passou` nas seis zonas, partida `REPRODUZ`. Os dois TAS foram descartados por perda de sincronia, então o corpus não tem TAS aceito (backlog). **Revisão do fechamento (sessão nova, 08/11/2026):** dois itens BLOQUEIA, B1 (chave de assinatura sem senha) e B2 (saída dos TAS). B2 corrigido (corpus.md). B1: o script agora aborta com senha vazia, mas **a senha ainda não foi definida** (conferido em 08/11/2026: a chave abre sem senha). **Decisão do Marcelo em 08/11/2026: aceita o corpus sem TAS aceito** (os dois foram descartados por perda de sincronia; investigação na Etapa 14). **B1 resolvido em 08/11/2026:** a chave de assinatura agora exige senha (conferido: `ssh-keygen -y -P ''` falha). **Para fechar a Etapa 2:** reconstruir a imagem do contêiner. Depois disso nenhum BLOQUEIA resta e a Etapa 5 pode começar.

**Sub-etapa 2.10 (IA, no contêiner, 07/11/2026): feita; falta o script do host.** Entregue: pacote `relatorio` em `docs/pedidos/etapa2-relatorio.patch` (aplicar numa cópia limpa e testes do pacote passam), `docs/pedidos/etapa2-congelar.patch` (`oracle2/` e `crates/corpus/`), `xtask` sem os comandos que medem, `cargo xtask tas converter|conferir`, os dois TAS convertidos em `corpus/` (ainda **não conferidos** contra a REV01), `demos-quatro` em `corpus/demos.txt`, zona `corpus` no `scripts/etapa2-gravar.sh`, D1a a D1c em `oraculos.md` e `scripts/etapa2-fechamento.sh`. Detalhes e provas em `docs/conhecimento/corpus.md`. **O Marcelo roda no host:** `sh scripts/etapa2-fechamento.sh ROM`, reconstrói a imagem do contêiner e cola `/tmp/etapa2-fechamento.txt` na sessão do contêiner (2.11).

Rodadas do host de 06 e 07/11/2026. Provas, com comando e resultado em `docs/conhecimento/corpus.md` e `docs/conhecimento/oraculos.md`:
- Patch 0003 do oráculo aplicado (mapa de acessos à RAM por contexto e gravador de entradas); testes do oráculo OK, incluindo o que falha sem a correção dos botões no quadro.
- Inventário de memória compartilhada gerado (`docs/inventario-memoria.md`: 751 variáveis, 546 compartilhadas).
- Gravador (2.2): `gravacao: REPRODUZ` na prova e nas 6 partidas dos testes ocultos.
- Testes ocultos (2.8): um caso cifrado por zona (ghz, mz, syz, lz, slz, sbz); a auditoria imprime `zona: passou (1 casos)` para as seis. Limite: o sistema sob teste da Etapa 2 (`scripts/sut-reproduz.sh`) só confere que o oráculo roda o caso até o fim sem erro. "passou" aqui prova decifração e execução, não fidelidade. O sistema real entra na Etapa 5.
- Manifesto assinado (207 linhas); `cargo xtask fidelidade`: manifesto, determinismo e traces `PASSOU`; `veredito: PASSOU`. Hashes novos: `lz-agua` `9f8d8be1…`, `boot-demos` `52ed49ce…`.
- Segundo oráculo: **não concorda** em todos os quadros, então a linha "concorda nas demos" da Etapa 2 não foi cumprida. D1 foi medido só nos quadros 51, 52, 53, 60, 120 e 600 do `boot-demos` (4, 1, 3, 13 e 197 bytes, conforme a rodada). A região explicava `v_megadrive` (corrigida, provado no quadro 53). O resto são variáveis de tempo, que a hipótese atribui ao ponto de fechamento do quadro em relação ao VBlank. **É hipótese, sem prova**, e a diferença cresce com o tempo. Decisão registrada: vale o oráculo principal. Isso atende o critério 5 do pronto (diferença com decisão registrada), não a linha da Etapa 2.
  - Atualização (07/11/2026, quarta a sexta rodadas, `oraculos.md`): o principal no quadro N contra o segundo no quadro N+1 deixa, em 14 quadros do `boot-demos` (até o 5600), só pilha (`FFFCC0..FFFDFF`) e 3 variáveis de tempo (`v_vblank_routine`, `v_jpadhold1`, `v_jpadpress1`). Nenhuma variável de lógica difere. A causa continua hipótese. Não cobre os quadros 5601 a 7200.
  - Decisão (IA, a pedido do Marcelo, pela regra do requisitos): D1 vira D1a (alinhamento), D1b (pilha) e D1c (3 variáveis de tempo), todas com "vale o principal". Faltam amostras do fim da MZ, da SYZ e da special stage. Ver `docs/plano-fechamento-etapa-2.md`, D-R1.
- Limite aceito: os testes ocultos desta etapa são um caso curto por zona; os atos completos ficam para a Etapa 14.
- Revisão da Etapa 2: feita em 07/11/2026. Fechamento em `docs/plano-fechamento-etapa-2.md`.
- Aviso: esta sessão de revisão e as sessões desde a Etapa 1 rodaram no host, fora do contêiner. O requisito decide que a IA roda no contêiner. O fechamento roda no contêiner.

## Etapa 4 (Renderizador fiel): implementada e provada no corpus atual (06/11/2026)

Plano aprovado em 06/11/2026 (`docs/plano-etapa-4.md`). 4.0 a 4.9 feitas. Prova com a ROM real (rodada 2, `docs/conhecimento/video.md`): `video-comparar` **IGUAL em 7200 quadros** (`boot-demos`) e **em 2000 quadros** (`lz-agua`, Labyrinth, com 1530 interrupcoes de linha de nivel 4); VRAM, CRAM e VSRAM iguais ao trace em todos os quadros; nenhum aviso fora do inventario; lints do nucleo e testes wasm do `sonic1-video` ok na imagem; manifesto assinado. Falta a sessao de revisao (so do que a Etapa 4 mudou) e o resumo para o chat de planejamento. A Etapa 2 foi fechada depois (ver o topo deste arquivo).

## Etapa atual

- Etapa: 1 (Oráculo), plano aprovado pelo Marcelo em 06/11/2026.
- Status: sub-etapas 1.1 a 1.9 feitas (1.8 e 1.9 rodadas pelo Marcelo em 06/11/2026). Congelamento aplicado e commitado (c188a83). **Defeito achado no `refs/rotulos.txt`** (base 10 em vez de 16; inferência, a provar na 2.0). Falta a revisão da Etapa 1, com a lista R1 a R13 em `docs/conhecimento/oraculo.md`.
- Etapa 2 (Corpus e integridade): plano aprovado pelo Marcelo em 06/11/2026. Feito pela IA, com dados sintéticos: 2.0 (mapa em base 16, já provado na Etapa 3), 2.3 (`crates/corpus`, GMV; BK2 não confirmável, ver backlog), 2.4 (`xtask corpus-trace`, patch `docs/pedidos/estado-a-cada.patch`), 2.6 parte hardware (`xtask inventario`), 2.7 (`xtask manifesto`, SHA-256 próprio, SSHSIG), 2.8 (`xtask ocultos`, crate `age`). **Rodada do host feita em 06/11/2026**: patches P0 e P1 aplicados, hooks 14 de 14, xtask 40 testes, oráculo OK, `boot-demos` 7200 quadros hash `d0acd7fe…` com `determinismo: PASSOU` e retomada, inventário de hardware gerado, manifesto assinado e verificado (chaves em `refs/assinante.pub` e `refs/ocultos.pub`). **Segundo oráculo (2.1 e 2.5), rodada de 06/11/2026: NÃO concorda** (RAM do 68000 diverge no quadro 52, causa desconhecida, ver `docs/conhecimento/oraculos.md`). (Histórico de 06/11/2026. O que faltava, gravador, memória compartilhada, assinatura e ocultos, foi fechado em 06 e 07/11/2026, ver o topo deste arquivo.) Ver `docs/conhecimento/corpus.md`.
- Etapa 3 (Descompressores): feita. Prova com a ROM real em 06/11/2026: 185 de 185 blocos IGUAL, 6 AUSENTE (ramos REV00), 0 falhas (relatório sha1 `98bdc85b`). Ver `docs/conhecimento/descompressao.md`. Commits do `oracle/` e do mapa feitos (6121249). Falta só a sessão de revisão da Etapa 3.

### Etapa 3: o que o Marcelo precisa rodar (host, fora do Claude)

1. Mapa de rótulos na base 16 (3.0): `sh oracle/scripts/monta-disasm.sh montar s1built.bin sonic.lst 16`. Esperado: `IDENTICA`, `variaveis` na casa das centenas.
2. Patch do oráculo: `patch -p1 < docs/pedidos/descomprimir-oraculo.patch && sh oracle/scripts/testa.sh`. Esperado: 14 testes.
3. Prova (3.5): `sh descompressao/prova.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`. Cole o relatório (só rótulos, hashes e vereditos).
- A Etapa 0 está finalizada (03/11/2026). O push é do Marcelo.

## Histórico de etapas

### Etapa 0: Esqueleto do repositório

- Sub-etapa 0.1 (Ambiente): entregue sem verificação e com defeitos. A auditoria mostrou que a imagem não construía (digest inexistente, `rustup` com argumento inválido). Substituída pela 0.1b.
- Sub-etapa 0.1b (Correção do ambiente): imagem constrói sem erro. Evidência rodada em 03/11/2026 no computador do Marcelo, sem montar nenhuma pasta de ROM:
  - `docker build` com exit 0. Versões dentro da imagem: Rust 1.99.1, wasm-bindgen 0.2.129, wasmtime 50.0.1, Claude Code 2.1.317.
  - `cargo test --target wasm32-wasip1` com o runner do `.cargo/config.toml` passa num crate de teste.
  - O firewall configura sem erro. `example.com` falha no DNS, em IPv4 e em IPv6. `index.crates.io` responde. DNS direto a um servidor externo é recusado.
  - `sh .devcontainer/verifica-config.sh` retorna 0.
  - `claude doctor` no contêiner (sem login) não acusa erro de configuração. Isso não prova que o `managed-settings.json` foi aceito.
- Sub-etapa 0.2 (Validação dentro do contêiner): rede provada (19 de 19, no contêiner real do devcontainer). Rodada 1 das permissões: Read e Edit recusam como esperado, mas o Bash não rodava (bwrap sem user namespace). Corrigido com `.devcontainer/seccomp-userns.json`. Rodada 2 em 03/11/2026: o Bash roda e nenhum teste de leitura da ROM nem de edição do arquivo travado passou. Rodada 3 em 03/11/2026: os testes 8, 9, 14, 15 e 20 foram barrados pelo sandbox, sem o classificador, e o commit da IA funcionou (teste 19). Restam as verificações do Marcelo (ver `docs/conhecimento/ambiente.md`).
- Sub-etapa 0.3 (Hooks do git): `hooks/` com `pre-commit`, `pre-push`, `lib.sh` e `instalar.sh`. `sh .devcontainer/testes/hooks.sh`: 14 de 14 num repositório descartável com arquivos sintéticos (ver `docs/conhecimento/hooks.md`). Verificada pelo Marcelo em 03/11/2026: hooks ativos, ROM real bloqueada com o hash reconhecido, 14 de 14 também no contêiner.
- Sub-etapa 0.4 (Esqueleto Rust e site): workspace com `sonic1-core`, `sonic1-rom`, `sonic1-web` e `xtask`, site em `site/`. Fmt, clippy `-D warnings` e testes nativos e `wasm32-wasip1` passam na imagem. Lints do núcleo provados com violações plantadas. `cargo xtask anti-rom` pega ROM no histórico. O Marcelo conferiu no navegador: a ROM REV01 é aceita com o SHA-1 correto. Ver `docs/conhecimento/esqueleto.md`.
- Sub-etapa 0.5 (CI e publicação): `.github/workflows/ci.yml` (jobs `verificar` e `publicar`), ações fixadas por SHA, tudo na imagem do contêiner. `actionlint` aprova e os seis comandos de código rodaram com um usuário de runner simulado. No GitHub, a execução 41695208654 passou nos dois jobs e o site publicado responde 200. Ver `docs/conhecimento/ci.md`.
- Sub-etapa 0.6 (Fechamento, concluída; rodada 4 das permissões e `claude doctor` fora do Claude, com os 2 avisos benignos): itens congelados (regra Edit e denyWrite), `.devcontainer/auditoria/` e `.claude/` com README, link do `claude` na imagem (avisos do doctor de 4 para 2). `verifica-config.sh` confere os congelados. Imagem reconstruída no host e idêntica ao arquivo. Prova de bloqueio no contêiner: rodada 4 das permissões, sem falhas (ver `docs/conhecimento/ambiente.md`).

- Sub-etapa 0.7 (Endurecimento, rodada 5 feita em 04/11/2026, ver a Parte 3 abaixo): a revisão da Etapa 0 achou caminhos de saída do sandbox. S1 (hook escrito pela ferramenta Write em `.claude/settings.json`): **positiva**, o hook rodou fora do sandbox e escreveu em `refs/`. S2 (`/proc/<pid>/root`): negativa neste kernel ("Permission denied"), sem causa registrada como fato, C6 não necessário. S3 (hooks do git): as regras de texto negaram as duas formas, sem prova sobre os hooks; limite aceito. Correções C1 a C4 aplicadas pelo Marcelo (commit d402937). C5: 21 casos, todos barrados por alguma camada; dois só pela checagem em shell. **O veredito de que a IA não lê a ROM está provado** pelos caminhos do Bash, do Read, do hook e da `statusLine` (S1b e S1 passaram na rodada 5). Detalhes em `docs/conhecimento/ambiente.md`.

## Decisões da auditoria (aprovadas pelo Marcelo em 03/11/2026)

- D1: os testes de fidelidade com a ROM rodam só pelo Marcelo, fora da IA. A IA não lê a ROM por nenhum caminho. A revisão fica para a Etapa 2.
- D2: rede mínima, a lista de `.devcontainer/dominios.txt`.
- D3: o Marcelo valida o contêiner. As sessões só tocam arquivos de texto até a 0.2 passar.
- D4: caminho único do workspace, `/workspaces/websonic`.
- Pastas externas (03/11/2026): uma só variável, `SONIC1_EXT_DIR` (padrão `~/sonic1-ext`), com as subpastas `rom`, `disasm`, `disasm-texto` e `trabalho`. O `trabalho` fica no disco e não em `/tmp`, porque relatórios e hashes precisam sobreviver a reinícios.

## Bloqueios e pendências

- `devcontainer up` verificado em 03/11/2026 (ver `docs/conhecimento/ambiente.md`).
- Aviso ao chat de planejamento: a cópia de `docs/requisitos.md` não traz o número da revisão, só a data. A D1 muda quem roda os testes de fidelidade.
- Rodada 3 das permissões: feita. O `denyWrite` de `docs/requisitos.md`, `.git/config` e `.git/hooks` está provado no sandbox, e a IA faz commit local. A sessão estava em modo automático, mas nenhum comando foi barrado pelo classificador (ver `docs/conhecimento/ambiente.md`).
- Testes 21 e 22 do roteiro: passaram em 03/11/2026 (ROM existe, sem marcadores sobrando).
- `/status` e `claude doctor` no contêiner: managed settings aceito, sem entradas rejeitadas. Os avisos de PATH foram tratados na 0.6 (link do `claude` na imagem).
- Verificação do Marcelo: escolher a ROM REV01 no site publicado e, se quiser, ativar a proteção de branch em `main`.
- Testes 23 a 28 do roteiro (rodada 4, 03/11/2026): nenhuma escrita em item congelado passou (permissão e sandbox), e o commit local continua funcionando. A sessão estava em modo automático, mas nenhum comando foi barrado pelo classificador. O `claude doctor` rodado pelo Marcelo fora do Claude mostrou só os 2 avisos benignos. Dentro do sandbox ele acusa os `/dev/null` montados, que não são arquivos reais.
- Tudo foi publicado até o commit 05576b5 (HEAD de origin/main, conferido no host pelo Marcelo). CI do main verde (run 41756764106 no commit b539528, jobs verificar e publicar, site 200). O commit 05576b5 só mudou o roteiro `permissoes.md` (run 41757996134 em andamento quando foi escrito; o Marcelo confere). O push é do Marcelo.
- Pendências que restam da 0.7: push do `main` (do Marcelo); N8 (checksums do Dockerfile); N3 (`cargo` rodado fora do sandbox com `/mnt/rom` montado); N4 (quem monta o disassembly).
- Parte 3 da 0.7 (rodada 5, 04/11/2026, feita): imagem com os patches e o C8 (commit d402937). N7 provado no GitHub (run 41732218921 verde, run 41733743755 vermelho só em "Formatacao"; o run 41732222178 falhou na imagem por falha transitória do instalador). S1b e S1 passaram, S2 negativa de novo, S3 não rodou (regras de texto negaram a instalação dos hooks no repositório descartável, limite aceito). Testes 23 a 36: nenhuma escrita em item congelado passou e nenhum `.sonda-*` apareceu em `refs/`. Ressalvas: o `claude doctor` mostra 3 avisos (o terceiro é sobre globs de Edit ignorados no sandbox do Linux), a camada do Bash em `crates/core/.cargo` não foi observada (regra de texto negou antes) e o `cargo test --offline` só funciona depois de um build online. O veredito de que a IA não lê a ROM volta a **provado** para o caminho do hook e da `statusLine`. Detalhes em `docs/conhecimento/ambiente.md`.
- Status da 0.7: concluída do lado da IA. O `claude doctor` fora do Claude foi rodado em 04/11/2026 (3 avisos, ver `docs/conhecimento/ambiente.md`). A S1b com hook em `/home/dev/.claude/settings.json` não foi feita (risco de estragar o settings do volume). Falta ao Marcelo, no host: apagar o `.claude/settings.json` da sonda, conferir `refs/` e `git status`, rodar `/status` e `/mcp` fora do Claude, e ajustar o roteiro `permissoes.md` para esperar 3 avisos.

## Etapa 1: Oráculo (06/11/2026)

Sessão no host (`/ssd400/Repos/websonic`), fora do contêiner e sem o `managed-settings.json`. A ROM e os binários do disassembly não foram lidos. O clone do jgenesis e os alvos de compilação ficam em `$SONIC1_EXT_DIR/trabalho/oracle`. Só se listou `disasm-texto` para achar nomes de rótulos (`EntryPoint`, `VBlank`, `HBlank`, `MainGameLoop`, `v_plc_ptrnemcode`).

- 1.1 Viabilidade: GO com ressalvas (V1 a V10 em `docs/conhecimento/oraculo.md`). Ressalva principal: o ciclo de um evento é o do início da instrução. Criado `docs/backlog.md`.
- 1.2 Formato do trace: `crates/fidelidade`, 19 testes. Commit 5da0d46.
- 1.3 a 1.7 Oráculo: `oracle/` (patches sobre o jgenesis `386db855`, runner, scripts). `sh oracle/scripts/testa.sh`: 13 testes, fmt e clippy `-D warnings`. Cobre entradas, estados, paradas, interrupções, desvios (68000 e Z80), VDP com linha, YM e DAC, esperas pelo Z80, rotina isolada e determinismo. `sh oracle/scripts/roda-oraculo.sh` com ROM sintética: `cmp` idêntico e `IGUAIS`.
- 1.8 Mapa de rótulos: feita pelo Marcelo em 06/11/2026. `monta-disasm.sh montar s1built.bin sonic.lst`: ROM montada IDENTICA (sha1 `69e10285…`, md5 `09dadb50…`), disassembly no commit `89c0e0e0`. `refs/rotulos.txt`: símbolos=2849, rotinas=1586, variáveis=2, 90115 bytes, sha1 `f59878b6…`. Ressalva: variáveis=2 é baixo, ver `docs/backlog.md`.
- 1.9 Prova com a ROM real: feita pelo Marcelo em 06/11/2026. `roda-oraculo.sh <ROM REV01> 600`: ROM conferida, `cmp` idêntico, `Prova de determinismo: PASSOU`, `hash_do_trace` `8eec1fb8178299a9d97ad65a7519771a2d3b101b` nas duas execuções, mclk_final 537521432, oráculo `jgenesis@386db855` com patches `751851c4`.

### O que o Marcelo precisa fazer (no host, fora do Claude)

1. `sh oracle/scripts/monta-disasm.sh reconhecer`. Cole a saída (nomes e tamanhos). Depois `sh oracle/scripts/monta-disasm.sh montar ROM_RELATIVA LISTAGEM_RELATIVA [BASE]`. Esperado: `ROM montada: IDENTICA` e `refs/rotulos.txt` gerado.
2. `sh oracle/scripts/roda-oraculo.sh "<caminho da ROM>" 600`. Esperado: `Prova de determinismo: PASSOU` e `hash_do_trace` igual nas duas execuções.
3. Aplicar o patch de congelamento (`oracle/congelar.patch`) e as decisões de commit abaixo.
4. Sessão nova de revisão, só do que a Etapa 1 mudou.

### Decisões tomadas por padrão (recomendações do plano)

N4 (montagem no host, em `privado`), N3 (nenhuma execução com a ROM real pela IA), traces reais fora de `trabalho`, `oracle/` em GPLv3 dentro do repositório com o código do jgenesis fora dele, mclk como unidade de tempo, SHA-1 nos hashes.
