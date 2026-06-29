# Executando o Suzaku

## Windows

Em um Prompt de Comando/PowerShell ou no Windows Terminal, basta executar o binário do Windows de 32 bits ou 64 bits apropriado.

### Erro ao tentar escanear um arquivo ou diretório com espaço no caminho

Ao usar o Prompt de Comando ou o PowerShell integrado do Windows, você pode receber um erro informando que o Suzaku não conseguiu carregar nenhum arquivo se houver um espaço no caminho do arquivo ou diretório.
Para carregar os arquivos de log corretamente, certifique-se de fazer o seguinte:
1. Coloque o caminho do arquivo ou diretório entre aspas duplas.
2. Se for um caminho de diretório, certifique-se de não incluir uma barra invertida como último caractere.

### Caracteres não sendo exibidos corretamente

Com a fonte padrão `Lucida Console` no Windows, vários caracteres usados no logo e nas tabelas não serão exibidos corretamente.
Você deve alterar a fonte para `Consalas` para corrigir isso.

## Linux

Primeiro, você precisa tornar o binário executável.

```bash
chmod +x ./suzaku
```

Em seguida, execute-o a partir do diretório raiz do Suzaku:

```bash
./suzaku
```

## macOS

A partir do Terminal ou do [iTerm2](https://iterm2.com/), primeiro você precisa tornar o binário executável.

```bash
chmod +x ./suzaku
```

Em seguida, tente executá-lo a partir do diretório raiz do Suzaku:

```bash
./suzaku
```

Na versão mais recente do macOS, você pode receber um erro de segurança ao tentar executá-lo.
Clique em "Cancelar" e, em seguida, nas Preferências do Sistema, abra "Segurança e Privacidade" e, na aba Geral, clique em "Permitir Mesmo Assim".
Depois disso, tente executá-lo novamente.

```bash
./suzaku
```

Um aviso será exibido, então basta clicar em "Abrir".
Agora você deve conseguir executar o suzaku.
