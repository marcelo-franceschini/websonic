# Estado do Projeto

Documento de acompanhamento do progresso das etapas.

## Etapa atual

- Etapa: 0 (Esqueleto do repositório).
- Sub-etapa: 0.1b (Correção do ambiente).
- Status da sub-etapa: implementada e commitada localmente. Faltam duas verificações do Marcelo (ver Pendências).

## Histórico de etapas

### Etapa 0: Esqueleto do repositório

- Sub-etapa 0.1 (Ambiente): entregue sem verificação e com defeitos. A auditoria mostrou que a imagem não construía (digest inexistente, `rustup` com argumento inválido). Substituída pela 0.1b.
- Sub-etapa 0.1b (Correção do ambiente): imagem constrói sem erro. Evidência rodada em 03/11/2026 no computador do Marcelo, sem montar nenhuma pasta de ROM:
  - `docker build` com exit 0. Versões dentro da imagem: Rust 1.99.1, wasm-bindgen 0.2.129, wasmtime 50.0.1, Claude Code 2.1.317.
  - `cargo test --target wasm32-wasip1` com o runner do `.cargo/config.toml` passa num crate de teste.
  - O firewall configura sem erro. `example.com` falha no DNS, em IPv4 e em IPv6. `index.crates.io` responde. DNS direto a um servidor externo é recusado.
  - `sh .devcontainer/verifica-config.sh` retorna 0.
  - `claude doctor` no contêiner (sem login) não acusa erro de configuração. Isso não prova que o `managed-settings.json` foi aceito.
- Sub-etapa 0.2 (Validação dentro do contêiner): não iniciada.
- Sub-etapa 0.3 (Hooks do git): não iniciada.
- Sub-etapa 0.4 (Esqueleto Rust e site): não iniciada.
- Sub-etapa 0.5 (CI e publicação): não iniciada.
- Sub-etapa 0.6 (Fechamento): não iniciada.

## Decisões da auditoria (aprovadas pelo Marcelo em 03/11/2026)

- D1: os testes de fidelidade com a ROM rodam só pelo Marcelo, fora da IA. A IA não lê a ROM por nenhum caminho. A revisão fica para a Etapa 2.
- D2: rede mínima, a lista de `.devcontainer/dominios.txt`.
- D3: o Marcelo valida o contêiner. As sessões só tocam arquivos de texto até a 0.2 passar.
- D4: caminho único do workspace, `/workspaces/websonic`.

## Bloqueios e pendências

- Verificação do Marcelo: abrir o contêiner pelo `devcontainer.json`, entrar no Claude Code e rodar `/status` e `claude doctor`, que não podem listar entradas rejeitadas do managed settings.
- Verificação do Marcelo: rodar `devcontainer up` com as quatro variáveis definidas e confirmar que o `verifica-host.sh` e o firewall (`postStartCommand`) passam.
- Aviso ao chat de planejamento: a cópia de `docs/requisitos.md` não traz o número da revisão, só a data. A D1 muda quem roda os testes de fidelidade.
- Sem push. O push é do Marcelo.
