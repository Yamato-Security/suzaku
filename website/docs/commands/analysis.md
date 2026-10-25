# Analysis Commands

## `aws-ct-metrics` command

Use this command to create metrics on fields inside AWS CloudTrail logs.
By default, it will scan the `eventName` field.
We are currently using this command to figure out which API calls are the most common in order to prioritize writing detection rules.

Any field can be aggregated with `-F`, including nested ones written in dot notation (`userIdentity.arn`), and several fields can be aggregated in a single scan.
For each value you get the number of events, its share of all events, and the first and last time that value was seen.

Where this differs from the [`aws-ct-summary` command](dfir-summary.md): `aws-ct-metrics` aggregates **across all events** to show you the overall distribution, while `aws-ct-summary` breaks the same information down **per user ARN**.
Use `aws-ct-metrics` to triage a new dataset (which source IPs, user agents and regions exist at all?) and `aws-ct-summary` to investigate what a specific principal did.

Fields that are missing from an event are counted as `-`, so the percentages are always a share of every event scanned.
For example, a large `-` share for `userIdentity.accessKeyId` tells you how much of the activity came from console sessions rather than access keys.

> Note: field names are **case-sensitive** and are validated before the scan starts. `-F sourceIPaddress` (lowercase `a`) is rejected with `Did you mean 'sourceIPAddress'?` instead of scanning the whole dataset and reporting a meaningless 100% `-`. Any sub-path of an API-specific container is accepted as-is (ex: `requestParameters.bucketName`, `additionalEventData.MFAUsed`).

> Note: temporary AWS STS access key IDs (`ASIA...`) are not counted as values of `accessKeyId` fields by default, because a new key per session inflates the results. The events themselves are still counted in the other fields. Add `-s` to count the keys too.

## Command usage
```
Usage: suzaku aws-ct-metrics <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory of multiple gz/json/jsonl/csv/parquet files
  -f, --file <FILE>      File path to one gz/json/jsonl/csv/parquet file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
      --file-date-to <DATE>    Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")

Output:
  -F, --field-name <FIELD_NAME,...>  The field(s) to generate metrics for. Comma-separate or repeat to aggregate several in a single scan, e.g. -F sourceIPAddress,userAgent [default: eventName]
  -C, --clobber                      Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>      Add GeoIP (ASN, city, country) info to IP addresses [alias: --GeoIP]
  -o, --output <FILE>                Save the results to a file
  -s, --include-sts-keys             Count temporary AWS STS access key IDs (ASIA...) as values of accessKeyId fields
  -t, --output-type <FORMAT,...>     Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [default: csv] [possible values: csv, json, jsonl, duckdb]

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### `aws-ct-metrics` command examples

* Output a table of `eventName` API calls to screen: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* Save to a CSV file: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
* Aggregate the five fields most useful for triage in a single scan: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -F sourceIPAddress,userAgent,userIdentity.arn,awsRegion,userIdentity.accessKeyId`
* Add ASN, city and country to the source IP addresses: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -F sourceIPAddress -G ../GeoLite2-DBs`
* Save to CSV and a DuckDB database at the same time: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -F sourceIPAddress,userAgent -o sample-metrics -t csv,duckdb`

### `aws-ct-metrics` output

The screen output prints one table per field, named after the field itself.
File output is a single flat table with a `Field` column, so all fields end up in one CSV/JSON/DuckDB file:

| Field | Value | Count | Percent | FirstSeen | LastSeen | SrcASN | SrcCity | SrcCountry |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| sourceIPAddress | 203.0.113.10 | 1,024 | 62.19% | 2021-07-05 13:03:12 | 2021-07-05 13:03:50 | Example ISP | London | United Kingdom |

In the CSV, JSON and screen output the `SrcASN`, `SrcCity` and `SrcCountry` columns only appear when `-G` is used, and only values that parse as an IP address are enriched (an AWS-service caller such as `cloudtrail.amazonaws.com` shows `-`). The DuckDB output always has the three columns so that the same query works against every file it writes; see below.

The DuckDB output is a single `metrics` table plus the [`suzaku_meta`](dfir-timeline.md#duckdb-output-schema) provenance table, so `SELECT * FROM metrics WHERE Field = 'sourceIPAddress' ORDER BY Count DESC` works directly. Values there are typed rather than rendered, and the table carries two columns the text formats do not:

| Column | Type | Notes |
|---|---|---|
| `Field` | `VARCHAR` | The CloudTrail field path, exactly as given to `-F` |
| `TimelineColumn` | `VARCHAR` | The `aws-ct-timeline` column holding the same fact (`sourceIPAddress` → `SrcIP`), or `NULL` when there is none |
| `Value` | `VARCHAR` | `NULL` when the event carried no value for `Field` |
| `Count` | `BIGINT` | |
| `FieldTotal` | `BIGINT` | Events counted for this `Field`, i.e. the denominator of `Percent` |
| `Percent` | `DOUBLE` | `Count / FieldTotal`, at full precision — unlike the `62.19%` the CSV renders, this sums back to 100 per field |
| `FirstSeen` / `LastSeen` | `TIMESTAMP` | |
| `SrcASN` / `SrcCity` / `SrcCountry` | `VARCHAR` | Always present, unlike in the CSV. `NULL` when `-G` was not used (`suzaku_meta.geoip_enabled` is `false`) or when the value is not an IP address |

So the exact share of a value is a query rather than a re-aggregation of rounded percentages:

```sql
SELECT Value, Count, Count * 100.0 / FieldTotal AS pct
FROM metrics
WHERE Field = 'sourceIPAddress' AND Value IS NOT NULL
ORDER BY Count DESC LIMIT 10;
```

`TimelineColumn` is read from `config/aws_profile.yaml`; run the command from the directory containing `config/` or the column is `NULL` for every row.
