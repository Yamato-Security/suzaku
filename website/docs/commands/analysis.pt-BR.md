# Comandos de Análise

## Comando `aws-ct-metrics`

Use este comando para criar métricas sobre campos dentro dos logs do AWS CloudTrail.
Por padrão, ele examinará o campo `eventName`.
Atualmente, estamos usando este comando para descobrir quais chamadas de API são as mais comuns, a fim de priorizar a escrita de regras de detecção.

## Uso do comando
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

### Exemplos do comando `aws-ct-metrics`

* Exibir uma tabela de chamadas de API do `eventName` na tela: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* Salvar em um arquivo CSV: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
