# MITRE ATT&CK แท็ก

กฎ Sigma สามารถมีฟิลด์ [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) ที่จัดหมวดหมู่การตรวจจับตามเฟรมเวิร์ก [MITRE ATT&CK®](https://attack.mitre.org/) (ทั้งกลยุทธ์ เทคนิค และกลุ่ม) รวมถึงระบบการจำแนกอื่น ๆ ได้ เนื่องจาก `tags` เป็นลิสต์ คำสั่ง `aws-ct-timeline` และ `azure-timeline` จึงแสดงผลรวมไว้ในคอลัมน์ **`Tags`** เดียว โดยเชื่อมแต่ละรายการด้วย ` ¦ ` (ตัวคั่นเดียวกับที่ Hayabusa ใช้) และย่อแต่ละรายการให้สั้นลงเพื่อให้คอลัมน์กระชับ

## ตัวอย่าง

กฎที่ถูกแท็กแบบนี้:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

จะถูกแสดงผลในคอลัมน์ `Tags` ดังนี้:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

ในเอาต์พุต JSON/JSONL ค่าจะถูกเก็บไว้เป็นสตริงแบบแบนเดียวกัน (ค่าจะ**ไม่**ถูกขยายเป็นอาร์เรย์) ดังนั้นคอลัมน์จึงเหมือนกันทั้งใน CSV, JSON และเทอร์มินัล

## วิธีย่อแต่ละแท็ก

| ประเภทแท็ก | ตัวอย่างอินพุต | เอาต์พุต | กฎ |
| --- | --- | --- | --- |
| กลยุทธ์ | `attack.credential-access` | `CredAccess` | ค้นหาใน `config/mitre_tactics.txt` (ดูด้านล่าง) |
| เทคนิค | `attack.t1562.001` | `T1562.001` | คำนำหน้า `attack.t` จะกลายเป็น `T` ตัวพิมพ์ใหญ่ ส่วนหมายเลขเทคนิค/เทคนิคย่อยจะคงไว้ตามเดิม |
| กลุ่ม | `attack.g0035` | `G0035` | คำนำหน้า `attack.g` จะกลายเป็น `G` ตัวพิมพ์ใหญ่ ส่วนหมายเลขกลุ่มจะคงไว้ตามเดิม |
| อื่น ๆ | `cve.2021.1234` | `cve.2021.1234` | คงไว้ไม่เปลี่ยนแปลง |

แท็กจะถูกจับคู่แบบไม่สนใจตัวพิมพ์เล็ก-ใหญ่ และการสะกดด้วยเครื่องหมายยัติภังค์กับขีดล่างจะถูกปฏิบัติเหมือนกัน ดังนั้น `attack.credential-access` และ `attack.credential_access` จึงกลายเป็น `CredAccess` ทั้งคู่

## ตารางการย่อกลยุทธ์

การย่อกลยุทธ์**ไม่ได้**ถูกฮาร์ดโค้ดไว้ แต่จะถูกอ่านขณะรันไทม์จาก `config/mitre_tactics.txt` ซึ่งเป็นตารางเดียวกับที่ [Hayabusa](https://github.com/Yamato-Security/hayabusa) ใช้ แต่ละบรรทัดเป็นคู่ `<full tag>,<abbreviation>` แบบง่าย ๆ ดังนั้นคุณจึงสามารถแก้ไขหรือเพิ่มการย่อได้โดยไม่ต้องบิลด์ Suzaku ใหม่:

| แท็กแบบเต็ม | ตัวย่อ |
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

> หมายเหตุ: หาก `config/mitre_tactics.txt` หายไป แท็กกลยุทธ์จะถูกส่งผ่านโดยไม่เปลี่ยนแปลง ส่วนการย่อเทคนิคและกลุ่มยังคงทำงานได้

## หมายเหตุเกี่ยวกับ "Stealth" กับ "Defense Evasion"

ตั้งแต่ [MITRE ATT&CK v19 (เมษายน 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/) เป็นต้นไป กลยุทธ์ **Defense Evasion** (`TA0005`) ได้ถูกเปลี่ยนชื่อเป็น **Stealth** และมีการแยกกลยุทธ์ **Impair Defenses** (`TA0112`) ออกมาต่างหาก Suzaku ปฏิบัติตามการตั้งชื่อใหม่นี้:

- ทั้ง `attack.stealth` และ `attack.defense-evasion` แบบเดิมต่างก็ถูกย่อเป็น **`Stealth`** ดังนั้นกฎเก่าที่ยังใช้แท็ก `attack.defense-evasion` จึงถูกแสดงด้วยชื่อกลยุทธ์ปัจจุบัน
- `attack.defense-impairment` ถูกย่อเป็น **`DefImpair`**

หากคุณต้องการใช้ป้ายกำกับแบบเดิม ให้แก้ไขบรรทัด `attack.defense-evasion` ใน `config/mitre_tactics.txt` (เช่น เปลี่ยนกลับเป็น `Evas`)
