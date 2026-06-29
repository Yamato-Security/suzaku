# Comandos de análisis

## Comando `aws-ct-metrics`

Use este comando para crear métricas sobre los campos dentro de los registros de AWS CloudTrail.
De forma predeterminada, escaneará el campo `eventName`.
Actualmente usamos este comando para averiguar cuáles llamadas a la API son las más comunes con el fin de priorizar la escritura de reglas de detección.

## Uso del comando
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

### Ejemplos del comando `aws-ct-metrics`

* Mostrar en pantalla una tabla de llamadas a la API de `eventName`: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* Guardar en un archivo CSV: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
