# 搜尋指令

## `aws-ct-search` 指令

使用此指令可在不撰寫 Sigma 規則的情況下，從 AWS CloudTrail 日誌中擷取所需的事件。
符合的事件會以與 `aws-ct-timeline` 相同的格式顯示或儲存，因此可以直接用熟悉的時間軸欄位檢視搜尋結果。

縮小事件範圍的方法有以下三種，並可組合使用：

| 選項 | 比對對象 | 多個值 |
|---|---|---|
| `-F, --filter FIELD:VALUE` | 單一欄位的值，**完全比對、區分大小寫** | 重複指定 `-F`；**所有**條件都必須符合（AND） |
| `-k, --keyword KEYWORD` | 事件 JSON 中任意位置的子字串，不區分大小寫（加上 `-c` 時區分） | 重複指定 `-k`；符合**任一**關鍵字即可（OR） |
| `-r, --regex REGEX` | 事件 JSON 中任意位置的正規表示式 | 單一模式（以未跳脫的管線字元 <code>&#124;</code> 指定多個候選） |

每個事件依下列順序檢查，只有通過所有檢查的事件才會輸出：時間範圍（`--timeline-start`, `--timeline-end`, `--time-offset`）→ `-F` → `-k` → `-r`。

若已知值所在的欄位（例如 `eventName:ConsoleLogin`），請使用 `-F`；若只知道字串而不知道出現位置，請使用 `-k` 或 `-r`。

### `-F, --filter` 的運作方式

* 格式為 `FIELD:VALUE`。引數會在**第一個** `:` 處分割，因此像 ARN 這類含有冒號的值可以直接指定。
* 巢狀欄位以點記法表示（`userIdentity.type`）。開頭的 `.` 可省略，因此 `.userIdentity.arn` 也可使用。
* 值前後的引號（`"..."` 或 `'...'`）會被移除。
* 數值與布林值會以字串進行比較，因此 `-F readOnly:false` 與 `-F responseElements.user.userId:12345` 都可使用。
* 沒有該欄位的事件不會符合。
* 無法指定陣列中的元素（沒有 `items.0.name` 這類語法）。
* 不含 `:` 的篩選條件會在掃描開始前被拒絕：`Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`。

> 注意：`aws-ct-metrics` 的 `-F` 是 `--field-name`，只接受欄位名稱。`aws-ct-search` 的 `-F` 是 `--filter`，一律使用 `FIELD:VALUE`。

## 指令用法
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

### `aws-ct-search` 指令範例

依欄位篩選（`-F`）：

* 找出主控台登入: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* 找出 root 使用者的 API 呼叫（巢狀欄位）: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* 找出特定 IAM 使用者的所有操作（值中含有冒號）: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* 找出被拒絕的呼叫: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* 找出 `us-east-1` 中非唯讀的 IAM 呼叫（多個篩選條件為 AND）: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* 找出對特定 S3 儲存貯體的存取（API 專屬欄位）: `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

依關鍵字（`-k`）與正規表示式（`-r`）搜尋：

* 找出任意位置提及某個 IP 位址的事件: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* 找出提及兩位使用者其中之一的事件（多個關鍵字為 OR）: `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* 區分大小寫的關鍵字搜尋: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* 找出停用 CloudTrail 的嘗試: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

組合條件與儲存結果：

* 將關鍵字搜尋限定於單一事件來源: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* 最近 7 天內失敗的主控台登入: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* 加上 GeoIP 資訊並儲存為 CSV 與 DuckDB: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* 儲存符合事件的原始 JSON: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search` 輸出

輸出欄位與 [`aws-ct-timeline`](dfir-timeline.md) 相同，DuckDB 輸出也遵循相同的 [DuckDB 輸出結構](dfir-timeline.md#duckdb-output-schema)。
最後會顯示掃描的事件數與符合的事件數：

```
Total events scanned: 209
Matching events: 1
```
