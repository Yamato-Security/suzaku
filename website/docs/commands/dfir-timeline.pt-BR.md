# Comandos de Timeline DFIR

## Comando `aws-ct-timeline`

Cria uma timeline DFIR do AWS CloudTrail com base nas regras Sigma na pasta `rules`.

## Uso do comando
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

### Exemplos do comando `aws-ct-timeline`

* Exibir alertas na tela: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Salvar os resultados em um arquivo CSV: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Salvar os resultados em arquivos CSV e JSONL: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### Perfil de saída do `aws-ct-timeline`

O Suzaku exibirá as informações com base no arquivo `config/aws_profile.yaml`:
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

* Qualquer valor de campo que comece com `.` (ex: `.eventTime`) será obtido do log do CloudTrail.
* Qualquer valor de campo que comece com `sigma.` (ex: `sigma.title`) será obtido da regra Sigma.
* Atualmente só oferecemos suporte a strings, mas planejamos oferecer suporte a outros tipos de valores de campo.

> Nota: Se você quiser exibir os dados JSON originais e garantir que não perca nenhuma informação de campo, basta adicionar a opção `-R, --raw-output` ao comando `aws-ct-timeline`.

### Esquema de saída do DuckDB {#duckdb-output-schema}

As saídas CSV e JSON são uma *renderização* do perfil acima; a saída do DuckDB é uma *interface de dados*, portanto é tipada e autodescritiva. As diferenças são intencionais e se aplicam a `aws-ct-timeline`, `azure-timeline`, `gws-timeline` e `aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| Valor ausente | `-` (ou vazio) | `NULL` |
| `Timestamp` | texto formatado | `TIMESTAMP` |
| `Level` | texto | `suzaku_level` (um `ENUM` ordenado por severidade) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (sem aspas no SQL) |
| `Tags` | uma única string unida por ` ¦ ` | `Tactics`, `TechniqueIDs`, `OtherTags` (cada uma `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | adicionadas apenas com `-G, --geo-ip` | sempre presentes (quando o perfil tem `SrcIP`), `NULL` quando `-G` não foi usado |
| Linhas duplicadas | mantidas | duplicatas exatas removidas, contagem registrada em `suzaku_meta` |

Cada arquivo também contém uma tabela `suzaku_meta` de uma linha, para que se saiba sem adivinhar o que o gerou:

| Coluna | Significado |
|---|---|
| `schema_version` | Versão do layout. Verifique-a antes de ler as outras tabelas. |
| `suzaku_version`, `command`, `command_line` | Qual versão do Suzaku, qual subcomando e qual invocação exata. |
| `generated_at` | Quando o arquivo foi gravado. |
| `timestamp_tz` | O fuso horário da coluna `Timestamp` — `UTC`, ou o deslocamento local com `-l, --localtime`. |
| `rules_version`, `rules_count` | Revisão do conjunto de regras (quando a pasta de regras é um checkout git) e quantas regras foram carregadas. |
| `geoip_enabled` | Se `-G, --geo-ip` foi executado. Diferencia um `SrcCountry` totalmente `NULL` ("o enriquecimento estava desligado") de uma célula `NULL` em um arquivo enriquecido ("este valor não é um endereço IP"). |
| `scanned_files`, `scanned_events` | Cobertura da execução. |
| `output_rows`, `duplicate_rows_removed` | Linhas gravadas e duplicatas exatas removidas na gravação. |

Nos comandos de linha do tempo baseados em regras, uma linha de `timeline` é **um evento × uma correspondência de regra**: um evento que corresponde a várias regras gera uma linha por correspondência, então `EventID` *não* é único. No `aws-ct-search`, cada evento correspondente gera uma linha.

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

O Suzaku faz checkpoint do banco de dados antes de encerrar, então o arquivo `.duckdb` está completo e pode ser aberto somente para leitura (copie-o depois que o comando terminar, não durante a execução).
