#!/bin/sh
# D1, passo 2 (host): a RAM dos dois oraculos nos quadros dados. Imprime so faixas, nomes e contagens.
# Uso: sh scripts/etapa2-d1.sh ROM [QUADROS...]   (padrao: 53 60 120 600)
set -eu
rom=${1:?uso: etapa2-d1.sh ROM [QUADROS...]}
shift
[ $# -gt 0 ] || set -- 53 60 120 600
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
acc=$ext/privado/acessos
bin=$ext/trabalho/oracle/alvo/release/oraculo
o2=$repo/oracle2/target/release/websonic-oracle2
nucleo=$ext/trabalho/oracle2/gpgx/genesis_plus_gx_libretro.so
cd "$repo"
for n in "$@"; do
    "$bin" rodar --rom "$rom" --quadros $((n + 1)) --saida /dev/null --despejar-ram "$acc/d1.p.$n.ram" >/dev/null
    "$o2" rodar --nucleo "$nucleo" --rom "$rom" --quadros $((n + 1)) --saida "$acc/d1.g.tsv" \
        --despejar "$n:$acc/d1.g.$n.ram" >/dev/null
    echo "--- quadro $n ---"
    "$o2" diferencas --a "$acc/d1.p.$n.ram" --b "$acc/d1.g.$n.ram" --rotulos refs/rotulos.txt | head -14
done
