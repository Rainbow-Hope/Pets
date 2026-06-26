# Instrucoes para baixar e aplicar os pets

Este repositorio publica cada pet em um pacote separado. Cada pacote contem:

- `pet.json`
- `spritesheet.webp`

Esses dois arquivos precisam ficar juntos dentro de uma pasta propria em `.codex/pets`.

## Baixar pelo GitHub

1. Abra o repositorio no GitHub.
2. Entre na pasta `downloads`.
3. Clique no `.zip` do pet que voce quer baixar, por exemplo `rainbow-hope.zip`.
4. Clique em `Download raw` para baixar o arquivo.

## Instalar no Windows

Depois de baixar um pacote, voce pode instalar com o PowerShell.

Exemplo para `rainbow-hope.zip`:

```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.codex\pets" | Out-Null
Expand-Archive -LiteralPath "$env:USERPROFILE\Downloads\rainbow-hope.zip" -DestinationPath "$env:USERPROFILE\.codex\pets" -Force
```

Exemplo para `natsu-chibi.zip`:

```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.codex\pets" | Out-Null
Expand-Archive -LiteralPath "$env:USERPROFILE\Downloads\natsu-chibi.zip" -DestinationPath "$env:USERPROFILE\.codex\pets" -Force
```

Ao final, a estrutura deve ficar assim:

```text
C:\Users\SEU_USUARIO\.codex\pets\rainbow-hope\pet.json
C:\Users\SEU_USUARIO\.codex\pets\rainbow-hope\spritesheet.webp
```

## Instalar no macOS ou Linux

```bash
mkdir -p ~/.codex/pets
unzip ~/Downloads/rainbow-hope.zip -d ~/.codex/pets
```

Troque `rainbow-hope.zip` pelo pacote que voce baixou.

## Aplicar no Codex

1. Feche e abra o Codex novamente se ele ja estava aberto.
2. Procure o pet instalado nas opcoes de pet/customizacao do Codex.
3. Se ele nao aparecer, confirme se `pet.json` e `spritesheet.webp` estao na mesma pasta.

## Instalar manualmente

Se preferir nao usar comandos:

1. Extraia o `.zip`.
2. Copie a pasta extraida, por exemplo `rainbow-hope`.
3. Cole essa pasta dentro de:

```text
Windows: %USERPROFILE%\.codex\pets
macOS/Linux: ~/.codex/pets
```

## Conferencia

Cada pasta de pet tambem tem `validation.json`, usado para conferir que o spritesheet tem o tamanho esperado e nao possui erros de atlas.
