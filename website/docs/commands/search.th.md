# คำสั่งการค้นหา

## คำสั่ง `aws-ct-search`

ใช้คำสั่งนี้เพื่อดึงเหตุการณ์ที่ต้องการออกจากล็อก AWS CloudTrail โดยไม่ต้องเขียนกฎ Sigma
เหตุการณ์ที่ตรงกันจะแสดงหรือบันทึกในรูปแบบเดียวกับ `aws-ct-timeline` จึงสามารถดูผลการค้นหาด้วยคอลัมน์ไทม์ไลน์ที่คุ้นเคยได้ทันที

มีสามวิธีในการจำกัดเหตุการณ์ และสามารถใช้ร่วมกันได้:

| ตัวเลือก | สิ่งที่ใช้เปรียบเทียบ | หลายค่า |
|---|---|---|
| `-F, --filter FIELD:VALUE` | ค่าของฟิลด์เดียว **ตรงกันทุกประการ แยกตัวพิมพ์เล็ก-ใหญ่** | ระบุ `-F` ซ้ำ; ต้องตรงกับตัวกรอง**ทั้งหมด** (AND) |
| `-k, --keyword KEYWORD` | สตริงย่อยที่ใดก็ได้ใน JSON ของเหตุการณ์ ไม่แยกตัวพิมพ์เล็ก-ใหญ่ (แยกเมื่อใช้ `-c`) | ระบุ `-k` ซ้ำ; ตรงกับคีย์เวิร์ด**ใดก็ได้**ก็เพียงพอ (OR) |
| `-r, --regex REGEX` | นิพจน์ปกติที่ใดก็ได้ใน JSON ของเหตุการณ์ | รูปแบบเดียว (ใช้เครื่องหมายไปป์ที่ไม่เอสเคป <code>&#124;</code> สำหรับทางเลือก) |

แต่ละเหตุการณ์จะถูกตรวจสอบตามลำดับนี้ และจะแสดงเฉพาะเหตุการณ์ที่ผ่านการตรวจสอบทั้งหมด: ช่วงเวลา (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`

ใช้ `-F` เมื่อทราบว่าค่าอยู่ในฟิลด์ใด (เช่น `eventName:ConsoleLogin`) และใช้ `-k` หรือ `-r` เมื่อทราบเพียงสตริงแต่ไม่ทราบว่าปรากฏที่ใด

### การทำงานของ `-F, --filter`

* รูปแบบคือ `FIELD:VALUE` อาร์กิวเมนต์จะถูกแยกที่ `:` **ตัวแรก** ดังนั้นค่าที่มีเครื่องหมายโคลอน เช่น ARN จึงใช้ได้ตามเดิม
* ฟิลด์ที่ซ้อนกันเขียนด้วยสัญกรณ์จุด (`userIdentity.type`) โดย `.` นำหน้าจะใส่หรือไม่ก็ได้ ดังนั้น `.userIdentity.arn` ก็ใช้ได้
* เครื่องหมายคำพูดรอบค่า (`"..."` หรือ `'...'`) จะถูกตัดออก
* ตัวเลขและค่าบูลีนจะเปรียบเทียบเป็นสตริง ดังนั้น `-F readOnly:false` และ `-F responseElements.user.userId:12345` จึงใช้ได้
* เหตุการณ์ที่ไม่มีฟิลด์นั้นจะไม่ตรงกัน
* ไม่สามารถระบุองค์ประกอบภายในอาร์เรย์ได้ (ไม่มีไวยากรณ์แบบ `items.0.name`)
* ตัวกรองที่ไม่มี `:` จะถูกปฏิเสธก่อนเริ่มสแกน: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`

> หมายเหตุ: ใน `aws-ct-metrics` `-F` มีความหมายต่างกัน คือ `--field-name` และรับเพียงชื่อฟิลด์ ส่วนใน `aws-ct-search` คือ `--filter` และรับ `FIELD:VALUE` เสมอ

## การใช้งานคำสั่ง
```
Usage: suzaku aws-ct-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory of multiple gz/json/parquet files
  -f, --file <FILE>      File path to one gz/json/parquet file

Filtering:
  -F, --filter <FILTER...>     Filter by specific field(s)
  -c, --preserve-case          Case-sensitive keyword search
  -k, --keyword <KEYWORD...>   Search by keyword(s)
  -r, --regex <REGEX>          Search by regular expression
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
      --file-date-to <DATE>    Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")

Output:
  -C, --clobber                   Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>   Add GeoIP (ASN, city, country) info to IP addresses
  -o, --output <FILE>             Save the results to a file
  -t, --output-type <FORMAT,...>  Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [default: csv] [possible values: csv, json, jsonl, duckdb]
      --raw-output                Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>   Number of threads to use (default: same as CPU cores)

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### ตัวอย่างคำสั่ง `aws-ct-search`

กรองตามฟิลด์ (`-F`):

* ค้นหาการเข้าสู่ระบบคอนโซล: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* ค้นหาการเรียก API โดยผู้ใช้ root (ฟิลด์ที่ซ้อนกัน): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* ค้นหาทุกการกระทำของผู้ใช้ IAM รายหนึ่ง (ค่ามีเครื่องหมายโคลอน): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* ค้นหาการเรียกที่ถูกปฏิเสธ: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* ค้นหาการเรียก IAM แบบเขียน (ไม่ใช่อ่านอย่างเดียว) ใน `us-east-1` (ตัวกรองหลายตัวเชื่อมด้วย AND): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* ค้นหาการเข้าถึง S3 bucket ที่ระบุ (ฟิลด์เฉพาะของ API): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

ค้นหาด้วยคีย์เวิร์ด (`-k`) และนิพจน์ปกติ (`-r`):

* ค้นหาเหตุการณ์ที่กล่าวถึงที่อยู่ IP ที่ใดก็ได้: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* ค้นหาเหตุการณ์ที่กล่าวถึงผู้ใช้คนใดคนหนึ่งจากสองคน (คีย์เวิร์ดหลายคำเชื่อมด้วย OR): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* ค้นหาคีย์เวิร์ดแบบแยกตัวพิมพ์เล็ก-ใหญ่: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* ค้นหาความพยายามปิดใช้งาน CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

รวมเงื่อนไขและบันทึกผลลัพธ์:

* ค้นหาคีย์เวิร์ดโดยจำกัดเฉพาะแหล่งเหตุการณ์เดียว: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* การเข้าสู่ระบบคอนโซลที่ล้มเหลวใน 7 วันที่ผ่านมา: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* บันทึกผลลัพธ์เป็น CSV และ DuckDB พร้อมข้อมูล GeoIP: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* บันทึก JSON ต้นฉบับของเหตุการณ์ที่ตรงกัน: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### ผลลัพธ์ของ `aws-ct-search`

คอลัมน์ผลลัพธ์เหมือนกับ [`aws-ct-timeline`](dfir-timeline.md) และผลลัพธ์ DuckDB ก็เป็นไปตาม[สคีมาผลลัพธ์ DuckDB](dfir-timeline.md#duckdb-output-schema) เดียวกัน
เมื่อจบจะแสดงจำนวนเหตุการณ์ที่สแกนและจำนวนที่ตรงกัน:

```
Total events scanned: 209
Matching events: 1
```
