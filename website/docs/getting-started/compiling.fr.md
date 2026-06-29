# Avancé : compilation depuis les sources (facultatif)

[Après avoir installé Rust](https://www.rust-lang.org/), vous pouvez compiler depuis les sources avec la commande suivante :

Note : pour compiler, vous avez généralement besoin de la dernière version de Rust.

```bash
cargo build --release
```

Vous pouvez télécharger la dernière version instable depuis la branche main ou la dernière version stable depuis la page [Releases](https://github.com/Yamato-Security/suzaku/releases).

Veillez à mettre à jour Rust régulièrement avec :

```bash
rustup update stable
```

Le binaire compilé sera généré dans le dossier `./target/release`.

## Mise à jour des paquets Rust

Vous pouvez mettre à jour vers les derniers crates Rust avant de compiler :

```bash
cargo update
```

> Veuillez nous informer si quelque chose ne fonctionne plus après la mise à jour.

## Notes de compilation sous macOS

Si vous recevez des erreurs de compilation concernant openssl, vous devrez installer [Homebrew](https://brew.sh/) puis installer les paquets suivants :

```bash
brew install pkg-config
brew install openssl
```

## Notes de compilation sous Linux

Si vous recevez des erreurs de compilation concernant openssl, vous devrez installer le paquet suivant.

Distributions basées sur Ubuntu :

```bash
sudo apt install libssl-dev
```

Distributions basées sur Fedora :

```bash
sudo yum install openssl-devel
```

## Compilation croisée de binaires Linux Intel MUSL

Pour Linux, nous recommandons de compiler des binaires GNU comme expliqué ci-dessus, mais vous pouvez vouloir créer des binaires MUSL pour une meilleure portabilité.
Dans ce cas, installez d'abord la cible :

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Compilez avec :

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **Avertissement : veillez à exécuter `rustup install stable-x86_64-unknown-linux-musl` chaque fois qu'une nouvelle version stable de Rust est disponible, car `rustup update stable` ne mettra pas à jour le compilateur pour la compilation croisée et vous pourriez recevoir des erreurs de compilation.**

Le binaire MUSL sera créé dans le répertoire `./target/x86_64-unknown-linux-musl/release/`.
Les binaires MUSL sont environ 15 % plus lents que les binaires GNU, cependant ils sont plus portables entre différentes versions et distributions de linux.

> Note : les binaires MUSL pour les systèmes Linux basés sur ARM ne fonctionneront probablement pas correctement.
