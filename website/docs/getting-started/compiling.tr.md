# Gelişmiş: Kaynaktan Derleme (İsteğe Bağlı)

[Rust'ı kurduktan sonra](https://www.rust-lang.org/), aşağıdaki komutla kaynaktan derleyebilirsiniz:

Not: Derlemek için genellikle Rust'ın en son sürümüne ihtiyacınız vardır.

```bash
cargo build --release
```

En son kararsız sürümü main dalından veya en son kararlı sürümü [Releases](https://github.com/Yamato-Security/suzaku/releases) sayfasından indirebilirsiniz.

Rust'ı düzenli olarak şu komutla güncellediğinizden emin olun:

```bash
rustup update stable
```

Derlenen ikili dosya `./target/release` klasörüne çıktı olarak verilecektir.

## Rust Paketlerini Güncelleme

Derlemeden önce en son Rust crate'lerine güncelleyebilirsiniz:

```bash
cargo update
```

> Güncellemeden sonra bir şey bozulursa lütfen bize bildirin.

## macOS Derleme Notları

openssl ile ilgili derleme hataları alırsanız, [Homebrew](https://brew.sh/) kurmanız ve ardından aşağıdaki paketleri kurmanız gerekecektir:

```bash
brew install pkg-config
brew install openssl
```

## Linux Derleme Notları

openssl ile ilgili derleme hataları alırsanız, aşağıdaki paketi kurmanız gerekecektir.

Ubuntu tabanlı dağıtımlar:

```bash
sudo apt install libssl-dev
```

Fedora tabanlı dağıtımlar:

```bash
sudo yum install openssl-devel
```

## Linux Intel MUSL İkili Dosyalarını Çapraz Derleme

Linux için, yukarıda açıklandığı gibi GNU ikili dosyalarını derlemenizi öneririz, ancak daha iyi taşınabilirlik için MUSL ikili dosyaları oluşturmak isteyebilirsiniz.
Bu durumda, önce hedefi kurun:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Şununla derleyin:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **Uyarı: Rust'ın yeni bir kararlı sürümü olduğunda her seferinde `rustup install stable-x86_64-unknown-linux-musl` komutunu çalıştırdığınızdan emin olun çünkü `rustup update stable` çapraz derleme için derleyiciyi güncellemez ve derleme hataları alabilirsiniz.**

MUSL ikili dosyası `./target/x86_64-unknown-linux-musl/release/` dizininde oluşturulacaktır.
MUSL ikili dosyaları GNU ikili dosyalarından yaklaşık %15 daha yavaştır, ancak farklı linux sürümleri ve dağıtımları arasında daha taşınabilirdir.

> Not: ARM tabanlı Linux sistemleri için MUSL ikili dosyaları muhtemelen doğru çalışmayacaktır.
