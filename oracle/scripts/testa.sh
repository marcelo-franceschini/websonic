#!/bin/sh
# Testes do oraculo (so ROMs sinteticas). Roda no contêiner ou no host. Nao toca na ROM real.
# Uso: sh oracle/scripts/testa.sh
set -eu

aqui=$(cd "$(dirname "$0")/.." && pwd -P)
sh "$aqui/scripts/prepara.sh"

base=${ORACLE_DIR:-}
if [ -z "$base" ]; then
    if [ -d /mnt/trabalho ]; then base=/mnt/trabalho/oracle; else base=$SONIC1_EXT_DIR/trabalho/oracle; fi
fi
export CARGO_TARGET_DIR="$base/alvo"
cd "$aqui/runner"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
echo "Testes do oraculo: OK."
