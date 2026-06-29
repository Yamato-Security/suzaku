# Avançado: Compilando a partir do código-fonte (Opcional)

[Após instalar o Rust](https://www.rust-lang.org/), você pode compilar a partir do código-fonte com o seguinte comando:

Observação: Para compilar, normalmente você precisa da versão mais recente do Rust.

```bash
cargo build --release
```

Você pode baixar a versão instável mais recente a partir da branch main ou a versão estável mais recente na página de [Releases](https://github.com/Yamato-Security/suzaku/releases).

Certifique-se de atualizar o Rust periodicamente com:

```bash
rustup update stable
```

O binário compilado será gerado na pasta `./target/release`.

## Atualizando os pacotes do Rust

Você pode atualizar para os crates mais recentes do Rust antes de compilar:

```bash
cargo update
```

> Por favor, avise-nos se algo quebrar após a atualização.

## Notas de compilação no macOS

Se você receber erros de compilação relacionados ao openssl, será necessário instalar o [Homebrew](https://brew.sh/) e, em seguida, instalar os seguintes pacotes:

```bash
brew install pkg-config
brew install openssl
```

## Notas de compilação no Linux

Se você receber erros de compilação relacionados ao openssl, será necessário instalar o seguinte pacote.

Distros baseadas em Ubuntu:

```bash
sudo apt install libssl-dev
```

Distros baseadas em Fedora:

```bash
sudo yum install openssl-devel
```

## Compilação cruzada de binários Linux Intel MUSL

Para Linux, recomendamos compilar binários GNU conforme explicado acima, mas você pode querer criar binários MUSL para melhor portabilidade.
Nesse caso, primeiro instale o target:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Compile com:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **Aviso: Certifique-se de executar `rustup install stable-x86_64-unknown-linux-musl` sempre que houver uma nova versão estável do Rust, pois `rustup update stable` não atualizará o compilador para compilação cruzada e você poderá receber erros de compilação.**

O binário MUSL será criado no diretório `./target/x86_64-unknown-linux-musl/release/`.
Os binários MUSL são cerca de 15% mais lentos que os binários GNU, porém são mais portáteis entre diferentes versões e distribuições do linux.

> Observação: Binários MUSL para sistemas Linux baseados em ARM provavelmente não funcionarão corretamente.
