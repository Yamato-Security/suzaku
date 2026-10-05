# Commandes de recherche

## Commande `aws-ct-search`

Utilisez cette commande pour extraire des journaux AWS CloudTrail les événements qui vous intéressent, sans écrire de règle Sigma.
Les événements trouvés sont affichés ou enregistrés dans le même format que `aws-ct-timeline`, ce qui permet de passer directement d'une recherche aux colonnes de chronologie que vous connaissez déjà.

Il existe trois façons de restreindre les événements, et elles peuvent être combinées :

| Option | Ce qui est comparé | Plusieurs valeurs |
|---|---|---|
| `-F, --filter FIELD:VALUE` | La valeur d'un champ, **correspondance exacte, sensible à la casse** | Répéter `-F` ; **tous** les filtres doivent correspondre (ET) |
| `-k, --keyword KEYWORD` | Une sous-chaîne n'importe où dans le JSON de l'événement, insensible à la casse (sensible avec `-c`) | Répéter `-k` ; **n'importe quel** mot-clé suffit (OU) |
| `-r, --regex REGEX` | Une expression régulière n'importe où dans le JSON de l'événement | Un seul motif (utilisez le caractère barre verticale non échappé <code>&#124;</code> pour les alternatives) |

Chaque événement est vérifié dans cet ordre, et seuls les événements qui passent toutes les vérifications sont affichés : plage horaire (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Utilisez `-F` lorsque vous savez quel champ contient la valeur (par exemple `eventName:ConsoleLogin`), et `-k` ou `-r` lorsque vous ne connaissez que la chaîne et pas l'endroit où elle apparaît.

### Fonctionnement de `-F, --filter`

* Le format est `FIELD:VALUE`. L'argument est découpé au **premier** `:`, donc les valeurs contenant des deux-points, comme les ARN, fonctionnent telles quelles.
* Les champs imbriqués s'écrivent en notation pointée (`userIdentity.type`). Le `.` initial est facultatif, donc `.userIdentity.arn` fonctionne aussi.
* Les guillemets autour de la valeur (`"..."` ou `'...'`) sont supprimés.
* Les nombres et les booléens sont comparés comme des chaînes, donc `-F readOnly:false` et `-F responseElements.user.userId:12345` fonctionnent.
* Un événement qui ne possède pas le champ ne correspond pas.
* Les éléments d'un tableau ne peuvent pas être ciblés (il n'existe pas de syntaxe `items.0.name`).
* Un filtre sans `:` est rejeté avant le début de l'analyse : `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Note : dans `aws-ct-metrics`, `-F` signifie autre chose : c'est `--field-name` et il n'accepte qu'un nom de champ. Dans `aws-ct-search`, c'est `--filter` et il prend toujours `FIELD:VALUE`.

## Utilisation de la commande
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

### Exemples de la commande `aws-ct-search`

Filtrer par champ (`-F`) :

* Trouver les connexions à la console: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Trouver les appels d'API effectués par l'utilisateur root (champ imbriqué): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Trouver toutes les actions d'un utilisateur IAM précis (la valeur contient des deux-points): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Trouver les appels refusés: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* Trouver les appels IAM en écriture (non en lecture seule) dans `us-east-1` (plusieurs filtres sont combinés par ET): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Trouver les accès à un bucket S3 précis (champ propre à l'API): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Rechercher par mot-clé (`-k`) et expression régulière (`-r`) :

* Trouver les événements qui mentionnent une adresse IP n'importe où: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* Trouver les événements qui mentionnent l'un de deux utilisateurs (plusieurs mots-clés sont combinés par OU): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Recherche par mot-clé sensible à la casse: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* Trouver les tentatives de désactivation de CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Combiner des conditions et enregistrer les résultats :

* Recherche par mot-clé limitée à une source d'événements: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Connexions à la console échouées au cours des 7 derniers jours: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Enregistrer les résultats en CSV et DuckDB avec les informations GeoIP: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Enregistrer le JSON d'origine des événements trouvés: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### Sortie de `aws-ct-search`

Les colonnes de sortie sont les mêmes que pour [`aws-ct-timeline`](dfir-timeline.md), et la sortie DuckDB suit le même [schéma de sortie DuckDB](dfir-timeline.md#duckdb-output-schema).
Le nombre d'événements analysés et trouvés est affiché à la fin :

```
Total events scanned: 209
Matching events: 1
```
