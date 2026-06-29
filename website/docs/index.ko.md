---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀)는 <a href="https://github.com/Yamato-Security">Yamato
Security</a>가 제작하고 <a href="https://www.rust-lang.org/">Rust</a>로 작성한 <strong>클라우드 로그를 위한 Sigma 기반 위협 헌팅 및 빠른 포렌식 타임라인
생성기</strong>입니다. <a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>를 떠올려보되,
Windows 이벤트 로그가 아닌 클라우드 로그를 위한 것이며 — AWS CloudTrail에 대한 네이티브
<a href="https://github.com/SigmaHQ/sigma">Sigma</a> 지원을 제공합니다(Azure 및 GCP는 계획 중).
</p>

<div class="hb-cta" markdown>
[시작하기 :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[명령어 레퍼런스 :material-console:](commands/index.md){ .md-button }
[GitHub에서 보기 :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Suzaku를 선택하는 이유

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __클라우드 네이티브 Sigma__

    ---

    클라우드 로그를 위한 네이티브 [Sigma](https://github.com/SigmaHQ/sigma) 탐지 — 현재는 AWS CloudTrail,
    Azure 및 GCP는 계획 중입니다. 상관관계 규칙과 거의 모든 필드 수정자를 지원합니다.

-   :material-timeline-clock:{ .lg .middle } __빠른 포렌식 타임라인__

    ---

    수천 건의 잡음 많은 클라우드 API 호출을 필요한 이벤트만 담은 하나의 분석하기 쉬운 **DFIR 타임라인**으로
    변환합니다.

-   :material-flash:{ .lg .middle } __Rust로 구현된 압도적인 속도__

    ---

    메모리 안전성을 갖추고 멀티스레드로 동작하며 독립 실행이 가능합니다. Windows, Linux, macOS에서
    `.json` 및 압축된 `.json.gz` 로그를 스캔합니다.

-   :material-chart-box:{ .lg .middle } __공격자 요약__

    ---

    API 사용 현황과 공격자 지표 — 소스 IP, 지리적 위치, 리전, 사용자 에이전트 — 를 요약하여
    신속하게 피벗할 수 있습니다.

-   :material-shield-search:{ .lg .middle } __행위 탐지__

    ---

    **시그니처에 의존하지 않고** 비정상 활동을 드러내어 새로운 공격을 놓치지 않습니다.

-   :material-export:{ .lg .middle } __유연한 출력__

    ---

    원하는 도구에서 분석할 수 있도록 결과를 **CSV, JSON, JSONL**로 저장합니다.

</div>

## 빠른 링크

<div class="grid cards" markdown>

-   __:material-book-open-variant: 처음이신가요?__

    [개요](overview/index.md)부터 시작한 다음,
    [시작하기](getting-started/index.md)로 이동하여 Suzaku를 다운로드하고 실행하세요.

-   __:material-console-line: CLI로 작업 중이신가요?__

    [명령어 목록](commands/index.md)과
    [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md),
    [DFIR Timeline](commands/dfir-timeline.md) 명령어에 대한 명령어별 레퍼런스를 살펴보세요.

-   __:material-puzzle: 더 깊이 알아보고 싶으신가요?__

    [네이티브 Sigma 지원](rules/index.md),
    [연계 프로젝트](resources/companion-projects.md), 그리고
    [기여하기](resources/contributing.md) 방법을 알아보세요.

</div>
