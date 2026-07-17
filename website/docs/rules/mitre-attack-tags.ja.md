# MITRE ATT&CK タグ

Sigma ルールには、検知内容を [MITRE ATT&CK®](https://attack.mitre.org/) フレームワーク（タクティクス・テクニック・グループ）やその他の分類体系に紐付ける [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) フィールドを指定できます。`tags` はリストであるため、`aws-ct-timeline` および `azure-timeline` コマンドではこれを1つの **`Tags`** カラムにまとめて出力します。各エントリは（Hayabusa と同じ区切り文字である） ` ¦ ` で連結し、カラムがコンパクトになるようそれぞれを略記します。

## 例

次のようにタグ付けされたルール:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

は、`Tags` カラムで次のように出力されます:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

JSON/JSONL 出力でも同じフラットな文字列として保持され（配列には展開されません）、CSV・JSON・ターミナルで同一の内容になります。

## 各タグの略記ルール

| タグの種類 | 入力例 | 出力 | ルール |
| --- | --- | --- | --- |
| タクティクス | `attack.credential-access` | `CredAccess` | `config/mitre_tactics.txt` を参照（後述） |
| テクニック | `attack.t1562.001` | `T1562.001` | `attack.t` を大文字の `T` に置き換え、テクニック／サブテクニック番号はそのまま |
| グループ | `attack.g0035` | `G0035` | `attack.g` を大文字の `G` に置き換え、グループ番号はそのまま |
| その他 | `cve.2021.1234` | `cve.2021.1234` | 変更しない |

タグは大文字・小文字を区別せずにマッチし、ハイフンとアンダースコアの表記は同一に扱われるため、`attack.credential-access` と `attack.credential_access` はどちらも `CredAccess` になります。

## タクティクスの略記テーブル

タクティクスの略記はハードコードされておらず、実行時に `config/mitre_tactics.txt`（Hayabusa と共通のテーブル）から読み込まれます。各行は `<フルタグ>,<略記>` という単純な組であり、Suzaku を再ビルドせずに略記を編集・追加できます:

| フルタグ | 略記 |
| --- | --- |
| `attack.reconnaissance` | Recon |
| `attack.resource-development` | ResDev |
| `attack.initial-access` | InitAccess |
| `attack.execution` | Exec |
| `attack.persistence` | Persis |
| `attack.privilege-escalation` | PrivEsc |
| `attack.stealth` | Stealth |
| `attack.defense-evasion` | Stealth |
| `attack.defense-impairment` | DefImpair |
| `attack.credential-access` | CredAccess |
| `attack.discovery` | Disc |
| `attack.lateral-movement` | LatMov |
| `attack.collection` | Collect |
| `attack.command-and-control` | C2 |
| `attack.exfiltration` | Exfil |
| `attack.impact` | Impact |

> 注: `config/mitre_tactics.txt` が存在しない場合、タクティクスのタグはそのまま出力されます（テクニックとグループの略記は引き続き機能します）。

## 「Stealth」と「Defense Evasion」について

[MITRE ATT&CK v19（2026年4月）](https://attack.mitre.org/resources/updates/updates-april-2026/) より、**Defense Evasion** タクティクス（`TA0005`）は **Stealth** に改名され、そこから **Impair Defenses** タクティクス（`TA0112`）が分離されました。Suzaku は新しい名称に従います:

- `attack.stealth` と従来の `attack.defense-evasion` はどちらも **`Stealth`** に略記されるため、`attack.defense-evasion` タグを使っている古いルールも現在のタクティクス名で表示されます。
- `attack.defense-impairment` は **`DefImpair`** に略記されます。

以前のラベルを使いたい場合は、`config/mitre_tactics.txt` の `attack.defense-evasion` の行を変更してください（例: `Evas` に戻す）。
