# Arama Komutları

## `aws-ct-search` komutu

Bu komutu, bir Sigma kuralı yazmadan AWS CloudTrail günlüklerinden ilgilendiğiniz olayları çıkarmak için kullanın.
Eşleşen olaylar `aws-ct-timeline` ile aynı biçimde yazdırılır veya kaydedilir; böylece aramadan doğrudan bildiğiniz zaman çizelgesi sütunlarına geçebilirsiniz.

Olayları daraltmanın üç yolu vardır ve bunlar birlikte kullanılabilir:

| Seçenek | Neyle eşleşir | Birden fazla değer |
|---|---|---|
| `-F, --filter FIELD:VALUE` | Tek bir alanın değeri, **tam eşleşme, büyük/küçük harfe duyarlı** | `-F` tekrarlanır; **tüm** filtreler eşleşmelidir (VE) |
| `-k, --keyword KEYWORD` | Olay JSON'unun herhangi bir yerindeki alt dize, büyük/küçük harfe duyarsız (`-c` ile duyarlı) | `-k` tekrarlanır; **herhangi bir** anahtar kelime yeterlidir (VEYA) |
| `-r, --regex REGEX` | Olay JSON'unun herhangi bir yerindeki düzenli ifade | Tek bir desen (alternatifler için kaçış uygulanmamış dikey çizgi karakterini <code>&#124;</code> kullanın) |

Her olay bu sırayla kontrol edilir ve yalnızca tüm kontrolleri geçen olaylar çıktılanır: zaman aralığı (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Değerin hangi alanda olduğunu biliyorsanız (örneğin `eventName:ConsoleLogin`) `-F`, yalnızca dizeyi bilip nerede geçtiğini bilmiyorsanız `-k` veya `-r` kullanın.

### `-F, --filter` nasıl çalışır

* Biçim `FIELD:VALUE` şeklindedir. Argüman **ilk** `:` işaretinden bölünür, bu nedenle ARN gibi iki nokta içeren değerler olduğu gibi çalışır.
* İç içe alanlar nokta gösterimiyle yazılır (`userIdentity.type`). Baştaki `.` isteğe bağlıdır, bu yüzden `.userIdentity.arn` da çalışır.
* Değeri çevreleyen tırnaklar (`"..."` veya `'...'`) kaldırılır.
* Sayılar ve boolean değerler dize olarak karşılaştırılır, bu nedenle `-F readOnly:false` ve `-F responseElements.user.userId:12345` çalışır.
* Alana sahip olmayan bir olay eşleşmez.
* Dizi içindeki öğeler adreslenemez (`items.0.name` gibi bir sözdizimi yoktur).
* `:` içermeyen bir filtre tarama başlamadan reddedilir: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Not: `aws-ct-metrics` içinde `-F` farklı bir anlama gelir; orada `--field-name`'dir ve yalnızca bir alan adı alır. `aws-ct-search` içinde `--filter`'dır ve her zaman `FIELD:VALUE` alır.

## Komut kullanımı
```
Usage: suzaku aws-ct-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory of multiple gz/json/parquet files
  -f, --file <FILE>      File path to one gz/json/parquet file

Filtering:
  -F, --filter <FILTER...>     Filter by specific field(s)
  -c, --preserve-case          Case-sensitive keyword search
  -k, --keyword <KEYWORD...>   Search by keyword(s)
  -r, --regex <REGEX>          Search by regular expression
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
      --file-date-to <DATE>    Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")

Output:
  -C, --clobber                   Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>   Add GeoIP (ASN, city, country) info to IP addresses
  -o, --output <FILE>             Save the results to a file
  -t, --output-type <FORMAT,...>  Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [default: csv] [possible values: csv, json, jsonl, duckdb]
      --raw-output                Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>   Number of threads to use (default: same as CPU cores)

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### `aws-ct-search` komut örnekleri

Alana göre filtreleme (`-F`):

* Konsol oturum açmalarını bulun: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Root kullanıcısının API çağrılarını bulun (iç içe alan): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Belirli bir IAM kullanıcısının yaptığı her şeyi bulun (değer iki nokta içerir): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Reddedilen çağrıları bulun: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* `us-east-1` içindeki salt okunur olmayan IAM çağrılarını bulun (birden fazla filtre VE ile birleşir): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Belirli bir S3 bucket'ına erişimi bulun (API'ye özgü alan): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Anahtar kelime (`-k`) ve düzenli ifade (`-r`) ile arama:

* Herhangi bir yerinde bir IP adresi geçen olayları bulun: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* İki kullanıcıdan birinin geçtiği olayları bulun (birden fazla anahtar kelime VEYA ile birleşir): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Büyük/küçük harfe duyarlı anahtar kelime araması: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* CloudTrail'i devre dışı bırakma girişimlerini bulun: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Koşulları birleştirme ve sonuçları kaydetme:

* Anahtar kelime aramasını tek bir olay kaynağıyla sınırlayın: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Son 7 gündeki başarısız konsol oturum açmaları: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Sonuçları GeoIP bilgisiyle CSV ve DuckDB olarak kaydedin: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Eşleşen olayların orijinal JSON'unu kaydedin: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search` çıktısı

Çıktı sütunları [`aws-ct-timeline`](dfir-timeline.md) ile aynıdır ve DuckDB çıktısı da aynı [DuckDB çıktı şemasını](dfir-timeline.md#duckdb-output-schema) izler.
Sonunda taranan ve eşleşen olay sayısı yazdırılır:

```
Total events scanned: 209
Matching events: 1
```
