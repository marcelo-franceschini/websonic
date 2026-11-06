#!/bin/sh
# Etapa 2, passo do host (Marcelo, fora do Claude): perfil das demos, traces do corpus,
# inventario de hardware e manifesto. Imprime so nomes, numeros e hashes.
# Uso: sh scripts/etapa2-host.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
set -eu
rom=${1:?uso: etapa2-host.sh ROM}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
traces=$ext/privado/traces
mkdir -p "$traces"

esperado=$(sed -n 's/^sha1: *//p' "$repo/refs/rom-hashes.txt")
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

sh "$repo/oracle/scripts/prepara.sh" >/dev/null
export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo"
(cd "$repo/oracle/runner" && cargo build --release -q)
bin=$CARGO_TARGET_DIR/release/oraculo
unset CARGO_TARGET_DIR

echo "=== PERFIL (so as linhas em que modo, zona, ato ou demo mudam) ==="
"$bin" rodar --rom "$rom" --quadros 12000 --saida /dev/null \
    --perfil-a-cada 60 --rotulos "$repo/refs/rotulos.txt" | grep '^perfil ' \
    | awk '{k=$3" "$4" "$5" "$6; if (k!=u) print; u=k}'

echo "=== CORPUS-TRACE ==="
cd "$repo"
cargo run -q -p xtask -- corpus-trace --rom "$rom" --oraculo "$bin" \
    --rotulos refs/rotulos.txt --saida "$traces" | tee "$traces/resumo.tsv"

echo "=== INVENTARIO ==="
cargo run -q -p xtask -- inventario --traces "$traces"

echo "=== MANIFESTO (sem assinar) ==="
grep -v '^determinismo' "$traces/resumo.tsv" > "$traces/hashes.tsv"
cargo run -q -p xtask -- manifesto gerar --traces "$traces/hashes.tsv"
echo "=== FIM ==="
