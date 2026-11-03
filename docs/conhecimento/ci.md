# CI e publicação

Sub-etapa 0.5. Um workflow só, `.github/workflows/ci.yml`, com dois jobs.

## Fluxo

1. `verificar` (em todo push e pull request): checkout com histórico completo, constrói a imagem de `.devcontainer/Dockerfile` (cache `type=gha`) e roda, dentro dela, nesta ordem: `cargo xtask anti-rom`, `fmt --check`, `clippy -D warnings`, `cargo test --workspace`, testes em `wasm32-wasip1` (`sonic1-core` e `sonic1-rom`) e `cargo xtask site`. O `dist/` fica como artefato por 7 dias.
2. `publicar` (só em push no `main`, depois de `verificar`): publica o `dist/` no GitHub Pages.

Os comandos passam por `.github/rodar-na-imagem.sh`, que roda `docker run` com o usuário do runner, `HOME` e `CARGO_HOME` em `/tmp` e `safe.directory` por variável de ambiente. O mesmo script serve para repetir o CI na máquina local (`IMAGEM=websonic-dev:0.1b`).

## Segurança

- `permissions: contents: read` no topo. Só o job `publicar` recebe `pages: write` e `id-token: write`.
- Sem `pull_request_target` e sem segredos. `persist-credentials: false` no checkout.
- Ações fixadas pelo SHA do commit, com a tag no comentário. SHAs obtidos de `git ls-remote` (tag descascada) em 03/11/2026:

| Ação | Tag | SHA |
| --- | --- | --- |
| `actions/checkout` | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `docker/setup-buildx-action` | v4.4.1 | `f87e5991a6d7451dcb8d9637bfbc97413f497069` |
| `docker/build-push-action` | v7.4.0 | `c3c9e263c25d99ce0380d002d59b67737d91b0dc` |
| `actions/upload-artifact` | v7.0.2 | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` |
| `actions/upload-pages-artifact` | v5.0.0 | `fc324d3547104276b827a68afc52ff2a11cc49c9` |
| `actions/deploy-pages` | v5.0.1 | `368f82528645a54fb793d4d04e342629a3f51346` |

- A imagem é construída a cada execução, com cache. Não há publicação no GHCR (evita `packages: write`).
- A checagem anti-ROM roda primeiro. É a quarta camada da proteção.

## Evidência (03/11/2026)

- `actionlint` 1.7.12 (imagem `rhysd/actionlint`, só como ferramenta de verificação) aceita o workflow. Um workflow plantado com `github.evnt.name` foi recusado, então o linter está ativo.
- Os seis comandos de código do CI rodaram com `rodar-na-imagem.sh` numa cópia do repositório com o usuário 12345 (como um runner, sem ser o `dev`): todos com código 0, e o `dist/` saiu com `app.js`, `index.html` e `pkg/`.

## Execução real no GitHub (03/11/2026)

Verificada com `gh` no commit `548e997`, execução 41695208654 (branch `main`):
- Os jobs `verificar` e `publicar` terminaram com sucesso. Todos os passos passaram: construção da imagem, anti-ROM, formatação, clippy, testes nativos, testes em WebAssembly, montagem do site, artefatos e deploy.
- O Pages está em modo `workflow`, com HTTPS forçado.
- `https://marcelo-franceschini.github.io/websonic/` responde 200 com o título "Sonic 1 em Rust", e `pkg/sonic1_web_bg.wasm` responde 200 (32846 bytes).

## Endurecimento da 0.7 (patches preparados, ainda não aplicados)

Patches em `/mnt/trabalho/0.7/patches/`, para o Marcelo aplicar no host. Mudam o fluxo assim:
1. Logo depois do checkout, **antes** de construir a imagem: `sh .github/anti-rom.sh` (shell puro, usa `hooks/lib.sh`). Reprova extensão, tamanho a partir de 262144, blob com o id da ROM, arquivo só no histórico e clone raso.
2. Depois do clippy geral: `sh .github/rodar-na-imagem.sh sh .github/checa-core.sh` (configuração do cargo, dependências e fontes do núcleo) e `sh .github/rodar-na-imagem.sh sh .github/lints-core.sh` (clippy do núcleo com `-F`, `--all-features`, `--all-targets`, em nativo, `wasm32-unknown-unknown` e `wasm32-wasip1`).
3. `rodar-na-imagem.sh` passa `RUSTFLAGS=-Dwarnings` e wrappers vazios: o `RUSTFLAGS` vale mais que `rustflags` de qualquer `.cargo/config.toml`. Os passos antigos (clippy geral, testes nativos e em WebAssembly, site) foram repetidos com esse `RUSTFLAGS` na imagem e passam.

Provas rodadas no contêiner (cópia limpa do `HEAD` com os patches): `testes/anti-rom.sh` 8 de 8, `checa-core.sh` OK, `lints-core.sh` OK nos três alvos, e a tabela do C5 em `ambiente.md`. Falta `actionlint` no `ci.yml` e a execução no GitHub.

## Ainda não provado

- O aceite da REV01 no site publicado (o Marcelo escolhe a ROM nele).
- A proteção de branch em `main`, que é configuração do GitHub.
- Que o CI **falha** quando deveria. A prova é um push de teste com um arquivo proibido, num branch descartável.

## Para o Marcelo

1. Feito: push, Pages em "GitHub Actions" e CI verde.
2. Abrir o site publicado e escolher a ROM REV01.
3. Exigir o job `verificar` em `main` (proteção de branch).
