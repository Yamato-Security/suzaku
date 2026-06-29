# အဆင့်မြင့်: Source မှ Compile ပြုလုပ်ခြင်း (ရွေးချယ်နိုင်သည်)

[Rust ကို install ပြုလုပ်ပြီးနောက်](https://www.rust-lang.org/) အောက်ပါ command ဖြင့် source မှ compile ပြုလုပ်နိုင်ပါသည်:

မှတ်ချက်: Compile ပြုလုပ်ရန်အတွက် ပုံမှန်အားဖြင့် Rust ၏ နောက်ဆုံးထွက် ဗားရှင်းကို လိုအပ်ပါသည်။

```bash
cargo build --release
```

main branch မှ နောက်ဆုံးထွက် တည်ငြိမ်မှုမရှိသေးသော ဗားရှင်းကိုဖြစ်စေ၊ [Releases](https://github.com/Yamato-Security/suzaku/releases) စာမျက်နှာမှ နောက်ဆုံးထွက် တည်ငြိမ်သော ဗားရှင်းကိုဖြစ်စေ download ပြုလုပ်နိုင်ပါသည်။

Rust ကို အောက်ပါအတိုင်း အခါအားလျော်စွာ update ပြုလုပ်ရန် သေချာပါစေ:

```bash
rustup update stable
```

Compile ပြုလုပ်ပြီးသော binary ကို `./target/release` folder အတွင်းသို့ ထုတ်ပေးပါမည်။

## Rust Packages များကို Update ပြုလုပ်ခြင်း

Compile မပြုလုပ်မီ နောက်ဆုံးထွက် Rust crates များသို့ update ပြုလုပ်နိုင်ပါသည်:

```bash
cargo update
```

> Update ပြုလုပ်ပြီးနောက် တစ်စုံတစ်ရာ ပျက်စီးသွားပါက ကျွန်ုပ်တို့အား အသိပေးပါ။

## macOS Compile ပြုလုပ်ခြင်းဆိုင်ရာ မှတ်ချက်များ

openssl နှင့်ပတ်သက်သော compile error များကို ရရှိပါက [Homebrew](https://brew.sh/) ကို install ပြုလုပ်ပြီးနောက် အောက်ပါ packages များကို install ပြုလုပ်ရန် လိုအပ်ပါမည်:

```bash
brew install pkg-config
brew install openssl
```

## Linux Compile ပြုလုပ်ခြင်းဆိုင်ရာ မှတ်ချက်များ

openssl နှင့်ပတ်သက်သော compile error များကို ရရှိပါက အောက်ပါ package ကို install ပြုလုပ်ရန် လိုအပ်ပါမည်။

Ubuntu-based distros:

```bash
sudo apt install libssl-dev
```

Fedora-based distros:

```bash
sudo yum install openssl-devel
```

## Linux Intel MUSL Binaries များကို Cross-compile ပြုလုပ်ခြင်း

Linux အတွက် အထက်တွင် ရှင်းပြထားသည့်အတိုင်း GNU binaries များကို compile ပြုလုပ်ရန် အကြံပြုပါသည်၊ သို့သော် ပိုမိုကောင်းမွန်သော portability အတွက် MUSL binaries များကို ဖန်တီးလိုနိုင်ပါသည်။
ထိုသို့ဖြစ်ပါက ပထမဦးစွာ target ကို install ပြုလုပ်ပါ:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

အောက်ပါအတိုင်း compile ပြုလုပ်ပါ:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **သတိ: `rustup update stable` သည် cross compile ပြုလုပ်ရန်အတွက် compiler ကို update မပြုလုပ်သောကြောင့် build error များ ရရှိနိုင်သည်။ ထို့ကြောင့် Rust ၏ တည်ငြိမ်သော ဗားရှင်းအသစ် ထွက်ရှိသည့်အခါတိုင်း `rustup install stable-x86_64-unknown-linux-musl` ကို run ရန် သေချာပါစေ။**

MUSL binary ကို `./target/x86_64-unknown-linux-musl/release/` directory တွင် ဖန်တီးပါမည်။
MUSL binaries များသည် GNU binaries များထက် ၁၅% ခန့် နှေးကွေးသော်လည်း linux ၏ မတူညီသော ဗားရှင်းများနှင့် distribution များတွင် ပိုမို portable ဖြစ်ပါသည်။

> မှတ်ချက်: ARM-based Linux စနစ်များအတွက် MUSL binaries များသည် မှန်ကန်စွာ run နိုင်လိမ့်မည်မဟုတ်ပါ။
