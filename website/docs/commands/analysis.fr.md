# Commandes d'analyse

## Commande `aws-ct-metrics`

Utilisez cette commande pour créer des métriques sur les champs contenus dans les journaux AWS CloudTrail.
Par défaut, elle analyse le champ `eventName`.
Nous utilisons actuellement cette commande pour déterminer quels appels d'API sont les plus courants afin de prioriser la rédaction des règles de détection.

## Utilisation de la commande
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

### Exemples de la commande `aws-ct-metrics`

* Afficher à l'écran un tableau des appels d'API `eventName` : `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* Enregistrer dans un fichier CSV : `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
