# 機能

* クロスプラットフォームサポート: Windows、Linux、macOS
* Rustで開発され、安全で高速。
* AWS CloudTrail、Azure/Entra ID/M365、Google Workspace Reports APIログをネイティブSigma検知で分析
* `.json`、`.jsonl`、圧縮された`.json.gz`、Parquet、および対応する[AWS Management ConsoleのCloudTrail CSV](../getting-started/console-csv-input.md)をマルチスレッドで高速スキャン
* フォレンジック調査やインシデント対応に適した、分析しやすい単一のタイムラインを作成
* 読み書きや編集が容易なYML形式の[Sigma](https://github.com/SigmaHQ/sigma)ルールで書かれたIoCシグネチャをネイティブに優れた形でサポート（相関ルールと、`expand`を除くすべてのフィールド修飾子に対応)
* API使用状況の概要、攻撃者に関するメトリクス（送信元IPアドレス、ジオロケーション、使用リージョン、ユーザーエージェントなど）を集約し、シグネチャに依存せず異常な活動を検出
* 結果をCSV、JSON、JSONL、DuckDB形式で保存可能
