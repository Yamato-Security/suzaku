# متقدم: التجميع من المصدر (اختياري)

[بعد تثبيت Rust](https://www.rust-lang.org/)، يمكنك التجميع من المصدر باستخدام الأمر التالي:

ملاحظة: للتجميع، تحتاج عادةً إلى أحدث إصدار من Rust.

```bash
cargo build --release
```

يمكنك تنزيل أحدث إصدار غير مستقر من الفرع الرئيسي أو أحدث إصدار مستقر من صفحة [Releases](https://github.com/Yamato-Security/suzaku/releases).

تأكد من تحديث Rust بشكل دوري باستخدام:

```bash
rustup update stable
```

سيتم إخراج الملف الثنائي المُجمَّع في المجلد `./target/release`.

## تحديث حزم Rust

يمكنك التحديث إلى أحدث صناديق Rust قبل التجميع:

```bash
cargo update
```

> يرجى إعلامنا إذا تعطّل أي شيء بعد التحديث.

## ملاحظات التجميع على macOS

إذا تلقيت أخطاء تجميع متعلقة بـ openssl، فستحتاج إلى تثبيت [Homebrew](https://brew.sh/) ثم تثبيت الحزم التالية:

```bash
brew install pkg-config
brew install openssl
```

## ملاحظات التجميع على Linux

إذا تلقيت أخطاء تجميع متعلقة بـ openssl، فستحتاج إلى تثبيت الحزمة التالية.

التوزيعات المبنية على Ubuntu:

```bash
sudo apt install libssl-dev
```

التوزيعات المبنية على Fedora:

```bash
sudo yum install openssl-devel
```

## التجميع المتقاطع للملفات الثنائية MUSL لنظام Linux Intel

بالنسبة لنظام Linux، نوصي بتجميع الملفات الثنائية GNU كما هو موضح أعلاه ولكن قد ترغب في إنشاء ملفات ثنائية MUSL لقابلية نقل أفضل.
في هذه الحالة، قم أولاً بتثبيت الهدف:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

قم بالتجميع باستخدام:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **تحذير: تأكد من تشغيل `rustup install stable-x86_64-unknown-linux-musl` كلما توفر إصدار مستقر جديد من Rust لأن `rustup update stable` لن يقوم بتحديث المُجمِّع الخاص بالتجميع المتقاطع وقد تتلقى أخطاء في البناء.**

سيتم إنشاء الملف الثنائي MUSL في المجلد `./target/x86_64-unknown-linux-musl/release/`.
الملفات الثنائية MUSL أبطأ بنحو 15% من الملفات الثنائية GNU، ومع ذلك، فهي أكثر قابلية للنقل عبر إصدارات وتوزيعات Linux المختلفة.

> ملاحظة: من المحتمل ألا تعمل الملفات الثنائية MUSL لأنظمة Linux المبنية على ARM بشكل صحيح.
