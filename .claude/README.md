# Configuração do Claude Code no projeto

Esta pasta não tem regras de permissão, de propósito.

As permissões, a rede e o sandbox ficam na imagem, em `/etc/claude-code/managed-settings.json`
(fonte: `.devcontainer/managed-settings.json`), com `allowManagedPermissionRulesOnly`. Com isso,
nenhum `settings.json` do projeto consegue liberar o que a imagem proíbe.

Os arquivos locais (`settings.local.json`) e os marcadores do sandbox (`.cc-writes/`) estão no `.gitignore`.
