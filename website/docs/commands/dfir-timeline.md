# DFIR Timeline Commands

## `aws-ct-timeline` command

Create an AWS CloudTrail DFIR timeline based on Sigma rules in the `rules` folder.

## Command usage
```
Usage:
  suzaku aws-ct-timeline <INPUT> [OPTIONS]

General Options:
  -h, --help              Show the help menu
  -r, --rules <DIR/FILE>  Specify a custom rule directory or file (default: ./rules)

Input:
  -d, --directory <DIR>  Directory of multiple gz/json/parquet files
  -f, --file <FILE>      File path to one gz/json/parquet file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
      --file-date-to <DATE>    Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")

Output:
  -C, --clobber                   Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>   Add GeoIP (ASN, city, country) info to IP addresses
  -m, --min-level <LEVEL>         Minimum level for rules to load (default: informational)
  -o, --output <FILE>             Save the results to a file
  -t, --output-type <FORMAT,...>  Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [default: csv] [possible values: csv, json, jsonl, duckdb]
      --raw-output                Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>   Number of threads to use (default: same as CPU cores)
  -s, --sort                      Sort results by timestamp before output (warning: holds every result in memory)

Display Settings:
  -K, --no-color    Disable color output
  -N, --no-summary  Do not display results summary
  -q, --quiet       Quiet mode: do not display the launch banner

Time Format:
  -l, --localtime  Output the timestamp in the local timezone (default: UTC)
```

### `aws-ct-timeline` command examples

* Output alerts to screen: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Save results to a CSV file: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Save results to CSV and JSONL files: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t csv,jsonl`
* Save results to a DuckDB database: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t duckdb`
* Save results to a CSV file sorted by time: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv -s`

### Sorting results (`-s, --sort`)

By default results are written as they are found, so their order follows the order the log files
were scanned in, which the filesystem decides and which can differ between runs. With `-s`, the
screen, CSV, JSON and JSONL results are held back and written at the end of the run in time order:

* Rows are ordered by the event time as an instant, so `-l, --localtime` and logs that mix `Z` and
  `+09:00` offsets still sort correctly.
* Rows whose time is missing or cannot be parsed are placed last.
* Rows with the same time are ordered by their content. Given the same detection results and
  output settings, detection rows have the same order regardless of file scan order.
  This does not guarantee byte-identical terminal output or DuckDB files. For `temporal_ordered`
  correlations, the input order of events with identical timestamps can affect whether a rule
  matches; output sorting does not resolve that detection-side limitation.
* A correlation result is placed at the time of its latest member event (see below).

The DuckDB output is always sorted by time and is not affected by `-s`; a run that writes only
DuckDB keeps no extra copy of the results.

Because every result is kept in memory until the end, `-s` uses noticeably more memory: about
1.3 KB per result, or about 3 KB per result with `--raw-output`. On the flaws.cloud CloudTrail
dataset (1.9 million detections) the peak grew by 2.4 GB (5.5 GB with `--raw-output`) on top of the
5.3 GB the run already used, while the run time grew by only about 2-3% (roughly 1-2 seconds on a
70-second run).

**Correlation timestamps.** A correlation result uses its latest member event's time. The
`Timestamp` column displays that instant in UTC or, with `-l`, in local time, preserving fractional
seconds. The summary always counts the result under its UTC date, including when `-l` is set.
This applies with or without `-s`, including to correlation rows in DuckDB output.

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

> Note: If you want to output the original JSON data and make sure you do not loose any field information, just add the `--raw-output` option to `aws-ct-timeline` command.

### DuckDB output schema

The CSV and JSON outputs are a *rendering* of the profile above; the DuckDB output is a *data
interface*, so it is typed and self-describing instead. The differences are deliberate and apply
to `aws-ct-timeline`, `azure-timeline`, `gws-timeline` and `aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| A missing value | `-` (or empty) | `NULL` |
| `Timestamp` | rendered text | `TIMESTAMP` |
| `Level` | text | `suzaku_level`, an `ENUM` ordered by severity |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (no quoting needed in SQL) |
| `Tags` | one ` ¦ `-joined string | `Tactics`, `TechniqueIDs`, `OtherTags`, each a `VARCHAR[]` |
| `SrcASN` / `SrcCity` / `SrcCountry` | added only under `-G, --geo-ip` | always present (when the profile has `SrcIP`), `NULL` when `-G` was not used |
| Duplicate rows | kept | exact duplicates removed, count reported in `suzaku_meta` |

