# DFIR-Timeline-Befehle

## `aws-ct-timeline`-Befehl

Erstellt eine AWS-CloudTrail-DFIR-Timeline basierend auf Sigma-Regeln im `rules`-Ordner.

## Befehlsverwendung
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

### `aws-ct-timeline`-Befehlsbeispiele

* Warnungen auf dem Bildschirm ausgeben: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Ergebnisse in einer CSV-Datei speichern: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Ergebnisse in CSV- und JSONL-Dateien speichern: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### `aws-ct-timeline`-Ausgabeprofil

Suzaku gibt Informationen basierend auf der Datei `config/aws_profile.yaml` aus:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
Level: 'sigma.level'
EventSource: '.eventSource'
EventName: '.eventName'
ErrorCode: '.errorCode'
UserName: '.userIdentity.userName'
UserType: '.userIdentity.type'
SrcIP: '.sourceIPAddress'
AWS-Region: '.awsRegion'
UserAccountID: '.userIdentity.accountId'
UserARN: '.userIdentity.arn'
UserAccessKeyID: '.userIdentity.accessKeyId'
UserPrincipalID: '.userIdentity.principalId'
UserAgent: '.userAgent'
ErrorMessage: '.errorMessage'
EventID: '.eventID'
RuleAuthor: 'sigma.author'
Tags: 'sigma.tags'
RuleID: 'sigma.id'
```

* Jeder Feldwert, der mit `.` beginnt (z. B. `.eventTime`), wird aus dem CloudTrail-Log übernommen.
* Jeder Feldwert, der mit `sigma.` beginnt (z. B. `sigma.title`), wird aus der Sigma-Regel übernommen.
* Derzeit unterstützen wir nur Zeichenketten, planen aber, weitere Typen von Feldwerten zu unterstützen.

> Hinweis: Wenn Sie die ursprünglichen JSON-Daten ausgeben und sicherstellen möchten, dass Sie keine Feldinformationen verlieren, fügen Sie einfach die Option `-R, --raw-output` zum Befehl `aws-ct-timeline` hinzu.

### DuckDB-Ausgabeschema {#duckdb-output-schema}

Die CSV- und JSON-Ausgaben sind eine *Darstellung* des obigen Profils; die DuckDB-Ausgabe ist dagegen eine *Datenschnittstelle* und daher typisiert und selbstbeschreibend. Die Unterschiede sind beabsichtigt und gelten für `aws-ct-timeline`, `azure-timeline`, `gws-timeline` und `aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| Fehlender Wert | `-` (oder leer) | `NULL` |
| `Timestamp` | formatierter Text | `TIMESTAMP` |
| `Level` | Text | `suzaku_level` (ein nach Schweregrad sortiertes `ENUM`) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (in SQL ohne Anführungszeichen nutzbar) |
| `Tags` | eine mit ` ¦ ` verbundene Zeichenkette | `Tactics`, `TechniqueIDs`, `OtherTags` (jeweils `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | nur mit `-G, --geo-ip` hinzugefügt | immer vorhanden (wenn das Profil `SrcIP` enthält), `NULL` ohne `-G` |
| Doppelte Zeilen | bleiben erhalten | exakte Duplikate werden entfernt, Anzahl in `suzaku_meta` |

Jede Datei enthält außerdem eine einzeilige Tabelle `suzaku_meta`, sodass ohne Raten erkennbar ist, was die Datei erzeugt hat:

| Spalte | Bedeutung |
|---|---|
| `schema_version` | Layout-Version. Prüfen Sie diese, bevor Sie die anderen Tabellen lesen. |
| `suzaku_version`, `command`, `command_line` | Welche Suzaku-Version, welcher Unterbefehl, welcher exakte Aufruf. |
| `generated_at` | Zeitpunkt, zu dem die Datei geschrieben wurde. |
| `timestamp_tz` | Die Zeitzone der Spalte `Timestamp` – `UTC` oder der lokale Offset bei `-l, --localtime`. |
| `rules_version`, `rules_count` | Revision des Regelsatzes (wenn der Regelordner ein Git-Checkout ist) und Anzahl der geladenen Regeln. |
| `geoip_enabled` | Ob `-G, --geo-ip` ausgeführt wurde. Unterscheidet ein durchgehend `NULL`-wertiges `SrcCountry` („Anreicherung war aus“) von einer `NULL`-Zelle in einer angereicherten Datei („dieser Wert ist keine IP-Adresse“). |
| `scanned_files`, `scanned_events` | Abdeckung des Laufs. |
| `output_rows`, `duplicate_rows_removed` | Geschriebene Zeilen und beim Schreiben entfernte exakte Duplikate. |

Bei den regelbasierten Timeline-Befehlen ist eine `timeline`-Zeile **ein Ereignis × ein Regeltreffer**: Ein Ereignis, das mehrere Regeln trifft, erzeugt pro Treffer eine Zeile, daher ist `EventID` *nicht* eindeutig. Bei `aws-ct-search` erzeugt jedes passende Ereignis eine Zeile.

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

Die Datenbank wird vor dem Beenden von Suzaku gecheckpointet, sodass die `.duckdb`-Datei vollständig ist und schreibgeschützt geöffnet werden kann (kopieren Sie sie nach Abschluss des Befehls, nicht während er läuft).
