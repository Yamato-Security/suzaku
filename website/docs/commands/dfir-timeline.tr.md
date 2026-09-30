# DFIR Zaman Çizelgesi Komutları

## `aws-ct-timeline` komutu

`rules` klasöründeki Sigma kurallarına dayalı bir AWS CloudTrail DFIR zaman çizelgesi oluşturur.

## Komut kullanımı
```
Usage: suzaku aws-ct-timeline [OPTIONS] <--directory <DIR>|--file <FILE>>

General Options:
  -r, --rules <DIR/FILE>  Specify a custom rule directory or file (default: ./rules)
  -h, --help              Show the help menu

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -C, --clobber                    Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>    Add GeoIP (ASN, city, country) info to IP addresses
  -m, --min-level <LEVEL>          Minimum level for rules to load (default: informational)
  -o, --output <FILE>              Save the results to a file
  -t, --output-type <OUTPUT_TYPE>  Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL [default: 1]
  -R, --raw-output                 Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>    Number of threads to use (default: same as CPU cores)

Display Settings:
  -K, --no-color               Disable color output
  -N, --no-summary             Do not display results summary
  -T, --no-frequency-timeline  Disable event frequency timeline (terminal needs to support Unicode)
  -q, --quiet                  Quiet mode: do not display the launch banner
```

### `aws-ct-timeline` komut örnekleri

* Uyarıları ekrana yazdırma: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Sonuçları bir CSV dosyasına kaydetme: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Sonuçları CSV ve JSONL dosyalarına kaydetme: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### `aws-ct-timeline` çıktı profili

Suzaku, bilgileri `config/aws_profile.yaml` dosyasına göre çıktılar:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
ErrorCode: '.errorCode'
ErrorMessage: '.errorMessage'
EventSource: '.eventSource'
AWS-Region: '.awsRegion'
SrcIP: '.sourceIPAddress'
UserAgent: '.userAgent'
UserName: '.userIdentity.userName'
UserType: '.userIdentity.type'
UserAccountID: '.userIdentity.accountId'
UserARN: '.userIdentity.arn'
UserPrincipalID: '.userIdentity.principalId'
UserAccessKeyID: '.userIdentity.accessKeyId'
EventID: '.eventID'
Tags: 'sigma.tags'
RuleID: 'sigma.id'
```

* `.` ile başlayan herhangi bir alan değeri (ör: `.eventTime`) CloudTrail günlüğünden alınır.
* `sigma.` ile başlayan herhangi bir alan değeri (ör: `sigma.title`) Sigma kuralından alınır.
* Şu anda yalnızca dizeleri destekliyoruz ancak diğer alan değeri türlerini de desteklemeyi planlıyoruz.

> Not: Orijinal JSON verisini çıktılamak ve herhangi bir alan bilgisini kaybetmediğinizden emin olmak istiyorsanız, `aws-ct-timeline` komutuna sadece `-R, --raw-output` seçeneğini ekleyin.

### DuckDB çıktı şeması {#duckdb-output-schema}

CSV ve JSON çıktıları yukarıdaki profilin bir *görüntülemesidir*; DuckDB çıktısı ise bir *veri arayüzüdür*, bu nedenle tiplidir ve kendi kendini tanımlar. Farklar kasıtlıdır ve `aws-ct-timeline`, `azure-timeline`, `gws-timeline` ve `aws-ct-search` için geçerlidir:

| | CSV / JSON | DuckDB |
|---|---|---|
| Eksik değer | `-` (veya boş) | `NULL` |
| `Timestamp` | biçimlendirilmiş metin | `TIMESTAMP` |
| `Level` | metin | `suzaku_level` (önem derecesine göre sıralı bir `ENUM`) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (SQL'de tırnak gerektirmez) |
| `Tags` | ` ¦ ` ile birleştirilmiş tek bir dize | `Tactics`, `TechniqueIDs`, `OtherTags` (her biri `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | yalnızca `-G, --geo-ip` ile eklenir | her zaman bulunur (profilde `SrcIP` varsa), `-G` kullanılmadığında `NULL` |
| Yinelenen satırlar | korunur | tam yinelenenler kaldırılır, sayısı `suzaku_meta` içinde raporlanır |

Her dosya ayrıca tek satırlık bir `suzaku_meta` tablosu içerir; böylece dosyayı neyin ürettiği tahmin etmeden anlaşılabilir:

| Sütun | Anlamı |
|---|---|
| `schema_version` | Düzen sürümü. Diğer tabloları okumadan önce bunu kontrol edin. |
| `suzaku_version`, `command`, `command_line` | Hangi Suzaku, hangi alt komut, tam olarak hangi çağrı. |
| `generated_at` | Dosyanın yazıldığı zaman. |
| `timestamp_tz` | `Timestamp` sütununun ifade edildiği saat dilimi — `UTC` veya `-l, --localtime` ile yerel ofset. |
| `rules_version`, `rules_count` | Kural seti revizyonu (kurallar klasörü bir git checkout ise) ve yüklenen kural sayısı. |
| `geoip_enabled` | `-G, --geo-ip` çalıştırılıp çalıştırılmadığı. Tamamı `NULL` olan bir `SrcCountry` sütununu ("zenginleştirme kapalıydı"), zenginleştirilmiş bir dosyadaki `NULL` hücreden ("bu değer bir IP adresi değil") ayırt eder. |
| `scanned_files`, `scanned_events` | Çalıştırmanın kapsamı. |
| `output_rows`, `duplicate_rows_removed` | Yazılan satırlar ve yazma sırasında kaldırılan tam yinelenenler. |

Kural tabanlı zaman çizelgesi komutlarında bir `timeline` satırı **bir olay × bir kural eşleşmesidir**: birden fazla kurala uyan bir olay her eşleşme için bir satır üretir, bu nedenle `EventID` benzersiz *değildir*. `aws-ct-search` için eşleşen her olay bir satır üretir.

```sql
-- Critical and high alerts in a time range, with their ATT&CK techniques.
-- `Level` is an ENUM, so cast the literal to compare by severity rather than alphabetically.
SELECT Timestamp, RuleTitle, EventName, SrcIP, TechniqueIDs
FROM timeline
WHERE Level >= 'high'::suzaku_level
  AND Timestamp BETWEEN TIMESTAMP '2024-01-01' AND TIMESTAMP '2024-02-01'
  AND ErrorCode IS NULL          -- the call succeeded
ORDER BY Timestamp;

-- ATT&CK technique coverage, no string parsing required
SELECT technique, count(*) AS hits
FROM (SELECT unnest(TechniqueIDs) AS technique FROM timeline)
GROUP BY 1 ORDER BY hits DESC;
```

Veritabanı Suzaku çıkmadan önce checkpoint edilir, bu nedenle `.duckdb` dosyası eksiksizdir ve salt okunur olarak açılabilir (dosyayı komut çalışırken değil, bittikten sonra kopyalayın).
