---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀), <a href="https://github.com/Yamato-Security">Yamato
Security</a> tarafından oluşturulan ve <a href="https://www.rust-lang.org/">Rust</a> ile yazılmış,
<strong>bulut günlükleri için Sigma tabanlı bir tehdit avcılığı ve hızlı adli zaman çizelgesi
oluşturucusudur</strong>. <a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>'yı
hayal edin, ancak Windows olay günlükleri yerine bulut günlükleri için — AWS CloudTrail için yerel
<a href="https://github.com/SigmaHQ/sigma">Sigma</a> desteğiyle (Azure ve GCP planlanıyor).
</p>

<div class="hb-cta" markdown>
[Başlayın :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Komut Referansı :material-console:](commands/index.md){ .md-button }
[GitHub'da Görüntüle :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Neden Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Buluta özgü Sigma__

    ---

    Bulut günlükleri için yerel [Sigma](https://github.com/SigmaHQ/sigma) tespiti — bugün AWS CloudTrail,
    Azure ve GCP planlanıyor. Korelasyon kuralları ve neredeyse tüm alan değiştiricileri desteklenir.

-   :material-timeline-clock:{ .lg .middle } __Hızlı adli zaman çizelgeleri__

    ---

    Binlerce gürültülü bulut API çağrısını, yalnızca ihtiyaç duyduğunuz olayları içeren tek bir,
    analiz edilmesi kolay **DFIR zaman çizelgesine** dönüştürün.

-   :material-flash:{ .lg .middle } __Rust ile inanılmaz hızlı__

    ---

    Bellek açısından güvenli, çok iş parçacıklı ve bağımsız. Windows, Linux ve macOS üzerinde `.json`
    ve sıkıştırılmış `.json.gz` günlüklerini tarar.

-   :material-chart-box:{ .lg .middle } __Saldırgan özetleri__

    ---

    Hızlı bir şekilde inceleme yapmak için API kullanımını ve saldırgan metriklerini — kaynak IP'ler,
    coğrafi konum, bölgeler, kullanıcı aracıları — özetleyin.

-   :material-shield-search:{ .lg .middle } __Davranış tespiti__

    ---

    **İmzalara dayanmadan** anormal etkinlikleri ortaya çıkarın, böylece yeni saldırıları kaçırmazsınız.

-   :material-export:{ .lg .middle } __Esnek çıktı__

    ---

    Tercih ettiğiniz araçta analiz için sonuçları **CSV, JSON ve JSONL** olarak kaydedin.

</div>

## Hızlı bağlantılar

<div class="grid cards" markdown>

-   __:material-book-open-variant: Yeni mi başladınız?__

    [Genel Bakış](overview/index.md) ile başlayın, ardından Suzaku'yu indirmek ve çalıştırmak için
    [Başlangıç](getting-started/index.md) bölümüne geçin.

-   __:material-console-line: CLI ile mi çalışıyorsunuz?__

    [Komut Listesine](commands/index.md) ve [Analiz](commands/analysis.md),
    [DFIR Özeti](commands/dfir-summary.md) ve [DFIR Zaman Çizelgesi](commands/dfir-timeline.md)
    komutları için komut başına referansa göz atın.

-   __:material-puzzle: Daha ileri mi gidiyorsunuz?__

    [Yerel Sigma Desteğini](rules/index.md), [Tamamlayıcı Projeleri](resources/companion-projects.md)
    ve nasıl [katkıda bulunacağınızı](resources/contributing.md) keşfedin.

</div>
