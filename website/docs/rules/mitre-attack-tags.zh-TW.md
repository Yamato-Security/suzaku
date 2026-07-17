# MITRE ATT&CK 標籤

Sigma 規則可以帶有 [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) 欄位，用來將偵測內容依據 [MITRE ATT&CK®](https://attack.mitre.org/) 框架（戰術、技術與群組）以及其他分類體系進行分類。由於 `tags` 是一個清單，`aws-ct-timeline` 與 `azure-timeline` 命令會將其彙整輸出於單一的 **`Tags`** 欄位中，各項目以 ` ¦ `（與 Hayabusa 使用的分隔符號相同）連接，並將每個項目略記，使欄位保持精簡。

## 範例

以如下方式標記標籤的規則：

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

會在 `Tags` 欄位中輸出為：

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

在 JSON/JSONL 輸出中，該值會保持為相同的扁平字串（**不會**展開成陣列），因此在 CSV、JSON 與終端機中欄位內容完全相同。

## 各標籤的略記規則

| 標籤種類 | 輸入範例 | 輸出 | 規則 |
| --- | --- | --- | --- |
| 戰術 | `attack.credential-access` | `CredAccess` | 於 `config/mitre_tactics.txt` 中查找（見下方） |
| 技術 | `attack.t1562.001` | `T1562.001` | 將 `attack.t` 前綴替換為大寫的 `T`；技術／子技術編號保持不變 |
| 群組 | `attack.g0035` | `G0035` | 將 `attack.g` 前綴替換為大寫的 `G`；群組編號保持不變 |
| 其他 | `cve.2021.1234` | `cve.2021.1234` | 保持不變 |

標籤比對時不區分大小寫，連字號與底線的寫法會視為相同，因此 `attack.credential-access` 與 `attack.credential_access` 都會變成 `CredAccess`。

## 戰術略記對照表

戰術略記並非硬式編碼，而是在執行階段從 `config/mitre_tactics.txt`（與 [Hayabusa](https://github.com/Yamato-Security/hayabusa) 共用的對照表）讀取。每一行都是簡單的 `<full tag>,<abbreviation>` 組合，因此你可以在不重新建置 Suzaku 的情況下編輯或擴充這些略記：

| 完整標籤 | 略記 |
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

> 注意：若 `config/mitre_tactics.txt` 不存在，戰術標籤會原樣輸出（技術與群組的略記仍可正常運作）。

## 關於「Stealth」與「Defense Evasion」的說明

自 [MITRE ATT&CK v19（2026 年 4 月）](https://attack.mitre.org/resources/updates/updates-april-2026/) 起，**Defense Evasion** 戰術（`TA0005`）已改名為 **Stealth**，並從中分離出獨立的 **Impair Defenses** 戰術（`TA0112`）。Suzaku 採用新的名稱：

- `attack.stealth` 與舊有的 `attack.defense-evasion` 都會略記為 **`Stealth`**，因此仍在使用 `attack.defense-evasion` 標籤的舊規則也會以目前的戰術名稱顯示。
- `attack.defense-impairment` 會略記為 **`DefImpair`**。

若你偏好舊的標籤，請修改 `config/mitre_tactics.txt` 中的 `attack.defense-evasion` 這一行（例如改回 `Evas`）。
