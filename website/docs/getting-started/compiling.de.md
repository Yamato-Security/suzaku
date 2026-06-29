# Fortgeschritten: Kompilieren aus dem Quellcode (Optional)

[Nach der Installation von Rust](https://www.rust-lang.org/) können Sie mit dem folgenden Befehl aus dem Quellcode kompilieren:

Hinweis: Zum Kompilieren benötigen Sie normalerweise die neueste Version von Rust.

```bash
cargo build --release
```

Sie können die neueste instabile Version vom main-Branch oder die neueste stabile Version von der Seite [Releases](https://github.com/Yamato-Security/suzaku/releases) herunterladen.

Stellen Sie sicher, dass Sie Rust regelmäßig aktualisieren mit:

```bash
rustup update stable
```

Die kompilierte Binärdatei wird im Ordner `./target/release` ausgegeben.

## Aktualisieren der Rust-Pakete

Sie können vor dem Kompilieren auf die neuesten Rust-Crates aktualisieren:

```bash
cargo update
```

> Bitte teilen Sie uns mit, falls nach dem Aktualisieren etwas nicht mehr funktioniert.

## Hinweise zum Kompilieren unter macOS

Wenn Sie Kompilierfehler bezüglich openssl erhalten, müssen Sie [Homebrew](https://brew.sh/) installieren und anschließend die folgenden Pakete installieren:

```bash
brew install pkg-config
brew install openssl
```

## Hinweise zum Kompilieren unter Linux

Wenn Sie Kompilierfehler bezüglich openssl erhalten, müssen Sie das folgende Paket installieren.

Ubuntu-basierte Distributionen:

```bash
sudo apt install libssl-dev
```

Fedora-basierte Distributionen:

```bash
sudo yum install openssl-devel
```

## Cross-Kompilieren von Linux Intel MUSL Binärdateien

Für Linux empfehlen wir, GNU-Binärdateien wie oben beschrieben zu kompilieren, aber Sie möchten möglicherweise MUSL-Binärdateien für bessere Portabilität erstellen.
Installieren Sie in diesem Fall zunächst das Target:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Kompilieren Sie mit:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **Warnung: Stellen Sie sicher, dass Sie `rustup install stable-x86_64-unknown-linux-musl` ausführen, sobald es eine neue stabile Version von Rust gibt, da `rustup update stable` den Compiler für das Cross-Kompilieren nicht aktualisiert und Sie möglicherweise Build-Fehler erhalten.**

Die MUSL-Binärdatei wird im Verzeichnis `./target/x86_64-unknown-linux-musl/release/` erstellt.
MUSL-Binärdateien sind etwa 15 % langsamer als die GNU-Binärdateien, sie sind jedoch portabler über verschiedene Versionen und Distributionen von Linux hinweg.

> Hinweis: MUSL-Binärdateien für ARM-basierte Linux-Systeme werden wahrscheinlich nicht korrekt ausgeführt.
