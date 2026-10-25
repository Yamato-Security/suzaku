# AWSコンソールのCSVを読み込む

SuzakuのAWS系コマンド（`aws-ct-timeline`、`aws-ct-metrics`、`aws-ct-summary`、`aws-ct-search`）は、AWS Management Consoleの **CloudTrail → イベント履歴 → イベントのダウンロード → CSV** で取得したCSVファイルを、`-f/--file` と `-d/--directory` の両方で読み込めます。

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/
```

CSVはS3にログを配信する証跡が無い場合の手段としては便利ですが、CloudTrailのレコードそのものではなく、各イベントの**要約**です。証跡のJSONログ（またはイベント履歴のJSONダウンロード）がある場合は、そちらを使ってください。

## 行の変換

各行はCloudTrailのレコードに組み立て直されるため、ルールやsummary・metricsコマンドはJSON入力と同じフィールド名を使います。

| CSVの列 | CloudTrailのフィールド | 備考 |
|---|---|---|
| `Event time` | `eventTime` | 空の行はスキップする |
| `Event source` | `eventSource` | |
| `Event name` | `eventName` | |
| `AWS region` | `awsRegion` | |
| `Source IP address` | `sourceIPAddress` | |
| `User agent` | `userAgent` | |
| `Error code` | `errorCode` | |
| `User name` | `userIdentity.userName` | ロールの場合はセッション名。`root`の場合は`userIdentity.type: Root`も補う |
| `AWS access key` | `userIdentity.accessKeyId` | |
| `Resources` | `resources[]` | `resourceName` → `ARN`、`resourceType` → `type` |
| `Read-only` | `readOnly` | `true`/`false` → 真偽値 |
| `Request ID` | `requestID` | |
| `Event ID` | `eventID` | |
| `Event type` | `eventType` | |
| `Recipient Account Id` | `recipientAccountId` | |
| `Event category` | `eventCategory` | |

空欄のセルはフィールドなしとして扱うため、`errorCode|exists`を条件とするルールが成功した呼び出しにマッチすることはありません。この表に無い列は、列名のまま残します。

対応しているのは英語の列名だけです。コンソールのCloudTrail CSVではないCSVは読み込みません。`-f`では理由を表示し、`-d`ではそのファイルをスキップしてスキャン済みに数えず、理由をエラーログに記録します。フィールド数が合わない行（多くは引用符の崩れ）はスキップし、行単位の問題はすべて行番号とともにエラーログに記録します。

## 制限事項

1. **必要なリージョンをすべてダウンロードする（`us-east-1`を含む）。** コンソールのダウンロードはリージョンごとです。コンソールへのサインインやIAMのイベントは`us-east-1`に記録されます。
2. **マッチしないルールがある。** CSVには`userIdentity.arn`/`type`/`accountId`/`principalId`、`errorMessage`、`requestParameters`、`responseElements`、`additionalEventData`がありません。これらを参照するルール（CloudTrailルール794本のうち約24本。ログイン失敗やMFAなしのログインなど、コンソールログイン系のルールを含む）はマッチしません。
3. **タイムラインの一部の列が空になる。** `UserARN`・`UserAccountID`・`UserPrincipalID`・`ErrorMessage`は常に空になり、`UserType`もroot以外は空になります。
4. **`aws-ct-summary`はユーザー名で集計する。** ARNが無いため、`UserARN`列に`User name`を入れて集計し、その件数を注意として表示します。ロールの場合はセッション名なので、同じセッション名を使う別のロールは1つのプリンシパルとして数えられます。ユーザー名が無い行（AWSのサービスがユーザーに代わって行う呼び出しなど）は除外されるため、同じ期間のJSONとは件数が異なることがあります。
5. **Google WorkspaceはCSVに対応していない。** 入力オプションはAWS・Azure・Google Workspaceのコマンドで共通のため、`-d`/`-f`のヘルプには`csv`が載っていますが、`gws-timeline`はCSV入力に対応していません。
