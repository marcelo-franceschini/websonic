# Estado do Projeto

Documento de acompanhamento do progresso das etapas.

## Etapa atual

- Etapa: 0 (Esqueleto do repositório).
- Sub-etapa: 0.2 (Validação dentro do contêiner), parte das permissões pendente.
- Status: a 0.1b está commitada. A 0.2 tem a rede provada e as permissões pendentes (ver Pendências).

## Histórico de etapas

### Etapa 0: Esqueleto do repositório

- Sub-etapa 0.1 (Ambiente): entregue sem verificação e com defeitos. A auditoria mostrou que a imagem não construía (digest inexistente, `rustup` com argumento inválido). Substituída pela 0.1b.
- Sub-etapa 0.1b (Correção do ambiente): imagem constrói sem erro. Evidência rodada em 03/11/2026 no computador do Marcelo, sem montar nenhuma pasta de ROM:
  - `docker build` com exit 0. Versões dentro da imagem: Rust 1.99.1, wasm-bindgen 0.2.129, wasmtime 50.0.1, Claude Code 2.1.317.
  - `cargo test --target wasm32-wasip1` com o runner do `.cargo/config.toml` passa num crate de teste.
  - O firewall configura sem erro. `example.com` falha no DNS, em IPv4 e em IPv6. `index.crates.io` responde. DNS direto a um servidor externo é recusado.
  - `sh .devcontainer/verifica-config.sh` retorna 0.
  - `claude doctor` no contêiner (sem login) não acusa erro de configuração. Isso não prova que o `managed-settings.json` foi aceito.
- Sub-etapa 0.2 (Validação dentro do contêiner): rede provada (19 de 19, no contêiner real do devcontainer). Rodada 1 das permissões: Read e Edit recusam como esperado, mas o Bash não rodava (bwrap sem user namespace). Corrigido com `.devcontainer/seccomp-userns.json`. Rodada 2 pendente (ver `docs/conhecimento/ambiente.md`).
- Sub-etapa 0.3 (Hooks do git): não iniciada.
- Sub-etapa 0.4 (Esqueleto Rust e site): não iniciada.
- Sub-etapa 0.5 (CI e publicação): não iniciada.
- Sub-etapa 0.6 (Fechamento): não iniciada.

## Decisões da auditoria (aprovadas pelo Marcelo em 03/11/2026)

- D1: os testes de fidelidade com a ROM rodam só pelo Marcelo, fora da IA. A IA não lê a ROM por nenhum caminho. A revisão fica para a Etapa 2.
- D2: rede mínima, a lista de `.devcontainer/dominios.txt`.
- D3: o Marcelo valida o contêiner. As sessões só tocam arquivos de texto até a 0.2 passar.
- D4: caminho único do workspace, `/workspaces/websonic`.
- Pastas externas (03/11/2026): uma só variável, `SONIC1_EXT_DIR` (padrão `~/sonic1-ext`), com as subpastas `rom`, `disasm`, `disasm-texto` e `trabalho`. O `trabalho` fica no disco e não em `/tmp`, porque relatórios e hashes precisam sobreviver a reinícios.

## Bloqueios e pendências

- Verificação do Marcelo: dentro do contêiner, entrar no Claude Code (login) e rodar `/status` e `claude doctor`, que não podem listar entradas rejeitadas do managed settings.
- `devcontainer up` verificado em 03/11/2026 (ver `docs/conhecimento/ambiente.md`).
- Aviso ao chat de planejamento: a cópia de `docs/requisitos.md` não traz o número da revisão, só a data. A D1 muda quem roda os testes de fidelidade.
- Verificação do Marcelo: recriar o contêiner (`devcontainer up --workspace-folder . --remove-existing-container`) e rodar de novo o roteiro `.devcontainer/testes/permissoes.md` (rodada 2).
- Sem push. O push é do Marcelo. O commit a5b0710 ainda não foi publicado.
