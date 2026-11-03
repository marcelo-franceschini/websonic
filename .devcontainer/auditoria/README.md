# Contêiner de auditoria

Reservado. Ainda não há configuração aqui, de propósito.

## O que os requisitos pedem

- Um segundo contêiner, **da mesma imagem** do contêiner de trabalho.
- **Sem iniciar o Claude Code.**
- Com os testes ocultos montados (cifrados, gravados e abertos só pelo Marcelo).
- Só o Marcelo roda a auditoria. A IA vê apenas "passou" ou "falhou" por zona.

## Por que está vazio

Os testes ocultos e o comparador só existem a partir das Etapas 1 e 2, e a decisão D1 deixa
a revisão com a ROM para a Etapa 2. Criar agora um `devcontainer.json` seria inventar montagens
e comandos sem requisito. Quando a Etapa 2 definir o formato do trace e dos testes ocultos,
esta pasta recebe o `devcontainer.json` (mesmo `Dockerfile`, sem o Claude Code e sem o
firewall do contêiner de trabalho, se a auditoria não precisar de rede) e o roteiro de uso.

## Pendente para a Etapa 2

- Formato e local dos testes ocultos cifrados e da chave.
- Como montar a pasta dos testes só neste contêiner.
- Comando que roda a auditoria e imprime só "passou" ou "falhou" por zona.
- Prova de que a IA não alcança esta pasta de testes pelo contêiner de trabalho.
