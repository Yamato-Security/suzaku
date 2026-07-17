# DFIRタイムラインコマンド

## `aws-ct-timeline`コマンド

AWS CloudTrailのDFIRタイムラインを、`rules`フォルダ内のsigmaルールに基づいて作成します。

## コマンド使用例
```
Usage: suzaku aws-ct-timeline [OPTIONS] <--directory <DIR>|--file <FILE>>

General Options:
  -r, --rules <DIR/FILE>  カスタムルールディレクトリのパス (デフォルト: ./rules)
  -h, --help              ヘルプメニューを表示する

Input:
  -d, --directory <DIR>  複数gz/jsonファイルのディレクトリパス
  -f, --file <FILE>      gz/jsonファイルのパス

Filtering:
      --timeline-start <DATE>  読み込むイベントの開始時刻 (例: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    読み込むイベントの終了時刻 (例: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   オフセットに基づいて直近のイベントをスキャン (例: 1y, 3M, 30d, 24h, 30m)

Output:
  -C, --clobber                    結果ファイルを上書きする
  -G, --GeoIP <MAXMIND-DB-DIR>     IPアドレスにGeoIP (ASN、都市、国)情報を追加する
  -m, --min-level <LEVEL>          読み込むルールの最小レベル (規定値: informational)
  -o, --output <FILE>              ファイルに結果を保存
  -t, --output-type <OUTPUT_TYPE>  ファイルタイプ 1: CSV (デフォルト), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL [デフォルト: 1]
  -R, --raw-output                 元のJSONログを出力する（JSON形式または標準出力のみ利用可能）
      --threads <THREAD NUMBER>    使用するスレッド数 (規定値: same as CPU cores)

Display Settings:
  -K, --no-color               カラーで出力しない
  -N, --no-summary             結果概要を出力しない
  -T, --no-frequency-timeline  結果タイムライン頻度を出力しない (ターミナルがUnicodeをサポートしている必要がある)
  -q, --quiet                  Quietモード: 起動バナーを表示しない
```

### `aws-ct-timeline`コマンドの例

* スクリーンにアラートを出力: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* CSVに保存: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* CSVとJSONLに保存: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### `aws-ct-timeline`出力プロフィール

Suzakuは`config/aws_profile.yaml`ファイルに基づいて情報を出力します:
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

* `.`（例: `.eventTime`）で始まるフィールド値は、CloudTrailログから取得されます。
* `sigma.`（例: `sigma.title`）で始まるフィールド値は、Sigmaルールから取得されます。
* 現在は文字列のみをサポートしていますが、将来的には他の型のフィールド値にも対応する予定です。

> 注意：元のJSONデータを出力し、フィールド情報を失わないようにしたい場合は、`aws-ct-timeline`コマンドに`-R, --raw-output`オプションを追加してください。
