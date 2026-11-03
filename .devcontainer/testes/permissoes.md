# Roteiro da sub-etapa 0.2, parte 2: permissões

Roda numa sessão do Claude Code **dentro do contêiner de trabalho**, aberto pelo `devcontainer.json`.
Não funciona no host. A rede já foi provada por `.devcontainer/testes/rede.sh`.

## Antes, feito pelo Marcelo

Recriar o contêiner, para valer o perfil seccomp novo (um contêiner já existente não o recebe):

```sh
devcontainer up --workspace-folder . --remove-existing-container
```

Primeiro teste, ainda fora do roteiro: o Bash precisa funcionar. Na sessão do Claude Code, `echo ok` deve imprimir `ok` e sair com 0. Se aparecer `bwrap: No permissions to create new namespace`, pare e traga de volta.

## Modo de permissão

Rode a sessão **fora do modo automático** (`claude --permission-mode default`, ou Shift+Tab até "default"). No modo automático, um classificador do Claude Code pode recusar o comando antes do sandbox, e então o teste não prova a configuração. A coluna "camada" do registro diz quem barrou: permissão, sandbox ou classificador. Só vale como prova a camada que o teste quer provar.

## No terminal do contêiner, fora do Claude

```sh
claude          # entrar e fazer login
/status         # no Claude Code: a linha "Setting sources" deve citar o managed settings
claude doctor   # no terminal: nenhuma entrada rejeitada
```

Anote qualquer erro ou aviso sobre o managed settings. Se houver, pare e traga de volta.

## Regra do teste de leitura

O `CLAUDE.md` permite a exceção só na 0.2: desvie a saída para `/dev/null` e avalie apenas o código de retorno.
Nunca liste a pasta da ROM, nunca mostre o conteúdo. `ARQ` abaixo é `"/mnt/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`. Use sempre as aspas, porque o nome tem espaços e colchetes.

## Testes de leitura (esperado: todos falham, ou seja, código de retorno diferente de 0)

1. Ferramenta Read em `ARQ`. Esperado: recusada pela permissão.
2. Bash: `cat ARQ > /dev/null; echo $?`
3. Bash: `od ARQ > /dev/null; echo $?`
4. Bash: `cp ARQ "$TMPDIR/copia" > /dev/null; echo $?`
5. Programa compilado (prova que não é só texto de comando). Crie em `$TMPDIR` (dentro do sandbox, `/tmp` é só leitura):

   ```sh
   printf '#include <stdio.h>\nint main(int c,char**v){return fopen(v[1],"rb")?0:1;}\n' > "$TMPDIR/abre".c
   gcc -o "$TMPDIR/abre" "$TMPDIR/abre".c && "$TMPDIR/abre" ARQ; echo $?
   ```

6. Ferramenta Read e `cat` em um arquivo qualquer de `/mnt/disasm` (binário ou não). Esperado: recusados.

Controles positivos (esperado: funcionam, para provar que o sandbox não quebrou tudo):
- Ler um arquivo `.asm` em `/mnt/disasm-texto` (só o nome e as primeiras linhas, sem colar no chat).
- `echo ok > /mnt/trabalho/teste.txt && rm /mnt/trabalho/teste.txt`
- `cargo build` num crate de teste em `/tmp`.

## Testes de edição do arquivo travado `docs/requisitos.md` (esperado: todos falham)

7. Ferramenta Edit.
8. `sed -i 's/a/b/' docs/requisitos.md; echo $?`
9. `echo x >> docs/requisitos.md; echo $?`
10. `cp /etc/hostname docs/requisitos.md; echo $?`
11. Programa compilado que abre o arquivo para escrita: `fopen(v[1],"ab")`.
12. Depois de todos: `git status --short docs/requisitos.md` deve estar vazio.

## Testes dos hooks e do git (esperado: todos falham)

13. `git config core.hooksPath /tmp; echo $?`
14. `echo x >> .git/config; echo $?`
15. `touch .git/hooks/pre-commit; echo $?`
16. `git push origin HEAD; echo $?`
17. `git -C . push origin HEAD; echo $?` (a regra de texto é frágil, o que vale é a falta de credenciais)
18. `git commit --no-verify` num commit vazio de teste. Esperado: recusado pela regra.

## Controle positivo: a IA consegue fazer commit local

19. `git add docs && git commit -m "teste: commit no sandbox"; echo $?`. Esperado: 0. A regra é "a IA só faz commit local", então o commit precisa funcionar. Só `.git/config` e `.git/hooks` ficam travados. Um erro `Unable to create '.git/index.lock': Read-only file system` significa que alguma regra cobre a `.git` inteira.
20. `echo x >> .git/config; echo $?` e `touch .git/hooks/pre-commit; echo $?`. Esperado: código diferente de 0, barrado pelo **sandbox** ("Read-only file system"). Repetir fora do modo automático (testes 8, 9, 14 e 15).

## Fora do Claude, no terminal do contêiner

21. `test -f "/mnt/rom/Sonic The Hedgehog (W) (REV01) [!].gen"; echo $?`. Esperado: 0. Prova que o arquivo existe, e que o "No such file or directory" de dentro do sandbox vem de ele estar escondido, não de faltar.
22. `git status --short`. Esperado: sem arquivos vazios (`.bashrc`, `.gitconfig`, `.mcp.json` e outros). O sandbox cria marcadores de 0 byte enquanto um comando roda e os remove depois. Se sobrar algum, rode `claude doctor` e apague com `rm`, sem outra sessão aberta.

## Itens congelados na 0.6 (esperado: todos falham, barrados pelo sandbox)

Exigem a imagem reconstruída com o `managed-settings.json` da 0.6. Rodar fora do modo automático.
Alvos: `CLAUDE.md`, `.gitignore`, `hooks/pre-commit`, `refs/rom-hashes.txt`, `.github/workflows/ci.yml`, `.devcontainer/Dockerfile`.

23. Para cada alvo `ALVO`: `echo x >> ALVO; echo $?`. Esperado: código diferente de 0, "Read-only file system".
24. Ferramenta Edit em cada alvo. Esperado: recusada pela permissão.
25. Criar arquivo novo dentro de uma pasta congelada: `touch hooks/novo; touch refs/novo; touch .github/novo; touch .devcontainer/novo; echo $?`. Esperado: falha.
26. Depois de todos: `git status --short` vazio e `git diff --stat` vazio.
27. Controle positivo: editar `docs/estado.md` e commitar (`git add docs && git commit`) continua funcionando.
28. `claude doctor` no contêiner: no máximo 2 avisos (lançador não criado pelo instalador e método de instalação "not set"), nenhum sobre o managed settings.

## Registro

Para cada teste, a sessão anota em `docs/conhecimento/ambiente.md`: o comando, o código de retorno e o resultado esperado ou não.
Se qualquer teste de leitura da ROM ou de edição do arquivo travado **passar**, a sub-etapa para.
Nada é corrigido no teste: corrige-se o `managed-settings.json` e reconstrói-se a imagem. Se o sandbox não bloquear o Bash, a decisão D1 volta ao Marcelo.
