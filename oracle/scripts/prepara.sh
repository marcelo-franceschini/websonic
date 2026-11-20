#!/bin/sh
# Prepara o jgenesis para o oraculo: clona no commit fixado, aplica os patches e cria o link
# `target/oracle-fonte`. Idempotente. Roda no contêiner (ORACLE_DIR=/mnt/trabalho/oracle) ou no host
# (ORACLE_DIR=$SONIC1_EXT_DIR/trabalho/oracle). Nao toca em /mnt/rom nem em /mnt/disasm.
set -eu

aqui=$(cd "$(dirname "$0")/.." && pwd -P)
base=${ORACLE_DIR:-}
if [ -z "$base" ]; then
    if [ -d /mnt/trabalho ]; then base=/mnt/trabalho/oracle
    elif [ -n "${SONIC1_EXT_DIR:-}" ]; then base=$SONIC1_EXT_DIR/trabalho/oracle
    else echo "Erro: defina ORACLE_DIR ou SONIC1_EXT_DIR." >&2; exit 1; fi
fi
commit=$(sed -n 's/^commit: *//p' "$aqui/jgenesis-commit.txt")
[ -n "$commit" ] || { echo "Erro: jgenesis-commit.txt sem commit." >&2; exit 1; }

mkdir -p "$base"
fonte="$base/jgenesis-src"
if [ ! -d "$fonte/.git" ]; then
    git clone -q https://github.com/jsgroth/jgenesis "$fonte"
fi
git -C "$fonte" checkout -q -f "$commit"
git -C "$fonte" clean -q -fdx -e target
for p in "$aqui"/patches/*.patch; do
    git -C "$fonte" apply "$p"
done

# O hash dos patches aplicados (diff contra o commit fixado, com arquivos novos) tem de bater
# com o registrado.
git -C "$fonte" add -A
obtido=$(git -C "$fonte" diff --cached "$commit" | sha1sum | cut -d' ' -f1)
git -C "$fonte" reset -q
esperado=$(cat "$aqui/patches/SHA1")
if [ "$obtido" != "$esperado" ]; then
    echo "Erro: hash dos patches aplicados ($obtido) difere do registrado ($esperado)." >&2
    exit 1
fi

mkdir -p "$aqui/../target"
ln -sfn "$fonte" "$aqui/../target/oracle-fonte"
echo "Oraculo preparado (link: target/oracle-fonte): $fonte (commit $commit, patches $esperado)."
