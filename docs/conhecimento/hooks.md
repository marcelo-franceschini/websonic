# Hooks do git

Camada 3 da proteção da ROM (ver `docs/requisitos.md`). Shell POSIX, em `hooks/`.

## O que bloqueia

| Verificação | Regra | Fonte |
| --- | --- | --- |
| Hash | O id do blob é o da ROM REV01. Nenhuma exceção vale. | `refs/rom-hashes.txt` (`blob_id`) |
| Tamanho | Tamanho maior ou igual ao limite. | `refs/limite-tamanho.txt` (`max_bytes`, 262144) |
| Extensão | `gen bin smd rom 32x sms iso zip 7z rar tar gz bz2 xz sav srm state* trace gst wav mp3 ogg flac vgm`, sem diferenciar maiúsculas. | lista em `hooks/lib.sh`, espelha o `.gitignore` |

Exceções por caminho (só extensão e tamanho) ficam em `refs/excecoes-extensoes.txt`, com aprovação do Marcelo.

## Como funciona

- `pre-commit` olha o índice. `pre-push` olha todos os objetos que sobem e ainda não estão em nenhuma ref remota. Assim um arquivo apagado num commit posterior continua barrado.
- O tamanho e o hash vêm do id do objeto no git, e não do nome. Renomear a ROM para `.txt` não escapa.
- Em dúvida, bloqueia. Se um arquivo de `refs/` faltar, o hook recusa.
- O hash do blob (`blob_id`) é o SHA-1 do objeto git, e não o SHA-1 do arquivo. O `refs/rom-hashes.txt` guarda os dois.
- O hook não lê a ROM. Compara identificadores.

## Ativação

No host, uma vez por clone: `sh hooks/instalar.sh`. Ele define `core.hooksPath = hooks`. O repositório é o mesmo no host e no contêiner, então vale nos dois. A IA não consegue rodar isso, porque não pode alterar a configuração do git.

## Limites conhecidos

- `git commit --no-verify` pula o `pre-commit`. O `pre-push` do Marcelo e o CI pegam o que escapar. Uma regra de texto do Claude Code não é barreira técnica.
- Os hooks leem `refs/` da árvore de trabalho. Na 0.6 os hooks e `refs/` são congelados contra edição da IA.
- Um arquivo com conteúdo já enviado antes (mesmo id de blob) não é reexaminado por nome no `pre-push`. Isso não afeta a ROM, que é pega pelo hash.

## Testes

- `sh .devcontainer/testes/hooks.sh`: 14 testes num repositório descartável, com um "remoto" local vazio e arquivos sintéticos. Não toca na ROM.
- `sh .devcontainer/testes/hooks-rom-real.sh`: com a ROM real. Rodado só pelo Marcelo, no host. A IA nunca roda.

## Evidência

Rodada em 03/11/2026: 14 testes, 0 falhas (`hooks.sh`). Cobre arquivo limpo, `.gen`, maiúsculas, nome com espaços e colchetes, 262143 e 262144 bytes, hash com nome inocente, exceção que não libera o hash, exceção que libera outras regras, `refs/` ausente (falha fechada), push limpo, push com ROM no histórico e remoto sem receber nada.

Um erro no primeiro teste do push veio do próprio teste: o conteúdo `x` já tinha sido enviado, e o git guarda por conteúdo. O teste passou a usar conteúdo único. O hook não mudou.
