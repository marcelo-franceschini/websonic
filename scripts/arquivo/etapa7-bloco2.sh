#!/bin/sh
# Etapa 7, bloco 2 (Marcelo, fora do Claude): compara o oraculo com o nucleo na abertura.
# Imprime so contagens, nomes, enderecos e vereditos. Nunca valores de memoria.
# Uso: sh scripts/etapa7-bloco2.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen" [QUADROS]
set -u
rom=${1:?uso: etapa7-bloco2.sh ROM [QUADROS]}
quadros=${2:-62}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
cd "$repo"
esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "=== OFICINA DO ORACULO ==="
sh oracle/scripts/prepara.sh | tail -1

# Soma as linhas "test result" de todos os binarios de teste (as ultimas linhas do cargo sao
# so as dos doctests, com zero testes). Se o cargo falhar, diz o codigo de saida.
resumir_testes() {
    saida=$("$@" 2>&1)
    codigo=$?
    printf '%s\n' "$saida" | awk '/test result:/ { ok += $4; falhou += $6 }
        END { printf "testes: %d passaram, %d falharam\n", ok, falhou }'
    [ "$codigo" -eq 0 ] || echo "testes: o cargo saiu com o codigo $codigo"
}

echo "=== TESTES DA ABERTURA (sinteticos) ==="
resumir_testes cargo test -q -p sonic1-core -p sonic1-teste-som
(cd diferencial-abertura && export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo-abertura" &&
    resumir_testes cargo test -q --release)

echo "=== COMPARACAO ($quadros quadros) ==="
(cd diferencial-abertura && CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo-abertura" \
    cargo run -q --release --bin abertura -- comparar "$rom" "$quadros" ../refs/rotulos.txt)
echo "=== FIM ==="
