#!/bin/sh
# Etapa 2, fechamento (plano em docs/plano-fechamento-etapa-2.md, secao 5). Roda no HOST, fora do
# Claude e fora do contêiner: usa a ROM, as chaves e o gravador com janela.
#
# Uso: sh scripts/etapa2-fechamento.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
#
# O comeco pede a sua atencao (uns 5 minutos): senhas das chaves e a partida na Green Hill.
# Depois roda sozinho, por horas. Pode ser repetido: o que ja foi feito e pulado.
# Escreve em /tmp/etapa2-fechamento.txt SO vereditos, contagens, hashes e nomes.
#
# Passos:
#   1. confere o repositorio limpo e aplica os dois patches da sub-etapa 2.10;
#   2. chaves: senha da chave de assinatura (cria se faltar), ssh-agent, protecao da chave dos
#      ocultos com senha e conferencia da senha;
#   3. grava a partida do corpus (de 1 a 2 minutos da Green Hill; tem de dar REPRODUZ);
#   4. confere os dois TAS contra a REV01 (o que perder a sincronia sai do corpus);
#   5. D1 nas amostras novas do segundo oraculo;
#   6. traces de todos os casos, manifesto com os ocultos, assinatura e verificacao;
#   7. relatorio completo (cargo fidelidade fidelidade);
#   8. auditoria dos ocultos;
#   9. mata o ssh-agent e grava o resumo.
set -eu
rom=${1:?uso: etapa2-fechamento.sh ROM}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
traces=$ext/privado/traces
bin=$ext/trabalho/oracle/alvo/release/oraculo
chave_assina=$HOME/.ssh/websonic_assinatura
chave_ocultos=$HOME/.config/websonic/ocultos.key
chave_ocultos_cifrada=$chave_ocultos.age
resumo=/tmp/etapa2-fechamento.txt
cd "$repo"
[ -x "$bin" ] || { echo "Erro: rode antes scripts/etapa2-final.sh (falta $bin)." >&2; exit 1; }
[ -f "$rom" ] || { echo "Erro: ROM nao encontrada: $rom" >&2; exit 1; }
[ -f "$chave_assina" ] || { echo "Erro: falta a chave de assinatura $chave_assina." >&2; exit 1; }
[ -d "$ext/ocultos" ] || { echo "Erro: faltam os testes ocultos em $ext/ocultos." >&2; exit 1; }
[ -t 0 ] || { echo "Erro: rode num terminal (o script pede senhas)." >&2; exit 1; }

# Termina sempre devolvendo o eco do terminal e matando o ssh-agent que este script criou.
agente_pid=
limpar() {
    stty echo 2>/dev/null || true
    if [ -n "$agente_pid" ]; then
        kill "$agente_pid" 2>/dev/null || true
    fi
}
trap limpar EXIT INT TERM

# Escreve na tela e no resumo. O resumo so recebe o que passa por aqui.
: > "$resumo"
dizer() { echo "$*" | tee -a "$resumo"; }
# Passa a saida de um comando para a tela (na hora, a rodada e longa), para o resumo e para
# $traces/ultimo.txt. O codigo de saida do comando fica de fora; quem precisa confere o conteudo.
registrar() {
    "$@" 2>&1 | tee "$traces/ultimo.txt" | tee -a "$resumo"
}
mkdir -p "$traces"

# Le uma senha sem eco. $1 = pergunta. A resposta fica na variavel SENHA.
ler_senha() {
    printf '%s: ' "$1" >&2
    stty -echo
    IFS= read -r SENHA
    stty echo
    printf '\n' >&2
    [ -n "$SENHA" ] || { echo "Erro: senha vazia." >&2; exit 1; }
}

dizer "=== 1. PATCHES E REPOSITORIO LIMPO ==="
if [ -n "$(git status --porcelain)" ]; then
    # Reexecucao: tudo bem se so o que os passos anteriores deixaram estiver sujo. Pede confirmacao.
    git status --short
    printf 'O repositorio tem mudancas. Continuar mesmo assim (s/N)? ' >&2
    read -r r
    [ "$r" = s ] || exit 1
fi
for p in etapa2-relatorio etapa2-congelar; do
    arq=docs/pedidos/$p.patch
    if git apply --check "$arq" 2>/dev/null; then
        git apply "$arq"
        dizer "$p: aplicado"
    elif git apply --check --reverse "$arq" 2>/dev/null; then
        dizer "$p: ja estava aplicado"
    else
        echo "Erro: $arq nao aplica nem esta aplicado. Veja 'git status'." >&2
        exit 1
    fi
done
sh .devcontainer/verifica-config.sh | tee -a "$resumo"

dizer "=== 2. CHAVES ==="
# 2a. Chave de assinatura: tem de ter senha. Sem senha, qualquer sessao no host poderia assinar.
if ssh-keygen -y -P '' -f "$chave_assina" > /dev/null 2>&1; then
    echo "A chave de assinatura NAO tem senha. Defina uma agora (ssh-keygen vai pedir)." >&2
    ssh-keygen -p -f "$chave_assina"
    dizer "chave de assinatura: senha criada"
else
    dizer "chave de assinatura: tem senha"
fi
# O agente guarda a chave so durante este script; o limpar mata o agente no fim.
eval "$(ssh-agent -s)" > /dev/null
agente_pid=$SSH_AGENT_PID
echo "Digite a senha da chave de assinatura (ssh-add)." >&2
ssh-add "$chave_assina"

