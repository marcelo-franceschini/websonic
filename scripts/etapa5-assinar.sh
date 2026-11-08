#!/bin/sh
# Etapa 5, fechamento: refaz os traces com o oraculo novo (o hash dos patches mudou), regera o
# manifesto com os arquivos novos, assina e roda o relatorio de fidelidade. PEDE SENHAS (a da
# chave de assinatura e a da chave dos ocultos): rode num terminal, direto:
#   sh scripts/etapa5-assinar.sh
# Escreve em /tmp/etapa5-assinar.txt so vereditos, contagens, hashes e nomes.
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
traces=$ext/privado/traces
rom="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
bin=$ext/trabalho/oracle/alvo/release/oraculo
# Chave nova (a senha da anterior foi perdida). Troca refs/assinante.pub antes do manifesto.
chave_assina=$HOME/.ssh/websonic_assinatura_nova
chave_ocultos_cifrada=$HOME/.config/websonic/ocultos.key.age
resumo=/tmp/etapa5-assinar.txt
cd "$repo"
[ -x "$bin" ] || { echo "Erro: rode antes scripts/etapa5-bloco-a.sh (falta $bin)." >&2; exit 1; }
[ -t 0 ] || { echo "Erro: rode num terminal (o script pede senhas)." >&2; exit 1; }
agente_pid=
limpar() {
    stty echo 2>/dev/null || true
    [ -z "$agente_pid" ] || kill "$agente_pid" 2>/dev/null || true
}
trap limpar EXIT INT TERM
: > "$resumo"
dizer() { echo "$*" | tee -a "$resumo"; }
registrar() { "$@" 2>&1 | tee "$traces/ultimo.txt" | tee -a "$resumo"; }

eval "$(ssh-agent -s)" > /dev/null
agente_pid=$SSH_AGENT_PID
echo "Digite a senha da chave de assinatura (ssh-add)." >&2
ssh-add "$chave_assina"
printf 'Senha da chave dos ocultos: ' >&2
stty -echo
IFS= read -r SENHA
stty echo
printf '\n' >&2
[ -n "$SENHA" ] || { echo "Erro: senha vazia." >&2; exit 1; }

if ! cmp -s "$chave_assina.pub" refs/assinante.pub; then
    cp "$chave_assina.pub" refs/assinante.pub
    dizer "refs/assinante.pub: trocado pela chave nova"
fi

dizer "=== 1. TRACES DO CORPUS (oraculo novo) ==="
registrar cargo fidelidade corpus-trace --rom "$rom" --oraculo "$bin" --rotulos refs/rotulos.txt --saida "$traces"
cp "$traces/ultimo.txt" "$traces/resumo-etapa5.tsv"
grep -q '^determinismo: PASSOU$' "$traces/resumo-etapa5.tsv" \
    || { echo "Erro: determinismo nao passou." >&2; exit 1; }
grep -v '^determinismo' "$traces/resumo-etapa5.tsv" > "$traces/hashes-etapa5.tsv"

dizer "=== 1b. COBERTURA COM OS TRACES NOVOS ==="
# O arquivo executados-68k.txt vem de scripts/etapa5-cobertura.sh e entra no manifesto.
acc=$ext/privado/etapa5
[ -s cobertura/executados-68k.txt ] || echo "Aviso: cobertura/executados-68k.txt vazio. Rode antes scripts/etapa5-cobertura.sh." >&2
cat "$acc/nucleo-testes.txt" "$acc/nucleo-rotinas.txt" > "$acc/nucleo-todos.txt"
cargo fidelidade cobertura relatorio --nucleo "$acc/nucleo-todos.txt" --executados cobertura/executados-68k.txt \
    --executados "$acc/executados-rotinas.txt" --mapa cobertura/mapa-decisoes.txt --saida "$acc/cobertura.tsv" \
    | grep -v '^id	' | cut -c1-160 | tee -a "$resumo"

dizer "=== 2. MANIFESTO E ASSINATURA ==="
cargo fidelidade manifesto gerar --traces "$traces/hashes-etapa5.tsv" --ocultos "$ext/ocultos" | tee -a "$resumo"
rm -f refs/manifesto.txt.sig
ssh-keygen -Y sign -f "$chave_assina.pub" -n websonic-manifesto refs/manifesto.txt
registrar cargo fidelidade manifesto verificar --pular-testes
grep -q '^assinatura: OK$' "$traces/ultimo.txt" || { echo "Erro: a assinatura nao verificou." >&2; exit 1; }

dizer "=== 3. RELATORIO COMPLETO ==="
registrar cargo fidelidade fidelidade --rom "$rom" --oraculo "$bin" --saida "$traces"

dizer "=== 4. AUDITORIA DOS OCULTOS ==="
printf '%s\n' "$SENHA" | ROM="$rom" ORACULO="$bin" cargo fidelidade ocultos auditar \
    --pasta "$ext/ocultos" --identidade "$chave_ocultos_cifrada" --sut "$repo/scripts/sut-reproduz.sh" \
    | tee -a "$resumo"
SENHA=
dizer "=== FIM === (resumo em $resumo)"
