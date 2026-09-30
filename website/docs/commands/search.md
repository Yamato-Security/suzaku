# Search Commands

## `aws-ct-search` command

Use this command to pull the events you care about out of AWS CloudTrail logs without writing a Sigma rule.
Matching events are printed or saved in the same format as `aws-ct-timeline`, so you can move straight from a search to the timeline columns you already know.

There are three ways to narrow down the events, and they can be combined:

| Option | What it matches | Multiple values |
|---|---|---|
| `-F, --filter FIELD:VALUE` | The value of one field, **exact match, case-sensitive** | Repeat `-F`; **every** filter must match (AND) |
| `-k, --keyword KEYWORD` | A substring anywhere in the event's JSON, case-insensitive (case-sensitive with `-c`) | Repeat `-k`; **any** keyword may match (OR) |
| `-r, --regex REGEX` | A regular expression anywhere in the event's JSON | One pattern (use the unescaped pipe character <code>&#124;</code> for alternatives) |

Each event is checked in this order, and only events that pass every check are output: time range (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Use `-F` when you know which field holds the value (for example `eventName:ConsoleLogin`), and `-k` or `-r` when you only know the string and not where it appears.

### How `-F, --filter` works

* The format is `FIELD:VALUE`. The argument is split at the **first** `:`, so values that contain colons, such as ARNs, work as-is.
* Nested fields are written in dot notation (`userIdentity.type`). A leading `.` is optional, so `.userIdentity.arn` also works.
* Quotes around the value (`"..."` or `'...'`) are stripped.
* Numbers and booleans are compared as strings, so `-F readOnly:false` and `-F responseElements.user.userId:12345` work.
* An event that does not have the field does not match.
* Elements inside arrays cannot be addressed (there is no `items.0.name` syntax).
* A filter without a `:` is rejected before the scan starts: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Note: `-F` means something different in `aws-ct-metrics`, where it is `--field-name` and only takes a field name. In `aws-ct-search` it is `--filter` and always takes `FIELD:VALUE`.

## Command usage
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

### `aws-ct-search` command examples

Filtering by field (`-F`):

* Find console logins: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Find API calls made by the root user (nested field): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Find everything a specific IAM user did (the value contains colons): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Find calls that were denied: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* Find write (non read-only) IAM calls in `us-east-1` (several filters are ANDed): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Find access to a specific S3 bucket (API-specific field): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Searching by keyword (`-k`) and regular expression (`-r`):

* Find events that mention an IP address anywhere: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* Find events that mention either of two users (several keywords are ORed): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Case-sensitive keyword search: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* Find attempts to disable CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Combining conditions and saving results:

* Keyword search limited to one event source: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Failed console logins in the last 7 days: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Save the results to CSV and DuckDB with GeoIP info: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Save the original JSON of the matching events: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search` output

The output columns are the same as [`aws-ct-timeline`](dfir-timeline.md), and the DuckDB output follows the same [DuckDB output schema](dfir-timeline.md#duckdb-output-schema).
The number of scanned and matching events is printed at the end:

```
Total events scanned: 209
Matching events: 1
```