Every file also carries a one-row `suzaku_meta` table so a reader can tell what produced it
without guessing:

| Column | Meaning |
|---|---|
| `schema_version` | Layout version. Check this before reading the other tables. |
| `suzaku_version`, `command`, `command_line` | Which Suzaku, which subcommand, which exact invocation. |
| `generated_at` | When the file was written. |
| `timestamp_tz` | The zone the `Timestamp` column is expressed in — `UTC`, or the local offset under `-l, --localtime`. |
| `rules_version`, `rules_count` | Ruleset revision (when the rules folder is a git checkout) and how many rules were loaded. |
| `geoip_enabled` | Whether `-G, --geo-ip` ran. Tells an all-`NULL` `SrcCountry` ("enrichment was off") apart from a `NULL` cell in an enriched file ("this value is not an IP address"). |
| `scanned_files`, `scanned_events` | Coverage of the run. |
| `output_rows`, `duplicate_rows_removed` | Rows written, and exact duplicates dropped on write. |

A `timeline` row is one **event × rule match**: an event matching several rules produces one row
per match, so `EventID` is *not* unique. That grain is also recorded as a table comment
(`SELECT comment FROM duckdb_tables()`).

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

The database is checkpointed before Suzaku exits, so the `.duckdb` file is complete and can be
opened read-only (copy it after the command finishes, not while it runs).

## `gws-timeline` command

Create a Google Workspace DFIR timeline based on Sigma rules in the `rules` folder.

### Input

`gws-timeline` reads Google Admin SDK Reports API `activities.list` output
(`"kind": "admin#reports#activity"`), as `.json`, `.jsonl`, or `.json.gz`. Any of the following
shapes are accepted:

* A bare activity object, or a JSON array of activity objects.
* A raw `activities.list` response page: `{"kind": "admin#reports#activities", "items": [...]}`. A
  page that returned nothing carries `"items": null` (or `[]`) and yields no events at all, so an
  empty page does not inflate the scanned-event count.

