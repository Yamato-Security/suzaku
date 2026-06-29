# 進階：從原始碼編譯（選用）

[安裝 Rust 之後](https://www.rust-lang.org/)，您可以使用以下指令從原始碼進行編譯：

注意：要進行編譯，您通常需要最新版本的 Rust。

```bash
cargo build --release
```

您可以從 main 分支下載最新的不穩定版本，或從 [Releases](https://github.com/Yamato-Security/suzaku/releases) 頁面下載最新的穩定版本。

請務必定期使用以下指令更新 Rust：

```bash
rustup update stable
```

編譯後的二進位檔會輸出至 `./target/release` 資料夾。

## 更新 Rust 套件

您可以在編譯之前更新至最新的 Rust crates：

```bash
cargo update
```

> 如果在您更新之後有任何功能損壞，請讓我們知道。

## macOS 編譯注意事項

如果您收到關於 openssl 的編譯錯誤，您需要安裝 [Homebrew](https://brew.sh/)，然後安裝以下套件：

```bash
brew install pkg-config
brew install openssl
```

## Linux 編譯注意事項

如果您收到關於 openssl 的編譯錯誤，您需要安裝以下套件。

Ubuntu 系列的發行版：

```bash
sudo apt install libssl-dev
```

Fedora 系列的發行版：

```bash
sudo yum install openssl-devel
```

## 交叉編譯 Linux Intel MUSL 二進位檔

對於 Linux，我們建議如上所述編譯 GNU 二進位檔，但您可能會想要建立 MUSL 二進位檔以獲得更好的可攜性。
在這種情況下，請先安裝目標：

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

使用以下指令進行編譯：

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **警告：每當有新的穩定版本的 Rust 時，請務必執行 `rustup install stable-x86_64-unknown-linux-musl`，因為 `rustup update stable` 不會更新用於交叉編譯的編譯器，您可能會收到建置錯誤。**

MUSL 二進位檔會建立於 `./target/x86_64-unknown-linux-musl/release/` 目錄中。
MUSL 二進位檔比 GNU 二進位檔約慢 15%，然而，它們在不同版本與不同的 linux 發行版之間更具可攜性。

> 注意：用於 ARM 架構 Linux 系統的 MUSL 二進位檔可能無法正確執行。
