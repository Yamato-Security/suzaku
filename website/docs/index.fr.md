---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) est un <strong>générateur de chronologies pour la chasse aux menaces et l'investigation forensique rapide basé sur Sigma, destiné aux journaux cloud</strong>, créé par <a href="https://github.com/Yamato-Security">Yamato
Security</a> et écrit en <a href="https://www.rust-lang.org/">Rust</a>. Imaginez
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>, mais pour les journaux cloud au lieu des
journaux d'événements Windows — avec une prise en charge native de <a href="https://github.com/SigmaHQ/sigma">Sigma</a> pour
AWS CloudTrail (Azure et GCP prévus).
</p>

<div class="hb-cta" markdown>
[Commencer :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Référence des commandes :material-console:](commands/index.md){ .md-button }
[Voir sur GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Pourquoi Suzaku ?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Sigma cloud-native__

    ---

    Détection [Sigma](https://github.com/SigmaHQ/sigma) native pour les journaux cloud — AWS CloudTrail aujourd'hui,
    Azure et GCP prévus. Règles de corrélation et quasiment tous les modificateurs de champs pris en charge.

-   :material-timeline-clock:{ .lg .middle } __Chronologies forensiques rapides__

    ---

    Transformez des milliers d'appels d'API cloud bruités en une unique **chronologie DFIR** facile à analyser, avec uniquement
    les événements dont vous avez besoin.

-   :material-flash:{ .lg .middle } __Ultra-rapide en Rust__

    ---

    Sûr en mémoire, multithread et autonome. Analyse les journaux `.json` et compressés `.json.gz` sur
    Windows, Linux et macOS.

-   :material-chart-box:{ .lg .middle } __Synthèses des attaquants__

    ---

    Synthétisez l'utilisation des API et les métriques des attaquants — adresses IP source, géolocalisation, régions, agents utilisateurs — pour
    pivoter rapidement.

-   :material-shield-search:{ .lg .middle } __Détection comportementale__

    ---

    Faites ressortir les activités anormales **sans dépendre des signatures**, afin de ne pas manquer les attaques inédites.

-   :material-export:{ .lg .middle } __Sortie flexible__

    ---

    Enregistrez les résultats au format **CSV, JSON et JSONL** pour les analyser dans l'outil de votre choix.

</div>

## Liens rapides

<div class="grid cards" markdown>

-   __:material-book-open-variant: Nouveau ici ?__

    Commencez par la [Présentation](overview/index.md), puis rendez-vous sur
    [Premiers pas](getting-started/index.md) pour télécharger et exécuter Suzaku.

-   __:material-console-line: Vous travaillez avec la CLI ?__

    Parcourez la [Liste des commandes](commands/index.md) et la référence par commande pour les commandes
    [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md) et
    [DFIR Timeline](commands/dfir-timeline.md).

-   __:material-puzzle: Aller plus loin ?__

    Explorez la [Prise en charge native de Sigma](rules/index.md), les
    [Projets associés](resources/companion-projects.md), et comment
    [contribuer](resources/contributing.md).

</div>
