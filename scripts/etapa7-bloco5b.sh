#!/bin/sh
# Etapa 7, bloco 5b: diagnostico do console nominal (so numeros). Rapido, so nativo.
# Uso: sh scripts/etapa7-bloco5b.sh ROM [QUADROS]
set -u
rom=${1:?uso: etapa7-bloco5b.sh ROM [QUADROS]}
quadros=${2:-330}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
cargo run -q --release -p sonic1-console --bin console_hashes -- "$rom" "$quadros" 2>&1 >/dev/null | grep '^diag'
cargo run -q --release -p sonic1-console --bin console_hashes -- "$rom" "$quadros" 2>/dev/null | grep -E '^erro' 
echo fim
