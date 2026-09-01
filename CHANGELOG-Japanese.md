# 変更点

## 2.0.1 [2026/08/24] - El Niño Release

**バグ修正:**

- `-G, --geo-ip` で一致したタイムラインイベントを GeoIP エンリッチメントしていても、`--raw-output` は `SrcASN`・`SrcCity`・`SrcCountry` を出力せず、GeoIP が無効であるかのように見えていた。生のターミナル出力、JSON、JSONL が同じプロファイル由来のエンリッチメント経路を使用するようにし、元のイベントフィールドと Sigma メタデータを保持したまま GeoIP フィールドを追加するようにした。 (#184) (@vitorparras)
- イベントのタイムスタンプ取得時に、出力プロファイルのフィールド指定をそのまま `Event::get` に渡していた問題を修正した。プロファイルの指定は先頭にドットが付き `|` 区切りのフォールバックリストにもなる（`.eventTime`、`.time|.eventTimestamp|.CreationTime`）一方、`Event::get` はドットなしのフィールド名を受け取るため、常に `None` が返っていた。この影響で、サマリーの `Dates with most total detections` が `n/a` となり、`first_event_time`/`last_event_time` が設定されず、さらに **Sigma の相関（correlation）ルールが一切発火しなかった**（同梱の相関ルール5本がすべて無効）。該当する3箇所すべてで、`Timestamp` カラムと同じ「存在する最初の候補を採用する」ルールの共通ヘルパー経由で解決するようにし、表示値と分析値が食い違わないようにした。 (#191) (@fukusuket)

**その他:**

- リリースに向けて`Cargo.lock`を更新した。宣言されているMSRV（`rust-version = "1.97.1"`）と互換性のある最新バージョンへ59パッケージを更新した。主なものは`arrow`/`parquet` 59.1 → 59.2、`clap` 4.6.5 → 4.6.6、`maxminddb` 0.30.0 → 0.30.3、`duckdb` 1.10504 → 1.10505、`libgit2-sys` 1.9.6 → 1.9.7、および`regex`・`serde_json`・`tempfile`・`rayon`・`icu_*`系である。`Cargo.toml`のバージョン指定は一切変更していないため、すべての更新は#189で設定した下限の範囲内に収まっている。なお`comfy-table` 8.0.0への更新は意図的に見送った。テーブルAPIが再設計されており（`modifiers`モジュールと`load_preset`が削除され、`TableStyle`の構造も変更）、Suzakuが描画するすべてのテーブルに手を入れるコード移行が必要になるため、パッチリリースに含めるべき変更ではないと判断した。 (@YamatoSecurity)

## 2.0.0 [2026/07/31] - Black Hat Arsenal USA 2026 Release

**新機能:**

- Azureログ用のDFIRタイムラインを作成する`azure-timeline`コマンドを追加した。 (#109) (@fukusuket)
- CloudTrailログを検索するための`aws-ct-search`コマンドを追加した。(#117) (@fukusuket)
- UUIDを指定してルールを読み込み対象から除外できる除外リストファイル（`config/aws_ignore_rule_list.txt`）に対応した。これにより、置き換えられた重複ルールをリポジトリに残したまま読み込まないようにできる。 (#136) (@YamatoSecurity)
- AWS系コマンド（`aws-ct-timeline`、`aws-ct-metrics`、`aws-ct-summary`、`aws-ct-search`）の `-f` と `-d` の両方で Parquet（`.parquet`）入力に対応した。各行を1つのJSONイベントに変換する。ネストされたstruct型カラムはネストされたオブジェクトになり、JSON文字列として格納されたCloudTrailの既知のエンベロープフィールド（`userIdentity`、`requestParameters`、`responseElements` など。Athena CTAS / Glue / Firehose パイプラインが生成する形式）はオブジェクトに復元してルールがネストされた値にマッチできるようにし、タイムゾーンなしの `eventTime` TIMESTAMPカラムはUTCとしてマークして時刻フィルタとサマリが機能するようにした。snappy、gzip、zstd、lz4圧縮に対応。 (@nishikawaakira)

**改善:**

- 10個の依存クレートのバージョン指定をワイルドカード（`"*"`）から明示的な下限付きに変更した。これにより、将来の破壊的変更を含むリリースが`cargo update`で自動的に取り込まれることがなくなり、意図的なバージョン更新が必要になる。うち6個は1.0未満のクレートであり、Cargoのセマンティックバージョニングではマイナーバージョンの更新がすべて破壊的変更として扱われる。これは#131で実際に発生した問題であり、ワイルドカード指定の`cidr-utils`が省略形のCIDR表記を受け付けないバックエンドに更新された結果、Suzaku側は何も変更していないにもかかわらず`--geo-ip`が起動時にパニックしていた。解決されるバージョンは変わらない（下限指定の有無にかかわらず`cargo update`は同一の`Cargo.lock`を生成する）。 (#189) (@YamatoSecurity)
- 実行を終了するエラーの表示色をすべて赤に統一し、`--no-color` にも対応させた。従来は色が統一されておらず、一部（存在しない入力パス、不正な `-m, --min-level`、ルールフォルダや出力プロファイルの未検出、`-C` なしでの出力ファイル上書き、`update-rules` の失敗）はターミナルの既定色で表示され、逆に一部は `--no-color` を無視する赤色のハードコードになっていた。これらを `fatal_error()` と共通の `error_msg()` ヘルパーに集約した。 (@fukusuket)
- ログ処理中に発生した警告・エラー（読み込めずスキップしたファイル、ディレクトリスキャンの失敗、相関イベント処理のエラー、悪用AWS APIリストの読み込み失敗など）を、ターミナルではなく `logs/errorlog-<YYYYMMDD_HHMMSS>.log` に出力するようにした。これによりプログレスバーの表示が崩れたり、結果サマリーがメッセージで埋もれたりしなくなる。ファイルは出力すべきメッセージがある場合のみ作成され、先頭行には実行したコマンドラインを出力する。実行の最後に `Warnings and errors: <件数> messages saved to <パス>` の1行のみを表示する。実行を中断する致命的なエラーは従来どおりターミナルにも表示する。 (@fukusuket)
- `aws-ct-summary` の `-t, --output-type` をタイムライン系コマンドと同様に形式**名**で指定する方式に変更し、**DuckDB** 出力を追加した。`csv`・`json`・`jsonl`・`duckdb` をカンマ区切り（または繰り返し）で指定できる（例: `-t csv,duckdb`）。CSV では各プリンシパルの API 呼び出しや属性が複数行のセルにまとめられており SQL で扱えないため、DuckDB 出力では**リレーショナル**な3テーブル構成にした。`summary`（プリンシパルごとに1行）、`summary_api_calls`（プリンシパルと API ごとに1行、`abused_success`/`abused_failed`/`other_success`/`other_failed` のラベル付き）、`summary_attributes`（プリンシパルと値ごとに1行、`aws_region`/`src_ip`/`access_key_id`/`user_agent` のラベル付き）である。これにより、悪用された API を呼び出したプリンシパルがどの送信元 IP を使っていたかといった、CSV では答えられない問いが通常の JOIN で解けるようになる。（破壊的変更: 数字 `-t 1..5` は他コマンドと同様に名前指定へ置き換えられた。） (@YamatoSecurity)
- `aws-ct-timeline`・`azure-timeline` の `-t, --output-type` を数字ではなく形式**名**を指定する方式に変更し、**DuckDB** 出力を追加した。`csv`・`json`・`jsonl`・`duckdb` をカンマ区切り（または繰り返し）で指定して、任意の組み合わせを同時に出力できる（例: `-t csv,duckdb`）。DuckDB 出力は、出力プロファイルの各項目を列に持つ `timeline` テーブルを含む自己完結型の `.duckdb` データベースファイルである。（破壊的変更: 数字 `-t 1..5` は名前指定に置き換えられた。`aws-ct-search` は同じオプションを共有するため同様に名前形式に対応する。） (@YamatoSecurity)
- `aws-ct-timeline` および `azure-timeline` の出力に `Tags` カラムを追加した。ルールの Sigma `tags` リストを（破棄せずに）Hayabusa のように ` ¦ ` 区切りの1つの文字列として出力する。ATT&CK のタクティクスは Hayabusa と共通の編集可能な `config/mitre_tactics.txt` テーブルを使って略記され（例: `attack.credential-access` は `CredAccess`）、テクニックやグループも短縮される（`attack.t1562.001` は `T1562.001`、`attack.g0035` は `G0035`）。タクティクスのハイフン表記とアンダースコア表記の両方に対応する。JSON 出力では値をフラットな文字列のまま保持する。 (#62) (@YamatoSecurity)
- `aws-ct-timeline` および `azure-timeline` コマンドに、イベントのタイムスタンプを（UTC ではなく）実行環境のローカルタイムゾーンで明示的な UTC オフセット付きで出力する `-l, --localtime` オプションを追加した（例: JST では `2023-07-10 12:27:45` が `2023-07-10 21:27:45+09:00` になる）。解析できないタイムスタンプは従来どおり UTC 表記にフォールバックする。 (#34) (@YamatoSecurity)
- `sigma-rust` をリリース版の `v0.7.1` に更新し、その他の依存クレートもすべて最新版に更新した。`sigma-rust` v0.7.1 は suzaku が利用している Sigma の相関（correlation）機能を維持したまま、YAML バックエンドを非推奨の `serde_yml`/`noyalib`（ルールやイベント中の64ビット符号なし整数の大きな値を精度の落ちた浮動小数点として解析していた）から、活発にメンテナンスされている `yaml_serde` に移行し、`u64` の正しい解析を回復した。 (@YamatoSecurity)
- `azure-timeline` が SigmaHQ の Microsoft 365 ルールを読み込み・マッチできるようにした。これらのルールは `logsource.service` を `audit`/`exchange`/`threat_detection`/`threat_management` として宣言しているが、従来は `m365` しか認識されず、アップストリームの m365 ルールがすべて読み込み時に破棄されていた。これらのサービスを `m365` と同じ Unified Audit Log の判別（`Workload`/`RecordType`）で振り分けるようにした。 (#137) (@YamatoSecurity)
- 異なるログソースの取り扱いを容易にするため、コードをリファクタリングした。 (@fukusuket)
- Microsoft Graph API JSON形式のAzureログに対応した。 (#113) (@fukusuket)
- `azure-timeline`が、Azure Monitorの診断設定BLOBやEvent Hubメッセージで使われる`{ "records": [...] }`形式のバッチエンベロープを展開するようにした（ファイル全体・行単位の両方に対応）。これにより、これらのエクスポートが単一のイベントではなくレコード単位で読み込まれるようになった。また、従来は読み込み時に破棄されていた`identity_protection`（`riskdetection`）および`privileged_identity_management`（`pim`）のルール種別も読み込み・マッチできるようにした。 (#130) (@YamatoSecurity)

- `azure-timeline`をMicrosoft 365統合監査ログ（Unified Audit Log）に対応させた。`AuditData`列・ラッパーを展開することで`Search-UnifiedAuditLog`のCSVエクスポート（およびJSON）を読み込み、UALのName/Value形式のプロパティバッグ（`ExtendedProperties`/`Parameters`など）をオブジェクトに変換してルールがネストされた値（例: `ExtendedProperties.UserAgent`）にマッチできるようにし、単一・整形済みのレコードオブジェクトも解析できるようにした。あわせて、時刻フィルタが未指定の場合にイベントが破棄されないよう修正し、`CreationTime`のタイムスタンプを解析できるようにし、ログソースのサービスとして`m365`を追加した。Azureの出力プロファイルも、従来のAzure Monitor専用の空欄になっていたカラムに代わり、DFIRで有用なM365のフィールド（`Workload`、`Operation`、`Result`、`User`、`SrcIP`、`TargetObject`、`UserAgent`、`AppId`、`LogonError`、および変更内容の`Parameters`/`ModifiedProperties`をまとめた`Details`）を出力するようにした。 (#129) (@YamatoSecurity)

- 既存の `--timeline-start/--timeline-end` オプション（ファイル内のイベントタイムスタンプに基づいて動作する）とは異なり、S3キーの日付プレフィックスに基づいてオブジェクトをフィルタリングする `--file-date-from/--file-date-to` オプションを追加した。 (#118) (@fukusuket)
- `aws-ct-summary`コマンドに、JSON形式で出力するための`-output-type`オプションを追加した。 (#123) (@fukusuket)
- `aws-ct-metrics`で複数フィールドを1回のスキャンでまとめて集計し、結果を拡充できるようにした。`-F, --field-name`はカンマ区切りのリストを受け付け（例: `-F sourceIPAddress,userAgent,userIdentity.arn,awsRegion,userIdentity.accessKeyId`）、フィールドごとにフルスキャンを繰り返すのではなく、1回のスキャンですべてのフィールドを集計する。各値について、その値の`FirstSeen`/`LastSeen`も出力するようにした。`-G, --geo-ip`を指定すると、IPアドレスとして解釈できる値に`SrcASN`/`SrcCity`/`SrcCountry`列が追加される。`-t, --output-type`でcsv/json/jsonl/duckdbを出力できる（DuckDB出力はそのままクエリできる単一の`metrics`テーブル）。`-s, --include-sts-keys`は`aws-ct-summary`と同様で、一時的な`ASIA...`のSTSキーは指定しない限り除外する。また、指定したフィールドを持たないイベントは破棄せず`-`として集計するようにしたため、割合はスキャンした全イベントに対する比率になる。さらに`-F`はスキャン開始前にCloudTrailのレコードフィールドと照合して検証するようにした。大文字小文字を誤った名前は、データセット全体をスキャンして全イベントを`-`として報告する代わりに、`'sourceIPaddress' is not a CloudTrail field. Did you mean 'sourceIPAddress'?`として即座に失敗する。APIごとに内容が異なるコンテナ（`requestParameters`・`responseElements`・`additionalEventData`など）配下の任意のパスは従来どおりそのまま指定できる。（破壊的変更: CSVの列が`EventName,Percent,Total`から`Field,Value,Count,Percent,FirstSeen,LastSeen`に変更された。） (@fukusuket)

**バグ修正:**

- `--geo-ip`がすべてのグローバルユニキャストIPv6アドレスを`Private`として報告し、データベースを参照していなかった問題を修正した。IPv6のグローバルユニキャスト空間全体である`2000::/3`がプライベート範囲として登録されていたため、日本のISPやAzureサインイン元などのパブリックアドレスが`SrcASN: Private` / `SrcCountry: -`と表示されていた。これは単に位置情報が欠けるだけでなく、事実と異なる情報を提示するものであり、分析者が`Private`を見て内部通信だと誤認する原因となる。IPv6もIPv4と同様に位置情報を付与するようにした。未指定・リンクローカル・ユニークローカル・マルチキャストアドレスは従来どおりプレースホルダーのままとなる。 (#182) (@YamatoSecurity)
- プロファイルの`SrcIP`フォールバックリストにおいて、先に並ぶフィールドが存在するものの利用できない値だった場合に`--geo-ip`の位置情報カラムが空になる問題を修正した。従来は*存在する*最初の候補を採用してから解析を試みていたため、`claims.ipaddr`が`""`、`"-"`、`host:port`形式（M365の`ClientIP`でよくある形式）などIPアドレスとして解析できない値を持っていると探索がそこで終了し、`callerIpAddress`にある有効なアドレスが参照されなかった。その結果、公開IPを含むレコードでも`SrcASN`/`SrcCity`/`SrcCountry`が`-`と表示されていた。IPアドレスとして解析できる最初の候補を採用するようにした。また`SrcIP`カラムも同じ選択ロジックを使うようにしたため、表示されるアドレスと位置情報を取得したアドレスが常に一致する。どの候補も解析できない場合は、`SrcIP`にはログに記録された値（AWSサービスによるイベントの`cloudtrail.amazonaws.com`など）がそのまま表示され、位置情報カラムは`-`となる。 (#183) (@YamatoSecurity)
- `aws-ct-metrics`のテーブルおよびCSVの1列目の見出しが、`-F`で別のフィールドを集計した場合でも常に`EventName`になっていた問題を修正した（`sourceIPAddress`を集計すると、IPアドレスの列に`EventName`という見出しが付いていた）。実際に集計したフィールド名を見出しに使用するようにした。また、`-o`で既存ファイルを無言で上書きしていたため、他のコマンドと同様に上書きには`-C, --clobber`を必要とするようにした。さらに、件数が同じ値は値でソートするようにしたため、`HashMap`の順序で行が入れ替わることがなくなり、同じ入力に対して常に同じ出力になる。 (@fukusuket)
- `Results Summary` の「Data reduction」行が、デバッグビルドでパニック（`attempt to subtract with overflow`）していた問題を修正した（リリースビルドでは約 1.8×10¹⁹ という無意味な件数、空入力では `NaN%` を表示していた）。相関（correlation）結果はベーススキャンで既に数えたイベントに対して `event_with_hits` を再度加算するため、`event_with_hits` が `total_events` を上回ることがあった。件数を飽和減算（saturating）で計算し、パーセンテージも空データセットに対してガードするようにした。 (#163) (@YamatoSecurity)
- `aws-ct-summary` が各エントリの時間範囲を誤って報告していた問題を修正した。集計対象のリージョン・送信元IP・アクセスキー・ユーザーエージェント・API それぞれの `first_seen`/`last_seen` が、キーを最初に挿入した時点のデータセット全体の最小/最大値で一度だけ設定され、その後更新されていなかったため、そのエントリ自身の初回/最終発生時刻ではなくデータセット全体の範囲を表示していた。各エントリが、実際にそのキーに該当したイベントの初回/最終時刻を追跡するようにした。 (#160) (@YamatoSecurity)
- 読み込めなかった入力ファイル（アクセス権限なし・UTF-8 として不正な内容・スキャン中に削除・破損した/サイズ超過の `.gz`）をスキャンがスキップする際に、無言で読み飛ばすのではなく警告（`[WARNING] Skipping <file>: <reason>`）を表示するようにした。従来はそのようなファイルも合計ファイル数には数えられつつ無言でスキップされ、報告されるカバレッジが過大になっていた。ディレクトリスキャンと単一ファイル入力の両方に適用され、gzip のサイズ上限警告もこの呼び出し側の1箇所に集約した。 (#161) (@YamatoSecurity)
- `aws-ct-search` が不正な `--regex` 値でパニックしていた問題を修正した（`.expect()` でコンパイルしていた）。不正なパターンは明確なエラーを表示してクリーンに終了するようにした。また、`FIELD:VALUE` のコロンを欠いた不正な `--filter` は、従来は無言で無視されていたが、起動時に拒否するようにした。さらに、`aws-ct-summary` が abused AWS API リストを開けなかった際の警告に、探索したパスと「全 API 呼び出しが非不正として分類される」旨を明記するようにした。 (#162) (@YamatoSecurity)
- `--geo-ip` によるエンリッチメント（`SrcASN`/`SrcCity`/`SrcCountry`）が `azure-timeline`/M365 ログでは無言で機能していなかった問題を修正した。送信元 IP を AWS CloudTrail にしか存在しない `sourceIPAddress` フィールドから固定で解決していたため、`SrcIP` を `callerIpAddress`/`ClientIP` などにマッピングする Azure/M365 プロファイルでは、ルーティング可能なパブリック IP を持つイベントでもこれらの列が常に `-` になっていた。送信元 IP を（他の列と同じ `|` 区切りのフォールバックで）プロファイルの `SrcIP` フィールド仕様から解決するようにし、Azure/M365 でも AWS と同様にエンリッチメントされるようにした。 (#159) (@YamatoSecurity)
- 不正な `--timeline-start` / `--timeline-end` / `--time-offset` の値を、イベントごとに解析して**全イベント**を無言で読み飛ばす（空のタイムライン・警告なし）のではなく、起動時に明確なエラーで拒否するようにした（例: RFC 3339 形式ではない `--timeline-start 2024-01-01`）。また、空のオフセット・末尾の空白・マルチバイトの末尾文字で `parse_offset` がパニックしていた問題（分割位置をトリム前の長さから求めていた）を修正した。 (#150) (@YamatoSecurity)
- スキャン終了時の「Rule Authors」サマリーで、27バイトを超え24バイト目がマルチバイト文字の途中に来る作者名を切り詰める際に発生していたパニック（`byte index 24 is not a char boundary`）を修正した。日本語などの非ASCII作者名（Sigma ルールパックで一般的）で起きていた。切り詰めをバイトではなく文字単位で行うようにし、完了済みの結果が破棄されないようにした。 (#148) (@YamatoSecurity)
- CSV/表計算ソフトの数式インジェクション（CWE-1236）をレポート出力で無害化した。CSV のセルは攻撃者が影響を与えられるクラウドログのフィールド（`userAgent`、プリンシパル ARN、エラー文字列など）に由来し、`=`・`+`・`-`・`@`・タブ・CR で始まる値は Excel/LibreOffice/Sheets で開いた際に数式として評価されてしまう。これらの値は全ての CSV 出力箇所でアポストロフィを前置（表計算ソフトはテキスト強制マーカーとして扱う）するようにした。JSON/JSONL と標準出力は変更しない。 (#146) (@YamatoSecurity)
- gzip の展開サイズを制限し、展開爆弾（decompression bomb）による OOM を防いだ。スキャン対象ツリー内の細工された・破損した `.gz` が数GB（DEFLATE は約1032:1）に展開し、スキャン全体が OOM で強制終了される可能性があった。`.gz` 入力は展開後 3 GiB を上限とし、上限を超えるファイルは実行を中断せず警告を表示してスキップするようにした。 (#147) (@YamatoSecurity)
- `--geo-ip` がタイムライン出力を破壊していた問題を修正した。レコードの `sourceIPAddress` が解析可能な IP アドレスでない場合（`cloudtrail.amazonaws.com` などの AWS サービスイベントでは一般的）、GeoIP ルックアップがその生の文字列を*すべて*の出力列に返し、`Timestamp`・`EventName`・`RuleTitle` などを上書きしていた。エンリッチメントを `SrcASN`・`SrcCity`・`SrcCountry` の3列のみに限定し、アドレスを解決できない場合は `-` を出力するようにした。 (#145) (@YamatoSecurity)
- スキャンをRustのパニックとバックトレースで中断させていた入力・ファイルシステム関連のエッジケースを堅牢化した。スキャン対象ツリー内のUTF-8として不正なファイル名（結果が出る前の初期ファイル数カウントを中断させていた）、読み取り不可のサブディレクトリやスキャン中に削除されたファイル、書き込み不可の `--output` パスは、パニックの代わりにクリーンで実行可能なエラーを表示するようになった。ウォークのエラーは報告され、出力エラーは `Cannot write to output file …` を表示して非ゼロで終了する。`aws-ct-timeline`・`azure-timeline`・`aws-ct-search`・`aws-ct-metrics`・`aws-ct-summary` に適用される。 また、UTF-8 として不正なファイル名は（実パスをスキャン処理全体で保持することで）カウントだけされてスキップされるのではなく実際に読み込まれるようになり、`aws-ct-summary` の JSON/JSONL 出力でも書き込み不可の `--output` パスをクリーンに報告するようにした。 (#149) (@YamatoSecurity)
- `aws-ct-timeline`・`aws-ct-metrics`・`aws-ct-search`・`aws-ct-summary` コマンドが JSONL 入力（1行に1つの CloudTrail イベント、または `{ "Records": [...] }` バッチ）を無言で読み飛ばしていた問題を修正した。パーサーはファイル全体を単一の JSON として読み込み、失敗するとイベントを1件も返していなかった。行単位の JSONL 解析にフォールバックするようにし、`.jsonl` 拡張子のファイルも認識・読み込みできるようにした。 (#139) (@YamatoSecurity)
- `-T, --no-frequency-timeline`オプションが機能していなかったため削除した。また、作者表示のロジックバグを修正した。 (#110) (@fukusuket)
- 結果がなくても出力ファイルは保存されていた。 (#114) (@fukusuket)
- `aws-ct-summary`は、破損または不完全なログファイルを処理する際にパニックを起こしていた。 (#119) (@fukusuket)

- `--geo-ip`が起動時に`invalid IP address syntax`でパニックを起こしていた。プライベートIP判定に使っていた省略形のCIDR文字列（`10/8`、`172.16/12`、`2000::/3`など）が`cidr`クレートで受け付けられなくなったことが原因である。`cidr-utils`への依存をやめ、IPv4は標準ライブラリの`Ipv4Addr::is_private()`で、IPv6は手動のプレフィックス判定でプライベート範囲を確認するようにした。あわせて、それまで使われていなかったGeoIPの国・都市キャッシュを利用するようにした。 (#132) (@fukusuket)

## 1.1.0 [2025/08/14] - Obon Release

**改善:**

- `-R, --raw-output` は、`-o` が指定されていない場合に、ターミナルに生のログを出力する。(#101) (@fukusuket)

## 1.0.1 [2025/08/07] - Black Hat Arsenal USA 2025 Release

**バグ修正:**

- 無効なファイルやディレクトリ入力に対するエラー処理の改善 (#99) (@fukusuket)

## 1.0.0 [2025/07/31] - Black Hat Arsenal USA 2025 Release

**新機能:**

- `aws-ct-timeline`コマンドで相関ルール(`event_count`、`value_count`、`temporal`、`temporal_order`)に対応した。(#97) (@fukusuket)

**改善:**

- レベル名は`aws-ct-timeline`で省略されるようになった。(#68) (@fukusuket)
- ルールが見つからない場合は、エラーメッセージを出力するようになった。 (#76) (@fukusuket)
- `aws-ct-timeline`コマンドに`--timeline-offset`、`--timeline-start`、`--timeline-end`オプションを追加した。 (#58) (@fukusuket)
- `aws-ct-timeline`コマンドは、マルチスレッドに対応した。 (#32, #93) (@hach1yon)

## 0.2.1 [2025/05/25] - AUSCERT/SINCON Release 2

- リリース名を修正し、readmeを更新した。 (@yamatosecurity)

## 0.2.0 [2025/05/22] - AUSCERT Release

**新機能:**

- `aws-ct-summary`: 一意のARNごとに、イベント総数、使用地域、ユーザータイプ、アクセスキー、ユーザーエージェントなどのサマリーを作成する。 (#53) (@fukusuket)

**改善:**

- `aws-ct-timeline`と`aws-ct-summary`コマンド結果の送信元IPアドレスにMaxmindのジオロケーション情報を追加した。(#16)(@fukusuket)
- `--aws-ct-timeline`コマンドに`-R, --raw-output`オプションを追加し、検出があった場合に元々のJSONデータを出力するようにした。 (#67) (@fukusuket)

**バグ修正s:**

- `aws-ct-metrics`コマンドのCSVヘッダーは正しくなかった。 (#72) (@fukusuket)

## 0.1.1 [2025/04/24] - AlphaOne Release

**バグ修正:**

- いくつかのSigmaフィールド情報が正しく出力されなかった。 (#61) (@fukusuket)

# 最初リリース

## 0.1.0 [2025/04/20] - AlphaOne Release

**新機能:**

- `aws-ct-metrics`: AWS CloudTrailイベントを集計する
- `aws-ct-timeline`: AWS CloudTrailログでSigmaルールを使って攻撃の痕跡を検出する
- `update-rules`: Sigmaルールの更新