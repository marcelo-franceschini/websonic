#!/bin/sh
# Etapa 6, bloco 2 (Marcelo, fora do Claude): o relatorio do som com a ROM real. Para cada ID de
# musica, efeito, especial e comando, o jogo roda no oraculo e o driver do nucleo roda a mesma
# chamada de UpdateMusic, de forma continua. Compara escritas no YM2612 e no PSG (incluindo as do
# DAC), o estado do driver na RAM, a RAM do Z80 e a posicao do Z80 em cada ponto gravado.
# Imprime so numeros, hashes e vereditos. Uso:
#   sh scripts/etapa6-bloco2.sh 2>&1 | tee /tmp/etapa6-bloco2.txt | tail -120
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
ROM="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
acc=$ext/privado/etapa6
mkdir -p "$acc"

esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$ROM" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "##### 1. TESTES DO NUCLEO E DO SOM (sem ROM) #####"
cargo test -q -p sonic1-core -p sonic1-teste-som 2>&1 | grep -E "test result|FAILED|error" | head -10
alvo=$ext/trabalho/oracle/alvo-dif
CARGO_TARGET_DIR=$alvo cargo test --release -q --manifest-path diferencial-som/Cargo.toml 2>&1 \
    | grep -E "test result|FAILED|error" | head -10

echo "##### 2. RELATORIO DO SOM (todos os IDs) #####"
CARGO_TARGET_DIR=$alvo cargo build --release -q --manifest-path diferencial-som/Cargo.toml
som=$alvo/release/som
set +e
"$som" --rom "$ROM" --rotulos refs/rotulos.txt --ids crates/teste-som/ids.txt \
    --saida "$acc/relatorio-som.txt" > "$acc/relatorio-som.log" 2>&1
codigo=$?
set -e
echo "codigo de saida: $codigo"
echo "--- resumo ---"
tail -1 "$acc/relatorio-som.log"
echo "--- IDs que DIVERGEM (cole isto) ---"
grep -E "DIVERGE|erro" "$acc/relatorio-som.log" | cut -c1-260 | head -80
echo "--- hash do relatorio ---"
sha1sum "$acc/relatorio-som.txt" | cut -d' ' -f1
echo "##### FIM #####"
