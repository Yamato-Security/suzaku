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

- 不正な `--timeline-start` / `--timeline-end` / `--time-offset` の値を、イベントごとに解析して**全イベント**を無言で読み飛ばす（空のタイムライン・警告なし）のではなく、起動時に明確なエラーで拒否するようにした（例: RFC 3339 形式ではない `--timeline-start 2024-01-01`）。また、空のオフセット・末尾の空白・マルチバイトの末尾文字で `parse_offset` がパニックしていた問題（分割位置をトリム前の長さから求めていた）を修正した。 (#150) (@YamatoSecurity)
- スキャン終了時の「Rule Authors」サマリーで、27バイトを超え24バイト目がマルチバイト文字の途中に来る作者名を切り詰める際に発生していたパニック（`byte index 24 is not a char boundary`）を修正した。日本語などの非ASCII作者名（Sigma ルールパックで一般的）で起きていた。切り詰めをバイトではなく文字単位で行うようにし、完了済みの結果が破棄されないようにした。 (#148) (@YamatoSecurity)
- CSV/表計算ソフトの数式インジェクション（CWE-1236）をレポート出力で無害化した。CSV のセルは攻撃者が影響を与えられるクラウドログのフィールド（`userAgent`、プリンシパル ARN、エラー文字列など）に由来し、`=`・`+`・`-`・`@`・タブ・CR で始まる値は Excel/LibreOffice/Sheets で開いた際に数式として評価されてしまう。これらの値は全ての CSV 出力箇所でアポストロフィを前置（表計算ソフトはテキスト強制マーカーとして扱う）するようにした。JSON/JSONL と標準出力は変更しない。 (#146) (@YamatoSecurity)
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