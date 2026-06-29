# ขั้นสูง: การคอมไพล์จากซอร์ส (ทางเลือก)

[หลังจากติดตั้ง Rust](https://www.rust-lang.org/) แล้ว คุณสามารถคอมไพล์จากซอร์สได้ด้วยคำสั่งต่อไปนี้:

หมายเหตุ: ในการคอมไพล์ โดยทั่วไปคุณจำเป็นต้องใช้ Rust เวอร์ชันล่าสุด

```bash
cargo build --release
```

คุณสามารถดาวน์โหลดเวอร์ชันที่ยังไม่เสถียรล่าสุดได้จาก main branch หรือเวอร์ชันที่เสถียรล่าสุดได้จากหน้า [Releases](https://github.com/Yamato-Security/suzaku/releases)

อย่าลืมอัปเดต Rust เป็นระยะด้วยคำสั่ง:

```bash
rustup update stable
```

ไบนารีที่คอมไพล์แล้วจะถูกสร้างออกมาในโฟลเดอร์ `./target/release`

## การอัปเดต Rust Packages

คุณสามารถอัปเดตเป็น Rust crates ล่าสุดก่อนการคอมไพล์ได้:

```bash
cargo update
```

> โปรดแจ้งให้เราทราบหากมีสิ่งใดเสียหายหลังจากที่คุณอัปเดต

## หมายเหตุการคอมไพล์บน macOS

หากคุณได้รับข้อผิดพลาดในการคอมไพล์เกี่ยวกับ openssl คุณจะต้องติดตั้ง [Homebrew](https://brew.sh/) และจากนั้นติดตั้งแพ็กเกจต่อไปนี้:

```bash
brew install pkg-config
brew install openssl
```

## หมายเหตุการคอมไพล์บน Linux

หากคุณได้รับข้อผิดพลาดในการคอมไพล์เกี่ยวกับ openssl คุณจะต้องติดตั้งแพ็กเกจต่อไปนี้

ดิสโทรที่ใช้พื้นฐาน Ubuntu:

```bash
sudo apt install libssl-dev
```

ดิสโทรที่ใช้พื้นฐาน Fedora:

```bash
sudo yum install openssl-devel
```

## การคอมไพล์ข้ามแพลตฟอร์มสำหรับไบนารี Linux Intel MUSL

สำหรับ Linux เราแนะนำให้คอมไพล์ไบนารี GNU ตามที่อธิบายไว้ข้างต้น แต่คุณอาจต้องการสร้างไบนารี MUSL เพื่อให้พกพาได้ดียิ่งขึ้น
ในกรณีนั้น ให้ติดตั้ง target ก่อน:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

คอมไพล์ด้วยคำสั่ง:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **คำเตือน: อย่าลืมรัน `rustup install stable-x86_64-unknown-linux-musl` ทุกครั้งที่มี Rust เวอร์ชันเสถียรใหม่ เนื่องจาก `rustup update stable` จะไม่อัปเดตคอมไพเลอร์สำหรับการคอมไพล์ข้ามแพลตฟอร์ม และคุณอาจได้รับข้อผิดพลาดในการ build**

ไบนารี MUSL จะถูกสร้างขึ้นในไดเรกทอรี `./target/x86_64-unknown-linux-musl/release/`
ไบนารี MUSL จะช้ากว่าไบนารี GNU ประมาณ 15% อย่างไรก็ตาม ไบนารีเหล่านี้พกพาได้ดีกว่าระหว่างเวอร์ชันและดิสทริบิวชันต่าง ๆ ของ linux

> หมายเหตุ: ไบนารี MUSL สำหรับระบบ Linux ที่ใช้พื้นฐาน ARM อาจจะทำงานได้ไม่ถูกต้อง
