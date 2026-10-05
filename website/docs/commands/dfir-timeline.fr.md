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

* Toute valeur de champ qui commence par `.` (ex : `.eventTime`) sera extraite du journal CloudTrail.
* Toute valeur de champ qui commence par `sigma.` (ex : `sigma.title`) sera extraite de la règle Sigma.
* Actuellement, nous ne prenons en charge que les chaînes de caractères, mais nous prévoyons de prendre en charge d'autres types de valeurs de champ.

> Note : Si vous souhaitez afficher les données JSON d'origine et vous assurer de ne perdre aucune information de champ, ajoutez simplement l'option `-R, --raw-output` à la commande `aws-ct-timeline`.

### Schéma de sortie DuckDB {#duckdb-output-schema}

Les sorties CSV et JSON sont un *rendu* du profil ci-dessus ; la sortie DuckDB est une *interface de données*, elle est donc typée et auto-descriptive. Ces différences sont voulues et s'appliquent à `aws-ct-timeline`, `azure-timeline`, `gws-timeline` et `aws-ct-search` :

| | CSV / JSON | DuckDB |
|---|---|---|
| Valeur manquante | `-` (ou vide) | `NULL` |
| `Timestamp` | texte mis en forme | `TIMESTAMP` |
| `Level` | texte | `suzaku_level` (un `ENUM` ordonné par sévérité) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (sans guillemets en SQL) |
| `Tags` | une seule chaîne jointe par ` ¦ ` | `Tactics`, `TechniqueIDs`, `OtherTags` (chacune de type `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | ajoutées uniquement avec `-G, --geo-ip` | toujours présentes (si le profil contient `SrcIP`), `NULL` sans `-G` |
| Lignes en double | conservées | doublons exacts supprimés, nombre indiqué dans `suzaku_meta` |

Chaque fichier contient aussi une table `suzaku_meta` d'une ligne, afin de savoir sans deviner ce qui l'a produit :

| Colonne | Signification |
|---|---|
| `schema_version` | Version de la structure. Vérifiez-la avant de lire les autres tables. |
| `suzaku_version`, `command`, `command_line` | Quelle version de Suzaku, quelle sous-commande, quelle invocation exacte. |
| `generated_at` | Date d'écriture du fichier. |
| `timestamp_tz` | Le fuseau de la colonne `Timestamp` — `UTC`, ou le décalage local avec `-l, --localtime`. |
| `rules_version`, `rules_count` | Révision du jeu de règles (si le dossier des règles est un checkout git) et nombre de règles chargées. |
| `geoip_enabled` | Indique si `-G, --geo-ip` a été exécuté. Permet de distinguer un `SrcCountry` entièrement `NULL` (« enrichissement désactivé ») d'une cellule `NULL` dans un fichier enrichi (« cette valeur n'est pas une adresse IP »). |
| `scanned_files`, `scanned_events` | Couverture de l'exécution. |
| `output_rows`, `duplicate_rows_removed` | Lignes écrites et doublons exacts supprimés à l'écriture. |

Pour les commandes de chronologie basées sur des règles, une ligne de `timeline` correspond à **un événement × une correspondance de règle** : un événement qui correspond à plusieurs règles produit une ligne par correspondance, donc `EventID` n'est *pas* unique. Avec `aws-ct-search`, chaque événement correspondant produit une ligne.

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

La base de données est checkpointée avant la fin de Suzaku ; le fichier `.duckdb` est donc complet et peut être ouvert en lecture seule (copiez-le une fois la commande terminée, pas pendant son exécution).
