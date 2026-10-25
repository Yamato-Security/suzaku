# 分析コマンド

## `aws-ct-metrics`コマンド

このコマンドは、AWS CloudTrailログ内のフィールドに関するメトリクスを作成するために使用します。
デフォルトでは、`eventName`フィールドをスキャンします。
このコマンドは、現在、最も一般的なAPI呼び出しを特定するために使用されており、検出ルールの優先順位を決定するために使用されます。

`-F`では任意のフィールドを集計でき、ドット記法によるネストしたフィールド(`userIdentity.arn`)も指定できます。また、複数のフィールドを1回のスキャンでまとめて集計できます。
各値について、イベント数・全体に占める割合・その値が最初と最後に観測された時刻が出力されます。

[`aws-ct-summary`コマンド](dfir-summary.md)との違いは集計軸です。`aws-ct-metrics`は**全イベント横断**の分布を出力し、`aws-ct-summary`は同じ情報を**ユーザARNごと**に分解して出力します。
新しいデータセットのトリアージ(そもそもどのようなソースIP・ユーザーエージェント・リージョンが存在するのか)には`aws-ct-metrics`を、特定のプリンシパルの活動を深掘りするには`aws-ct-summary`を使用してください。

イベントに存在しないフィールドは`-`として集計されるため、割合は常にスキャンした全イベントに対する比率になります。
例えば`userIdentity.accessKeyId`の`-`の割合からは、アクセスキーではなくコンソール操作によるアクティビティがどれくらいあるのかが分かります。

> 注意: フィールド名は**大文字・小文字を区別**し、スキャン開始前に検証されます。`-F sourceIPaddress`(小文字の`a`)はデータセット全体をスキャンして無意味な`-` 100%を出力する代わりに、`Did you mean 'sourceIPAddress'?`というエラーで拒否されます。APIごとに内容が異なるコンテナ配下は任意のパスをそのまま指定できます(例: `requestParameters.bucketName`、`additionalEventData.MFAUsed`)。

> 注意: AWS STSの一時アクセスキーID(`ASIA...`)は、セッションごとに新しいキーが発行され結果が膨らむため、デフォルトでは`accessKeyId`系フィールドの値として集計しません。イベント自体は他のフィールドで集計されます。キーも集計する場合は`-s`を指定してください。

## コマンド使用例
```
Usage: suzaku aws-ct-metrics <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  複数gz/jsonファイルのディレクトリパス
  -f, --file <FILE>      gz/jsonファイルのパス

Filtering:
      --timeline-start <DATE>  読み込むイベントの開始時刻 (例: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    読み込むイベントの終了時刻 (例: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   オフセットに基づいて直近のイベントをスキャン (例: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  AWSLogsのS3パスの日付構造に基づいて開始日でファイルを絞り込む (例: "20240101")
      --file-date-to <DATE>    AWSLogsのS3パスの日付構造に基づいて終了日でファイルを絞り込む (例: "20241231")

Output:
  -F, --field-name <FIELD_NAME,...>  メトリクスを作成するフィールド。カンマ区切りまたは複数回指定で、1回のスキャンで複数フィールドを集計する (例: -F sourceIPAddress,userAgent) [デフォルト: eventName]
  -C, --clobber                      保存時にファイルを上書きする
  -s, --include-sts-keys             AWS STSの一時アクセスキーID(ASIA...)をaccessKeyId系フィールドの値として集計する
  -G, --geo-ip <MAXMIND-DB-DIR>      IPアドレスにGeoIP情報(ASN、市、国)を追加する [エイリアス: --GeoIP]
  -o, --output <FILE>                結果をファイルに保存する
  -t, --output-type <FORMAT,...>     出力形式(-o指定時のみ有効): csv (デフォルト), json, jsonl, duckdb。カンマ区切りまたは複数回指定で同時に出力する (例: -t csv,duckdb) [デフォルト: csv]

General Options:
  -h, --help  ヘルプメニューを表示する

Display Settings:
  -K, --no-color  カラーで出力しない
  -q, --quiet     Quietモード: 起動バナーを表示しない
```

