# Estado do Projeto

Documento de acompanhamento do progresso das etapas.

## Etapa 2 (Corpus e integridade): revisada, aprovada com três ressalvas (07/11/2026)

Revisão feita por sessão nova em 07/11/2026 (só do que a Etapa 2 mudou). Nenhum achado BLOQUEIA. Três linhas do "Pronto quando" só se sustentam por corte aprovado no plano ou por decisão registrada, e o Marcelo precisa aceitar de forma explícita:
- Segundo oráculo: a linha diz "concorda nas demos" e isso **não** foi cumprido (ver abaixo). Vale só a via do critério 5 do pronto (diferença com decisão registrada).
- TAS: nenhum TAS no corpus e a conferência contra a REV01 não foi implementada (corte padrão do plano, seção 1). O conversor GMV só foi provado com dado sintético.
- Demos: o caso `boot-demos` cobre duas demos e o início da terceira, não as quatro.
Detalhes, evidência e pendências em `docs/backlog.md` e no fim de `docs/conhecimento/corpus.md`.

Rodadas do host de 06 e 07/11/2026. Provas, com comando e resultado em `docs/conhecimento/corpus.md` e `docs/conhecimento/oraculos.md`:
- Patch 0003 do oráculo aplicado (mapa de acessos à RAM por contexto e gravador de entradas); testes do oráculo OK, incluindo o que falha sem a correção dos botões no quadro.
- Inventário de memória compartilhada gerado (`docs/inventario-memoria.md`: 751 variáveis, 546 compartilhadas).
- Gravador (2.2): `gravacao: REPRODUZ` na prova e nas 6 partidas dos testes ocultos.
- Testes ocultos (2.8): um caso cifrado por zona (ghz, mz, syz, lz, slz, sbz); a auditoria imprime `zona: passou (1 casos)` para as seis. Limite: o sistema sob teste da Etapa 2 (`scripts/sut-reproduz.sh`) só confere que o oráculo roda o caso até o fim sem erro. "passou" aqui prova decifração e execução, não fidelidade. O sistema real entra na Etapa 5.
- Manifesto assinado (207 linhas); `cargo xtask fidelidade`: manifesto, determinismo e traces `PASSOU`; `veredito: PASSOU`. Hashes novos: `lz-agua` `9f8d8be1…`, `boot-demos` `52ed49ce…`.
- Segundo oráculo: **não concorda** em todos os quadros, então a linha "concorda nas demos" da Etapa 2 não foi cumprida. D1 foi medido só nos quadros 51, 52, 53, 60, 120 e 600 do `boot-demos` (4, 1, 3, 13 e 197 bytes, conforme a rodada). A região explicava `v_megadrive` (corrigida, provado no quadro 53). O resto são variáveis de tempo, que a hipótese atribui ao ponto de fechamento do quadro em relação ao VBlank. **É hipótese, sem prova**, e a diferença cresce com o tempo. Decisão registrada: vale o oráculo principal. Isso atende o critério 5 do pronto (diferença com decisão registrada), não a linha da Etapa 2.
  - Atualização (07/11/2026, quarta a sexta rodadas, `oraculos.md`): o principal no quadro N contra o segundo no quadro N+1 deixa, em 14 quadros do `boot-demos` (até o 5600), só pilha (`FFFCC0..FFFDFF`) e 3 variáveis de tempo (`v_vblank_routine`, `v_jpadhold1`, `v_jpadpress1`). Nenhuma variável de lógica difere. A causa continua hipótese. Não cobre os quadros 5601 a 7200.
  - Decisão que falta do Marcelo: aprovar a exclusão da pilha na comparação entre oráculos e aceitar "fase de 1 quadro" como a forma de concordar nas demos.
- Limite aceito: os testes ocultos desta etapa são um caso curto por zona; os atos completos ficam para a Etapa 14.
- Revisão da Etapa 2: feita em 07/11/2026. Falta o Marcelo aceitar as três ressalvas acima.

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
