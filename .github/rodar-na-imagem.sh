#!/bin/sh
# Roda um comando dentro da imagem de trabalho, com o repositorio montado.
# Usado pelo CI e igual ao que se faz localmente. A imagem vem de IMAGEM
# (padrao: websonic-dev:ci, construida pelo workflow a partir de .devcontainer/Dockerfile).
#
# O usuario e o do runner, para nao deixar arquivos de root na pasta. O HOME e o
# CARGO_HOME vao para /tmp, que e gravavel por qualquer usuario. O safe.directory
# entra por variavel de ambiente (o dono do repositorio pode diferir do usuario).
#
# RUSTFLAGS e explicito de proposito: ele vale mais que build.rustflags e target.*.rustflags de
# qualquer .cargo/config.toml, entao um --cap-lints=allow vindo de la nao desliga os lints.
# Os wrappers vazios desligam build.rustc-wrapper. (Testado em 0.7, caso C5.)
#
# MONTAGEM_RO=1 monta o checkout so para leitura (:ro). Usado nos passos que conferem o nucleo,
# para eles nao poderem reescrever os proprios scripts. Quem usa precisa mandar o alvo do cargo
# para fora do checkout (CARGO_TARGET_DIR=/tmp/...).
set -eu
imagem=${IMAGEM:-websonic-dev:ci}
usuario=${USUARIO:-$(id -u):$(id -g)}
exec docker run --rm \
    --user "$usuario" \
    -e HOME=/tmp -e CARGO_HOME=/tmp/cargo \
    -e RUSTFLAGS=-Dwarnings -e RUSTC_WRAPPER= -e RUSTC_WORKSPACE_WRAPPER= \
    -e GIT_CONFIG_COUNT=1 -e GIT_CONFIG_KEY_0=safe.directory -e GIT_CONFIG_VALUE_0='*' \
    -v "$PWD":/workspaces/websonic${MONTAGEM_RO:+:ro} -w /workspaces/websonic \
    "$imagem" "$@"
