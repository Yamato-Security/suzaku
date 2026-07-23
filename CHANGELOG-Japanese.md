# 変更点

## 2.0.0 [xxxx/xx/xx]

**新機能:**

- Azureログ用のDFIRタイムラインを作成する`azure-timeline`コマンドを追加した。 (#109) (@fukusuket)
- CloudTrailログを検索するための`aws-ct-search`コマンドを追加した。(#117) (@fukusuket)
- UUIDを指定してルールを読み込み対象から除外できる除外リストファイル（`config/aws_ignore_rule_list.txt`）に対応した。これにより、置き換えられた重複ルールをリポジトリに残したまま読み込まないようにできる。 (#136) (@YamatoSecurity)

**改善:**

- `aws-ct-timeline` および `azure-timeline` の出力に `Tags` カラムを追加した。ルールの Sigma `tags` リストを（破棄せずに）Hayabusa のように ` ¦ ` 区切りの1つの文字列として出力する。ATT&CK のタクティクスは Hayabusa と共通の編集可能な `config/mitre_tactics.txt` テーブルを使って略記され（例: `attack.credential-access` は `CredAccess`）、テクニックやグループも短縮される（`attack.t1562.001` は `T1562.001`、`attack.g0035` は `G0035`）。タクティクスのハイフン表記とアンダースコア表記の両方に対応する。JSON 出力では値をフラットな文字列のまま保持する。 (#62) (@YamatoSecurity)
- `aws-ct-timeline` および `azure-timeline` コマンドに、イベントのタイムスタンプを（UTC ではなく）実行環境のローカルタイムゾーンで明示的な UTC オフセット付きで出力する `-l, --localtime` オプションを追加した（例: JST では `2023-07-10 12:27:45` が `2023-07-10 21:27:45+09:00` になる）。解析できないタイムスタンプは従来どおり UTC 表記にフォールバックする。 (#34) (@YamatoSecurity)
- `sigma-rust` をリリース版の `v0.7.1` に更新し、その他の依存クレートもすべて最新版に更新した。`sigma-rust` v0.7.1 は suzaku が利用している Sigma の相関（correlation）機能を維持したまま、YAML バックエンドを非推奨の `serde_yml`/`noyalib`（ルールやイベント中の64ビット符号なし整数の大きな値を精度の落ちた浮動小数点として解析していた）から、活発にメンテナンスされている `yaml_serde` に移行し、`u64` の正しい解析を回復した。 (@YamatoSecurity)
- `azure-timeline` が SigmaHQ の Microsoft 365 ルールを読み込み・マッチできるようにした。これらのルールは `logsource.service` を `audit`/`exchange`/`threat_detection`/`threat_management` として宣言しているが、従来は `m365` しか認識されず、アップストリームの m365 ルールがすべて読み込み時に破棄されていた。これらのサービスを `m365` と同じ Unified Audit Log の判別（`Workload`/`RecordType`）で振り分けるようにした。 (#137) (@YamatoSecurity)
- 異なるログソースの取り扱いを容易にするため、コードをリファクタリングした。 (@fukusuket)
- Microsoft Graph API JSON形式のAzureログに対応した。 (#113) (@fukusuket)
- 既存の `--timeline-start/--timeline-end` オプション（ファイル内のイベントタイムスタンプに基づいて動作する）とは異なり、S3キーの日付プレフィックスに基づいてオブジェクトをフィルタリングする `--file-date-from/--file-date-to` オプションを追加した。 (#118) (@fukusuket)
- `aws-ct-summary`コマンドに、JSON形式で出力するための`-output-type`オプションを追加した。 (#123) (@fukusuket)

**バグ修正:**

- `Results Summary` の「Data reduction」行が、デバッグビルドでパニック（`attempt to subtract with overflow`）していた問題を修正した（リリースビルドでは約 1.8×10¹⁹ という無意味な件数、空入力では `NaN%` を表示していた）。相関（correlation）結果はベーススキャンで既に数えたイベントに対して `event_with_hits` を再度加算するため、`event_with_hits` が `total_events` を上回ることがあった。件数を飽和減算（saturating）で計算し、パーセンテージも空データセットに対してガードするようにした。 (#163) (@YamatoSecurity)
- `aws-ct-summary` が各エントリの時間範囲を誤って報告していた問題を修正した。集計対象のリージョン・送信元IP・アクセスキー・ユーザーエージェント・API それぞれの `first_seen`/`last_seen` が、キーを最初に挿入した時点のデータセット全体の最小/最大値で一度だけ設定され、その後更新されていなかったため、そのエントリ自身の初回/最終発生時刻ではなくデータセット全体の範囲を表示していた。各エントリが、実際にそのキーに該当したイベントの初回/最終時刻を追跡するようにした。 (#160) (@YamatoSecurity)
- 読み込めなかった入力ファイル（アクセス権限なし・UTF-8 として不正な内容・スキャン中に削除・破損した/サイズ超過の `.gz`）をスキャンがスキップする際に、無言で読み飛ばすのではなく警告（`[WARNING] Skipping <file>: <reason>`）を表示するようにした。従来はそのようなファイルも合計ファイル数には数えられつつ無言でスキップされ、報告されるカバレッジが過大になっていた。ディレクトリスキャンと単一ファイル入力の両方に適用され、gzip のサイズ上限警告もこの呼び出し側の1箇所に集約した。 (#161) (@YamatoSecurity)
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