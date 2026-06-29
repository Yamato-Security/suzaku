# Perintah Analisis

## Perintah `aws-ct-metrics`

Gunakan perintah ini untuk membuat metrik pada field di dalam log AWS CloudTrail.
Secara default, perintah ini akan memindai field `eventName`.
Saat ini kami menggunakan perintah ini untuk mencari tahu panggilan API mana yang paling umum agar dapat memprioritaskan penulisan aturan deteksi.

## Penggunaan perintah
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -F, --field-name <FIELD_NAME>  The field to generate metrics for [default: eventName]
  -o, --output <FILE>            Output CSV

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
```

### Contoh perintah `aws-ct-metrics`

* Menampilkan tabel panggilan API `eventName` ke layar: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* Menyimpan ke file CSV: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