# 2b. Chave dos ocultos: protege com senha na primeira vez e confere a senha antes das horas de rodada.
if [ ! -f "$chave_ocultos_cifrada" ]; then
    [ -f "$chave_ocultos" ] || { echo "Erro: falta a chave dos ocultos ($chave_ocultos)." >&2; exit 1; }
    ler_senha "Defina a senha da chave dos ocultos"
    senha1=$SENHA
    ler_senha "Repita a senha"
    [ "$SENHA" = "$senha1" ] || { echo "Erro: as senhas nao batem." >&2; exit 1; }
    printf '%s\n' "$SENHA" | cargo fidelidade ocultos proteger \
        --identidade "$chave_ocultos" --saida "$chave_ocultos_cifrada" --publica refs/ocultos.pub
    # Confere que a cifrada abre ANTES de apagar a clara.
    printf '%s\n' "$SENHA" | cargo fidelidade ocultos conferir \
        --identidade "$chave_ocultos_cifrada" --publica refs/ocultos.pub
    shred -u "$chave_ocultos" 2>/dev/null || rm -f "$chave_ocultos"
    dizer "chave dos ocultos: protegida com senha, copia sem senha apagada"
    senha1=
else
    ler_senha "Senha da chave dos ocultos"
    printf '%s\n' "$SENHA" | cargo fidelidade ocultos conferir \
        --identidade "$chave_ocultos_cifrada" --publica refs/ocultos.pub
    dizer "chave dos ocultos: senha conferida"
    if [ -f "$chave_ocultos" ]; then
        echo "Aviso: ainda existe a copia sem senha ($chave_ocultos). Apague-a." >&2
    fi
fi
senha_ocultos=$SENHA

dizer "=== 3. PARTIDA DO CORPUS ==="
if [ -f corpus/partida-ghz1.s1in ]; then
    dizer "partida-ghz1: ja existe no corpus"
else
    echo "Jogue de 1 a 2 minutos da Green Hill e feche a janela pelo X." >&2
    # O script de gravacao termina com erro se nao der REPRODUZ; o set -e para aqui.
    sh scripts/etapa2-gravar.sh "$rom" corpus partida-ghz1 | tee -a "$resumo"
    [ -f corpus/partida-ghz1.s1in ] || { echo "Erro: a partida nao entrou no corpus (nao deu REPRODUZ?)." >&2; exit 1; }
fi

dizer "=== 4. TAS CONTRA A REV01 ==="
for tas in tas-3570m tas-1937m; do
    if [ ! -f "corpus/$tas.s1in" ]; then
        dizer "$tas: nao esta no corpus (ja descartado ou nunca convertido)"
        continue
    fi
    # --aplicar: se perder a sincronia, sai do corpus e vai para corpus/descartados.txt.
    registrar cargo xtask tas conferir --nome "$tas" --rom "$rom" --oraculo "$bin" --aplicar
done

dizer "=== 5. D1 NAS AMOSTRAS NOVAS (segundo oraculo) ==="
registrar sh scripts/etapa2-d1c.sh "$rom" 6100 6600 7100 7600 8400 10200 11000

dizer "=== 6. TRACES, MANIFESTO E ASSINATURA ==="
registrar cargo fidelidade corpus-trace --rom "$rom" --oraculo "$bin" \
    --rotulos refs/rotulos.txt --saida "$traces"
# A saida ficou em ultimo.txt; confere o veredito de determinismo.
cp "$traces/ultimo.txt" "$traces/resumo-fechamento.tsv"
grep -q '^determinismo: PASSOU$' "$traces/resumo-fechamento.tsv" \
    || { echo "Erro: determinismo nao passou. Veja $traces/resumo-fechamento.tsv." >&2; exit 1; }
grep -v '^determinismo' "$traces/resumo-fechamento.tsv" > "$traces/hashes-fechamento.tsv"
cargo fidelidade manifesto gerar --traces "$traces/hashes-fechamento.tsv" --ocultos "$ext/ocultos" \
    | tee -a "$resumo"
# Assina com a chave que esta no ssh-agent (por isso -f aponta para a publica).
rm -f refs/manifesto.txt.sig  # senao o ssh-keygen pergunta "Overwrite (y/n)?" e trava a rodada sem ninguem
ssh-keygen -Y sign -f "$chave_assina.pub" -n websonic-manifesto refs/manifesto.txt
registrar cargo fidelidade manifesto verificar --pular-testes
grep -q '^assinatura: OK$' "$traces/ultimo.txt" \
    || { echo "Erro: a assinatura nao verificou." >&2; exit 1; }

dizer "=== 7. RELATORIO COMPLETO ==="
registrar cargo fidelidade fidelidade --rom "$rom" --oraculo "$bin" --saida "$traces"

dizer "=== 8. AUDITORIA DOS OCULTOS ==="
printf '%s\n' "$senha_ocultos" | ROM="$rom" ORACULO="$bin" cargo fidelidade ocultos auditar \
    --pasta "$ext/ocultos" --identidade "$chave_ocultos_cifrada" --sut "$repo/scripts/sut-reproduz.sh" \
    | tee -a "$resumo"
senha_ocultos=

dizer "=== 9. FIM ==="
dizer "casos do corpus: $(ls corpus/*.s1in | wc -l) entradas, $(wc -l < corpus/descartados.txt 2>/dev/null || echo 0) linhas em descartados.txt"
dizer "manifesto: $(wc -l < refs/manifesto.txt) linhas"
# Hash dos arquivos novos do corpus (so metadados: nomes e hashes das entradas).
for f in corpus/*.meta; do
    dizer "$(basename "$f"): $(grep -h '^quadros:\|^entradas_sha1:' "$f" | tr '\n' ' ')"
done
dizer "resumo em $resumo. Cole-o na sessao do contêiner (sub-etapa 2.11) e reconstrua a imagem do contêiner."
