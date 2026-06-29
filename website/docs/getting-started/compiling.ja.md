# アドバンス: ソースからのコンパイル (任意)

[Rustをインストールした後](https://www.rust-lang.org/)、以下のコマンドでソースからコンパイルできます。

注意: コンパイルには通常、最新バージョンのRustが必要です

```bash
cargo build --release
```

最新の開発版はメインブランチから、または最新の安定版は[Releases](https://github.com/Yamato-Security/suzaku/releases)ページからダウンロードできます

Rustを定期的に更新してください:

```bash
rustup update stable
```

コンパイルされたバイナリは`./target/release` フォルダに作成されます。

## Rustパッケージの更新

コンパイルする前に、最新のRustクレートに更新できます:

```bash
cargo update
```

> アップデート後に何かが壊れた場合は、お知らせください。

## macOSのコンパイルの注意点

opensslのコンパイルエラーが発生した場合は、[Homebrew](https://brew.sh/)をインストールし、次のパッケージをインストールしてください。

```bash
brew install pkg-config
brew install openssl
```

## Linuxのコンパイルの注意点

opensslのコンパイルエラーが発生した場合は、次のパッケージをインストールしてください。

Ubuntuベースのディストリビューション:

```bash
sudo apt install libssl-dev
```

Fedoraベースのディストリビューション:

```bash
sudo yum install openssl-devel
```

## LinuxのIntel MUSLバイナリのクロスコンパイル

Linuxの場合、上記のようにGNUバイナリをコンパイルすることを推奨しますが、より高い移植性を求める場合はMUSLバイナリを作成することもできます。
その場合は、まず以下のようにターゲットをインストールしてください。

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Compile with:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **警告: 新しい安定版のRustがリリースされるたびに、`rustup install stable-x86_64-unknown-linux-musl`を実行してください。そうしないと、クロスコンパイル用のコンパイラが更新されず、ビルドエラーが発生する可能性があります。**

MUSLバイナリは、`./target/x86_64-unknown-linux-musl/release/`ディレクトリに作成されます。
GNUバイナリよりも約15%遅くなりますが、Linuxの異なるバージョンやディストリビューション間での移植性が高くなります。

> 注意：ARMベースのLinuxシステム向けのMUSLバイナリは、正しく動作しない可能性があります。
