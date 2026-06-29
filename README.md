<div align="center">
 <p>
    <img alt="Suzaku Logo" src="logo.jpeg" width="60%">
 </p>

 <p>
   <b>Suzaku (朱雀) is a Sigma-based threat hunting and fast forensics timeline generator for cloud logs.</b><br/>
   Created by <a href="https://github.com/Yamato-Security">Yamato Security</a> and written in
   <a href="https://www.rust-lang.org/">Rust</a> — imagine
   <a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>, but for cloud logs.
 </p>

 <p>
    <a href="https://github.com/Yamato-Security/suzaku/releases"><img src="https://img.shields.io/github/v/release/Yamato-Security/suzaku?color=blue&label=Stable%20Version&style=flat"/></a>
    <a href="https://github.com/Yamato-Security/suzaku/releases"><img src="https://img.shields.io/github/downloads/Yamato-Security/suzaku/total?style=flat&label=GitHub%F0%9F%A6%85Downloads&color=blue"/></a>
    <a href="https://github.com/Yamato-Security/suzaku/stargazers"><img src="https://img.shields.io/github/stars/Yamato-Security/suzaku?style=flat&label=GitHub%F0%9F%A6%85Stars"/></a>
    <a href="https://github.com/Yamato-Security/suzaku/blob/main/LICENSE.txt"><img src="https://img.shields.io/badge/License-AGPLv3-blue.svg?style=flat"/></a>
    <a href="https://www.blackhat.com/us-25/arsenal/schedule/index.html#cloud-log-fast-forensics-with-yamato-securitys-suzaku-45630"><img src="https://img.shields.io/badge/Black%20Hat%20Arsenal%20USA-2025-blue"></a>
    <a href="https://twitter.com/SecurityYamato"><img src="https://img.shields.io/twitter/follow/SecurityYamato?style=social"/></a>
 </p>

 <h2>
   📖 <a href="https://yamato-security.github.io/suzaku/">Read the Documentation&nbsp;→</a>
 </h2>

 <sub>
   Available in 15 languages — English · 日本語 · 繁體中文 · 한국어 · Deutsch · Türkçe · Français ·
   Español · Português (Brasil) · Українська · हिन्दी · Bahasa Indonesia · မြန်မာဘာသာ · ไทย · العربية
 </sub>
</div>

---

## 🦅 About

Suzaku (朱雀) — the "Vermilion Bird" that rules the southern heavens above the clouds — is a
**threat hunting and fast forensics timeline generator for cloud logs**, written in memory-safe
[Rust](https://www.rust-lang.org/). Think of [Hayabusa](https://github.com/Yamato-Security/hayabusa)
but for cloud logs instead of Windows event logs, with native
[Sigma](https://github.com/SigmaHQ/sigma) detection for AWS CloudTrail (Azure and GCP planned).

Among thousands of cloud API calls, Suzaku finds the attacks in the noise and gives you a DFIR
timeline with only the events you need — plus summaries of attacker activity (source IPs,
geo-location, regions, user agents) to pivot on.

## 📖 Documentation

All documentation now lives on a dedicated, searchable, multi-language site:

> ### 👉 **[yamato-security.github.io/suzaku](https://yamato-security.github.io/suzaku/)**

| Section | |
| --- | --- |
| 🚀 [Getting Started](https://yamato-security.github.io/suzaku/getting-started/) | Download, install and run Suzaku |
| ⌨️ [Command Reference](https://yamato-security.github.io/suzaku/commands/) | Analysis, DFIR Summary and DFIR Timeline commands |
| 🧩 [Native Sigma Support](https://yamato-security.github.io/suzaku/rules/) | Sigma detection and correlation rules |
| 📦 [Resources](https://yamato-security.github.io/suzaku/resources/companion-projects/) | Companion projects, changelog, contributing |

## ⬇️ Download

Grab the latest binaries from the [**Releases page**](https://github.com/Yamato-Security/suzaku/releases),
or see [Getting Started](https://yamato-security.github.io/suzaku/getting-started/) for building from source.

## 🗂️ Looking for the old README?

The previous single-page README is preserved unchanged:

- 📄 [**OLD-README.md**](OLD-README.md) — English
- 📄 [**OLD-README-Japanese.md**](OLD-README-Japanese.md) — 日本語

## 🤝 Contributing & License

Contributions and bug reports are welcome — see
[Contributing & Support](https://yamato-security.github.io/suzaku/resources/contributing/).
Suzaku is released under the [GNU AGPLv3](LICENSE.txt) license.

---

<div align="center">
  Made with 🦅 by <a href="https://github.com/Yamato-Security">Yamato Security</a>
  &nbsp;·&nbsp; <a href="https://twitter.com/SecurityYamato">@SecurityYamato</a>
</div>
