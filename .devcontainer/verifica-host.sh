#!/bin/sh
# Roda no computador do Marcelo, antes de criar o contêiner (initializeCommand).
# Confere a pasta externa SONIC1_EXT_DIR e suas subpastas. So testa se as pastas existem:
# nao lista nem le o conteudo delas.
#
#   $SONIC1_EXT_DIR/
#   ├─ rom/          a ROM REV01 (montada em /mnt/rom, so leitura)
#   ├─ disasm/       o s1disasm completo (montado em /mnt/disasm, so leitura)
#   ├─ disasm-texto/ so os .asm do disassembly (montado em /mnt/disasm-texto, so leitura)
#   └─ trabalho/     traces e relatorios (montado em /mnt/trabalho, leitura e escrita)
set -eu

erro() {
    echo "Erro: $*" >&2
    exit 1
}

base=${SONIC1_EXT_DIR:-}
[ -n "$base" ] || erro "a variavel SONIC1_EXT_DIR nao esta definida. Veja o README."
[ -d "$base" ] || erro "SONIC1_EXT_DIR aponta para '$base', que nao e uma pasta. Crie-a (veja o README)."

# Caminhos reais, para a checagem abaixo nao ser enganada por links simbolicos.
base=$(cd "$base" && pwd -P)
repo=$(cd "$(dirname "$0")/.." && pwd -P)

# Protecao da camada 1: a pasta externa nunca pode ficar dentro do repositorio.
case "$base/" in
    "$repo"/*) erro "SONIC1_EXT_DIR ($base) esta dentro do repositorio ($repo). Mova-a para fora." ;;
esac

# A pasta de trabalho so tem conteudo gerado, entao pode ser criada aqui.
mkdir -p "$base/trabalho"

# As outras tres tem conteudo do Marcelo. Se faltarem, e erro, para nao montar uma pasta vazia por engano.
for sub in rom disasm disasm-texto; do
    [ -d "$base/$sub" ] || erro "falta a pasta $base/$sub. Veja o README."
done
