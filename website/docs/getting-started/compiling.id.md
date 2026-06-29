# Lanjutan: Mengompilasi Dari Sumber (Opsional)

[Setelah menginstal Rust](https://www.rust-lang.org/), Anda dapat mengompilasi dari sumber dengan perintah berikut:

Catatan: Untuk mengompilasi, Anda biasanya memerlukan Rust versi terbaru.

```bash
cargo build --release
```

Anda dapat mengunduh versi tidak stabil terbaru dari branch main atau versi stabil terbaru dari halaman [Releases](https://github.com/Yamato-Security/suzaku/releases).

Pastikan untuk memperbarui Rust secara berkala dengan:

```bash
rustup update stable
```

Biner yang telah dikompilasi akan dihasilkan di folder `./target/release`.

## Memperbarui Paket Rust

Anda dapat memperbarui ke crate Rust terbaru sebelum mengompilasi:

```bash
cargo update
```

> Mohon beri tahu kami jika ada yang rusak setelah Anda memperbarui.

## Catatan Kompilasi macOS

Jika Anda menerima kesalahan kompilasi tentang openssl, Anda perlu menginstal [Homebrew](https://brew.sh/) dan kemudian menginstal paket-paket berikut:

```bash
brew install pkg-config
brew install openssl
```

## Catatan Kompilasi Linux

Jika Anda menerima kesalahan kompilasi tentang openssl, Anda perlu menginstal paket berikut.

Distro berbasis Ubuntu:

```bash
sudo apt install libssl-dev
```

Distro berbasis Fedora:

```bash
sudo yum install openssl-devel
```

## Kompilasi Silang Biner Linux Intel MUSL

Untuk Linux, kami menyarankan untuk mengompilasi biner GNU seperti yang dijelaskan di atas, tetapi Anda mungkin ingin membuat biner MUSL untuk portabilitas yang lebih baik.
Dalam kasus tersebut, instal target terlebih dahulu:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Kompilasi dengan:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **Peringatan: Pastikan untuk menjalankan `rustup install stable-x86_64-unknown-linux-musl` setiap kali ada versi stabil Rust yang baru karena `rustup update stable` tidak akan memperbarui kompiler untuk kompilasi silang dan Anda mungkin menerima kesalahan build.**

Biner MUSL akan dibuat di direktori `./target/x86_64-unknown-linux-musl/release/`.
Biner MUSL sekitar 15% lebih lambat daripada biner GNU, namun, biner ini lebih portabel di berbagai versi dan distribusi linux.

> Catatan: Biner MUSL untuk sistem Linux berbasis ARM kemungkinan tidak akan berjalan dengan benar.