### `aws-ct-metrics`コマンドの例

* `eventName`のAPIコール数をテーブル形式で出力: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* CSVに保存: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
* トリアージで有用な5つのフィールドを1回のスキャンでまとめて集計: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -F sourceIPAddress,userAgent,userIdentity.arn,awsRegion,userIdentity.accessKeyId`
* ソースIPアドレスにASN・市・国を追加: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -F sourceIPAddress -G ../GeoLite2-DBs`
* CSVとDuckDBデータベースに同時に保存: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -F sourceIPAddress,userAgent -o sample-metrics -t csv,duckdb`

### `aws-ct-metrics`コマンドの出力

画面出力では、フィールドごとに1つのテーブルが表示され、1列目の見出しはそのフィールド名になります。
ファイル出力は`Field`列を持つ1つのフラットなテーブルであり、すべてのフィールドが1つのCSV/JSON/DuckDBファイルにまとまります:

| Field | Value | Count | Percent | FirstSeen | LastSeen | SrcASN | SrcCity | SrcCountry |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| sourceIPAddress | 203.0.113.10 | 1,024 | 62.19% | 2021-07-05 13:03:12 | 2021-07-05 13:03:50 | Example ISP | London | United Kingdom |

CSV・JSON・画面出力では、`SrcASN`・`SrcCity`・`SrcCountry`列は`-G`を指定した場合のみ出力され、IPアドレスとして解釈できる値のみが対象となります(`cloudtrail.amazonaws.com`のようなAWSサービスからの呼び出しは`-`になります)。DuckDB出力では、同じクエリがどのファイルに対しても動くように、この3列は常に出力されます(下記参照)。

DuckDB出力は単一の`metrics`テーブルと[`suzaku_meta`](dfir-timeline.md#duckdb-output-schema)テーブル(実行情報)で構成されるため、`SELECT * FROM metrics WHERE Field = 'sourceIPAddress' ORDER BY Count DESC`のようにそのままクエリできます。値は表示用の文字列ではなく型付きで格納され、テキスト形式には無い2つの列が追加されます:

| 列 | 型 | 説明 |
|---|---|---|
| `Field` | `VARCHAR` | `-F`で指定したCloudTrailのフィールド名そのもの |
| `TimelineColumn` | `VARCHAR` | 同じ情報を保持する`aws-ct-timeline`の列名(`sourceIPAddress` → `SrcIP`)。対応する列が無い場合は`NULL` |
| `Value` | `VARCHAR` | そのイベントが`Field`の値を持たない場合は`NULL` |
| `Count` | `BIGINT` | |
| `FieldTotal` | `BIGINT` | この`Field`で集計されたイベント数、すなわち`Percent`の分母 |
| `Percent` | `DOUBLE` | `Count / FieldTotal`を丸めずに格納。CSVの`62.19%`と異なり、フィールドごとの合計が100になります |
| `FirstSeen` / `LastSeen` | `TIMESTAMP` | |
| `SrcASN` / `SrcCity` / `SrcCountry` | `VARCHAR` | CSVと異なり常に出力されます。`-G`未指定時(`suzaku_meta.geoip_enabled`が`false`)や、値がIPアドレスでない場合は`NULL` |

そのため、丸められた割合を再集計するのではなく、正確な割合をクエリで求められます:

```sql
SELECT Value, Count, Count * 100.0 / FieldTotal AS pct
FROM metrics
WHERE Field = 'sourceIPAddress' AND Value IS NOT NULL
ORDER BY Count DESC LIMIT 10;
```

`TimelineColumn`は`config/aws_profile.yaml`から読み込まれます。`config/`があるディレクトリで実行しない場合、この列は全行`NULL`になります。
