#!/bin/sh
# Etapa 2, fechamento, passo 1 (Marcelo, fora do Claude): testes do oraculo com o patch novo,
# mapa de acessos a RAM (inventario de memoria compartilhada), diagnostico do D1 e build do
# gravador. Imprime so nomes, numeros, enderecos e contagens. Nenhum valor da RAM.
# Uso: sh scripts/etapa2-final.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
set -eu
rom=${1:?uso: etapa2-final.sh ROM}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
traces=$ext/privado/traces
acc=$ext/privado/acessos
mkdir -p "$traces" "$acc"
cd "$repo"

esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "=== TESTES DO ORACULO (com o patch 0003) ==="
sh oracle/scripts/testa.sh 2>&1 | grep -E "test result|Testes do oraculo|error|FAILED" | head -12

echo "=== OFICINA DO ORACULO ==="
sh oracle/scripts/prepara.sh | tail -1
export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo"
(cd oracle/runner && cargo build --release -q)
bin=$CARGO_TARGET_DIR/release/oraculo
unset CARGO_TARGET_DIR

echo "=== MAPA DE ACESSOS A RAM (P2) ==="
"$bin" rodar --rom "$rom" --quadros 7200 --saida /dev/null --acessos "$acc/boot-demos.acc" | grep '^quadros='
"$bin" rodar --rom "$rom" --quadros 2000 --saida /dev/null --entradas corpus/lz-agua.s1in \
    --acessos "$acc/lz-agua.acc" | grep '^quadros='
cargo run -q -p xtask -- inventario-memoria --acessos "$acc/boot-demos.acc" --acessos "$acc/lz-agua.acc"
grep -E '^## ' docs/inventario-memoria.md

echo "=== DIAGNOSTICO DO D1 (so enderecos e nomes) ==="
nucleo=$ext/trabalho/oracle2/gpgx/genesis_plus_gx_libretro.so
(cd oracle2 && cargo build --release -q)
o2=$repo/oracle2/target/release/websonic-oracle2
for n in 51 52; do
    "$bin" rodar --rom "$rom" --quadros $((n + 1)) --saida /dev/null --despejar-ram "$acc/d1.p.$n.ram" >/dev/null
    "$o2" rodar --nucleo "$nucleo" --rom "$rom" --quadros $((n + 1)) --saida "$acc/d1.g.tsv" \
        --despejar "$n:$acc/d1.g.$n.ram" >/dev/null
    echo "--- quadro $n ---"
    "$o2" diferencas --a "$acc/d1.p.$n.ram" --b "$acc/d1.g.$n.ram" --rotulos refs/rotulos.txt
done

echo "=== GRAVADOR (build do jgenesis com janela, patch 0003) ==="
CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo-gui" cargo build --release -q \
    --manifest-path "$repo/target/oracle-fonte/Cargo.toml" -p jgenesis-gui
ls "$ext/trabalho/oracle/alvo-gui/release/jgenesis" >/dev/null && echo "gravador: construido"
echo "=== FIM ==="
