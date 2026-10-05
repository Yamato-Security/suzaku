# คำสั่งสำหรับ DFIR Timeline

## คำสั่ง `aws-ct-timeline`

สร้างไทม์ไลน์ DFIR ของ AWS CloudTrail โดยอ้างอิงจากกฎ Sigma ในโฟลเดอร์ `rules`

## วิธีการใช้งานคำสั่ง
```
Usage: suzaku aws-ct-timeline [OPTIONS] <--directory <DIR>|--file <FILE>>

General Options:
  -r, --rules <DIR/FILE>  Specify a custom rule directory or file (default: ./rules)
  -h, --help              Show the help menu

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -C, --clobber                    Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>    Add GeoIP (ASN, city, country) info to IP addresses
  -m, --min-level <LEVEL>          Minimum level for rules to load (default: informational)
  -o, --output <FILE>              Save the results to a file
  -t, --output-type <OUTPUT_TYPE>  Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL [default: 1]
  -R, --raw-output                 Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>    Number of threads to use (default: same as CPU cores)

Display Settings:
  -K, --no-color               Disable color output
  -N, --no-summary             Do not display results summary
  -T, --no-frequency-timeline  Disable event frequency timeline (terminal needs to support Unicode)
  -q, --quiet                  Quiet mode: do not display the launch banner
```

### ตัวอย่างการใช้คำสั่ง `aws-ct-timeline`

* แสดงการแจ้งเตือนบนหน้าจอ: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* บันทึกผลลัพธ์ลงไฟล์ CSV: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* บันทึกผลลัพธ์ลงไฟล์ CSV และ JSONL: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### โปรไฟล์ผลลัพธ์ของ `aws-ct-timeline`

