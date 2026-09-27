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
  -s, --sort                       出力前に結果をタイムスタンプ順に並べ替える (注意: 全結果をメモリに保持する)

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
* 時刻順に並べ替えてCSVに保存: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv -s`

### 結果の並べ替え (`-s, --sort`)

デフォルトでは、結果は見つかった順にそのまま出力されます。そのため出力順はログファイルの走査順に従い、走査順はファイルシステムが決めるので実行ごとに変わることがあります。`-s`を指定すると、画面・CSV・JSON・JSONLの結果を実行の最後まで保持し、時刻順に並べてから出力します。

* イベント時刻を時点(UTCの瞬間)として比較します。`-l, --localtime`を使った場合や、`Z`と`+09:00`のオフセットが混在するログでも正しく並びます。
* 時刻がない行や解釈できない行は末尾に置きます。
* 同じ時刻の行は内容で並べます。同じ検出結果集合・出力設定であれば、ファイルの走査順に関係なく検出行が同じ順序になります。端末出力全体やDuckDBファイルのバイト一致は保証しません。`temporal_ordered`の相関では、同時刻イベントの入力順が検出成否に影響する場合があり、出力のソートでは解決しません。
* 相関ルールの結果は、構成イベントのうち最も新しい時刻の位置に置きます(下記参照)。

DuckDB出力は常に時刻順にソートされるため、`-s`の影響を受けません。DuckDBだけを出力する場合、結果の追加コピーは保持しません。

全結果を最後までメモリに保持するため、`-s`はメモリ使用量が増えます。目安は1結果あたり約1.3KB、`--raw-output`併用時は約3KBです。flaws.cloudのCloudTrailデータ(190万件の検知)では、`-s`なしでも使う5.3GBに加えて、ピークメモリが2.4GB(`--raw-output`併用時は5.5GB)増えました。一方、実行時間の増加は約2〜3%(約70秒の実行で1〜2秒)でした。

**相関ルールのタイムスタンプ:** 構成イベントのうち最も新しい時刻を代表時刻とします。`Timestamp`列はUTC、または`-l`指定時はローカル時刻で表示し、小数秒も保持します。サマリーの日別集計は、`-l`指定時も常にUTCの日付を使います。これは`-s`の有無に関係なく、DuckDBの相関行にも適用されます。

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
