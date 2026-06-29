# 分析コマンド

## `aws-ct-metrics`コマンド

このコマンドは、AWS CloudTrailログ内のフィールドに関するメトリクスを作成するために使用します。
デフォルトでは、`eventName`フィールドをスキャンします。
このコマンドは、現在、最も一般的なAPI呼び出しを特定するために使用されており、検出ルールの優先順位を決定するために使用されます。

## コマンド使用例
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  複数gz/jsonファイルのディレクトリパス
  -f, --file <FILE>      gz/jsonファイルのパス

Filtering:
      --timeline-start <DATE>  読み込むイベントの開始時刻 (例: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    読み込むイベントの終了時刻 (例: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   オフセットに基づいて直近のイベントをスキャン (例: 1y, 3M, 30d, 24h, 30m)

Output:
  -F, --field-name <FIELD_NAME>  メトリクスを作成するフィールド [デフォルト: eventName]
  -o, --output <FILE>            CSVに保存

Display Settings:
  -K, --no-color  カラーで出力しない
  -q, --quiet     Quietモード: 起動バナーを表示しない

General Options:
  -h, --help  ヘルプメニューを表示する
```

### `aws-ct-metrics`コマンドの例

* `eventName`のAPIコール数をテーブル形式で出力: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* CSVに保存: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
