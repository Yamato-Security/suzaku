---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) — це <strong>генератор хронологій для пошуку загроз і швидкої форензики
на основі Sigma для хмарних журналів</strong>, створений <a href="https://github.com/Yamato-Security">Yamato
Security</a> та написаний мовою <a href="https://www.rust-lang.org/">Rust</a>. Уявіть собі
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>, але для хмарних журналів замість
журналів подій Windows — з рідною підтримкою <a href="https://github.com/SigmaHQ/sigma">Sigma</a> для
AWS CloudTrail (Azure та GCP заплановані).
</p>

<div class="hb-cta" markdown>
[Почати роботу :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Довідник команд :material-console:](commands/index.md){ .md-button }
[Переглянути на GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Чому Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Хмарно-орієнтований Sigma__

    ---

    Рідне виявлення [Sigma](https://github.com/SigmaHQ/sigma) для хмарних журналів — AWS CloudTrail вже сьогодні,
    Azure та GCP заплановані. Підтримуються правила кореляції та майже всі модифікатори полів.

-   :material-timeline-clock:{ .lg .middle } __Швидкі форензичні хронології__

    ---

    Перетворіть тисячі шумних викликів хмарного API на єдину, просту для аналізу **DFIR-хронологію** лише з
    тими подіями, які вам потрібні.

-   :material-flash:{ .lg .middle } __Блискавична швидкість на Rust__

    ---

    Безпечний для пам'яті, багатопотоковий та автономний. Сканує журнали `.json` та стиснені `.json.gz` на
    Windows, Linux та macOS.

-   :material-chart-box:{ .lg .middle } __Зведення про зловмисників__

    ---

    Підбивайте підсумки використання API та метрики зловмисників — вихідні IP, геолокацію, регіони, агенти користувачів — щоб
    швидко робити висновки.

-   :material-shield-search:{ .lg .middle } __Виявлення поведінки__

    ---

    Виявляйте аномальну активність **без покладання на сигнатури**, щоб не пропустити нові атаки.

-   :material-export:{ .lg .middle } __Гнучкий вивід__

    ---

    Зберігайте результати у **CSV, JSON та JSONL** для аналізу в інструменті на ваш вибір.

</div>

## Швидкі посилання

<div class="grid cards" markdown>

-   __:material-book-open-variant: Уперше тут?__

    Почніть з [Огляду](overview/index.md), а потім перейдіть до
    [Початку роботи](getting-started/index.md), щоб завантажити та запустити Suzaku.

-   __:material-console-line: Працюєте з CLI?__

    Перегляньте [Список команд](commands/index.md) та довідник по кожній команді для
    команд [Аналізу](commands/analysis.md), [Зведення DFIR](commands/dfir-summary.md) та
    [Хронології DFIR](commands/dfir-timeline.md).

-   __:material-puzzle: Хочете більше?__

    Дослідіть [Рідну підтримку Sigma](rules/index.md),
    [Супутні проєкти](resources/companion-projects.md) та те, як
    [зробити внесок](resources/contributing.md).

</div>
