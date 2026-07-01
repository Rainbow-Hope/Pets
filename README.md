# Pets

Repositorio publico dos pets to-RusH para baixar e instalar individualmente no Codex.

## Downloads

| Pet | Preview | Download individual | Arquivos |
| --- | --- | --- | --- |
| Kogure Chibi | ![Kogure Chibi](pets/kogure-chibi/preview.gif) | [kogure-chibi.zip](downloads/kogure-chibi.zip) | [pasta](pets/kogure-chibi/) |
| Zuko Chibi | ![Zuko Chibi](pets/zuko-chibi/preview.gif) | [zuko-chibi.zip](downloads/zuko-chibi.zip) | [pasta](pets/zuko-chibi/) |
| Rainbow Hope | ![Rainbow Hope](pets/rainbow-hope/preview.gif) | [rainbow-hope.zip](downloads/rainbow-hope.zip) | [pasta](pets/rainbow-hope/) |
| Natsu Chibi | ![Natsu Chibi](pets/natsu-chibi/preview.gif) | [natsu-chibi.zip](downloads/natsu-chibi.zip) | [pasta](pets/natsu-chibi/) |
| Arco-Íris 2.0 | ![Arco-Íris 2.0](pets/arco-iris-2-0/preview.gif) | [arco-iris-2-0.zip](downloads/arco-iris-2-0.zip) | [pasta](pets/arco-iris-2-0/) |
| Pokemon Ranger & Ukulele Pichu | ![Pokemon Ranger & Ukulele Pichu](pets/pokemon-ranger-guardian-signs/preview.gif) | [pokemon-ranger-guardian-signs.zip](downloads/pokemon-ranger-guardian-signs.zip) | [pasta](pets/pokemon-ranger-guardian-signs/) |
| Steven Filme | ![Steven Filme](pets/steven-filme/preview.gif) | [steven-filme.zip](downloads/steven-filme.zip) | [pasta](pets/steven-filme/) |

## Fusões

| Pet | Preview | Download individual | Arquivos |
| --- | --- | --- | --- |
| Kogure-Zuko | ![Kogure-Zuko](pets/fusões/kogure-zuko/preview.gif) | [kogure-zuko.zip](downloads/fusões/kogure-zuko.zip) | [pasta](pets/fusões/kogure-zuko/) |
| Zuko-Kogure | ![Zuko-Kogure](pets/fusões/zuko-kogure/preview.gif) | [zuko-kogure.zip](downloads/fusões/zuko-kogure.zip) | [pasta](pets/fusões/zuko-kogure/) |
| Natsu Ranger | ![Natsu Ranger](pets/fusões/natsu-ranger/preview.gif) | [natsu-ranger.zip](downloads/fusões/natsu-ranger.zip) | [pasta](pets/fusões/natsu-ranger/) |
| Ranger Natsu | ![Ranger Natsu](pets/fusões/ranger-natsu/preview.gif) | [ranger-natsu.zip](downloads/fusões/ranger-natsu.zip) | [pasta](pets/fusões/ranger-natsu/) |
| Obsidiana | ![Obsidiana](pets/fusões/obsidiana/preview.gif) | [obsidiana.zip](downloads/fusões/obsidiana.zip) | [pasta](pets/fusões/obsidiana/) |
| Pedra do Sol | ![Pedra do Sol](pets/fusões/pedra-do-sol/preview.gif) | [pedra-do-sol.zip](downloads/fusões/pedra-do-sol.zip) | [pasta](pets/fusões/pedra-do-sol/) |
| Quartzo Fumê | ![Quartzo Fumê](pets/fusões/quartzo-fume/preview.gif) | [quartzo-fume.zip](downloads/fusões/quartzo-fume.zip) | [pasta](pets/fusões/quartzo-fume/) |
| Sardonyx | ![Sardonyx](pets/fusões/sardonyx/preview.gif) | [sardonyx.zip](downloads/fusões/sardonyx.zip) | [pasta](pets/fusões/sardonyx/) |

## Executáveis autônomos para rodar fora do Códex

Além dos pets para o Códex, este repositório também contém executáveis
portáteis para Windows em [Executar fora do Códex](<Executar fora do Códex/>).
Eles não são instaladores: é para manter a pasta da edição junta e abrir o
`.exe` diretamente. Não precisa instalar Rust, Python, Códex, WebView, serviço
ou runtime separado.

O pet padrão inicial de todas as edições é Rainbow Hope.

