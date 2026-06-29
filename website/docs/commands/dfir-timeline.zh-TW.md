# DFIR 時間軸指令

## `aws-ct-timeline` 指令

根據 `rules` 資料夾中的 Sigma 規則建立 AWS CloudTrail DFIR 時間軸。

## 指令用法
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

### `aws-ct-timeline` 指令範例

* 將警示輸出至螢幕：`./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* 將結果儲存為 CSV 檔案：`./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* 將結果儲存為 CSV 與 JSONL 檔案：`./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### `aws-ct-timeline` 輸出設定檔

Suzaku 會根據 `config/default_profile.yaml` 檔案輸出資訊：
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
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
RuleID: 'sigma.id'
```

* 任何以 `.` 開頭的欄位值（例如：`.eventTime`）將取自 CloudTrail 日誌。
* 任何以 `sigma.` 開頭的欄位值（例如：`sigma.title`）將取自 Sigma 規則。
* 目前我們僅支援字串，但計劃支援其他類型的欄位值。

> 注意：如果您想輸出原始 JSON 資料並確保不遺失任何欄位資訊，只需在 `aws-ct-timeline` 指令中加上 `-R, --raw-output` 選項即可。
