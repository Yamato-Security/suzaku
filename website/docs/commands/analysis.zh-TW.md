# 分析指令

## `aws-ct-metrics` 指令

使用此指令可針對 AWS CloudTrail 日誌內的欄位建立統計指標。
預設情況下，它會掃描 `eventName` 欄位。
我們目前使用此指令來找出哪些 API 呼叫最為常見，以便優先撰寫偵測規則。

## 指令用法
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

### `aws-ct-metrics` 指令範例

* 將 `eventName` API 呼叫以表格形式輸出至螢幕：`./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* 儲存至 CSV 檔案：`./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
