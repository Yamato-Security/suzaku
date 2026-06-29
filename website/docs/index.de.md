---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) ist ein <strong>Sigma-basierter Threat-Hunting- und schneller Forensik-Timeline-
Generator für Cloud-Logs</strong>, erstellt von <a href="https://github.com/Yamato-Security">Yamato
Security</a> und geschrieben in <a href="https://www.rust-lang.org/">Rust</a>. Stellen Sie sich
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a> vor, aber für Cloud-Logs anstelle von
Windows-Ereignisprotokollen — mit nativer <a href="https://github.com/SigmaHQ/sigma">Sigma</a>-Unterstützung für
AWS CloudTrail (Azure und GCP geplant).
</p>

<div class="hb-cta" markdown>
[Erste Schritte :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Befehlsreferenz :material-console:](commands/index.md){ .md-button }
[Auf GitHub ansehen :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Warum Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Cloud-natives Sigma__

    ---

    Native [Sigma](https://github.com/SigmaHQ/sigma)-Erkennung für Cloud-Logs — AWS CloudTrail heute,
    Azure und GCP geplant. Korrelationsregeln und nahezu alle Feld-Modifikatoren werden unterstützt.

-   :material-timeline-clock:{ .lg .middle } __Schnelle Forensik-Timelines__

    ---

    Verwandeln Sie Tausende verrauschter Cloud-API-Aufrufe in eine einzige, leicht zu analysierende **DFIR-Timeline** mit nur
    den Ereignissen, die Sie benötigen.

-   :material-flash:{ .lg .middle } __Rasend schnell in Rust__

    ---

    Speichersicher, multithreaded und eigenständig. Scannt `.json`- und komprimierte `.json.gz`-Logs unter
    Windows, Linux und macOS.

-   :material-chart-box:{ .lg .middle } __Angreifer-Zusammenfassungen__

    ---

    Fassen Sie API-Nutzung und Angreifer-Metriken zusammen — Quell-IPs, Geolokalisierung, Regionen, User-Agents — um
    schnell zu pivotieren.

-   :material-shield-search:{ .lg .middle } __Verhaltenserkennung__

    ---

    Decken Sie abnormale Aktivitäten auf, **ohne sich auf Signaturen zu verlassen**, damit Ihnen keine neuartigen Angriffe entgehen.

-   :material-export:{ .lg .middle } __Flexible Ausgabe__

    ---

    Speichern Sie Ergebnisse als **CSV, JSON und JSONL** zur Analyse in dem Tool Ihrer Wahl.

</div>

## Schnelllinks

<div class="grid cards" markdown>

-   __:material-book-open-variant: Neu hier?__

    Beginnen Sie mit der [Übersicht](overview/index.md), und gehen Sie dann zu
    [Erste Schritte](getting-started/index.md), um Suzaku herunterzuladen und auszuführen.

-   __:material-console-line: Arbeiten Sie mit der CLI?__

    Durchsuchen Sie die [Befehlsliste](commands/index.md) und die Referenz pro Befehl für die Befehle
    [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md) und
    [DFIR Timeline](commands/dfir-timeline.md).

-   __:material-puzzle: Wollen Sie weitergehen?__

    Erkunden Sie [Native Sigma-Unterstützung](rules/index.md), die
    [Begleitprojekte](resources/companion-projects.md) und wie Sie
    [beitragen](resources/contributing.md) können.

</div>
