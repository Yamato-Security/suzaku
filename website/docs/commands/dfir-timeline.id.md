# Perintah DFIR Timeline

## Perintah `aws-ct-timeline`

Membuat timeline DFIR AWS CloudTrail berdasarkan aturan Sigma di folder `rules`.

## Penggunaan perintah
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

### Contoh perintah `aws-ct-timeline`

* Menampilkan alert ke layar: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Menyimpan hasil ke file CSV: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Menyimpan hasil ke file CSV dan JSONL: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### Profil output `aws-ct-timeline`

Suzaku akan menampilkan informasi berdasarkan file `config/aws_profile.yaml`:
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

* Setiap nilai field yang diawali dengan `.` (ex: `.eventTime`) akan diambil dari log CloudTrail.
* Setiap nilai field yang diawali dengan `sigma.` (ex: `sigma.title`) akan diambil dari aturan Sigma.
* Saat ini kami hanya mendukung string namun berencana untuk mendukung tipe nilai field lainnya.

> Catatan: Jika Anda ingin menampilkan data JSON asli dan memastikan Anda tidak kehilangan informasi field apa pun, cukup tambahkan opsi `-R, --raw-output` ke perintah `aws-ct-timeline`.

### Skema output DuckDB {#duckdb-output-schema}

Output CSV dan JSON adalah *tampilan* dari profil di atas; output DuckDB adalah *antarmuka data*, sehingga bertipe dan mendeskripsikan dirinya sendiri. Perbedaan ini disengaja dan berlaku untuk `aws-ct-timeline`, `azure-timeline`, `gws-timeline`, dan `aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| Nilai yang tidak ada | `-` (atau kosong) | `NULL` |
| `Timestamp` | teks yang diformat | `TIMESTAMP` |
| `Level` | teks | `suzaku_level` (`ENUM` yang diurutkan berdasarkan tingkat keparahan) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (tanpa tanda kutip di SQL) |
| `Tags` | satu string yang digabung dengan ` ¦ ` | `Tactics`, `TechniqueIDs`, `OtherTags` (masing-masing `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | hanya ditambahkan dengan `-G, --geo-ip` | selalu ada (jika profil memiliki `SrcIP`), `NULL` jika `-G` tidak digunakan |
| Baris duplikat | dipertahankan | duplikat persis dihapus, jumlahnya dicatat di `suzaku_meta` |

Setiap file juga memiliki tabel `suzaku_meta` satu baris, sehingga pembaca dapat mengetahui apa yang menghasilkannya tanpa menebak:

| Kolom | Arti |
|---|---|
| `schema_version` | Versi tata letak. Periksa ini sebelum membaca tabel lainnya. |
| `suzaku_version`, `command`, `command_line` | Versi Suzaku, subperintah, dan pemanggilan persis yang digunakan. |
| `generated_at` | Waktu file ditulis. |
| `timestamp_tz` | Zona waktu kolom `Timestamp` — `UTC`, atau offset lokal dengan `-l, --localtime`. |
| `rules_version`, `rules_count` | Revisi ruleset (jika folder rules adalah checkout git) dan jumlah aturan yang dimuat. |
| `geoip_enabled` | Apakah `-G, --geo-ip` dijalankan. Membedakan `SrcCountry` yang seluruhnya `NULL` ("pengayaan dimatikan") dari sel `NULL` pada file yang diperkaya ("nilai ini bukan alamat IP"). |
| `scanned_files`, `scanned_events` | Cakupan eksekusi. |
| `output_rows`, `duplicate_rows_removed` | Baris yang ditulis, dan duplikat persis yang dihapus saat penulisan. |

Untuk perintah linimasa berbasis aturan, satu baris `timeline` adalah **satu peristiwa × satu kecocokan aturan**: peristiwa yang cocok dengan beberapa aturan menghasilkan satu baris per kecocokan, sehingga `EventID` *tidak* unik. Pada `aws-ct-search`, setiap peristiwa yang cocok menghasilkan satu baris.

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

Suzaku melakukan checkpoint basis data sebelum keluar, sehingga file `.duckdb` lengkap dan dapat dibuka dalam mode hanya-baca (salin setelah perintah selesai, bukan saat sedang berjalan).
