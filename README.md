# Reconstrução do Sonic 1

Reconstrução do jogo Sonic the Hedgehog (1991) do Mega Drive em Rust, com foco em fidelidade estrita e aprendizado.

## Pré-requisitos no host

Defina as variáveis no seu perfil de shell:

```sh
export SONIC1_ROM_DIR=~/sonic1-ext/rom
export SONIC1_DISASM_DIR=~/sonic1-ext/disasm
export SONIC1_DISASM_TEXTO_DIR=~/sonic1-ext/disasm-texto
export SONIC1_TRABALHO_DIR=~/sonic1-ext/trabalho
```

Crie os diretórios locais:

```sh
mkdir -p "$SONIC1_ROM_DIR" "$SONIC1_TRABALHO_DIR/restrito"
```

## Como construir e abrir o contêiner de trabalho

Com a ferramenta de linha de comando do Dev Container:

```sh
devcontainer up --workspace-folder .
devcontainer exec --workspace-folder . claude
```

Se abrir pelo VS Code ou Cursor:
- Desative o compartilhamento de credenciais do Git nas configurações do editor (`dev.containers.gitCredentialHelperConfigLocation: "none"`).
- Use sempre o terminal integrado para rodar o Claude Code.

## Como congelar um item

Para congelar arquivos contra qualquer edição da IA:
1. Abra `.devcontainer/managed-settings.json`.
2. Adicione regras `Edit(caminho)` na lista `deny`.
3. Reconstrua a imagem do contêiner (`devcontainer build --workspace-folder .`).
4. As configurações gerenciadas em `/etc/claude-code/managed-settings.json` bloqueiam qualquer alteração, tanto pelas ferramentas da IA quanto por comandos de shell.
