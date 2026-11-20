#!/bin/sh
# Etapa 8, bloco 2 (Marcelo, fora do Claude): relatorio de sintese diferencial com a ROM real.
# Executa os chips de referencia Nuked-OPN2 e Nuked-PSG contra o nosso sintetizador em Rust nativo.
# Camadas testadas:
# (a) bateria sintetica completa (algoritmos, canais, envelopes, DAC, tons e ruido do PSG);
# (b) escritas reais do corpus extraidas dos traces do oraculo (boot-demos, demos-quatro, lz-agua, partida-ghz1).
# Compara amostra a amostra em blocos de 4096 amostras (SHA-1 por bloco).
# Imprime apenas contagens, hashes e vereditos. Os traces temporarios sao apagados no fim.
# Uso:
#   sh scripts/etapa8-bloco2.sh 2>&1 | tee /tmp/etapa8-bloco2.txt
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"

ROM="${1:-$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen}"
ext="${SONIC1_EXT_DIR:-$HOME/sonic1-ext}"
acc="$ext/privado/etapa8"
tmp="$ext/privado/trace-sintese"
mkdir -p "$acc"
rm -rf "$tmp"; mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT

esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$ROM" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "##### 1. TESTES UNITARIOS DO SINTETIZADOR #####"
cargo test -q -p sonic1-audio 2>&1 | grep -E "test result|FAILED|error" | head -10

echo "##### 2. TESTES DIFERENCIAIS CONTRA NUKED-OPN2 E NUKED-PSG #####"
CARGO_TARGET_DIR="$ext/trabalho/nuked/alvo" cargo test --release -q --manifest-path diferencial-sintese/Cargo.toml 2>&1 \
    | grep -E "test result|FAILED|error" | head -10

echo "##### 3. COMPILACAO DO COMPARADOR E DO ORACULO #####"
CARGO_TARGET_DIR="$ext/trabalho/nuked/alvo" cargo build --release -q --manifest-path diferencial-sintese/Cargo.toml --bin sintese
sintese="$ext/trabalho/nuked/alvo/release/sintese"

export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo"
(cd oracle/runner && cargo build --release -q)
oraculo="$CARGO_TARGET_DIR/release/oraculo"
unset CARGO_TARGET_DIR

echo "##### 4. GERACAO DOS TRACES DO CORPUS (temporarios) #####"
rodar() {
    nome=$1; q=$2; ent=${3:-}
    if [ -n "$ent" ]; then set -- --entradas "$ent"; else set --; fi
    "$oraculo" rodar --rom "$ROM" --quadros "$q" --saida "$tmp/$nome.a.s1t" "$@" | grep -E '^(quadros|mclk_final)='
    echo "caso $nome: $(du -m "$tmp/$nome.a.s1t" | cut -f1) MB de trace"
}
rodar boot-demos 7200
rodar lz-agua 2000 corpus/lz-agua.s1in
rodar partida-ghz1 "$(wc -l < corpus/partida-ghz1.s1in)" corpus/partida-ghz1.s1in
rodar demos-quatro 11700

echo "##### 5. RELATORIO DA SINTESE DIFERENCIAL (bateria sintetica + corpus) #####"
set +e
"$sintese" --sintetico --traces "$tmp" --saida "$acc/relatorio-sintese.txt" > "$acc/relatorio-sintese.log" 2>&1
codigo=$?
set -e

cat "$acc/relatorio-sintese.log"
echo "codigo de saida: $codigo"
echo "--- hash do relatorio ---"
sha1sum "$acc/relatorio-sintese.txt" | cut -d' ' -f1
echo "##### FIM DO BLOCO 2 #####"
