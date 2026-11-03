# Estado do Projeto

Documento de acompanhamento do progresso das etapas.

## Etapa atual

- Etapa: 0 (Esqueleto do repositório).
- Sub-etapa: 0.5 (CI e publicação), implementada. Falta o push e a conferência no GitHub pelo Marcelo.
- Status: a 0.1b e a 0.2 estão commitadas e provadas. Nada aguarda verificação do Marcelo, exceto o push.

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
- Sub-etapa 0.5 (CI e publicação): `.github/workflows/ci.yml` (jobs `verificar` e `publicar`), ações fixadas por SHA, tudo na imagem do contêiner. `actionlint` aprova e os seis comandos de código rodaram com um usuário de runner simulado. A execução no GitHub não foi provada. Ver `docs/conhecimento/ci.md`.
- Sub-etapa 0.6 (Fechamento): não iniciada.

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
- `/status` e `claude doctor` no contêiner: managed settings aceito, sem entradas rejeitadas. Avisos de PATH e método de instalação, sem efeito na segurança, a tratar na 0.6.
- Verificação do Marcelo: `git push`, Pages em "GitHub Actions", CI verde, site publicado aceitando a REV01 (ver `docs/conhecimento/ci.md`).
- Sem push. O push é do Marcelo. Os commits locais até 5264427 ainda não foram publicados.
