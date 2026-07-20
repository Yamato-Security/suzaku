# DFIR Timeline Commands

## `aws-ct-timeline` command

Create an AWS CloudTrail DFIR timeline based on Sigma rules in the `rules` folder.

## Command usage
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
  -t, --output-type <FORMAT,...>   Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [possible values: csv, json, jsonl, duckdb]
  -R, --raw-output                 Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>    Number of threads to use (default: same as CPU cores)

Display Settings:
  -K, --no-color               Disable color output
  -N, --no-summary             Do not display results summary
  -T, --no-frequency-timeline  Disable event frequency timeline (terminal needs to support Unicode)
  -q, --quiet                  Quiet mode: do not display the launch banner
```

### `aws-ct-timeline` command examples

* Output alerts to screen: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Save results to a CSV file: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Save results to CSV and JSONL files: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t csv,jsonl`
* Save results to a DuckDB database: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t duckdb`

### `aws-ct-timeline` output profile

Suzaku will output information based on the `config/aws_profile.yaml` file:
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

* Any field value that starts with `.` (ex: `.eventTime`) will be taken from the CloudTrail log.
* Any field value that starts with `sigma.` (ex: `sigma.title`) will be taken from the Sigma rule.
* Currently we only support strings but plan on supporting other types of field values.

> Note: If you want to output the original JSON data and make sure you do not loose any field information, just add the `-R, --raw-output` option to `aws-ct-timeline` command.
