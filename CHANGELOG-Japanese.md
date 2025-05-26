# 変更点

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