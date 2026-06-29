---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) is a <strong>Sigma-based threat hunting and fast forensics timeline
generator for cloud logs</strong>, created by <a href="https://github.com/Yamato-Security">Yamato
Security</a> and written in <a href="https://www.rust-lang.org/">Rust</a>. Imagine
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>, but for cloud logs instead of
Windows event logs — with native <a href="https://github.com/SigmaHQ/sigma">Sigma</a> support for
AWS CloudTrail (Azure and GCP planned).
</p>

<div class="hb-cta" markdown>
[Get Started :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Command Reference :material-console:](commands/index.md){ .md-button }
[View on GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Why Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Cloud-native Sigma__

    ---

    Native [Sigma](https://github.com/SigmaHQ/sigma) detection for cloud logs — AWS CloudTrail today,
    Azure and GCP planned. Correlation rules and nearly all field modifiers supported.

-   :material-timeline-clock:{ .lg .middle } __Fast forensics timelines__

    ---

    Turn thousands of noisy cloud API calls into a single, easy-to-analyze **DFIR timeline** with only
    the events you need.

-   :material-flash:{ .lg .middle } __Blazing fast in Rust__

    ---

    Memory-safe, multi-threaded and standalone. Scans `.json` and compressed `.json.gz` logs on
    Windows, Linux and macOS.

-   :material-chart-box:{ .lg .middle } __Attacker summaries__

    ---

    Summarize API usage and attacker metrics — source IPs, geo-location, regions, user agents — to
    pivot quickly.

-   :material-shield-search:{ .lg .middle } __Behavior detection__

    ---

    Surface abnormal activity **without relying on signatures**, so you don't miss novel attacks.

-   :material-export:{ .lg .middle } __Flexible output__

    ---

    Save results to **CSV, JSON and JSONL** for analysis in your tool of choice.

</div>

## Quick links

<div class="grid cards" markdown>

-   __:material-book-open-variant: New here?__

    Start with the [Overview](overview/index.md), then head to
    [Getting Started](getting-started/index.md) to download and run Suzaku.

-   __:material-console-line: Working with the CLI?__

    Browse the [Command List](commands/index.md) and the per-command reference for
    [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md) and
    [DFIR Timeline](commands/dfir-timeline.md) commands.

-   __:material-puzzle: Going further?__

    Explore [Native Sigma Support](rules/index.md), the
    [Companion Projects](resources/companion-projects.md), and how to
    [contribute](resources/contributing.md).

</div>
