# คำสั่งการวิเคราะห์

## คำสั่ง `aws-ct-metrics`

ใช้คำสั่งนี้เพื่อสร้างเมตริกบนฟิลด์ภายในล็อก AWS CloudTrail
โดยค่าเริ่มต้น จะทำการสแกนฟิลด์ `eventName`
ปัจจุบันเราใช้คำสั่งนี้เพื่อค้นหาว่า API call ใดที่พบบ่อยที่สุด เพื่อจัดลำดับความสำคัญในการเขียนกฎการตรวจจับ

## การใช้งานคำสั่ง
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -F, --field-name <FIELD_NAME>  The field to generate metrics for [default: eventName]
  -o, --output <FILE>            Output CSV

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
```

### ตัวอย่างคำสั่ง `aws-ct-metrics`

* แสดงตารางของ API call `eventName` ออกสู่หน้าจอ: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* บันทึกเป็นไฟล์ CSV: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
