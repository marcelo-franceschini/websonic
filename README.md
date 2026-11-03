# Reconstrução do Sonic 1

Reconstrução do jogo Sonic the Hedgehog (1991) do Mega Drive em Rust, com foco em fidelidade estrita e aprendizado.

A fonte única de verdade é `docs/requisitos.md`. O andamento está em `docs/estado.md`.

## Pré-requisitos no host

Docker e a ferramenta `devcontainer` (ou VS Code/Cursor com a extensão Dev Containers).

Defina as variáveis no perfil do shell:

```sh
export SONIC1_ROM_DIR=~/sonic1-ext/rom
export SONIC1_DISASM_DIR=~/sonic1-ext/disasm
export SONIC1_DISASM_TEXTO_DIR=~/sonic1-ext/disasm-texto
export SONIC1_TRABALHO_DIR=~/sonic1-ext/trabalho
```

Crie as quatro pastas (todas ficam fora do repositório):

```sh
mkdir -p "$SONIC1_ROM_DIR" "$SONIC1_DISASM_DIR" "$SONIC1_DISASM_TEXTO_DIR" "$SONIC1_TRABALHO_DIR"
```

O script `.devcontainer/verifica-host.sh` roda sozinho antes de criar o contêiner e recusa a montagem se alguma variável faltar ou não apontar para uma pasta.

### O que é cada pasta

| Pasta | Contêiner | Acesso | Conteúdo |
| --- | --- | --- | --- |
| `SONIC1_ROM_DIR` | `/mnt/rom` | só leitura, bloqueada para a IA | A ROM REV01. |
| `SONIC1_DISASM_DIR` | `/mnt/disasm` | só leitura, bloqueada para a IA | O s1disasm completo, no commit de `refs/disasm-commit.txt`. Tem binários. |
| `SONIC1_DISASM_TEXTO_DIR` | `/mnt/disasm-texto` | só leitura | Cópia só dos arquivos `.asm` do disassembly, sem binários. É o que a IA lê. |
| `SONIC1_TRABALHO_DIR` | `/mnt/trabalho` | leitura e escrita | Traces e relatórios. |

Como gerar a cópia só de texto (rode no host, depois de baixar o disassembly no commit fixado):

```sh
cd "$SONIC1_DISASM_DIR"
find . -type f -name '*.asm' -exec cp --parents {} "$SONIC1_DISASM_TEXTO_DIR" \;
```

O comando copia só os arquivos terminados em `.asm`. Os arquivos `.asm` trazem alguns dados da ROM em texto, e o requisito de origem dos dados (cada tabela vem da ROM) continua valendo.

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

## Como congelar um item

Para congelar arquivos contra edição da IA:

1. Abra `.devcontainer/managed-settings.json`.
2. Adicione `Edit(//workspaces/websonic/caminho)` em `permissions.deny` e o caminho em `sandbox.filesystem.denyWrite`.
3. Reconstrua a imagem do contêiner (`devcontainer build --workspace-folder .`).

A regra `Edit` vale para as ferramentas do Claude Code e para comandos de shell que ele reconhece (`sed`, `tee`, redirecionamentos). Só o `denyWrite` do sandbox vale para qualquer programa. Por isso as duas regras andam juntas. A prova disso é feita na sub-etapa 0.2.
