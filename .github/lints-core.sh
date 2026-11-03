#!/bin/sh
# Lints do nucleo (sonic1-core), passo separado do clippy geral do CI.
#
# O que este passo garante, mesmo que o crate do nucleo seja alterado:
# - -F (forbid) na linha de comando: um #[allow] ou #![allow] no codigo vira erro (E0453).
# - CLIPPY_CONF_DIR aponta para .github/clippy-core/, que e congelado. O clippy.toml do crate
#   deixa de valer (a variavel tem prioridade), entao esvaziar a lista de tipos proibidos nao ajuda.
# - --all-features e --all-targets: codigo atras de feature ou em cfg(test) tambem e examinado.
# - Tres alvos: nativo, wasm32-unknown-unknown (o do site) e wasm32-wasip1 (o dos testes).
#   Codigo atras de cfg(target_arch = "wasm32") so existe nos dois ultimos.
#
# Rode pela imagem: sh .github/rodar-na-imagem.sh sh .github/lints-core.sh
set -u

aqui=$(cd "$(dirname "$0")/.." && pwd)
cd "$aqui" || exit 1

# Prioridade sobre qualquer clippy.toml do crate. Caminho absoluto, pois o cargo muda de pasta.
CLIPPY_CONF_DIR="$aqui/.github/clippy-core"
export CLIPPY_CONF_DIR
# Um [env] com force = true em .cargo/config.toml sobrescreve a variavel acima. Por isso o valor
# tambem vai pela linha de comando (--config), que vale mais que qualquer arquivo de config.
conf_clippy_valor="env.CLIPPY_CONF_DIR.value=\"$CLIPPY_CONF_DIR\""

# RUSTFLAGS vale mais que build.rustflags e target.*.rustflags dos arquivos de config. Fixar aqui
# impede um --cap-lints=allow vindo de la. Wrappers vazios impedem build.rustc-wrapper.
RUSTFLAGS="-Dwarnings"
export RUSTFLAGS
RUSTC_WRAPPER=""
RUSTC_WORKSPACE_WRAPPER=""
export RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
unset CARGO_ENCODED_RUSTFLAGS

falhou=0
for alvo in nativo wasm32-unknown-unknown wasm32-wasip1; do
    if [ "$alvo" = nativo ]; then
        sel=""
    else
        sel="--target $alvo"
    fi
    echo "== clippy do nucleo: $alvo =="
    # shellcheck disable=SC2086  # $sel e vazio ou "--target X", sem espacos no valor.
    cargo clippy --locked --config "$conf_clippy_valor" --config env.CLIPPY_CONF_DIR.force=true -p sonic1-core --all-features --all-targets $sel -- \
        -D warnings \
        -F unsafe_code \
        -F clippy::float_arithmetic \
        -F clippy::arithmetic_side_effects \
        -F clippy::cast_possible_truncation \
        -F clippy::cast_possible_wrap \
        -F clippy::cast_sign_loss \
        -F clippy::disallowed_types \
        || falhou=1
done

[ "$falhou" -eq 0 ] && echo "lints do nucleo: OK" || echo "lints do nucleo: FALHOU" >&2
exit "$falhou"
