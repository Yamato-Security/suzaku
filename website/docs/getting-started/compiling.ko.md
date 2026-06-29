# 고급: 소스에서 컴파일하기 (선택 사항)

[Rust를 설치한 후](https://www.rust-lang.org/), 다음 명령으로 소스에서 컴파일할 수 있습니다:

참고: 컴파일하려면 일반적으로 최신 버전의 Rust가 필요합니다.

```bash
cargo build --release
```

메인 브랜치에서 최신 불안정 버전을 다운로드하거나 [Releases](https://github.com/Yamato-Security/suzaku/releases) 페이지에서 최신 안정 버전을 다운로드할 수 있습니다.

다음 명령으로 Rust를 주기적으로 업데이트하세요:

```bash
rustup update stable
```

컴파일된 바이너리는 `./target/release` 폴더에 출력됩니다.

## Rust 패키지 업데이트

컴파일하기 전에 최신 Rust 크레이트로 업데이트할 수 있습니다:

```bash
cargo update
```

> 업데이트 후 문제가 발생하면 알려주세요.

## macOS 컴파일 참고 사항

openssl에 관한 컴파일 오류가 발생하면 [Homebrew](https://brew.sh/)를 설치한 다음 아래 패키지를 설치해야 합니다:

```bash
brew install pkg-config
brew install openssl
```

## Linux 컴파일 참고 사항

openssl에 관한 컴파일 오류가 발생하면 아래 패키지를 설치해야 합니다.

Ubuntu 기반 배포판:

```bash
sudo apt install libssl-dev
```

Fedora 기반 배포판:

```bash
sudo yum install openssl-devel
```

## Linux Intel MUSL 바이너리 크로스 컴파일

Linux의 경우 위에서 설명한 대로 GNU 바이너리를 컴파일하는 것을 권장하지만, 더 나은 이식성을 위해 MUSL 바이너리를 생성하고 싶을 수 있습니다.
그러한 경우, 먼저 타겟을 설치하세요:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

다음 명령으로 컴파일하세요:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **경고: `rustup update stable`은 크로스 컴파일을 위한 컴파일러를 업데이트하지 않으므로 빌드 오류가 발생할 수 있습니다. 따라서 Rust의 새로운 안정 버전이 나올 때마다 반드시 `rustup install stable-x86_64-unknown-linux-musl`을 실행하세요.**

MUSL 바이너리는 `./target/x86_64-unknown-linux-musl/release/` 디렉터리에 생성됩니다.
MUSL 바이너리는 GNU 바이너리보다 약 15% 느리지만, 다양한 버전과 배포판의 linux에서 더 이식성이 좋습니다.

> 참고: ARM 기반 Linux 시스템용 MUSL 바이너리는 아마도 올바르게 실행되지 않을 것입니다.
