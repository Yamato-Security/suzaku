# Analysis Commands

## `aws-ct-metrics` command

Use this command to create metrics on fields inside AWS CloudTrail logs.
By default, it will scan the `eventName` field.
We are currently using this command to figure out which API calls are the most common in order to prioritize writing detection rules.

## Command usage
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -F, --field-name <FIELD_NAME>  The field to generate metrics for [default: eventName]
  -o, --output <FILE>            Output CSV

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
```

### `aws-ct-metrics` command examples

* Output a table of `eventName` API calls to screen: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* Save to a CSV file: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
