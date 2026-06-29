# Avanzado: Compilar desde el código fuente (opcional)

[Después de instalar Rust](https://www.rust-lang.org/), puede compilar desde el código fuente con el siguiente comando:

Nota: Para compilar, normalmente necesita la última versión de Rust.

```bash
cargo build --release
```

Puede descargar la última versión inestable desde la rama main o la última versión estable desde la página de [Releases](https://github.com/Yamato-Security/suzaku/releases).

Asegúrese de actualizar Rust periódicamente con:

```bash
rustup update stable
```

El binario compilado se generará en la carpeta `./target/release`.

## Actualizar paquetes de Rust

Puede actualizar a los últimos crates de Rust antes de compilar:

```bash
cargo update
```

> Háganos saber si algo deja de funcionar después de actualizar.

## Notas de compilación en macOS

Si recibe errores de compilación relacionados con openssl, deberá instalar [Homebrew](https://brew.sh/) y luego instalar los siguientes paquetes:

```bash
brew install pkg-config
brew install openssl
```

## Notas de compilación en Linux

Si recibe errores de compilación relacionados con openssl, deberá instalar el siguiente paquete.

Distribuciones basadas en Ubuntu:

```bash
sudo apt install libssl-dev
```

Distribuciones basadas en Fedora:

```bash
sudo yum install openssl-devel
```

## Compilación cruzada de binarios MUSL de Linux Intel

Para Linux, recomendamos compilar binarios GNU como se explicó anteriormente, pero es posible que desee crear binarios MUSL para una mejor portabilidad.
En ese caso, primero instale el target:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Compile con:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **Advertencia: Asegúrese de ejecutar `rustup install stable-x86_64-unknown-linux-musl` cada vez que haya una nueva versión estable de Rust, ya que `rustup update stable` no actualizará el compilador para la compilación cruzada y podría recibir errores de compilación.**

El binario MUSL se creará en el directorio `./target/x86_64-unknown-linux-musl/release/`.
Los binarios MUSL son aproximadamente un 15% más lentos que los binarios GNU; sin embargo, son más portables entre diferentes versiones y distribuciones de Linux.

> Nota: Es probable que los binarios MUSL para sistemas Linux basados en ARM no se ejecuten correctamente.
