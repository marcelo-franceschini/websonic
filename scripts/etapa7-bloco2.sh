#!/bin/sh
# Etapa 7, bloco 2 (Marcelo, fora do Claude): compara o oraculo com o nucleo na abertura.
# Imprime so contagens, nomes, enderecos e vereditos. Nunca valores de memoria.
# Uso: sh scripts/etapa7-bloco2.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen" [QUADROS]
set -u
rom=${1:?uso: etapa7-bloco2.sh ROM [QUADROS]}
quadros=${2:-61}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
cd "$repo"
esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "=== OFICINA DO ORACULO ==="
sh oracle/scripts/prepara.sh | tail -1

echo "=== TESTES DA ABERTURA (sinteticos) ==="
cargo test -q -p sonic1-core -p sonic1-teste-som 2>&1 | tail -3
(cd diferencial-abertura && CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo-abertura" cargo test -q --release 2>&1 | tail -3)

echo "=== COMPARACAO ($quadros quadros) ==="
(cd diferencial-abertura && CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo-abertura" \
    cargo run -q --release --bin abertura -- comparar "$rom" "$quadros" ../refs/rotulos.txt)
echo "=== FIM ==="
