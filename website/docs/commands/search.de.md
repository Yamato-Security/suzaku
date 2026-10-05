# Suchbefehle

## `aws-ct-search`-Befehl

Verwenden Sie diesen Befehl, um die für Sie relevanten Ereignisse aus AWS-CloudTrail-Logs zu extrahieren, ohne eine Sigma-Regel zu schreiben.
Treffer werden im gleichen Format wie bei `aws-ct-timeline` ausgegeben oder gespeichert, sodass Sie direkt mit den bekannten Zeitleistenspalten weiterarbeiten können.

Es gibt drei Möglichkeiten, die Ereignisse einzugrenzen, und sie lassen sich kombinieren:

| Option | Was verglichen wird | Mehrere Werte |
|---|---|---|
| `-F, --filter FIELD:VALUE` | Der Wert eines Feldes, **exakte Übereinstimmung, Groß-/Kleinschreibung beachtet** | `-F` wiederholen; **alle** Filter müssen passen (UND) |
| `-k, --keyword KEYWORD` | Eine Teilzeichenkette irgendwo im JSON des Ereignisses, ohne Beachtung der Groß-/Kleinschreibung (mit `-c` mit Beachtung) | `-k` wiederholen; **irgendein** Schlüsselwort genügt (ODER) |
| `-r, --regex REGEX` | Ein regulärer Ausdruck irgendwo im JSON des Ereignisses | Ein Muster (Alternativen mit dem nicht maskierten Pipe-Zeichen <code>&#124;</code>) |

Jedes Ereignis wird in dieser Reihenfolge geprüft, und nur Ereignisse, die alle Prüfungen bestehen, werden ausgegeben: Zeitraum (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Verwenden Sie `-F`, wenn Sie wissen, in welchem Feld der Wert steht (zum Beispiel `eventName:ConsoleLogin`), und `-k` oder `-r`, wenn Sie nur die Zeichenkette kennen, aber nicht, wo sie vorkommt.

### So funktioniert `-F, --filter`

* Das Format ist `FIELD:VALUE`. Das Argument wird am **ersten** `:` geteilt, sodass Werte mit Doppelpunkten wie ARNs unverändert funktionieren.
* Verschachtelte Felder werden in Punktnotation angegeben (`userIdentity.type`). Ein führender `.` ist optional, daher funktioniert auch `.userIdentity.arn`.
* Anführungszeichen um den Wert (`"..."` oder `'...'`) werden entfernt.
* Zahlen und boolesche Werte werden als Zeichenketten verglichen, daher funktionieren `-F readOnly:false` und `-F responseElements.user.userId:12345`.
* Ein Ereignis ohne das Feld passt nicht.
* Elemente in Arrays können nicht adressiert werden (es gibt keine Syntax wie `items.0.name`).
* Ein Filter ohne `:` wird vor Beginn des Scans abgelehnt: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Hinweis: In `aws-ct-metrics` bedeutet `-F` etwas anderes, nämlich `--field-name`, und akzeptiert nur einen Feldnamen. In `aws-ct-search` ist es `--filter` und erwartet immer `FIELD:VALUE`.

## Befehlsverwendung
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

### Beispiele für den `aws-ct-search`-Befehl

Nach Feld filtern (`-F`):

* Konsolenanmeldungen finden: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* API-Aufrufe des Root-Benutzers finden (verschachteltes Feld): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Alle Aktionen eines bestimmten IAM-Benutzers finden (der Wert enthält Doppelpunkte): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Abgelehnte Aufrufe finden: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* Schreibende (nicht schreibgeschützte) IAM-Aufrufe in `us-east-1` finden (mehrere Filter werden UND-verknüpft): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Zugriffe auf einen bestimmten S3-Bucket finden (API-spezifisches Feld): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Nach Schlüsselwort (`-k`) und regulärem Ausdruck (`-r`) suchen:

* Ereignisse finden, die irgendwo eine IP-Adresse enthalten: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* Ereignisse finden, die einen von zwei Benutzern enthalten (mehrere Schlüsselwörter werden ODER-verknüpft): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Suche nach Schlüsselwort mit Beachtung der Groß-/Kleinschreibung: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* Versuche finden, CloudTrail zu deaktivieren: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Bedingungen kombinieren und Ergebnisse speichern:

* Schlüsselwortsuche auf eine Ereignisquelle beschränken: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Fehlgeschlagene Konsolenanmeldungen der letzten 7 Tage: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Ergebnisse mit GeoIP-Informationen als CSV und DuckDB speichern: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Das ursprüngliche JSON der gefundenen Ereignisse speichern: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search`-Ausgabe

Die Ausgabespalten sind dieselben wie bei [`aws-ct-timeline`](dfir-timeline.md), und die DuckDB-Ausgabe folgt demselben [DuckDB-Ausgabeschema](dfir-timeline.md#duckdb-output-schema).
Am Ende wird die Anzahl der durchsuchten und der gefundenen Ereignisse ausgegeben:

```
Total events scanned: 209
Matching events: 1
```
