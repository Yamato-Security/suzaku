# Özellikler

* Çapraz platform desteği: Windows, Linux, macOS.
* Bellek güvenli, hızlı ve bağımsız olması için Rust ile geliştirilmiştir.
* AWS CloudTrail, Azure/Entra ID/M365 ve Google Workspace Reports API günlüklerini yerel Sigma algılamasıyla analiz edin.
* `.json`, `.jsonl`, sıkıştırılmış `.json.gz`, Parquet ve desteklenen [AWS Management Console CloudTrail CSV](../getting-started/console-csv-input.md) dosyalarını çoklu iş parçacığıyla tarayın.
* Adli soruşturmalar ve olay müdahalesi için analiz edilmesi kolay tek bir zaman çizelgesi oluşturun.
* Okuması/oluşturması/düzenlemesi kolay YML tabanlı [Sigma](https://github.com/SigmaHQ/sigma) kurallarıyla yazılmış IoC imzaları için mükemmel yerel destek. ([expand](https://sigmahq.io/docs/basics/modifiers.html#expand) dışındaki korelasyon kuralları ve tüm alan değiştiricileri desteklenir.)
* İmzalara güvenmeden anormal etkinliği keşfetmek için tüm API kullanımının bir özetini, saldırgan hakkındaki metrikleri (kaynak IP adresleri, coğrafi konum, kullanılan bölgeler, kullanıcı aracıları vb.) oluşturun.
* Sonuçları CSV, JSON, JSONL ve DuckDB olarak kaydedin.
