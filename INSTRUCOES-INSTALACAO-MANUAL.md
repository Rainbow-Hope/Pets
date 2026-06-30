# Instalação manual dos pets

Cada pacote de pet contém:

- `pet.json`
- `spritesheet.webp`

Esses arquivos devem permanecer juntos dentro de uma pasta própria em
`.codex/pets`.

## Baixar pelo GitHub

1. Abra o repositório no GitHub.
2. Entre em `downloads`.
3. Para pets de fusão, entre em `downloads/fusões`.
4. Abra o ZIP desejado.
5. Clique em `Download raw`.

## Windows

Exemplo com `rainbow-hope.zip`:

```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.codex\pets" | Out-Null
Expand-Archive -LiteralPath "$env:USERPROFILE\Downloads\rainbow-hope.zip" -DestinationPath "$env:USERPROFILE\.codex\pets" -Force
```

Troque `rainbow-hope.zip` pelo arquivo baixado.

## macOS ou Linux

```bash
mkdir -p ~/.codex/pets
unzip ~/Downloads/rainbow-hope.zip -d ~/.codex/pets
```

Troque `rainbow-hope.zip` pelo arquivo baixado.

## Sem comandos

1. Extraia o ZIP.
2. Copie a pasta extraída, por exemplo `rainbow-hope`.
3. Cole a pasta em:

```text
Windows: %USERPROFILE%\.codex\pets
macOS/Linux: ~/.codex/pets
```

Ao final:

```text
.codex/pets/rainbow-hope/pet.json
.codex/pets/rainbow-hope/spritesheet.webp
```

## Aplicar no Codex

1. Feche e abra o Codex novamente se ele estiver aberto.
2. Procure o pet nas opções de personalização.
3. Se não aparecer, confirme que `pet.json` e `spritesheet.webp` estão na mesma
   pasta.

As pastas públicas também incluem `validation.json`, que registra a validação
do atlas.