Suzaku จะแสดงผลข้อมูลตามไฟล์ `config/aws_profile.yaml`:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
Level: 'sigma.level'
EventSource: '.eventSource'
EventName: '.eventName'
ErrorCode: '.errorCode'
UserName: '.userIdentity.userName'
UserType: '.userIdentity.type'
SrcIP: '.sourceIPAddress'
AWS-Region: '.awsRegion'
UserAccountID: '.userIdentity.accountId'
UserARN: '.userIdentity.arn'
UserAccessKeyID: '.userIdentity.accessKeyId'
UserPrincipalID: '.userIdentity.principalId'
UserAgent: '.userAgent'
ErrorMessage: '.errorMessage'
EventID: '.eventID'
RuleAuthor: 'sigma.author'
Tags: 'sigma.tags'
RuleID: 'sigma.id'
```

* ค่าฟิลด์ใด ๆ ที่ขึ้นต้นด้วย `.` (ex: `.eventTime`) จะถูกดึงมาจากบันทึก CloudTrail
* ค่าฟิลด์ใด ๆ ที่ขึ้นต้นด้วย `sigma.` (ex: `sigma.title`) จะถูกดึงมาจากกฎ Sigma
* ปัจจุบันเรารองรับเฉพาะสตริงเท่านั้น แต่มีแผนที่จะรองรับค่าฟิลด์ประเภทอื่น ๆ ด้วย

> หมายเหตุ: หากคุณต้องการแสดงผลข้อมูล JSON ต้นฉบับและทำให้แน่ใจว่าจะไม่สูญเสียข้อมูลฟิลด์ใด ๆ เพียงเพิ่มออปชัน `-R, --raw-output` ให้กับคำสั่ง `aws-ct-timeline`

### สคีมาผลลัพธ์ DuckDB {#duckdb-output-schema}

ผลลัพธ์ CSV และ JSON เป็น*การแสดงผล*ของโปรไฟล์ข้างต้น ส่วนผลลัพธ์ DuckDB เป็น*อินเทอร์เฟซข้อมูล* จึงมีชนิดข้อมูลและอธิบายตัวเองได้ ความแตกต่างเหล่านี้ตั้งใจออกแบบไว้และใช้กับ `aws-ct-timeline`, `azure-timeline`, `gws-timeline` และ `aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| ค่าที่ไม่มี | `-` (หรือว่าง) | `NULL` |
| `Timestamp` | ข้อความที่จัดรูปแบบแล้ว | `TIMESTAMP` |
| `Level` | ข้อความ | `suzaku_level` (`ENUM` ที่เรียงตามระดับความรุนแรง) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (ไม่ต้องใส่เครื่องหมายคำพูดใน SQL) |
| `Tags` | สตริงเดียวที่เชื่อมด้วย ` ¦ ` | `Tactics`, `TechniqueIDs`, `OtherTags` (แต่ละคอลัมน์เป็น `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | เพิ่มเฉพาะเมื่อใช้ `-G, --geo-ip` | มีอยู่เสมอ (เมื่อโปรไฟล์มี `SrcIP`) เป็น `NULL` เมื่อไม่ได้ใช้ `-G` |
| แถวที่ซ้ำกัน | คงไว้ | ลบแถวที่ซ้ำกันทุกประการ และบันทึกจำนวนไว้ใน `suzaku_meta` |

ทุกไฟล์ยังมีตาราง `suzaku_meta` หนึ่งแถว เพื่อให้ทราบได้โดยไม่ต้องเดาว่าอะไรเป็นผู้สร้างไฟล์:

| คอลัมน์ | ความหมาย |
|---|---|
| `schema_version` | เวอร์ชันของโครงสร้าง ตรวจสอบค่านี้ก่อนอ่านตารางอื่น |
| `suzaku_version`, `command`, `command_line` | Suzaku เวอร์ชันใด คำสั่งย่อยใด และบรรทัดคำสั่งที่เรียกใช้จริง |
| `generated_at` | เวลาที่เขียนไฟล์ |
| `timestamp_tz` | เขตเวลาของคอลัมน์ `Timestamp` — `UTC` หรือค่าชดเชยเวลาท้องถิ่นเมื่อใช้ `-l, --localtime` |
| `rules_version`, `rules_count` | รุ่นของชุดกฎ (เมื่อโฟลเดอร์ rules เป็น git checkout) และจำนวนกฎที่โหลด |
| `geoip_enabled` | ระบุว่าได้เรียกใช้ `-G, --geo-ip` หรือไม่ ช่วยแยก `SrcCountry` ที่เป็น `NULL` ทั้งหมด ("ปิดการเสริมข้อมูล") ออกจากเซลล์ `NULL` ในไฟล์ที่เสริมข้อมูลแล้ว ("ค่านี้ไม่ใช่ที่อยู่ IP") |
| `scanned_files`, `scanned_events` | ขอบเขตของการรัน |
| `output_rows`, `duplicate_rows_removed` | จำนวนแถวที่เขียน และแถวที่ซ้ำกันทุกประการที่ถูกลบระหว่างเขียน |

สำหรับคำสั่งไทม์ไลน์ที่อิงกฎ หนึ่งแถวของ `timeline` คือ **หนึ่งเหตุการณ์ × หนึ่งการตรงกับกฎ** เหตุการณ์ที่ตรงกับหลายกฎจะสร้างหนึ่งแถวต่อการตรงกันแต่ละครั้ง ดังนั้น `EventID` จึง*ไม่ใช่*ค่าที่ไม่ซ้ำกัน สำหรับ `aws-ct-search` แต่ละเหตุการณ์ที่ตรงกันจะสร้างหนึ่งแถว

```sql
-- Critical and high alerts in a time range, with their ATT&CK techniques.
-- `Level` is an ENUM, so cast the literal to compare by severity rather than alphabetically.
SELECT Timestamp, RuleTitle, EventName, SrcIP, TechniqueIDs
FROM timeline
WHERE Level >= 'high'::suzaku_level
  AND Timestamp BETWEEN TIMESTAMP '2024-01-01' AND TIMESTAMP '2024-02-01'
  AND ErrorCode IS NULL          -- the call succeeded
ORDER BY Timestamp;

-- ATT&CK technique coverage, no string parsing required
SELECT technique, count(*) AS hits
FROM (SELECT unnest(TechniqueIDs) AS technique FROM timeline)
GROUP BY 1 ORDER BY hits DESC;
```

Suzaku จะ checkpoint ฐานข้อมูลก่อนจบการทำงาน ไฟล์ `.duckdb` จึงสมบูรณ์และเปิดแบบอ่านอย่างเดียวได้ (ให้คัดลอกหลังคำสั่งทำงานเสร็จ ไม่ใช่ระหว่างที่กำลังทำงาน)
