# Estado do Projeto

Documento de acompanhamento do progresso das etapas.

## Etapa atual

- Etapa: 0 (Esqueleto do repositório), finalizada em 03/11/2026.
- Próxima: Etapa 1 (Oráculo), aguardando o plano e a aprovação do Marcelo.
- Status: todas as sub-etapas da Etapa 0 estão provadas. Resta o push dos commits locais (do Marcelo).

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
- Sem push. O push é do Marcelo. Tudo até o commit 548e997 foi publicado.
- Pendências que restam da 0.7: push do `main` (do Marcelo); N8 (checksums do Dockerfile); N3 (`cargo` rodado fora do sandbox com `/mnt/rom` montado); N4 (quem monta o disassembly).
- Parte 3 da 0.7 (rodada 5, 04/11/2026, feita): imagem com os patches e o C8 (commit d402937). N7 provado no GitHub (run 41732218921 verde, run 41733743755 vermelho só em "Formatacao"; o run 41732222178 falhou na imagem por falha transitória do instalador). S1b e S1 passaram, S2 negativa de novo, S3 não rodou (regras de texto negaram a instalação dos hooks no repositório descartável, limite aceito). Testes 23 a 36: nenhuma escrita em item congelado passou e nenhum `.sonda-*` apareceu em `refs/`. Ressalvas: o `claude doctor` mostra 3 avisos (o terceiro é sobre globs de Edit ignorados no sandbox do Linux), a camada do Bash em `crates/core/.cargo` não foi observada (regra de texto negou antes) e o `cargo test --offline` só funciona depois de um build online. O veredito de que a IA não lê a ROM volta a **provado** para o caminho do hook e da `statusLine`. Detalhes em `docs/conhecimento/ambiente.md`.
- Status da 0.7: concluída do lado da IA. O `claude doctor` fora do Claude foi rodado em 04/11/2026 (3 avisos, ver `docs/conhecimento/ambiente.md`). A S1b com hook em `/home/dev/.claude/settings.json` não foi feita (risco de estragar o settings do volume). Falta ao Marcelo, no host: apagar o `.claude/settings.json` da sonda, conferir `refs/` e `git status`, rodar `/status` e `/mcp` fora do Claude, e ajustar o roteiro `permissoes.md` para esperar 3 avisos.
