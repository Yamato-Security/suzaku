---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) adalah <strong>generator timeline threat hunting dan forensik cepat
berbasis Sigma untuk log cloud</strong>, yang dibuat oleh <a href="https://github.com/Yamato-Security">Yamato
Security</a> dan ditulis dalam <a href="https://www.rust-lang.org/">Rust</a>. Bayangkan
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>, tetapi untuk log cloud alih-alih
log event Windows — dengan dukungan <a href="https://github.com/SigmaHQ/sigma">Sigma</a> native untuk
AWS CloudTrail (Azure dan GCP direncanakan).
</p>

<div class="hb-cta" markdown>
[Mulai :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Referensi Perintah :material-console:](commands/index.md){ .md-button }
[Lihat di GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
</div>

<p class="hb-badges">
<a href="https://github.com/Yamato-Security/suzaku/releases"><img src="https://img.shields.io/github/v/release/Yamato-Security/suzaku?color=blue&label=Stable%20Version&style=flat"/></a>
<a href="https://github.com/Yamato-Security/suzaku/releases"><img src="https://img.shields.io/github/downloads/Yamato-Security/suzaku/total?style=flat&label=GitHub%F0%9F%A6%85Downloads&color=blue"/></a>
<a href="https://github.com/Yamato-Security/suzaku/stargazers"><img src="https://img.shields.io/github/stars/Yamato-Security/suzaku?style=flat&label=GitHub%F0%9F%A6%85Stars"/></a>
<a href="https://github.com/Yamato-Security/suzaku/blob/main/LICENSE.txt"><img src="https://img.shields.io/badge/License-AGPLv3-blue.svg?style=flat"/></a>
<a href="https://www.blackhat.com/us-25/arsenal/schedule/index.html#cloud-log-fast-forensics-with-yamato-securitys-suzaku-45630"><img src="https://img.shields.io/badge/Black%20Hat%20Arsenal%20USA-2025-blue"></a>
<a href="https://twitter.com/SecurityYamato"><img src="https://img.shields.io/twitter/follow/SecurityYamato?style=social"/></a>
</p>

</div>

---

## Mengapa Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Sigma cloud-native__

    ---

    Deteksi [Sigma](https://github.com/SigmaHQ/sigma) native untuk log cloud — AWS CloudTrail saat ini,
    Azure dan GCP direncanakan. Aturan korelasi dan hampir semua field modifier didukung.

-   :material-timeline-clock:{ .lg .middle } __Timeline forensik cepat__

    ---

    Ubah ribuan panggilan API cloud yang bising menjadi satu **timeline DFIR** yang mudah dianalisis dengan hanya
    event yang Anda butuhkan.

-   :material-flash:{ .lg .middle } __Sangat cepat dengan Rust__

    ---

    Memory-safe, multi-threaded dan standalone. Memindai log `.json` dan `.json.gz` terkompresi pada
    Windows, Linux dan macOS.

-   :material-chart-box:{ .lg .middle } __Ringkasan penyerang__

    ---

    Ringkas penggunaan API dan metrik penyerang — IP sumber, geo-location, region, user agent — untuk
    melakukan pivot dengan cepat.

-   :material-shield-search:{ .lg .middle } __Deteksi perilaku__

    ---

    Munculkan aktivitas abnormal **tanpa mengandalkan signature**, sehingga Anda tidak melewatkan serangan baru.

-   :material-export:{ .lg .middle } __Output fleksibel__

    ---

    Simpan hasil ke **CSV, JSON dan JSONL** untuk analisis dengan tool pilihan Anda.

</div>

## Tautan cepat

<div class="grid cards" markdown>

-   __:material-book-open-variant: Baru di sini?__

    Mulai dengan [Ikhtisar](overview/index.md), lalu menuju ke
    [Memulai](getting-started/index.md) untuk mengunduh dan menjalankan Suzaku.

-   __:material-console-line: Bekerja dengan CLI?__

    Telusuri [Daftar Perintah](commands/index.md) dan referensi per-perintah untuk
    perintah [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md) dan
    [DFIR Timeline](commands/dfir-timeline.md).

-   __:material-puzzle: Ingin melangkah lebih jauh?__

    Jelajahi [Dukungan Sigma Native](rules/index.md),
    [Proyek Pendamping](resources/companion-projects.md), dan cara
    [berkontribusi](resources/contributing.md).

</div>
