# Commandes de chronologie DFIR

## Commande `aws-ct-timeline`

Crée une chronologie DFIR AWS CloudTrail basée sur les règles Sigma du dossier `rules`.

## Utilisation de la commande
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

### Exemples de la commande `aws-ct-timeline`

* Afficher les alertes à l'écran : `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Enregistrer les résultats dans un fichier CSV : `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Enregistrer les résultats dans des fichiers CSV et JSONL : `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### Profil de sortie de `aws-ct-timeline`

Suzaku affichera les informations en fonction du fichier `config/aws_profile.yaml` :
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

* Toute valeur de champ qui commence par `.` (ex : `.eventTime`) sera extraite du journal CloudTrail.
* Toute valeur de champ qui commence par `sigma.` (ex : `sigma.title`) sera extraite de la règle Sigma.
* Actuellement, nous ne prenons en charge que les chaînes de caractères, mais nous prévoyons de prendre en charge d'autres types de valeurs de champ.

> Note : Si vous souhaitez afficher les données JSON d'origine et vous assurer de ne perdre aucune information de champ, ajoutez simplement l'option `-R, --raw-output` à la commande `aws-ct-timeline`.
