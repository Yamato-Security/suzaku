# Comandos de cronología DFIR

## Comando `aws-ct-timeline`

Crea una cronología DFIR de AWS CloudTrail basada en las reglas Sigma de la carpeta `rules`.

## Uso del comando
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

### Ejemplos del comando `aws-ct-timeline`

* Mostrar alertas en pantalla: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Guardar resultados en un archivo CSV: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Guardar resultados en archivos CSV y JSONL: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### Perfil de salida de `aws-ct-timeline`

Suzaku mostrará la información basándose en el archivo `config/aws_profile.yaml`:
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

* Cualquier valor de campo que comience con `.` (ej: `.eventTime`) se tomará del registro de CloudTrail.
* Cualquier valor de campo que comience con `sigma.` (ej: `sigma.title`) se tomará de la regla Sigma.
* Actualmente solo admitimos cadenas de texto, pero planeamos admitir otros tipos de valores de campo.

> Nota: Si desea mostrar los datos JSON originales y asegurarse de no perder ninguna información de campo, simplemente añada la opción `-R, --raw-output` al comando `aws-ct-timeline`.

### Esquema de salida de DuckDB {#duckdb-output-schema}

Las salidas CSV y JSON son una *representación* del perfil anterior; la salida de DuckDB es una *interfaz de datos*, por lo que es tipada y autodescriptiva. Las diferencias son intencionadas y se aplican a `aws-ct-timeline`, `azure-timeline`, `gws-timeline` y `aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| Valor ausente | `-` (o vacío) | `NULL` |
| `Timestamp` | texto formateado | `TIMESTAMP` |
| `Level` | texto | `suzaku_level` (un `ENUM` ordenado por severidad) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (sin comillas en SQL) |
| `Tags` | una sola cadena unida con ` ¦ ` | `Tactics`, `TechniqueIDs`, `OtherTags` (cada una `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | se añaden solo con `-G, --geo-ip` | siempre presentes (si el perfil tiene `SrcIP`), `NULL` cuando no se usó `-G` |
| Filas duplicadas | se conservan | se eliminan los duplicados exactos y su número se registra en `suzaku_meta` |

Cada archivo incluye además una tabla `suzaku_meta` de una fila, para saber sin adivinar qué lo generó:

| Columna | Significado |
|---|---|
| `schema_version` | Versión del diseño. Compruébela antes de leer las demás tablas. |
| `suzaku_version`, `command`, `command_line` | Qué versión de Suzaku, qué subcomando y qué invocación exacta. |
| `generated_at` | Cuándo se escribió el archivo. |
| `timestamp_tz` | La zona horaria de la columna `Timestamp`: `UTC`, o el desfase local con `-l, --localtime`. |
| `rules_version`, `rules_count` | Revisión del conjunto de reglas (cuando la carpeta de reglas es un checkout de git) y número de reglas cargadas. |
| `geoip_enabled` | Si se ejecutó `-G, --geo-ip`. Distingue un `SrcCountry` totalmente `NULL` ("el enriquecimiento estaba desactivado") de una celda `NULL` en un archivo enriquecido ("este valor no es una dirección IP"). |
| `scanned_files`, `scanned_events` | Cobertura de la ejecución. |
| `output_rows`, `duplicate_rows_removed` | Filas escritas y duplicados exactos eliminados al escribir. |

En los comandos de cronología basados en reglas, una fila de `timeline` es **un evento × una coincidencia de regla**: un evento que coincide con varias reglas genera una fila por coincidencia, por lo que `EventID` *no* es único. En `aws-ct-search`, cada evento coincidente genera una fila.

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

Suzaku hace un checkpoint de la base de datos antes de salir, así que el archivo `.duckdb` está completo y puede abrirse en modo de solo lectura (cópielo cuando termine el comando, no mientras se ejecuta).
