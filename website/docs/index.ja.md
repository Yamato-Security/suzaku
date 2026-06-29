---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong>（朱雀）は、<a href="https://github.com/Yamato-Security">Yamato Security</a>
によって作られた、<strong>クラウドログ向けの Sigma ベースのスレットハンティング兼高速フォレンジック
タイムライン生成ツール</strong>です。<a href="https://www.rust-lang.org/">Rust</a> で記述されています。
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a> の「クラウドログ版」とお考えください。
AWS CloudTrail に対する <a href="https://github.com/SigmaHQ/sigma">Sigma</a> のネイティブ対応を備えています
（Azure・GCP は対応予定）。
</p>

<div class="hb-cta" markdown>
[はじめる :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[コマンド一覧 :material-console:](commands/index.md){ .md-button }
[GitHub で見る :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## なぜ Suzaku なのか？

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __クラウドネイティブな Sigma__

    ---

    クラウドログに対する [Sigma](https://github.com/SigmaHQ/sigma) のネイティブ検知。現在は AWS
    CloudTrail に対応し、Azure・GCP は対応予定。相関ルールとほぼ全てのフィールド修飾子をサポート。

-   :material-timeline-clock:{ .lg .middle } __高速フォレンジックタイムライン__

    ---

    膨大でノイズの多いクラウド API 呼び出しを、必要なイベントだけに絞った解析しやすい単一の
    **DFIR タイムライン**に変換します。

-   :material-flash:{ .lg .middle } __Rust による圧倒的な速さ__

    ---

    メモリセーフ・マルチスレッド・スタンドアロン。Windows・Linux・macOS で `.json` および圧縮
    `.json.gz` ログをスキャンします。

-   :material-chart-box:{ .lg .middle } __攻撃者のサマリ__

    ---

    API の利用状況や攻撃者のメトリクス（送信元 IP、地理情報、リージョン、ユーザーエージェント）を
    要約し、素早くピボットできます。

-   :material-shield-search:{ .lg .middle } __振る舞い検知__

    ---

    **シグネチャに依存せず**異常な活動を浮かび上がらせ、新種の攻撃も見逃しません。

-   :material-export:{ .lg .middle } __柔軟な出力__

    ---

    結果を **CSV・JSON・JSONL** で保存し、お好みのツールで解析できます。

</div>

## クイックリンク

<div class="grid cards" markdown>

-   __:material-book-open-variant: はじめての方へ__

    まずは[概要](overview/index.md)を読み、[はじめる](getting-started/index.md)で
    Suzaku のダウンロードと実行を行いましょう。

-   __:material-console-line: CLI を使う__

    [コマンド一覧](commands/index.md)や、[分析](commands/analysis.md)・
    [DFIR サマリー](commands/dfir-summary.md)・[DFIR タイムライン](commands/dfir-timeline.md)
    の各コマンドリファレンスへ。

-   __:material-puzzle: さらに活用する__

    [ネイティブ Sigma サポート](rules/index.md)、[関連プロジェクト](resources/companion-projects.md)、
    [貢献方法](resources/contributing.md)を見てみましょう。

</div>
