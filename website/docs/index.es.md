---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) es un <strong>generador de líneas de tiempo de caza de amenazas y análisis forense rápido basado en Sigma
para registros en la nube</strong>, creado por <a href="https://github.com/Yamato-Security">Yamato
Security</a> y escrito en <a href="https://www.rust-lang.org/">Rust</a>. Imagina
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>, pero para registros en la nube en lugar de
registros de eventos de Windows — con soporte nativo de <a href="https://github.com/SigmaHQ/sigma">Sigma</a> para
AWS CloudTrail (Azure y GCP planificados).
</p>

<div class="hb-cta" markdown>
[Comenzar :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Referencia de comandos :material-console:](commands/index.md){ .md-button }
[Ver en GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## ¿Por qué Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Sigma nativo para la nube__

    ---

    Detección nativa con [Sigma](https://github.com/SigmaHQ/sigma) para registros en la nube — AWS CloudTrail hoy,
    Azure y GCP planificados. Reglas de correlación y casi todos los modificadores de campo compatibles.

-   :material-timeline-clock:{ .lg .middle } __Líneas de tiempo forenses rápidas__

    ---

    Convierte miles de ruidosas llamadas a la API de la nube en una única **línea de tiempo DFIR** fácil de analizar con solo
    los eventos que necesitas.

-   :material-flash:{ .lg .middle } __Increíblemente rápido en Rust__

    ---

    Seguro en memoria, multihilo e independiente. Escanea registros `.json` y comprimidos `.json.gz` en
    Windows, Linux y macOS.

-   :material-chart-box:{ .lg .middle } __Resúmenes de atacantes__

    ---

    Resume el uso de la API y las métricas de los atacantes — IPs de origen, geolocalización, regiones, agentes de usuario — para
    pivotar rápidamente.

-   :material-shield-search:{ .lg .middle } __Detección de comportamiento__

    ---

    Detecta actividad anormal **sin depender de firmas**, para que no te pierdas ataques novedosos.

-   :material-export:{ .lg .middle } __Salida flexible__

    ---

    Guarda los resultados en **CSV, JSON y JSONL** para analizarlos con la herramienta de tu elección.

</div>

## Enlaces rápidos

<div class="grid cards" markdown>

-   __:material-book-open-variant: ¿Eres nuevo aquí?__

    Comienza con la [Visión general](overview/index.md), luego dirígete a
    [Comenzar](getting-started/index.md) para descargar y ejecutar Suzaku.

-   __:material-console-line: ¿Trabajas con la CLI?__

    Explora la [Lista de comandos](commands/index.md) y la referencia por comando para los comandos de
    [Análisis](commands/analysis.md), [Resumen DFIR](commands/dfir-summary.md) y
    [Línea de tiempo DFIR](commands/dfir-timeline.md).

-   __:material-puzzle: ¿Quieres ir más allá?__

    Explora el [Soporte nativo de Sigma](rules/index.md), los
    [Proyectos complementarios](resources/companion-projects.md), y cómo
    [contribuir](resources/contributing.md).

</div>
