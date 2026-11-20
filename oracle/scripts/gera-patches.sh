#!/bin/sh
# Regenera patches/*.patch e patches/SHA1 a partir da branch `websonic-oracle` do clone do jgenesis.
# Uso: depois de editar o clone em ORACLE_DIR/jgenesis-src na branch websonic-oracle e commitar.
set -eu

aqui=$(cd "$(dirname "$0")/.." && pwd -P)
base=${ORACLE_DIR:-}
if [ -z "$base" ]; then
    if [ -d /mnt/trabalho ]; then base=/mnt/trabalho/oracle
    else base=$SONIC1_EXT_DIR/trabalho/oracle; fi
fi
fonte="$base/jgenesis-src"
commit=$(sed -n 's/^commit: *//p' "$aqui/jgenesis-commit.txt")

rm -f "$aqui"/patches/*.patch
git -C "$fonte" format-patch -q --no-signature -o "$aqui/patches" "$commit"..websonic-oracle
git -C "$fonte" diff "$commit" websonic-oracle | sha1sum | cut -d' ' -f1 | tr -d '\n' > "$aqui/patches/SHA1"
echo "Patches: $(ls "$aqui"/patches/*.patch | wc -l). SHA1 dos patches: $(cat "$aqui/patches/SHA1")"
