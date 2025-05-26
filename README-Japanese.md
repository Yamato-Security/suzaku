<div align="center">
 <p>
    <img alt="Suzaku Logo" src="logo.jpeg" width="60%">
 </p>
  [<a href="README.md">English</a>] | [<b>日本語</b>]
</div>

---

<p align="center">
    <a href="https://gist.github.com/cheerfulstoic/d107229326a01ff0f333a1d3476e068d"><img src="https://img.shields.io/badge/Maintenance%20Level-Actively%20Developed-brightgreen.svg" /></a>
    <a href="https://github.com/Yamato-Security/suzaku/commits/main/"><img src="https://img.shields.io/github/commit-activity/t/Yamato-Security/suzaku/main" /></a>
    <a href="https://twitter.com/SecurityYamato"><img src="https://img.shields.io/twitter/follow/SecurityYamato?style=social"/></a>
</p>


# Suzaku について

Suzaku（朱雀）は、中国神話において、雲の上を飛び、南の天を支配する神である["Vermilion Bird"](https://en.wikipedia.org/wiki/Vermilion_Bird)を意味します。

Suzakuは、クラウドログのための脅威ハンティングおよび高速フォレンジックタイムライン生成ツールです。
（[Hayabusa](https://github.com/Yamato-Security/hayabusa)をWindowsイベントログではなくクラウドログ用にしたものを想像してください。） 
現在、AWS CloudTrailログに対する基本的なネイティブの[sigma](https://github.com/SigmaHQ/sigma)サポートを備えた形で積極的に開発が進められています。 
AWSの次には、AzureおよびGCPログのサポートを計画しています。

クラウドログには、数千もの異なるAPI呼び出しがあり、手動で確認するには膨大なイベントが存在します。 
Suzakuは、ノイズの中から攻撃を見つけるだけでなく、迅速なフォレンジック調査を行うために必要なイベントとデータのみを含むDFIRタイムラインを提供するよう設計されています。 
また、高レベルで何が起こったのかを迅速に発見し、攻撃者が行ったイベントを見逃さないようにするための要約や検索機能なども作成する予定です。

# 関連プロジェクト

* [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) - Suzakuのためのルールセット。Sigmaルールを使用して、Suzakuで検出できるようにします。
* [suzaku-sample-data](https://github.com/Yamato-Security/suzaku-sample-data) - Suzakuのサンプルデータセット。Suzakuを使用して、サンプルデータを分析することができます。

# 目次


- [Suzaku について](#suzaku-について)
- [関連プロジェクト](#関連プロジェクト)
- [目次](#目次)
- [スクリーンショット](#スクリーンショット)
  - [スタートアップ](#スタートアップ)
  - [DFIRタイムライン](#dfirタイムライン)
  - [検知頻度のタイムライン](#検知頻度のタイムライン)
  - [検知結果サマリ](#検知結果サマリ)
- [機能](#機能)
- [ダウンロード](#ダウンロード)
- [Gitクローン](#gitクローン)
- [アドバンス: ソースからのコンパイル (任意)](#アドバンス-ソースからのコンパイル-任意)
  - [Rustパッケージの更新](#rustパッケージの更新)
  - [macOSのコンパイルの注意点](#macosのコンパイルの注意点)
  - [Linuxのコンパイルの注意点](#linuxのコンパイルの注意点)
  - [LinuxのMUSLバイナリのクロスコンパイル](#linuxのmuslバイナリのクロスコンパイル)
- [Suzakuの実行](#suzakuの実行)
  - [Windows](#windows)
    - [パスにスペースが含まれるファイルまたはディレクトリをスキャンしようとするとエラーが発生した場合](#パスにスペースが含まれるファイルまたはディレクトリをスキャンしようとするとエラーが発生した場合)
    - [文字が正常に表示されない場合](#文字が正常に表示されない場合)
  - [Linux](#linux)
  - [macOS](#macos)
- [コマンドの一覧](#コマンドの一覧)
  - [分析コマンド](#分析コマンド)
  - [要約コマンド](#要約コマンド)
  - [DFIRタイムラインコマンド](#dfirタイムラインコマンド)
  - [一般コマンド](#一般コマンド)
- [コマンドの詳細](#コマンドの詳細)
  - [分析コマンド](#分析コマンド-1)
    - [`aws-ct-metrics`コマンド](#aws-ct-metricsコマンド)
    - [コマンド使用例](#コマンド使用例)
      - [`aws-ct-metrics`コマンドの例](#aws-ct-metricsコマンドの例)
  - [DFIRサマリーコマンド](#dfirサマリーコマンド)
    - [`aws-ct-summary`コマンド](#aws-ct-summaryコマンド)
      - [`AbusedAPIs-Success`の例](#abusedapis-successの例)
      - [`AbusedAPIs-Failed`の例](#abusedapis-failedの例)
      - [`OtherAPIs-Success`の例](#otherapis-successの例)
      - [`AWS-Regions`の例](#aws-regionsの例)
      - [`SrcIPs`の例](#srcipsの例)
      - [`UserType`の例](#usertypeの例)
      - [`UserAccessKeyIDs`の例](#useraccesskeyidsの例)
      - [`UserAgents`の例](#useragentsの例)
    - [コマンド使用例](#コマンド使用例-1)
      - [`aws-ct-summary`コマンドの例](#aws-ct-summaryコマンドの例)
  - [DFIRタイムラインコマンド](#dfirタイムラインコマンド-1)
    - [`aws-ct-timeline`コマンド](#aws-ct-timelineコマンド)
    - [コマンド使用例](#コマンド使用例-2)
      - [`aws-ct-timeline`コマンドの例](#aws-ct-timelineコマンドの例)
      - [`aws-ct-timeline`出力プロフィール](#aws-ct-timeline出力プロフィール)
- [貢献](#貢献)
- [バグの報告](#バグの報告)
- [ライセンス](#ライセンス)
- [コントリビューター](#コントリビューター)
- [謝辞](#謝辞)
- [Twitter](#twitter)

# スクリーンショット

## スタートアップ

![Suzaku Startup](screenshots/Startup.png)

## DFIRタイムライン

![Terminal Output](screenshots/TerminalOutput.png)

## 検知頻度のタイムライン

![Detection Frequency Timeline](screenshots/DetectionFrequencyTimeline.png)

## 検知結果サマリ

![Results Summary](screenshots/ResultsSummary.png)

# 機能

* クロスプラットフォームサポート: Windows、Linux、macOS
* Rustで開発され、安全で高速。
* `.json`または圧縮された`.json.gz`ファイルがスキャン可能。
* 単一のタイムラインを作成し、フォレンジック調査やインシデントレスポンスを容易にします。
* 読みやすく、作成や編集が簡単なYML形式の[Sigma](https://github.com/SigmaHQ/sigma)で記述されたIoCシグネチャに基づく脅威ハンティング。
* 結果をCSV、JSON、JSONL形式で保存可能。

# ダウンロード

最新の安定版Suzakuを、[Releases](https://github.com/Yamato-Security/suzaku/releases)ページからコンパイル済みバイナリとしてダウンロードするか、ソースコードをコンパイルしてください。

以下のアーキテクチャ向けにバイナリを提供しています:
- Linux ARM 64-bit GNU (`suzaku-x.x.x-lin-aarch64-gnu`)
- Linux Intel 64-bit GNU (`suzaku-x.x.x-lin-x64-gnu`)
- Linux Intel 64-bit MUSL (`suzaku-x.x.x-lin-x64-musl`)
- macOS ARM 64-bit (`suzaku-x.x.x-mac-aarch64`)
- macOS Intel 64-bit (`suzaku-x.x.x-mac-x64`)
- Windows ARM 64-bit (`suzaku-x.x.x-win-aarch64.exe`)
- Windows Intel 64-bit (`suzaku-x.x.x-win-x64.exe`)
- Windows Intel 32-bit (`suzaku-x.x.x-win-x86.exe`)

> [何らかの理由でLinux ARM MUSLバイナリが正常に動作しません](https://github.com/Yamato-Security/hayabusa/issues/1332) so we do not provide that binary. It is out of our control, so we plan on providing it in the future when it gets fixed.

# Gitクローン

`git clone`でリポジトリをクローンし、ソースコードからバイナリをコンパイルすることができます。

**警告** リポジトリのメインブランチは開発用であり、まだ正式にリリースされていない新機能にアクセスできますが、バグがある可能性があるため、安定版ではないと見なしてください。

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **注意:** `--recursive`オプションを忘れると、`rules`フォルダがgitサブモジュールとして管理されているため、クローンされません。

`rules`フォルダは、gitサブモジュールとして管理されているため、`git pull --recurse-submodules`で最新のSuzakuルールを取得できます。

```bash
./suzaku update-rules
```

更新に失敗したら、`rules`フォルダの名前を変更して、もう一度試してください。

>>警告: 更新時に、rulesフォルダ内のルールおよび設定ファイルは、suzaku-rulesリポジトリ内の最新のルールおよび設定ファイルに置き換えられます。
>>既存のファイルに加えた変更は上書きされるため、更新前に編集したファイルのバックアップを作成することをお勧めします。
>>ただし、rulesフォルダ内に**新しい**ルールを追加した場合、それらは更新時に上書きまたは削除されることはありません。

# アドバンス: ソースからのコンパイル (任意)

Rustがインストールされている場合、以下のコマンドでソースコードからコンパイルできます。

注意: コンパイルには通常、最新バージョンのRustが必要です

```bash
cargo build --release
```

最新の開発版はメインブランチから、または最新の安定版は[Releases](https://github.com/Yamato-Security/suzaku/releases)ページからダウンロードできます

Rustを定期的に更新してください:

```bash
rustup update stable
```

コンパイルされたバイナリは`./target/release` フォルダに作成されます。

## Rustパッケージの更新

コンパイルする前に、最新のRustクレートに更新できます:

```bash
cargo update
```

> アップデート後に何かが壊れた場合は、お知らせください。

## macOSのコンパイルの注意点

opensslのコンパイルエラーが発生した場合は、[Homebrew](https://brew.sh/)をインストールし、次のパッケージをインストールしてください。

```bash
brew install pkg-config
brew install openssl
```

## Linuxのコンパイルの注意点

opensslのコンパイルエラーが発生した場合は、次のパッケージをインストールしてください。

Ubuntuベースのディストリビューション:

```bash
sudo apt install libssl-dev
```

Fedoraベースのディストリビューション:

```bash
sudo yum install openssl-devel
```

## LinuxのMUSLバイナリのクロスコンパイル

Linux OSでは、まずターゲットをインストールします。

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Compile with:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **警告: 新しい安定版のRustがリリースされるたびに、`rustup install stable-x86_64-unknown-linux-musl`を実行してください。そうしないと、クロスコンパイル用のコンパイラが更新されず、ビルドエラーが発生する可能性があります。**

MUSLバイナリは、`./target/x86_64-unknown-linux-musl/release/`ディレクトリに作成されます。
GNUバイナリよりも約15%遅くなりますが、Linuxの異なるバージョンやディストリビューション間での移植性が高くなります。

# Suzakuの実行

## Windows

コマンドプロンプトまたはPowerShellを開き、Suzakuのルートディレクトリに移動して、次のコマンドを実行します。

### パスにスペースが含まれるファイルまたはディレクトリをスキャンしようとするとエラーが発生した場合

コマンドプロンプトやPowerShellの組み込み機能を使用している場合、ファイルやディレクトリパスにスペースが含まれていると、Suzakuがファイルを読み込めないというエラーが表示されることがあります。
ログファイルを正しく読み込むために、以下を確認してください。
1. ファイルやディレクトリパスを二重引用符で囲んでいること
2. ディレクトリの場合、最後の文字にバックスラッシュを含めないこと

### 文字が正常に表示されない場合

デフォルトフォントの`Lucida Console`では、ロゴやテーブルで使用されるさまざまな文字が正しく表示されません。
`Consalas`フォントに変更することで、文字が正しく表示されます。

## Linux

バイナリに実行権限を付与します。

```bash
chmod +x ./suzaku
```

Suzakuのルートディレクトリから実行します。

```bash
./suzaku
```

## macOS

ターミナルや[iTerm2](https://iterm2.com/)で、最初にバイナリに実行権限を付与する必要があります。

```bash
chmod +x ./suzaku
```

そして、Suzakuのルートディレクトリから実行します。

```bash
./suzaku
```

最新のmacOSでは、実行しようとするとセキュリティエラーが表示されることがあります。
キャンセルをクリックして、から"セキュリティとプライバシー"を開き、Generalタブから"許可"をクリックします。
その後、再度実行してください。

```bash
./suzaku
```

警告のポップアップが表示されるので、"開く"をクリックしてください。
これでsuzakuを実行できるようになります。

# コマンドの一覧

## 分析コマンド
* `aws-ct-metrics`: AWS CloudTrailログのメトリクスを作成する

## 要約コマンド
* `aws-ct-summary`: ARNからのアクティビティの要約を生成する

## DFIRタイムラインコマンド
* `aws-ct-timeline`: AWS CloudTrailログのDFIRタイムラインを作成する
* `update-rules`: Sigmaルールを更新する

## 一般コマンド
* `help`: コマンドのヘルプメニューを表示する

# コマンドの詳細

## 分析コマンド

### `aws-ct-metrics`コマンド

このコマンドは、AWS CloudTrailログ内のフィールドに関するメトリクスを作成するために使用します。
デフォルトでは、`eventName`フィールドをスキャンします。
このコマンドは、現在、最も一般的なAPI呼び出しを特定するために使用されており、検出ルールの優先順位を決定するために使用されます。

### コマンド使用例
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  複数gz/jsonファイルのディレクトリパス
  -f, --file <FILE>      gz/jsonファイルのパス

Output:
  -F, --field-name <FIELD_NAME>  メトリクスを作成するフィールド [デフォルト: eventName]
  -o, --output <FILE>            CSVに保存

Display Settings:
  -K, --no-color  カラーで出力しない
  -q, --quiet     Quietモード: 起動バナーを表示しない

General Options:
  -h, --help  ヘルプメニューを表示する
  ```

#### `aws-ct-metrics`コマンドの例

* `eventName`のAPIコール数をテーブル形式で出力: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* CSVに保存: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`

## DFIRサマリーコマンド

### `aws-ct-summary`コマンド

このコマンドは、ユーザーARNに基づいて以下の情報の要約を作成します:
- イベントの総数（APIコール） (`NumOfEvents`)
- ログ内で見つかった最初のAPIコールのタイムスタンプ (`FirstTimestamp`)
- ログ内で見つかった最後のAPIコールのタイムスタンプ (`LastTimestamp`)
- 成功した一般的に悪用されるAPIコール (`AbusedAPIs-Success`)
- 試行されたが失敗した一般的に悪用されるAPIコール (`AbusedAPIs-Failed`)
- 一般的に悪用されるAPIコールのリストにない他のAPIコールで成功したもの (`OtherAPIs-Success`)
- 一般的に悪用されるAPIコールのリストにない他のAPIコールで試行されたが失敗したもの (`OtherAPIs-Failed`)
- APIコールが行われたAWSリージョン (`AWS-Regions`)
- APIコールのソースIPアドレス (`SrcIPs`)
- ユーザータイプ (`UserTypes`)
- ユーザーアクセスキーID (`UserAccessKeyIDs`)
- APIコールを行ったソースのユーザーエージェント (`UserAgents`)

※ 注意: 一般的に悪用されるAPI呼び出しは、[https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv](https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv)にホストされている設定ファイルから取得できます。このファイルは時間とともに更新され、`update-rules`コマンドを実行するたびにローカルで同期されます。

この結果は、アナリストが特定のシグネチャに依存せずに侵害されたアカウントや攻撃を発見するための情報を提供することを目的としています。
例えば、特定のユーザーが疑わしいAPIコールを行っていないか、通常使用されないリージョンを使用していないか、疑わしいソースIPアドレスやユーザーエージェントからログインしていないかを確認できます。
攻撃者がその期間中に悪用した可能性のあるアクセスキーを特定し、元のJSONログでそれらのキーワードをピボットして攻撃者の活動のタイムラインを作成することができます。

> 警告: セル内のデータが非常に多く、Excelなどのプログラムではうまく表示されない可能性があります。MacではNumbers、WindowsではTimeline Explorerなどを使用してください。

#### `AbusedAPIs-Success`の例
```
Unique APIs: 11 | Total APIs 477,373
415,552 - RunInstances (ec2.amazonaws.com) - Spin up EC2 instances (crypto mining, tools) (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
28,907 - GetBucketAcl (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-10-21 13:59:40)
10,095 - GetCallerIdentity (sts.amazonaws.com) - Current credentials recon (2019-08-23 06:00:07 ~ 2019-08-23 06:04:14)
9,936 - ListBuckets (s3.amazonaws.com) - S3 recon (2019-08-23 06:00:07 ~ 2019-08-23 06:14:53)
9,168 - DescribeInstances (ec2.amazonaws.com) - EC2 and network layout recon (2019-08-23 06:00:07 ~ 2019-08-23 06:04:20)
3,658 - DescribeVpcs (ec2.amazonaws.com) - EC2 and network layout recon (2019-08-21 08:03:03 ~ 2019-09-12 20:00:44)
19 - ListGroups (greengrass.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-10-19 23:49:25)
14 - DescribeInstances (opsworks.amazonaws.com) - EC2 and network layout recon (2019-08-21 08:03:03 ~ 2019-10-19 23:49:22)
12 - GetBucketPolicy (s3.amazonaws.com) - S3 recon (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
7 - ListGroups (resource-groups.amazonaws.com) - IAM enumeration (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
5 - StartInstances (ec2.amazonaws.com) - Spin up EC2 instances (crypto mining, tools) (2019-08-21 08:03:03 ~ 2019-12-12 07:07:31)
```

#### `AbusedAPIs-Failed`の例
```
Unique APIs: 23 | Total APIs 20,464
11,603 - AssumeRole (sts.amazonaws.com) - Lateral movement via roles (2019-08-21 08:03:03 ~ 2019-09-18 07:04:12)
7,279 - GetBucketAcl (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-09-09 09:01:26)
515 - GetBucketPolicy (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-10-01 19:11:07)
331 - ListUsers (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-08-29 14:53:14)
210 - ListSecrets (secretsmanager.amazonaws.com) - Find credential storage locations (2019-08-21 08:03:03 ~ 2019-10-19 23:49:30)
153 - ListGroups (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-09-12 15:24:39)
148 - ListRoles (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-09-12 15:20:56)
112 - ListAccessKeys (iam.amazonaws.com) - Enumerates keys on IAM users (2019-08-21 08:03:03 ~ 2019-09-16 14:28:15)
31 - ListGroups (greengrass.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2020-02-25 14:41:24)
...
```

#### `OtherAPIs-Success`の例
```
Unique APIs: 289 | Total APIs 143,759
98,689 - DescribeSnapshots (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:50:59)
10,679 - DescribeSpotPriceHistory (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:32)
3,740 - DescribeReservedInstancesOfferings (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:30)
2,372 - DescribeSnapshotAttribute (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-08-24 12:38:34)
2,307 - CreateDefaultVpc (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:17)
1,532 - DescribeKeyPairs (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:16)
1,504 - DescribeSecurityGroups (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:00:40)
1,495 - DescribeImages (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:26)
1,438 - CreateKeyPair (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:16)
1,402 - DescribeVolumes (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-03 12:06:20)
1,217 - DescribeSubnets (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:00:43)
...
```

#### `AWS-Regions`の例
```
Total regions: 16
167,155 - us-west-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:14:53)
113,328 - us-east-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:04:14)
65,718 - ap-northeast-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:22:55)
64,787 - ap-northeast-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:34:57)
```

#### `SrcIPs`の例
```
Total source IPs: 5,293
634,454 - 5.205.62.253 (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
23,498 - 193.29.252.218 (2019-08-21 08:03:00 ~ 2019-10-17 09:11:22)
15,925 - 155.63.17.217 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
9,067 - 253.0.255.253 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
7,078 - 163.21.250.220 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
6,575 - 236.9.245.88 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
5,138 - 84.252.252.117 (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
4,946 - 24.251.252.2 (2019-08-21 08:03:00 ~ 2019-09-30 06:36:13)
4,225 - 211.111.151.81 (2019-08-21 08:03:00 ~ 2019-09-12 19:53:35)
```

#### `UserType`の例
```
IAMUser
```

#### `UserAccessKeyIDs`の例
```
Total access key ids: 629
667,476 - AKIA01U43UX3RBRDXF4Q (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
218,544 - ASIARF55FBMFZBXLKDFW (2019-08-21 11:31:47 ~ 2019-08-23 13:00:28)
12,677 - AKIA1ZBTOEKWKVHP6GHZ (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
8,822 - ASIAGD2JRX0V6RJGWR59 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
4,940 - ASIAUNHV6EHIK5MNPEKF (2019-08-21 08:03:00 ~ 2019-09-30 06:36:17)
```

>　一時的なAWS STSアクセスキーIDは大量に存在するため、デフォルトではフィルタリングしています。これらを含めたい場合は、`-s, --include-sts-keys`オプションを追加してください。

#### `UserAgents`の例
```
Total user agents: 7,760
351,022 - Boto3/1.9.201 Python/2.7.12 Linux/4.4.0-159-generic Botocore/1.12.201 (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
283,430 - Boto3/1.9.201 Python/2.7.12 Linux/4.4.0-157-generic Botocore/1.12.201 (2019-08-21 11:31:47 ~ 2019-08-23 13:00:28)
23,467 - [Boto3/1.15.13 Python/3.8.5 Darwin/19.6.0 Botocore/1.18.13 Resource] (2017-02-12 21:15:12 ~ 2020-10-07 16:05:52)
15,924 - Boto3/1.7.4 Python/2.7.12 Linux/4.4.0-119-generic Botocore/1.10.4 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
9,400 - aws-sdk-java/1.11.301 Linux/4.9.93-linuxkit-aufs Java_HotSpot(TM)_64-Bit_Server_VM/25.172-b11 java/1.8.0_172 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
6,372 - [Boto3/1.7.65 Python/3.5.2 Linux/4.13.0-37-generic Botocore/1.10.65 Resource] (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
5,352 - AWSPowerShell/3.3.365.0 .NET_Runtime/4.0 .NET_Framework/4.0 OS/Microsoft_Windows_NT_10.0.01985.0 WindowsPowerShell/5.0 ClientSync (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
4,945 - Boto3/1.9.231 Python/2.7.15+ Linux/4.15.0-64-generic Botocore/1.12.231 (2019-08-21 08:03:00 ~ 2019-09-30 06:36:13)
4,599 - [aws-cli/1.16.301 Python/3.7.6 Linux/5.4.0-kali3-amd64 botocore/1.13.37] (2019-08-21 08:03:00 ~ 2020-02-09 22:00:32)
3,909 - Boto3/1.14.28 Python/3.8.5 Linux/5.7.0-kali1-amd64 Botocore/1.17.28 (2019-01-08 20:30:01 ~ 2020-09-11 17:35:39)
3,450 - Boto3/1.4.2 Python/2.7.13+ Linux/4.9.0-3-amd64 Botocore/1.5.19 (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
3,198 - Boto3/1.4.2 Python/2.7.14 Linux/4.13.0-1-amd64 Botocore/1.5.19 (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
```

> `aws`クライアントツールは、ユーザーエージェントにOS情報を含めるため、攻撃者のOS（例: `kali`）からのAPIコールを検出することが可能です。

### コマンド使用例
```
Usage:
  suzaku aws-ct-summary <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  複数gz/jsonファイルのディレクトリパス
  -f, --file <FILE>      gz/jsonファイルのパス

Filtering:
  -s, --include-sts-keys  一時的なAWS STSアクセスキーIDを結果に含める

Output:
  -o, --output <FILE>           CSVファイルに結果を保存
  -D, --hide-descriptions       一般的に悪用されるAPIコールの説明を非表示にする
  -G, --GeoIP <MAXMIND-DB-DIR>  IPアドレスにGeoIP (ASN、都市、国)情報を追加する

Display Settings:
  -K, --no-color               カラーで出力しない
  -q, --quiet                  Quietモード: 起動バナーを表示しない

General Options:
  -h, --help  ヘルプメニューを表示する
  ```

#### `aws-ct-summary`コマンドの例

* CSVに結果を保存する: `./suzaku aws-ct-summary -d ../suzaku-sample-data -o sample-summary.csv`

## DFIRタイムラインコマンド

### `aws-ct-timeline`コマンド

AWS CloudTrailのDFIRタイムラインを、`rules`フォルダ内のsigmaルールに基づいて作成します。

### コマンド使用例
```
Usage: suzaku aws-ct-timeline [OPTIONS] <--directory <DIR>|--file <FILE>>

General Options:
  -r, --rules <DIR/FILE>  カスタムルールディレクトリのパス (デフォルト: ./rules)
  -h, --help              ヘルプメニューを表示する

Input:
  -d, --directory <DIR>  複数gz/jsonファイルのディレクトリパス
  -f, --file <FILE>      gz/jsonファイルのパス

Output:
  -o, --output <FILE>              ファイルに結果を保存
  -t, --output-type <OUTPUT_TYPE>  ファイルタイプ 1: CSV (デフォルト), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL [デフォルト: 1]
  -C, --clobber                    結果ファイルを上書きする
  -G, --GeoIP <MAXMIND-DB-DIR>     IPアドレスにGeoIP (ASN、都市、国)情報を追加する
  -R, --raw-output                 元のJSONログを出力する（JSON形式のみ利用可能）

Display Settings:
  -K, --no-color               カラーで出力しない
  -N, --no-summary             結果概要を出力しない
  -T, --no-frequency-timeline  結果タイムライン頻度を出力しない (ターミナルがUnicodeをサポートしている必要がある)
  -q, --quiet                  Quietモード: 起動バナーを表示しない
  ```

#### `aws-ct-timeline`コマンドの例

* スクリーンにアラートを出力: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* CSVに保存: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* CSVとJSONLに保存: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

#### `aws-ct-timeline`出力プロフィール

Suzakuは`config/default_profile.yaml`ファイルに基づいて情報を出力します:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
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
RuleID: 'sigma.id'
```

* `.`（例: `.eventTime`）で始まるフィールド値は、CloudTrailログから取得されます。
* `sigma.`（例: `sigma.title`）で始まるフィールド値は、Sigmaルールから取得されます。
* 現在は文字列のみをサポートしていますが、将来的には他の型のフィールド値にも対応する予定です。

# 貢献

私たちは、あらゆる形での貢献を歓迎しています。
プルリクエスト、ルールの作成、サンプルログの提供が最もありがたいですが、機能リクエストやバグの報告なども大歓迎です。

**私たちのツールやリソースを気に入っていただけた場合は、GitHubでスターを付けてサポートを示してください！**

# バグの報告

* 見つけたバグは[こちら](https://github.com/Yamato-Security/suzaku/issues/new?assignees=&labels=bug&template=bug_report.md&title=%5Bbug%5D)から報告してください
* このプロジェクトは現在積極的にメンテナンスされており、報告されたバグを修正する準備が整っています
* Suzakuルールに関する問題（誤検知、バグなど）を見つけた場合は、suzaku-rulesの[こちら](https://github.com/Yamato-Security/suzaku-rules/issues/new)に報告してください
* Sigmaルールに関する問題（誤検知、バグなど）を見つけた場合は、上流のSigmaHQの[こちら](https://github.com/SigmaHQ/sigma/issues)に報告してください。

# ライセンス

* Suzakuは[AGPLv3](https://www.gnu.org/licenses/agpl-3.0.en.html)の下でリリースされており、すべてのルールは[Detection Rule License (DRL) 1.1](https://github.com/SigmaHQ/sigma/blob/master/LICENSE.Detection.Rules.md)の下でリリースされています。
* Suzakuは、社内利用、SaaSソリューション、コンサルティング業務などで自由に使用できます。
  ただし、SaaSソリューションの一環としてSuzakuを使用し、それに改良を加えた場合は、その改良をオープンソース化し、プロジェクトに還元することをお願いしています

# コントリビューター

* DustInDark (コア開発者)
* Fukusuke Takahashi (コア開発者)
* Zach Mathis (プロジェクトリーダー, ツールデザイン, ルール作成, テスト,　など...) (@yamatosecurity)

# 謝辞

* [Flaws.cloud](http://flaws.cloud/)
* [Invictus-ir](https://www.invictus-ir.com/)
* [Sigma](https://github.com/SigmaHQ/sigma)
* [sigma-rust](https://github.com/jopohl/sigma-rust)
* [Stratus Red Team](https://stratus-red-team.cloud/)
* [Traildiscover.cloud](https://traildiscover.cloud/)

# Twitter

最新のニュースを受け取るには、[@SecurityYamato](https://twitter.com/SecurityYamato)をフォローしてください。