| Tipo | Pasta | Executável | Resumo |
| --- | --- | --- | --- |
| Normal | [Normal](<Executar fora do Códex/Normal/>) | `DesktopPets.exe` | versão completa, com vários pets, importação ZIP pelo menu e o auxiliar `AdicionarTodosOsPets.exe` já incluído |
| Leve / Micro | [Micro](<Executar fora do Códex/Leves/Micro/>) | `DesktopPetsMicro.exe` | quase completa, limitada a 4 pets, com menor frequência de atualização |
| Leve / Nano | [Nano](<Executar fora do Códex/Leves/Nano/>) | `DesktopPetsNano.exe` | equilibrada, 1 pet com movimento automático e interface menor |
| Leve / Pico | [Pico](<Executar fora do Códex/Leves/Pico/>) | `DesktopPetsPico.exe` | essencial, 1 pet fixo, menor atividade esperada |

Arquivos que devem ficar juntos em cada pasta:

- o executável da edição;
- `config.json`;
- a pasta `pets`;
- `LEIA-ME.txt`;
- `DIFERENCAS-ENTRE-EDICOES.txt`.

Não mova somente o `.exe`, porque a edição usa os arquivos da própria pasta.
Para fechar, mudar tamanho e escolher pet, use o menu do pet com clique direito.

### Auxiliar opcional para adicionar todos os pets

`AdicionarTodosOsPets.exe` é um programa auxiliar opcional e autônomo. Ele
adiciona à edição atual todos os pets embutidos no próprio auxiliar e também os
pets válidos encontrados em `%USERPROFILE%\.codex\pets`, quando essa pasta
existir.

- A versão Normal já contém o auxiliar na mesma pasta.
- Para Micro, Nano ou Pico, copie o auxiliar de
  [Auxiliar opcional - Todos os Pets](<Executar fora do Códex/Leves/Auxiliar opcional - Todos os Pets/>)
  para a mesma pasta da edição leve escolhida antes de executar.
- O auxiliar não modifica `.codex\pets`, não altera o Códex e não substitui
  pets que já existem na pasta de destino.
- Depois de importar os pets, o auxiliar pode ser apagado sem remover os pets
  copiados para a pasta `pets`.

Veja também:

- [Instruções das edições autônomas](<Executar fora do Códex/LEIA-ME.txt>)
- [Diferenças entre Normal, Micro, Nano e Pico](<Executar fora do Códex/DIFERENCAS-ENTRE-EDICOES.txt>)

## Como instalar

Baixe o [Instalador-Pets-Windows.zip](Instalador-Pets-Windows.zip) para instalar
os pets por uma interface portátil no Windows.

- [Instalação manual](INSTRUCOES-INSTALACAO-MANUAL.md)
- [Instalação com o instalador para Windows](INSTRUCOES-INSTALADOR.md)
- [Índice de instruções](INSTRUCOES.md)

Resumo rapido com o instalador:

1. Baixe `Instalador-Pets-Windows.zip`.
2. Extraia o ZIP e abra `Instalador-Pets.exe`.
3. Baixe o `.zip` do pet desejado na tabela acima, sem extrair.
4. No instalador, clique em `Selecionar ZIP`, escolha o pacote do pet e clique em `Instalar pet`.
5. Reinicie o Codex se ele ja estiver aberto.

Python nao e necessario para usar o instalador pronto. Veja o passo a passo
completo em [INSTRUCOES-INSTALADOR.md](INSTRUCOES-INSTALADOR.md).

Resumo rapido manual:

1. Baixe o `.zip` do pet desejado na tabela acima.
2. Extraia o `.zip` dentro da pasta de pets do Codex.
3. Reinicie o Codex se ele ja estiver aberto.

No Windows, a pasta final deve ficar assim:

```text
%USERPROFILE%\.codex\pets\nome-do-pet\pet.json
%USERPROFILE%\.codex\pets\nome-do-pet\spritesheet.webp
```

## Estrutura

- `pets/`: arquivos individuais de cada pet.
- `pets/fusões/`: arquivos individuais dos pets de fusao.
- `downloads/`: pacotes `.zip` prontos para download e instalacao.
- `downloads/fusões/`: pacotes `.zip` das fusoes.
- `Executar fora do Códex/`: edições autônomas para Windows que rodam por
  clique direto, sem instalação.
- `installer/`: código-fonte, testes e build do instalador portátil.
- `Instalador-Pets-Windows.zip`: executável portátil para Windows.
- `INSTRUCOES.md`: índice dos métodos de instalação.

Os arquivos nas pastas antigas de criacao ficam fora do Git; so os pacotes finais entram no repositorio publico.
