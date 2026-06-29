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

Suzaku gibt Informationen basierend auf der Datei `config/default_profile.yaml` aus:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
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
RuleID: 'sigma.id'
```

* Jeder Feldwert, der mit `.` beginnt (z. B. `.eventTime`), wird aus dem CloudTrail-Log übernommen.
* Jeder Feldwert, der mit `sigma.` beginnt (z. B. `sigma.title`), wird aus der Sigma-Regel übernommen.
* Derzeit unterstützen wir nur Zeichenketten, planen aber, weitere Typen von Feldwerten zu unterstützen.

> Hinweis: Wenn Sie die ursprünglichen JSON-Daten ausgeben und sicherstellen möchten, dass Sie keine Feldinformationen verlieren, fügen Sie einfach die Option `-R, --raw-output` zum Befehl `aws-ct-timeline` hinzu.
