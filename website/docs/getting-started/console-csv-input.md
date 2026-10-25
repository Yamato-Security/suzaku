# Reading AWS Console CSV Downloads

Suzaku's AWS commands (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) read the CSV file you get from **CloudTrail → Event history → Download events → CSV** in the AWS Management Console, with both `-f/--file` and `-d/--directory`.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

The CSV is a convenient fallback when no trail delivers logs to S3, but it is a **summary** of each event, not the CloudTrail record. Prefer the JSON logs from a trail (or the JSON download from Event history) whenever you have them.

## How a row is converted

Each row is rebuilt as a CloudTrail record, so rules and the summary/metrics commands use the same field names as with JSON input:

| CSV column | CloudTrail field | Notes |
|---|---|---|
| `Event time` | `eventTime` | A row without it is skipped |
| `Event source` | `eventSource` | |
| `Event name` | `eventName` | |
| `AWS region` | `awsRegion` | |
| `Source IP address` | `sourceIPAddress` | |
| `User agent` | `userAgent` | |
| `Error code` | `errorCode` | |
| `User name` | `userIdentity.userName` | The session name for a role. `root` also sets `userIdentity.type: Root` |
| `AWS access key` | `userIdentity.accessKeyId` | |
| `Resources` | `resources[]` | `resourceName` → `ARN`, `resourceType` → `type` |
| `Read-only` | `readOnly` | `true`/`false` → boolean |
| `Request ID` | `requestID` | |
| `Event ID` | `eventID` | |
| `Event type` | `eventType` | |
| `Recipient Account Id` | `recipientAccountId` | |
| `Event category` | `eventCategory` | |

An empty cell becomes an absent field, so a rule testing `errorCode|exists` does not match a successful call. A column not in this table is kept under its own name.

Only the English column names are recognised. A CSV that is not a console CloudTrail download is not read: `-f` prints the reason, and `-d` skips the file, does not count it as scanned, and writes the reason to the error log. Rows with a wrong number of fields (usually broken quoting) are skipped, and every row problem is written to the error log with its line number.

## Limitations

1. **Download every region you need, including `us-east-1`.** The console downloads one region at a time. Console sign-ins and IAM events are recorded in `us-east-1`.
2. **Some rules cannot match.** The CSV has no `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` or `additionalEventData`. Rules on those fields — about 24 of the 794 CloudTrail rules, including the console-login rules such as failed logins and logins without MFA — do not match.
3. **Some timeline columns are empty.** `UserARN`, `UserAccountID`, `UserPrincipalID` and `ErrorMessage` are always empty, and `UserType` is empty except for root.
4. **`aws-ct-summary` groups by user name.** Without an ARN, events are grouped by `User name` in the `UserARN` column, and the command prints a note with the number of such events. For a role this is the session name, so different roles used with the same session name are counted as one principal. Rows with no user name (such as calls AWS services make on a user's behalf) are left out, so the counts can differ from a JSON export of the same period.
5. **Google Workspace does not read CSV.** The `-d`/`-f` help lists `csv` because the input options are shared by the AWS, Azure and Google Workspace commands; `gws-timeline` does not accept CSV input.
