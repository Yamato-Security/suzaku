# Analysebefehle

## `aws-ct-metrics`-Befehl

Verwenden Sie diesen Befehl, um Metriken zu Feldern innerhalb von AWS CloudTrail-Logs zu erstellen.
Standardmäßig wird das Feld `eventName` durchsucht.
Wir verwenden diesen Befehl derzeit, um herauszufinden, welche API-Aufrufe am häufigsten vorkommen, um das Schreiben von Erkennungsregeln zu priorisieren.

## Befehlsverwendung
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

### Beispiele für den `aws-ct-metrics`-Befehl

* Eine Tabelle der `eventName`-API-Aufrufe auf dem Bildschirm ausgeben: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* In einer CSV-Datei speichern: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
