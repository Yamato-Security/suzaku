---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) é um <strong>gerador de timeline para caça a ameaças baseada em Sigma e
forense rápida de logs de nuvem</strong>, criado pela <a href="https://github.com/Yamato-Security">Yamato
Security</a> e escrito em <a href="https://www.rust-lang.org/">Rust</a>. Imagine o
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>, mas para logs de nuvem em vez de
logs de eventos do Windows — com suporte nativo a <a href="https://github.com/SigmaHQ/sigma">Sigma</a> para
AWS CloudTrail (Azure e GCP planejados).
</p>

<div class="hb-cta" markdown>
[Começar :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Referência de Comandos :material-console:](commands/index.md){ .md-button }
[Ver no GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Por que Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Sigma nativo para nuvem__

    ---

    Detecção nativa com [Sigma](https://github.com/SigmaHQ/sigma) para logs de nuvem — AWS CloudTrail hoje,
    Azure e GCP planejados. Regras de correlação e quase todos os modificadores de campo são suportados.

-   :material-timeline-clock:{ .lg .middle } __Timelines de forense rápida__

    ---

    Transforme milhares de chamadas de API de nuvem ruidosas em uma única e fácil de analisar **timeline de DFIR** apenas com
    os eventos de que você precisa.

-   :material-flash:{ .lg .middle } __Extremamente rápido em Rust__

    ---

    Seguro em memória, multi-thread e autônomo. Analisa logs `.json` e `.json.gz` compactados no
    Windows, Linux e macOS.

-   :material-chart-box:{ .lg .middle } __Resumos de atacantes__

    ---

    Resuma o uso de APIs e métricas de atacantes — IPs de origem, geolocalização, regiões, user agents — para
    fazer pivôs rapidamente.

-   :material-shield-search:{ .lg .middle } __Detecção de comportamento__

    ---

    Identifique atividades anormais **sem depender de assinaturas**, para que você não perca ataques inéditos.

-   :material-export:{ .lg .middle } __Saída flexível__

    ---

    Salve os resultados em **CSV, JSON e JSONL** para análise na ferramenta de sua escolha.

</div>

## Links rápidos

<div class="grid cards" markdown>

-   __:material-book-open-variant: Novo por aqui?__

    Comece pela [Visão geral](overview/index.md), depois siga para
    [Primeiros passos](getting-started/index.md) para baixar e executar o Suzaku.

-   __:material-console-line: Trabalhando com a CLI?__

    Navegue pela [Lista de Comandos](commands/index.md) e pela referência de cada comando para os comandos de
    [Análise](commands/analysis.md), [Resumo de DFIR](commands/dfir-summary.md) e
    [Timeline de DFIR](commands/dfir-timeline.md).

-   __:material-puzzle: Indo além?__

    Explore o [Suporte Nativo a Sigma](rules/index.md), os
    [Projetos Complementares](resources/companion-projects.md) e como
    [contribuir](resources/contributing.md).

</div>
