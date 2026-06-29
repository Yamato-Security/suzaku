---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) 是一套<strong>以 Sigma 為基礎、針對雲端日誌的威脅獵捕與快速鑑識時間軸
產生器</strong>，由 <a href="https://github.com/Yamato-Security">Yamato
Security</a> 開發，並以 <a href="https://www.rust-lang.org/">Rust</a> 撰寫。想像一下
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>，但對象是雲端日誌而非
Windows 事件日誌——並原生支援針對 AWS CloudTrail 的 <a href="https://github.com/SigmaHQ/sigma">Sigma</a>（Azure 與 GCP 規劃中）。
</p>

<div class="hb-cta" markdown>
[開始使用 :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[指令參考 :material-console:](commands/index.md){ .md-button }
[在 GitHub 上檢視 :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## 為什麼選擇 Suzaku？

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __雲端原生 Sigma__

    ---

    針對雲端日誌的原生 [Sigma](https://github.com/SigmaHQ/sigma) 偵測——目前支援 AWS CloudTrail，
    Azure 與 GCP 規劃中。支援關聯規則以及幾乎所有的欄位修飾子。

-   :material-timeline-clock:{ .lg .middle } __快速鑑識時間軸__

    ---

    將數以千計、雜訊繁多的雲端 API 呼叫，轉化為一份僅包含你所需事件、易於分析的 **DFIR 時間軸**。

-   :material-flash:{ .lg .middle } __以 Rust 打造、速度驚人__

    ---

    記憶體安全、多執行緒且獨立運行。可在 Windows、Linux 與 macOS 上掃描 `.json` 與壓縮過的 `.json.gz` 日誌。

-   :material-chart-box:{ .lg .middle } __攻擊者摘要__

    ---

    彙整 API 使用情形與攻擊者指標——來源 IP、地理位置、區域、使用者代理——以便快速進行樞紐分析。

-   :material-shield-search:{ .lg .middle } __行為偵測__

    ---

    **不依賴特徵碼**即可揭露異常活動，讓你不會錯過新型態的攻擊。

-   :material-export:{ .lg .middle } __彈性的輸出__

    ---

    可將結果儲存為 **CSV、JSON 與 JSONL**，以便使用你慣用的工具進行分析。

</div>

## 快速連結

<div class="grid cards" markdown>

-   __:material-book-open-variant: 第一次使用？__

    請從[總覽](overview/index.md)開始，接著前往
    [開始使用](getting-started/index.md)以下載並執行 Suzaku。

-   __:material-console-line: 正在使用 CLI？__

    瀏覽[指令清單](commands/index.md)，以及
    [Analysis](commands/analysis.md)、[DFIR Summary](commands/dfir-summary.md) 與
    [DFIR Timeline](commands/dfir-timeline.md) 各指令的詳細參考。

-   __:material-puzzle: 想更進一步？__

    探索[原生 Sigma 支援](rules/index.md)、
    [相關專案](resources/companion-projects.md)，以及如何
    [貢獻](resources/contributing.md)。

</div>
