#!/bin/sh
# Ativa os hooks versionados em hooks/. Rode no HOST, uma vez por clone.
# A IA nao consegue rodar isto: ela nao pode alterar a configuracao do git.
# Como o repositorio e o mesmo no host e no conteiner, a ativacao vale nos dois.
set -eu
cd "$(git rev-parse --show-toplevel)"
chmod +x hooks/pre-commit hooks/pre-push hooks/instalar.sh
git config core.hooksPath hooks
echo "hooks ativos: core.hooksPath = $(git config core.hooksPath)"
