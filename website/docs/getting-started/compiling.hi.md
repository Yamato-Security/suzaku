# उन्नत: स्रोत से कंपाइल करना (वैकल्पिक)

[Rust इंस्टॉल करने के बाद](https://www.rust-lang.org/), आप निम्नलिखित कमांड के साथ स्रोत से कंपाइल कर सकते हैं:

नोट: कंपाइल करने के लिए, आपको आमतौर पर Rust के नवीनतम संस्करण की आवश्यकता होती है।

```bash
cargo build --release
```

आप main ब्रांच से नवीनतम अस्थिर संस्करण या [Releases](https://github.com/Yamato-Security/suzaku/releases) पेज से नवीनतम स्थिर संस्करण डाउनलोड कर सकते हैं।

Rust को समय-समय पर इसके साथ अपडेट करना सुनिश्चित करें:

```bash
rustup update stable
```

कंपाइल की गई बाइनरी `./target/release` फोल्डर में आउटपुट की जाएगी।

## Rust पैकेज अपडेट करना

आप कंपाइल करने से पहले नवीनतम Rust crates में अपडेट कर सकते हैं:

```bash
cargo update
```

> कृपया हमें बताएं यदि अपडेट करने के बाद कुछ टूट जाता है।

## macOS कंपाइलिंग नोट्स

यदि आपको openssl के बारे में कंपाइल त्रुटियाँ मिलती हैं, तो आपको [Homebrew](https://brew.sh/) इंस्टॉल करना होगा और फिर निम्नलिखित पैकेज इंस्टॉल करने होंगे:

```bash
brew install pkg-config
brew install openssl
```

## Linux कंपाइलिंग नोट्स

यदि आपको openssl के बारे में कंपाइल त्रुटियाँ मिलती हैं, तो आपको निम्नलिखित पैकेज इंस्टॉल करना होगा।

Ubuntu-आधारित distros:

```bash
sudo apt install libssl-dev
```

Fedora-आधारित distros:

```bash
sudo yum install openssl-devel
```

## Linux Intel MUSL बाइनरीज़ को क्रॉस-कंपाइल करना

Linux के लिए, हम ऊपर बताए अनुसार GNU बाइनरीज़ कंपाइल करने की सलाह देते हैं लेकिन आप बेहतर पोर्टेबिलिटी के लिए MUSL बाइनरीज़ बनाना चाह सकते हैं।
उस स्थिति में, पहले target इंस्टॉल करें:

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

इसके साथ कंपाइल करें:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **चेतावनी: जब भी Rust का कोई नया स्थिर संस्करण हो तो `rustup install stable-x86_64-unknown-linux-musl` चलाना सुनिश्चित करें क्योंकि `rustup update stable` क्रॉस कंपाइलिंग के लिए कंपाइलर को अपडेट नहीं करेगा और आपको बिल्ड त्रुटियाँ मिल सकती हैं।**

MUSL बाइनरी `./target/x86_64-unknown-linux-musl/release/` डायरेक्टरी में बनाई जाएगी।
MUSL बाइनरीज़ GNU बाइनरीज़ की तुलना में लगभग 15% धीमी होती हैं, हालाँकि, वे linux के विभिन्न संस्करणों और वितरणों में अधिक पोर्टेबल होती हैं।

> नोट: ARM-आधारित Linux सिस्टम के लिए MUSL बाइनरीज़ शायद सही ढंग से नहीं चलेंगी।
