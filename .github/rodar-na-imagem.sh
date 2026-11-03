#!/bin/sh
# Roda um comando dentro da imagem de trabalho, com o repositorio montado.
# Usado pelo CI e igual ao que se faz localmente. A imagem vem de IMAGEM
# (padrao: websonic-dev:ci, construida pelo workflow a partir de .devcontainer/Dockerfile).
#
# O usuario e o do runner, para nao deixar arquivos de root na pasta. O HOME e o
# CARGO_HOME vao para /tmp, que e gravavel por qualquer usuario. O safe.directory
# entra por variavel de ambiente (o dono do repositorio pode diferir do usuario).
set -eu
imagem=${IMAGEM:-websonic-dev:ci}
usuario=${USUARIO:-$(id -u):$(id -g)}
exec docker run --rm \
    --user "$usuario" \
    -e HOME=/tmp -e CARGO_HOME=/tmp/cargo \
    -e GIT_CONFIG_COUNT=1 -e GIT_CONFIG_KEY_0=safe.directory -e GIT_CONFIG_VALUE_0='*' \
    -v "$PWD":/workspaces/websonic -w /workspaces/websonic \
    "$imagem" "$@"
