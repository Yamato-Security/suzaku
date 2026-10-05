# Comandos de búsqueda

## Comando `aws-ct-search`

Use este comando para extraer de los registros de AWS CloudTrail los eventos que le interesan sin escribir una regla Sigma.
Los eventos coincidentes se muestran o guardan en el mismo formato que `aws-ct-timeline`, por lo que puede pasar directamente de una búsqueda a las columnas de línea de tiempo que ya conoce.

Hay tres formas de acotar los eventos y se pueden combinar:

| Opción | Qué compara | Varios valores |
|---|---|---|
| `-F, --filter FIELD:VALUE` | El valor de un campo, **coincidencia exacta, distingue mayúsculas y minúsculas** | Repita `-F`; **todos** los filtros deben coincidir (Y) |
| `-k, --keyword KEYWORD` | Una subcadena en cualquier parte del JSON del evento, sin distinguir mayúsculas y minúsculas (las distingue con `-c`) | Repita `-k`; basta con que coincida **cualquier** palabra clave (O) |
| `-r, --regex REGEX` | Una expresión regular en cualquier parte del JSON del evento | Un solo patrón (use el carácter de barra vertical sin escapar <code>&#124;</code> para alternativas) |

Cada evento se comprueba en este orden y solo se muestran los eventos que superan todas las comprobaciones: intervalo de tiempo (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Use `-F` cuando sepa qué campo contiene el valor (por ejemplo `eventName:ConsoleLogin`), y `-k` o `-r` cuando solo conozca la cadena y no dónde aparece.

### Cómo funciona `-F, --filter`

* El formato es `FIELD:VALUE`. El argumento se divide en el **primer** `:`, así que los valores que contienen dos puntos, como los ARN, funcionan tal cual.
* Los campos anidados se escriben con notación de puntos (`userIdentity.type`). El `.` inicial es opcional, por lo que `.userIdentity.arn` también funciona.
* Se eliminan las comillas alrededor del valor (`"..."` o `'...'`).
* Los números y booleanos se comparan como cadenas, así que `-F readOnly:false` y `-F responseElements.user.userId:12345` funcionan.
* Un evento que no tiene el campo no coincide.
* No se pueden indicar elementos dentro de arrays (no existe la sintaxis `items.0.name`).
* Un filtro sin `:` se rechaza antes de iniciar el escaneo: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Nota: en `aws-ct-metrics`, `-F` significa otra cosa: es `--field-name` y solo acepta un nombre de campo. En `aws-ct-search` es `--filter` y siempre recibe `FIELD:VALUE`.

## Uso del comando
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

### Ejemplos del comando `aws-ct-search`

Filtrar por campo (`-F`):

* Buscar inicios de sesión en la consola: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Buscar llamadas a la API realizadas por el usuario root (campo anidado): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Buscar todo lo que hizo un usuario IAM concreto (el valor contiene dos puntos): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Buscar llamadas denegadas: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* Buscar llamadas IAM de escritura (no de solo lectura) en `us-east-1` (varios filtros se combinan con Y): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Buscar accesos a un bucket S3 concreto (campo específico de la API): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Buscar por palabra clave (`-k`) y expresión regular (`-r`):

* Buscar eventos que mencionen una dirección IP en cualquier parte: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* Buscar eventos que mencionen a cualquiera de dos usuarios (varias palabras clave se combinan con O): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Búsqueda por palabra clave que distingue mayúsculas y minúsculas: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* Buscar intentos de desactivar CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Combinar condiciones y guardar resultados:

* Búsqueda por palabra clave limitada a un origen de eventos: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Inicios de sesión fallidos en la consola en los últimos 7 días: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Guardar los resultados en CSV y DuckDB con información GeoIP: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Guardar el JSON original de los eventos coincidentes: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### Salida de `aws-ct-search`

Las columnas de salida son las mismas que las de [`aws-ct-timeline`](dfir-timeline.md), y la salida de DuckDB sigue el mismo [esquema de salida de DuckDB](dfir-timeline.md#duckdb-output-schema).
Al final se muestra el número de eventos escaneados y coincidentes:

```
Total events scanned: 209
Matching events: 1
```
