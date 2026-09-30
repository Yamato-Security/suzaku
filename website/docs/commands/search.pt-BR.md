# Comandos de Busca

## Comando `aws-ct-search`

Use este comando para extrair dos logs do AWS CloudTrail os eventos que interessam, sem escrever uma regra Sigma.
Os eventos correspondentes são exibidos ou salvos no mesmo formato do `aws-ct-timeline`, para que você passe direto da busca para as colunas de linha do tempo que já conhece.

Há três formas de restringir os eventos, e elas podem ser combinadas:

| Opção | O que é comparado | Vários valores |
|---|---|---|
| `-F, --filter FIELD:VALUE` | O valor de um campo, **correspondência exata, diferencia maiúsculas de minúsculas** | Repita `-F`; **todos** os filtros devem corresponder (E) |
| `-k, --keyword KEYWORD` | Uma substring em qualquer parte do JSON do evento, sem diferenciar maiúsculas de minúsculas (diferencia com `-c`) | Repita `-k`; basta corresponder a **qualquer** palavra-chave (OU) |
| `-r, --regex REGEX` | Uma expressão regular em qualquer parte do JSON do evento | Um único padrão (use o caractere de barra vertical sem escape <code>&#124;</code> para alternativas) |

Cada evento é verificado nesta ordem, e somente os eventos que passam em todas as verificações são exibidos: intervalo de tempo (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Use `-F` quando souber qual campo contém o valor (por exemplo `eventName:ConsoleLogin`), e `-k` ou `-r` quando souber apenas a string, mas não onde ela aparece.

### Como `-F, --filter` funciona

* O formato é `FIELD:VALUE`. O argumento é dividido no **primeiro** `:`, então valores que contêm dois-pontos, como ARNs, funcionam como estão.
* Campos aninhados são escritos em notação de ponto (`userIdentity.type`). O `.` inicial é opcional, então `.userIdentity.arn` também funciona.
* As aspas ao redor do valor (`"..."` ou `'...'`) são removidas.
* Números e booleanos são comparados como strings, então `-F readOnly:false` e `-F responseElements.user.userId:12345` funcionam.
* Um evento que não tem o campo não corresponde.
* Elementos dentro de arrays não podem ser referenciados (não existe a sintaxe `items.0.name`).
* Um filtro sem `:` é rejeitado antes do início da varredura: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Nota: em `aws-ct-metrics`, `-F` significa outra coisa: é `--field-name` e aceita apenas um nome de campo. Em `aws-ct-search` é `--filter` e sempre recebe `FIELD:VALUE`.

## Uso do comando
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

### Exemplos do comando `aws-ct-search`

Filtrar por campo (`-F`):

* Encontrar logins no console: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Encontrar chamadas de API feitas pelo usuário root (campo aninhado): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Encontrar tudo o que um usuário IAM específico fez (o valor contém dois-pontos): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Encontrar chamadas negadas: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* Encontrar chamadas IAM de escrita (não somente leitura) em `us-east-1` (vários filtros são combinados com E): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Encontrar acessos a um bucket S3 específico (campo específico da API): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Buscar por palavra-chave (`-k`) e expressão regular (`-r`):

* Encontrar eventos que mencionam um endereço IP em qualquer lugar: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* Encontrar eventos que mencionam um de dois usuários (várias palavras-chave são combinadas com OU): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Busca por palavra-chave diferenciando maiúsculas de minúsculas: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* Encontrar tentativas de desativar o CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Combinar condições e salvar resultados:

* Busca por palavra-chave limitada a uma origem de eventos: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Logins no console com falha nos últimos 7 dias: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Salvar os resultados em CSV e DuckDB com informações de GeoIP: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Salvar o JSON original dos eventos correspondentes: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### Saída do `aws-ct-search`

As colunas de saída são as mesmas do [`aws-ct-timeline`](dfir-timeline.md), e a saída do DuckDB segue o mesmo [esquema de saída do DuckDB](dfir-timeline.md#duckdb-output-schema).
Ao final, são exibidos o número de eventos varridos e de eventos correspondentes:

```
Total events scanned: 209
Matching events: 1
```
