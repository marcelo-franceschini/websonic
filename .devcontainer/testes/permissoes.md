# Roteiro da sub-etapa 0.2, parte 2: permissões

Roda numa sessão do Claude Code **dentro do contêiner de trabalho**, aberto pelo `devcontainer.json`.
Não funciona no host. A rede já foi provada por `.devcontainer/testes/rede.sh`.

## Antes, feito pelo Marcelo (no terminal do contêiner, fora do Claude)

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
4. Bash: `cp ARQ /tmp/copia > /dev/null; echo $?`
5. Programa compilado (prova que não é só texto de comando). Crie em `/tmp`:

   ```sh
   printf '#include <stdio.h>\nint main(int c,char**v){return fopen(v[1],"rb")?0:1;}\n' > /tmp/abre.c
   gcc -o /tmp/abre /tmp/abre.c && /tmp/abre ARQ; echo $?
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

## Registro

Para cada teste, a sessão anota em `docs/conhecimento/ambiente.md`: o comando, o código de retorno e o resultado esperado ou não.
Se qualquer teste de leitura da ROM ou de edição do arquivo travado **passar**, a sub-etapa para.
Nada é corrigido no teste: corrige-se o `managed-settings.json` e reconstrói-se a imagem. Se o sandbox não bloquear o Bash, a decisão D1 volta ao Marcelo.
