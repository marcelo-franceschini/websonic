#!/bin/sh
# D1, teste da hipotese do ponto de fechamento do quadro (host, fora do Claude).
# Hipotese: os dois oraculos fecham o quadro em pontos diferentes em relacao ao VBlank.
# Previsao: ao comparar o quadro N de um com o quadro N+1 do outro, as diferencas de tempo
# caem muito. Se nenhuma combinacao cair, a hipotese esta errada.
# Imprime so contagens de bytes, faixas e nomes de variaveis (nunca valores).
# Uso: sh scripts/etapa2-d1b.sh ROM [QUADROS...]   (padrao: 60 120 600 2000)
set -eu
rom=${1:?uso: etapa2-d1b.sh ROM [QUADROS...]}
shift
[ $# -gt 0 ] || set -- 60 120 600 2000
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
acc=$ext/privado/acessos
bin=$ext/trabalho/oracle/alvo/release/oraculo
o2=$repo/oracle2/target/release/websonic-oracle2
nucleo=$ext/trabalho/oracle2/gpgx/genesis_plus_gx_libretro.so
cd "$repo"
mkdir -p "$acc"
for n in "$@"; do
    m=$((n + 1))
    # Principal: RAM depois dos quadros n e m. Segundo oraculo: idem.
    "$bin" rodar --rom "$rom" --quadros $((n + 1)) --saida /dev/null --despejar-ram "$acc/d1b.p.$n.ram" >/dev/null
    "$bin" rodar --rom "$rom" --quadros $((m + 1)) --saida /dev/null --despejar-ram "$acc/d1b.p.$m.ram" >/dev/null
    "$o2" rodar --nucleo "$nucleo" --rom "$rom" --quadros $((m + 1)) --saida "$acc/d1b.g.tsv" \
        --despejar "$n:$acc/d1b.g.$n.ram" >/dev/null
    "$o2" rodar --nucleo "$nucleo" --rom "$rom" --quadros $((m + 1)) --saida "$acc/d1b.g.tsv" \
        --despejar "$m:$acc/d1b.g.$m.ram" >/dev/null
    for par in "$n $n" "$n $m" "$m $n"; do
        set -- $par
        echo "--- principal quadro $1 contra segundo oraculo quadro $2 ---"
        "$o2" diferencas --a "$acc/d1b.p.$1.ram" --b "$acc/d1b.g.$2.ram" --rotulos refs/rotulos.txt | head -8
    done
done
