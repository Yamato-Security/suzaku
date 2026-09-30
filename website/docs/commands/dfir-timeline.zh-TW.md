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

Suzaku 會根據 `config/aws_profile.yaml` 檔案輸出資訊：
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

* 任何以 `.` 開頭的欄位值（例如：`.eventTime`）將取自 CloudTrail 日誌。
* 任何以 `sigma.` 開頭的欄位值（例如：`sigma.title`）將取自 Sigma 規則。
* 目前我們僅支援字串，但計劃支援其他類型的欄位值。

> 注意：如果您想輸出原始 JSON 資料並確保不遺失任何欄位資訊，只需在 `aws-ct-timeline` 指令中加上 `-R, --raw-output` 選項即可。

### DuckDB 輸出結構 {#duckdb-output-schema}

CSV 與 JSON 輸出是上述設定檔的*呈現結果*；DuckDB 輸出則是*資料介面*，因此採用具型別且自我描述的格式。這些差異是刻意設計的，適用於 `aws-ct-timeline`、`azure-timeline`、`gws-timeline` 與 `aws-ct-search`：

| | CSV / JSON | DuckDB |
|---|---|---|
| 缺少的值 | `-` (或空白) | `NULL` |
| `Timestamp` | 格式化後的文字 | `TIMESTAMP` |
| `Level` | 文字 | `suzaku_level` (依嚴重程度排序的 `ENUM`) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (在 SQL 中不需加引號) |
| `Tags` | 以 ` ¦ ` 串接的單一字串 | `Tactics`, `TechniqueIDs`, `OtherTags` (皆為 `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | 僅在使用 `-G, --geo-ip` 時加入 | 一律存在（設定檔含有 `SrcIP` 時），未使用 `-G` 時為 `NULL` |
| 重複的列 | 保留 | 完全相同的重複列會被移除，數量記錄於 `suzaku_meta` |

每個檔案也都包含一列的 `suzaku_meta` 資料表，讓讀取者無需猜測就能得知檔案的產生來源：

| 欄位 | 說明 |
|---|---|
| `schema_version` | 版面配置版本。讀取其他資料表前請先檢查此值。 |
| `suzaku_version`, `command`, `command_line` | Suzaku 版本、子指令，以及實際執行的指令列。 |
| `generated_at` | 檔案寫入的時間。 |
| `timestamp_tz` | `Timestamp` 欄位所使用的時區：`UTC`，或使用 `-l, --localtime` 時的本地時差。 |
| `rules_version`, `rules_count` | 規則集的修訂版本（當 rules 資料夾為 git checkout 時）以及載入的規則數量。 |
| `geoip_enabled` | 是否執行了 `-G, --geo-ip`。可藉此區分 `SrcCountry` 全為 `NULL`（未啟用補充資訊）與已補充檔案中的 `NULL` 儲存格（該值不是 IP 位址）。 |
| `scanned_files`, `scanned_events` | 此次執行所涵蓋的範圍。 |
| `output_rows`, `duplicate_rows_removed` | 寫入的列數，以及寫入時移除的完全重複列數。 |

對於以規則為基礎的時間軸指令，`timeline` 的一列代表**一個事件 × 一個規則比對**：符合多條規則的事件會依每次比對各產生一列，因此 `EventID` *並非*唯一。在 `aws-ct-search` 中，每個符合的事件會產生一列。

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

Suzaku 結束前會對資料庫執行 checkpoint，因此 `.duckdb` 檔案是完整的，可以唯讀方式開啟（請在指令執行結束後再複製，不要在執行期間複製）。
