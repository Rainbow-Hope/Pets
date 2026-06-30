# Instalação com o instalador portátil

## Requisitos para usar

- Windows 10 ou Windows 11.
- Codex instalado.
- Um pacote ZIP de pet baixado deste repositório.

Python não é necessário para executar o instalador. O programa não precisa ser
instalado e não solicita privilégios de administrador.

## Baixar e abrir

1. Baixe [Instalador-Pets-Windows.zip](Instalador-Pets-Windows.zip).
2. Extraia o ZIP pelo Explorador de Arquivos do Windows.
3. Abra a pasta `Instalador-Pets-Windows`.
4. Execute `Instalador-Pets.exe`.

O executável não possui assinatura digital. O Windows SmartScreen pode exibir
um aviso. Confirme que o arquivo veio deste repositório e, se decidir
continuar, use **Mais informações** e **Executar assim mesmo**.

## Instalar um pet

1. Volte para a lista de pets no [README](README.md).
2. Baixe o ZIP individual do pet desejado, sem extraí-lo.
   Exemplo: `rainbow-hope.zip`, `kogure-chibi.zip` ou `downloads/fusões/sardonyx.zip`.
3. No instalador, clique em **Selecionar ZIP**.
4. Escolha o pacote ZIP do pet.
5. Confira o nome, o ID e o destino.
6. Clique em **Instalar pet**.
7. Reinicie o Codex se ele já estiver aberto.

O instalador copia o pet para:

```text
%USERPROFILE%\.codex\pets\<id-do-pet>
```

Dentro dessa pasta devem ficar:

```text
pet.json
spritesheet.webp
```

## Usar as opções do programa

Fluxo normal:

1. Clique em **Selecionar ZIP**.
2. Escolha um pacote de pet baixado deste repositório.
3. Se o pet ainda não existir no computador, clique em **Instalar pet**.

Quando o pet já existe:

1. Use **Verificar se é idêntico** se quiser comparar o tamanho ocupado em bits.
2. Se o programa informar que é idêntico, o pet já está presente.
3. Se o programa informar que é distinto, escolha **Atualizar** para substituir ou **Instalar como cópia** para manter os dois.
4. Use **Instalar como cópia** quando quiser dar outro nome ao pet baixado.

O programa não altera outros pets fora da pasta de destino exibida na tela.

## Caminho antigo manual

Se preferir não usar o instalador:

1. Baixe o ZIP do pet desejado.
2. Extraia o conteúdo para `%USERPROFILE%\.codex\pets`.
3. Confira se o caminho final ficou como:

```text
%USERPROFILE%\.codex\pets\<id-do-pet>\pet.json
%USERPROFILE%\.codex\pets\<id-do-pet>\spritesheet.webp
```

4. Reinicie o Codex se ele já estiver aberto.

## Quando o pet já existe

O programa oferece:

- **Verificar se é idêntico**: comparação opcional por tamanho.
- **Atualizar**: substitui o pet usando backup e restauração em caso de falha.
- **Instalar como cópia**: solicita outro nome e cria uma pasta e um ID novos.
- **Cancelar**: não altera arquivos.

### Verificação em bits

A verificação não compara o conteúdo dos arquivos.

1. Soma o tamanho de todos os arquivos do pacote em bits.
2. Compara com a soma dos arquivos instalados.
3. Se as somas forem diferentes, considera os pets distintos.
4. Se forem iguais, compara a lista de caminhos.
5. Depois compara o tamanho exato em bits de cada arquivo correspondente.

Arquivos diferentes podem ter exatamente o mesmo tamanho. Portanto, esse
resultado é uma verificação de armazenamento, não uma prova criptográfica de
conteúdo idêntico.

### Instalar como cópia

O novo nome altera:

- a pasta do pet;
- o campo `id` do `pet.json`;
- o campo `displayName` do `pet.json`.

O spritesheet e a descrição permanecem os mesmos.

## Solução de problemas

- **Pacote inválido:** baixe novamente o ZIP individual do pet.
- **Sem permissão:** confirme que sua conta pode gravar em
  `%USERPROFILE%\.codex\pets`.
- **Pet não aparece:** reinicie o Codex.
- **Aviso do SmartScreen:** confira a origem antes de permitir a execução.

## Requisitos para recompilar

Somente desenvolvedores que desejam gerar outro executável precisam de:

- Python 3.14;
- `pip`;
- PyInstaller 6.21.0;
- PowerShell.

O código e o script de build estão em `installer/`.
