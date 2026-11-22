#!/bin/sh
# 2.1 e 2.5 no host (Marcelo): roda a ROM no Genesis Plus GX (libretro) e compara os hashes de
# RAM do 68000, quadro a quadro, com o trace do oraculo principal. Imprime so contagens.
# Uso: sh scripts/etapa2-oraculo2.sh ROM [QUADROS]
set -eu
rom=${1:?uso: etapa2-oraculo2.sh ROM [QUADROS]}
quadros=${2:-7200}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
nucleo=$ext/trabalho/oracle2/gpgx/genesis_plus_gx_libretro.so
traces=$ext/privado/traces
echo "nucleo: commit $(git -C "$ext/trabalho/oracle2/gpgx" rev-parse HEAD), sha1 $(sha1sum "$nucleo" | cut -d' ' -f1)"
(cd "$repo/oracle2" && cargo build --release -q)
bin=$repo/oracle2/target/release/websonic-oracle2
"$bin" rodar --nucleo "$nucleo" --rom "$rom" --quadros "$quadros" --saida "$traces/boot-demos.g.tsv"
"$bin" comparar --trace "$traces/boot-demos.a.s1t" --tsv "$traces/boot-demos.g.tsv"
