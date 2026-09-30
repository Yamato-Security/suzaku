# Perintah Pencarian

## Perintah `aws-ct-search`

Gunakan perintah ini untuk mengambil peristiwa yang Anda perlukan dari log AWS CloudTrail tanpa menulis aturan Sigma.
Peristiwa yang cocok ditampilkan atau disimpan dalam format yang sama dengan `aws-ct-timeline`, sehingga Anda bisa langsung beralih dari pencarian ke kolom linimasa yang sudah Anda kenal.

Ada tiga cara untuk mempersempit peristiwa, dan ketiganya dapat digabungkan:

| Opsi | Yang dicocokkan | Beberapa nilai |
|---|---|---|
| `-F, --filter FIELD:VALUE` | Nilai satu field, **kecocokan persis, peka huruf besar/kecil** | Ulangi `-F`; **semua** filter harus cocok (AND) |
| `-k, --keyword KEYWORD` | Substring di mana saja dalam JSON peristiwa, tidak peka huruf besar/kecil (peka dengan `-c`) | Ulangi `-k`; cukup cocok dengan **salah satu** kata kunci (OR) |
| `-r, --regex REGEX` | Ekspresi reguler di mana saja dalam JSON peristiwa | Satu pola (gunakan karakter pipa tanpa escape <code>&#124;</code> untuk alternatif) |

Setiap peristiwa diperiksa dengan urutan ini, dan hanya peristiwa yang lolos semua pemeriksaan yang ditampilkan: rentang waktu (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Gunakan `-F` bila Anda tahu field mana yang berisi nilainya (misalnya `eventName:ConsoleLogin`), dan `-k` atau `-r` bila Anda hanya tahu string-nya tetapi tidak tahu di mana ia muncul.

### Cara kerja `-F, --filter`

* Formatnya adalah `FIELD:VALUE`. Argumen dipisah pada `:` **pertama**, sehingga nilai yang mengandung titik dua, seperti ARN, dapat digunakan apa adanya.
* Field bersarang ditulis dengan notasi titik (`userIdentity.type`). Tanda `.` di awal bersifat opsional, jadi `.userIdentity.arn` juga dapat digunakan.
* Tanda kutip di sekitar nilai (`"..."` atau `'...'`) dihapus.
* Angka dan boolean dibandingkan sebagai string, sehingga `-F readOnly:false` dan `-F responseElements.user.userId:12345` dapat digunakan.
* Peristiwa yang tidak memiliki field tersebut tidak cocok.
* Elemen di dalam array tidak dapat dirujuk (tidak ada sintaks `items.0.name`).
* Filter tanpa `:` ditolak sebelum pemindaian dimulai: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Catatan: di `aws-ct-metrics`, `-F` memiliki arti lain, yaitu `--field-name`, dan hanya menerima nama field. Di `aws-ct-search`, `-F` adalah `--filter` dan selalu menerima `FIELD:VALUE`.

## Penggunaan perintah
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

### Contoh perintah `aws-ct-search`

Memfilter berdasarkan field (`-F`):

* Menemukan login konsol: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Menemukan panggilan API oleh pengguna root (field bersarang): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Menemukan semua yang dilakukan pengguna IAM tertentu (nilai mengandung titik dua): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Menemukan panggilan yang ditolak: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* Menemukan panggilan IAM yang menulis (bukan hanya-baca) di `us-east-1` (beberapa filter digabung dengan AND): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Menemukan akses ke bucket S3 tertentu (field khusus API): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Mencari berdasarkan kata kunci (`-k`) dan ekspresi reguler (`-r`):

* Menemukan peristiwa yang menyebut alamat IP di mana saja: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* Menemukan peristiwa yang menyebut salah satu dari dua pengguna (beberapa kata kunci digabung dengan OR): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Pencarian kata kunci yang peka huruf besar/kecil: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* Menemukan upaya menonaktifkan CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Menggabungkan kondisi dan menyimpan hasil:

* Pencarian kata kunci yang dibatasi pada satu sumber peristiwa: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Login konsol yang gagal dalam 7 hari terakhir: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Menyimpan hasil ke CSV dan DuckDB dengan informasi GeoIP: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Menyimpan JSON asli dari peristiwa yang cocok: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### Output `aws-ct-search`

Kolom output sama dengan [`aws-ct-timeline`](dfir-timeline.md), dan output DuckDB mengikuti [skema output DuckDB](dfir-timeline.md#duckdb-output-schema) yang sama.
Jumlah peristiwa yang dipindai dan yang cocok ditampilkan di akhir:

```
Total events scanned: 209
Matching events: 1
```
