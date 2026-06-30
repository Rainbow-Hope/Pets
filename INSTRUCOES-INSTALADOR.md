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

1. Baixe o ZIP do pet desejado, sem extraí-lo.
2. No instalador, clique em **Selecionar ZIP**.
3. Escolha o pacote.
4. Confira o nome, o ID e o destino.
5. Clique em **Instalar pet**.

O destino é:

```text
%USERPROFILE%\.codex\pets\<id-do-pet>
```

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
