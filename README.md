# Reconstrução do Sonic 1

Reconstrução do jogo Sonic the Hedgehog (1991) do Mega Drive em Rust, com foco em fidelidade estrita e aprendizado.

A fonte única de verdade é `docs/requisitos.md`. O andamento está em `docs/estado.md`.

## Pré-requisitos no host

Docker e a ferramenta `devcontainer` (ou VS Code/Cursor com a extensão Dev Containers).

### Pasta externa

Tudo o que não pode entrar no git fica numa pasta fora do repositório, apontada por **uma** variável, `SONIC1_EXT_DIR`. O padrão do projeto é `~/sonic1-ext`.

```text
~/sonic1-ext/
├─ rom/            a ROM REV01
├─ disasm/         o s1disasm completo, no commit de refs/disasm-commit.txt
├─ disasm-texto/   só os .asm do disassembly
└─ trabalho/       traces e relatórios
```

| Subpasta | No contêiner | Acesso | Conteúdo |
| --- | --- | --- | --- |
| `rom` | `/mnt/rom` | só leitura, bloqueada para a IA | A ROM REV01. |
| `disasm` | `/mnt/disasm` | só leitura, bloqueada para a IA | O disassembly completo. Tem binários. |
| `disasm-texto` | `/mnt/disasm-texto` | só leitura | Cópia só dos `.asm`. É o que a IA lê. |
| `trabalho` | `/mnt/trabalho` | leitura e escrita | Traces e relatórios. Fica no disco, e não em `/tmp`, porque os relatórios e os hashes precisam sobreviver a reinícios. |

Defina a variável no perfil do shell.

sh, bash e zsh:

```sh
export SONIC1_EXT_DIR=~/sonic1-ext
```

fish (o `-U` grava para todas as sessões e o `-x` exporta):

```fish
set -Ux SONIC1_EXT_DIR ~/sonic1-ext
```

Abra um terminal novo, ou use o mesmo terminal onde definiu a variável, antes de subir o contêiner.

O script `.devcontainer/verifica-host.sh` roda sozinho antes de criar o contêiner. Ele:

- recusa a montagem se a variável faltar ou não apontar para uma pasta;
- recusa se a pasta externa estiver dentro do repositório;
- cria `trabalho` se ela não existir;
- recusa se faltar `rom`, `disasm` ou `disasm-texto`, para não montar uma pasta vazia por engano.

Ele só testa se as pastas existem. Não lista nem lê o conteúdo.

### Gerar a cópia só de texto

Depois de colocar o disassembly em `disasm`, no commit fixado em `refs/disasm-commit.txt`, rode no host:

```sh
sh .devcontainer/gera-disasm-texto.sh
```

O script recria `disasm-texto` do zero com os arquivos terminados em `.asm`, mantendo a estrutura de pastas. Os `.asm` trazem alguns dados da ROM em texto, e a regra de origem dos dados dos requisitos continua valendo.

## Como construir e abrir o contêiner de trabalho

O caminho do workspace no contêiner é `/workspaces/websonic`.

Com a ferramenta de linha de comando do Dev Container:

```sh
devcontainer up --workspace-folder .
devcontainer exec --workspace-folder . claude
```

Se abrir pelo VS Code ou Cursor:

- Desative o compartilhamento de credenciais do Git (`dev.containers.gitCredentialHelperConfigLocation: "none"`).
- Use sempre o terminal integrado para rodar o Claude Code.

## Conferir a configuração sem o Docker

```sh
sh .devcontainer/verifica-config.sh
```

Confere se a lista de domínios do `managed-settings.json` coincide com `dominios.txt` e se as chaves estão no nível certo (uma chave no nível errado é ignorada em silêncio).

## Hooks do git

Ativam-se uma vez por clone, no host:

```sh
sh hooks/instalar.sh
```

Bloqueiam ROM por extensão, tamanho e hash, no commit e no push. Detalhes e limites em `docs/conhecimento/hooks.md`. Teste sintético: `sh .devcontainer/testes/hooks.sh`.

## Código e comandos

Dentro do contêiner (ou na imagem):

```sh
cargo test --workspace
cargo xtask site        # monta dist/
cargo xtask anti-rom    # procura ROM no repositório
```

Estrutura e decisões em `docs/conhecimento/esqueleto.md`.

## CI

O GitHub Actions roda anti-ROM, formatação, clippy, testes (nativo e WebAssembly) e publica o site no Pages, tudo na imagem do contêiner. Detalhes em `docs/conhecimento/ci.md`.

## Como congelar um item

Para congelar arquivos contra edição da IA:

1. Abra `.devcontainer/managed-settings.json`.
2. Adicione `Edit(//workspaces/websonic/caminho)` em `permissions.deny` e o caminho em `sandbox.filesystem.denyWrite`.
3. Reconstrua a imagem do contêiner (`devcontainer build --workspace-folder .`).

A regra `Edit` vale para as ferramentas do Claude Code e para comandos de shell que ele reconhece (`sed`, `tee`, redirecionamentos). Só o `denyWrite` do sandbox vale para qualquer programa. Por isso as duas regras andam juntas. A prova disso é feita na sub-etapa 0.2. Itens já congelados (0.6): `CLAUDE.md`, `.gitignore`, `docs/requisitos.md`, `hooks/`, `refs/`, `.github/` e `.devcontainer/`. O `verifica-config.sh` confere que cada um tem as duas regras.
