# 検索コマンド

## `aws-ct-search`コマンド

このコマンドは、Sigmaルールを書かずに、AWS CloudTrailログから目的のイベントを抽出するために使用します。
マッチしたイベントは`aws-ct-timeline`と同じ形式で表示・保存されるため、検索結果をそのまま見慣れたタイムラインの列で確認できます。

イベントの絞り込み方法は次の3つで、組み合わせて使用できます。

| オプション | マッチ対象 | 複数指定 |
|---|---|---|
| `-F, --filter FIELD:VALUE` | 1つのフィールドの値に対する**完全一致(大文字・小文字を区別)** | `-F`を繰り返し指定。**すべて**の条件にマッチする必要がある(AND) |
| `-k, --keyword KEYWORD` | イベントのJSON全体に対する部分一致。大文字・小文字を区別しない(`-c`指定時は区別する) | `-k`を繰り返し指定。**いずれか**にマッチすればよい(OR) |
| `-r, --regex REGEX` | イベントのJSON全体に対する正規表現 | 1つのパターン(複数の候補はエスケープしないパイプ文字<code>&#124;</code>で指定) |

各イベントは次の順に判定され、すべてを通過したイベントだけが出力されます: 時間範囲(`--timeline-start`、`--timeline-end`、`--time-offset`) → `-F` → `-k` → `-r`。

値が入っているフィールドが分かっている場合(例: `eventName:ConsoleLogin`)は`-F`を、文字列は分かるがどこに現れるか分からない場合は`-k`または`-r`を使用してください。

### `-F, --filter`の仕様

* 書式は`FIELD:VALUE`です。引数は**最初の**`:`で分割されるため、ARNのように値に`:`を含む場合もそのまま指定できます。
* ネストしたフィールドはドット記法で指定します(`userIdentity.type`)。先頭の`.`は省略可能なため、`.userIdentity.arn`でも指定できます。
* 値を囲む引用符(`"..."`や`'...'`)は取り除かれます。
* 数値や真偽値は文字列として比較されるため、`-F readOnly:false`や`-F responseElements.user.userId:12345`のように指定できます。
* 指定したフィールドを持たないイベントはマッチしません。
* 配列内の要素は指定できません(`items.0.name`のような記法はありません)。
* `:`を含まないフィルタは、スキャン開始前に`Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`というエラーで拒否されます。

> 注意: `aws-ct-metrics`の`-F`は`--field-name`で、フィールド名のみを指定する別のオプションです。`aws-ct-search`の`-F`は`--filter`で、常に`FIELD:VALUE`を指定します。

## コマンド使用例
```
Usage: suzaku aws-ct-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  複数gz/json/parquetファイルのディレクトリパス
  -f, --file <FILE>      gz/json/parquetファイルのパス

Filtering:
  -F, --filter <FILTER...>     特定のフィールドで絞り込む
  -c, --preserve-case          大文字・小文字を区別してキーワード検索する
  -k, --keyword <KEYWORD...>   キーワードで検索する
  -r, --regex <REGEX>          正規表現で検索する
      --timeline-start <DATE>  読み込むイベントの開始時刻 (例: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    読み込むイベントの終了時刻 (例: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   オフセットに基づいて直近のイベントをスキャン (例: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  AWSLogsのS3パスの日付構造に基づいて開始日でファイルを絞り込む (例: "20240101")
      --file-date-to <DATE>    AWSLogsのS3パスの日付構造に基づいて終了日でファイルを絞り込む (例: "20241231")

Output:
  -C, --clobber                   保存時にファイルを上書きする
  -G, --geo-ip <MAXMIND-DB-DIR>   IPアドレスにGeoIP情報(ASN、市、国)を追加する
  -o, --output <FILE>             結果をファイルに保存する
  -t, --output-type <FORMAT,...>  出力形式(-o指定時のみ有効): csv (デフォルト), json, jsonl, duckdb。カンマ区切りまたは複数回指定で同時に出力する (例: -t csv,duckdb) [デフォルト: csv]
      --raw-output                元のJSONログを出力する (JSON形式または標準出力のみ)
      --threads <THREAD NUMBER>   使用するスレッド数 (デフォルト: CPUコア数)

General Options:
  -h, --help  ヘルプメニューを表示する

Display Settings:
  -K, --no-color  カラーで出力しない
  -q, --quiet     Quietモード: 起動バナーを表示しない
```

### `aws-ct-search`コマンドの例

フィールドによる絞り込み(`-F`):

* コンソールログインを抽出: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* rootユーザーによるAPI呼び出しを抽出(ネストしたフィールド): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* 特定のIAMユーザーの操作をすべて抽出(値に`:`を含む): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* 拒否された呼び出しを抽出: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* `us-east-1`での読み取り専用ではないIAMの呼び出しを抽出(複数の条件はAND): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* 特定のS3バケットへのアクセスを抽出(APIごとに異なるフィールド): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

キーワード(`-k`)と正規表現(`-r`)による検索:

* IPアドレスがどこかに含まれるイベントを抽出: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* 2人のユーザーのいずれかが含まれるイベントを抽出(複数のキーワードはOR): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* 大文字・小文字を区別してキーワード検索: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* CloudTrailを無効化しようとした操作を抽出: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

条件の組み合わせと結果の保存:

* 1つのイベントソースに限定してキーワード検索: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* 直近7日間の失敗したコンソールログインを抽出: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* GeoIP情報を付けてCSVとDuckDBに保存: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* マッチしたイベントの元のJSONを保存: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search`の出力

出力される列は[`aws-ct-timeline`](dfir-timeline.md)と同じで、DuckDB出力も同じ[DuckDB出力スキーマ](dfir-timeline.md#duckdb-output-schema)に従います。
最後に、スキャンしたイベント数とマッチしたイベント数が表示されます:

```
Total events scanned: 209
Matching events: 1
```