Get activities from the [Admin SDK Reports API `activities.list`
endpoint](https://developers.google.com/workspace/admin/reports/reference/rest/v1/activities/list)
(one export per `applicationName`: `admin`, `login`, `drive`, `calendar`, `token`,
`user_accounts`, `saml`, `groups`, `mobile`, `gmail`, ...), [GAM](https://github.com/GAM-team/GAM),
or any export that preserves the activity JSON.

> The Admin console's CSV report export is **not** supported — it is a different, lossy schema
> that drops fields Sigma rules and the output profile depend on. Export via the Reports API (or
> a tool that wraps it) instead.

### Command usage
```
Usage:
  suzaku gws-timeline <INPUT> [OPTIONS]

General Options:
  -h, --help              Show the help menu
  -r, --rules <DIR/FILE>  Specify a custom rule directory or file (default: ./rules)

Input:
  -d, --directory <DIR>  Directory of multiple gz/json/parquet files
  -f, --file <FILE>      File path to one gz/json/parquet file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
      --file-date-to <DATE>    Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")

Output:
  -C, --clobber                   Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>   Add GeoIP (ASN, city, country) info to IP addresses
  -m, --min-level <LEVEL>         Minimum level for rules to load (default: informational)
  -o, --output <FILE>             Save the results to a file
  -t, --output-type <FORMAT,...>  Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [default: csv] [possible values: csv, json, jsonl, duckdb]
      --raw-output                Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>   Number of threads to use (default: same as CPU cores)
  -s, --sort                      Sort results by timestamp before output (warning: holds every result in memory)

Display Settings:
  -K, --no-color    Disable color output
  -N, --no-summary  Do not display results summary
  -q, --quiet       Quiet mode: do not display the launch banner

Time Format:
  -l, --localtime  Output the timestamp in the local timezone (default: UTC)
```

> `gws-timeline` shares its input/output options with `aws-ct-timeline` and `azure-timeline`, so
> the general `-d/-f` help text above mentions `parquet`. Those other formats are still read when
> they turn up in a scanned directory — they simply will not contain Reports API activities, so
> they contribute no events. The shapes `gws-timeline` actually understands are the `.json` /
> `.jsonl` / `.json.gz` ones described under **Input** above.

### `gws-timeline` command examples

* Output alerts to screen: `./suzaku gws-timeline -d ./gws-logs`
* Save results to a CSV file: `./suzaku gws-timeline -d ./gws-logs -o gws-timeline.csv`
* Save results to CSV and JSONL files: `./suzaku gws-timeline -d ./gws-logs -o gws-timeline -t csv,jsonl`
* Save results to a DuckDB database: `./suzaku gws-timeline -d ./gws-logs -o gws-timeline -t duckdb`
* Save results to a CSV file sorted by time: `./suzaku gws-timeline -d ./gws-logs -o gws-timeline.csv -s` (see **Sorting results** under `aws-ct-timeline`)

### Normalization

Each activity's `events[]` array is split into one output record per sub-event before rules are
matched — Sigma cannot see inside an array, and a single activity commonly carries more than one
semantically distinct sub-event (e.g. an admin password reset paired with a forced
sign-out-on-next-login). For each sub-event, `gws-timeline`:

* Copies every top-level activity field (`id`, `actor`, `ipAddress`, `networkInfo`, ...) except `events`.
* Folds the sub-event's `parameters[]` array into flat top-level fields, keyed by parameter name,
  taking whichever value field is present:

    | Parameter field | Becomes |
    |---|---|
    | `value` | the string as-is |
    | `intValue` | an integer (the API transports it as a string; an unparseable one is kept verbatim) |
    | `boolValue` | a JSON boolean |
    | `multiValue` | a list of strings |
    | `multiIntValue` | a list of integers, parsed the same way as `intValue` |
    | `messageValue` | an object, by folding its own nested `parameter[]` array recursively (so a rule can reach `SETTING_METADATA.rule_type`) |
    | `multiMessageValue` | a list of such objects |

    A parameter with none of those is dropped: the Reports API omits default values, so a bare
    `{"name": "is_suspicious"}` means `false`, and rules on `is_suspicious: true` match only
    explicit trues.
* Adds a lowercase alias for every `UPPER_CASE` parameter name (e.g. `NEW_VALUE` also becomes
  `new_value`), so both the upstream SigmaHQ `gworkspace` rules (which use the lowercase Filebeat
  naming) and rules written against the raw Reports API field names can match. An alias is only
  ever *added*, never allowed to overwrite: where an application emits a natively lowercase
  parameter of the same name (Drive's `new_value` is a list, Admin's `NEW_VALUE` a string), the
  native key keeps its own value and both spellings can hold different values in one record.
* Adds `eventName` (the sub-event's `name`), `eventType` (the sub-event's `type`), `eventService`
  (`"<applicationName>.googleapis.com"`, e.g. `admin.googleapis.com`, and omitted entirely when
  the activity carries no `id.applicationName`), `eventIndex` (the sub-event's position in
  `events[]`), and `eventCount` (the total number of sub-events in the activity).
* Copies the sub-event's own `resourceIds` onto its record, so each record carries the resources
  of the sub-event it came from rather than those of the activity's first sub-event.
* Writes the folded parameters **first** and everything else over them. A parameter name is
  free-form data from the logged event, so one can be called `eventName` or `id`; the activity's
  own fields and the fields Suzaku synthesizes always win such a collision.

An activity is split into at most 10,000 records. Real activities are far below that (the largest
are Calendar invites with one `add_event_guest` sub-event per guest), and the cap keeps a crafted
input from exhausting memory. When it applies, a warning is written to the error log and
`eventCount` still reports the activity's real length.

> Because the timeline's unit is the sub-event, **Total events** in the summary counts normalized
> records, not the API activities they came from. One activity with three sub-events counts as
> three.

For example, an admin resetting a user's password and forcing a sign-in change arrives as one
activity with two sub-events:

Before (one activity, two sub-events):
```json
{
  "kind": "admin#reports#activity",
  "id": {
    "time": "2024-01-15T14:22:07.000Z",
    "uniqueQualifier": "-1234567890123456789",
    "applicationName": "admin",
    "customerId": "C0example"
  },
  "actor": { "email": "admin@example.test", "callerType": "USER" },
  "ipAddress": "203.0.113.10",
  "events": [
    {
      "type": "USER_SETTINGS",
      "name": "CHANGE_PASSWORD",
      "parameters": [{ "name": "USER_EMAIL", "value": "victim@example.test" }]
    },
    {
      "type": "USER_SETTINGS",
      "name": "CHANGE_PASSWORD_ON_NEXT_LOGIN",
      "parameters": [
        { "name": "USER_EMAIL", "value": "victim@example.test" },
        { "name": "NEW_VALUE", "value": "true" }
      ]
    }
  ]
}
```

After (two normalized records):
```json
[
  {
    "kind": "admin#reports#activity",
    "id": { "time": "2024-01-15T14:22:07.000Z", "uniqueQualifier": "-1234567890123456789", "applicationName": "admin", "customerId": "C0example" },
    "actor": { "email": "admin@example.test", "callerType": "USER" },
    "ipAddress": "203.0.113.10",
    "eventName": "CHANGE_PASSWORD",
    "eventType": "USER_SETTINGS",
    "eventService": "admin.googleapis.com",
    "eventIndex": 0,
    "eventCount": 2,
    "USER_EMAIL": "victim@example.test",
    "user_email": "victim@example.test"
  },
  {
    "kind": "admin#reports#activity",
    "id": { "time": "2024-01-15T14:22:07.000Z", "uniqueQualifier": "-1234567890123456789", "applicationName": "admin", "customerId": "C0example" },
    "actor": { "email": "admin@example.test", "callerType": "USER" },
    "ipAddress": "203.0.113.10",
    "eventName": "CHANGE_PASSWORD_ON_NEXT_LOGIN",
    "eventType": "USER_SETTINGS",
    "eventService": "admin.googleapis.com",
    "eventIndex": 1,
    "eventCount": 2,
    "USER_EMAIL": "victim@example.test",
    "user_email": "victim@example.test",
    "NEW_VALUE": "true",
    "new_value": "true"
  }
]
```

### Sigma rules matched

`gws-timeline` selects rules on `logsource.service` alone — `logsource.product` is not consulted.
A rule is loaded when its service is `google_workspace` or `google_workspace.<applicationName>`
(e.g. `google_workspace.login`, `google_workspace.drive`), and it matches a record whose `kind` is
`admin#reports#activity` and, for the dotted form, whose `id.applicationName` is that application.
This covers SigmaHQ's `rules/cloud/gcp/gworkspace` rule set as well as Suzaku's own rules under
`suzaku-rules`' `suzaku/gws/` directory.

The application list is a **closed set of 18**: a `google_workspace.<app>` service outside it is
not loaded at all, so a typo in a rule's `service:` is reported at load time instead of silently
matching nothing during the scan.

`admin`, `login`, `drive`, `calendar`, `token`, `user_accounts`, `saml`, `groups`, `mobile`,
`gmail`, `chat`, `meet`, `chrome`, `rules`, `context_aware_access`, `access_transparency`, `keep`,
`vault`.

As with the AWS and Azure rule sets, an ignore-list file lets a known-superseded or duplicate rule
stay in the rules repository without being loaded: `config/gws_ignore_rule_list.txt`, read from
the `config/` directory of the rules folder passed to `-r` (it ships with `suzaku-rules`, not with
Suzaku itself).

### `gws-timeline` output profile

Suzaku will output information based on the `config/gws_profile.yaml` file:
```yaml
Timestamp: '.id.time'
RuleTitle: 'sigma.title'
Level: 'sigma.level'
Application: '.id.applicationName'
EventName: '.eventName'
EventType: '.eventType'
Actor: '.actor.email|.actor.key'
ActorType: '.actor.callerType'
SrcIP: '.ipAddress'
IpASN: '.networkInfo.ipAsn'
IpRegion: '.networkInfo.regionCode'
Target: '.USER_EMAIL|.affected_email_address|.doc_title|.event_title|.GROUP_EMAIL|.ROLE_NAME|.APPLICATION_NAME|.API_CLIENT_NAME|.SETTING_NAME'
OldValue: '.OLD_VALUE|.old_value'
NewValue: '.NEW_VALUE|.new_value'
LoginType: '.login_type'
Challenge: '.login_challenge_method'
Suspicious: '.is_suspicious'
Resource: '.doc_id|.event_id'
CustomerId: '.id.customerId'
UniqueQualifier: '.id.uniqueQualifier'
RuleAuthor: 'sigma.author'
Tags: 'sigma.tags'
RuleID: 'sigma.id'
```

* Any field value that starts with `.` (ex: `.id.time`) is taken from the normalized activity
  record described under **Normalization** above.
* Any field value that starts with `sigma.` (ex: `sigma.title`) is taken from the Sigma rule.
* `IpASN` and `IpRegion` are populated directly from Google's own `networkInfo.ipAsn` /
  `networkInfo.regionCode` fields, at no cost — Google Workspace activities already carry ASN and
  region. Passing `-G, --geo-ip` additionally appends `SrcASN`, `SrcCity` and `SrcCountry` from the
  MaxMind databases by looking up `SrcIP`, same as the other commands.

> Note: If you want to output the original JSON data and make sure you do not lose any field
> information, add the `--raw-output` option to the `gws-timeline` command.
