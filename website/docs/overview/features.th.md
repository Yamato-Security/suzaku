# คุณสมบัติ

* รองรับการทำงานข้ามแพลตฟอร์ม: Windows, Linux, macOS
* พัฒนาด้วย Rust เพื่อความปลอดภัยของหน่วยความจำ ความรวดเร็ว และการทำงานแบบสแตนด์อโลน
* วิเคราะห์ log ของ AWS CloudTrail, Azure/Entra ID/M365 และ Google Workspace Reports API ด้วยการตรวจจับ Sigma แบบเนทีฟ
* สแกน `.json`, `.jsonl`, `.json.gz` แบบบีบอัด, Parquet และ [CloudTrail CSV จาก AWS Management Console ที่รองรับ](../getting-started/console-csv-input.md) แบบมัลติเธรด
* สร้างไทม์ไลน์เดียวที่วิเคราะห์ได้ง่ายสำหรับการสืบสวนทางนิติวิทยาศาสตร์และการตอบสนองต่อเหตุการณ์
* รองรับ IoC signatures ที่เขียนด้วยกฎ [Sigma](https://github.com/SigmaHQ/sigma) แบบ YML ที่อ่าน/สร้าง/แก้ไขได้ง่ายอย่างยอดเยี่ยมในระดับเนทีฟ (รองรับ Correlation rules และ field modifiers ทั้งหมด ยกเว้น [expand](https://sigmahq.io/docs/basics/modifiers.html#expand))
* สร้างสรุปการใช้งาน API ทั้งหมด รวมถึงเมตริกเกี่ยวกับผู้โจมตี (ที่อยู่ IP ต้นทาง, ตำแหน่งทางภูมิศาสตร์, ภูมิภาคที่ใช้, user agents ฯลฯ) เพื่อค้นหากิจกรรมที่ผิดปกติโดยไม่ต้องพึ่งพา signatures
* บันทึกผลลัพธ์เป็น CSV, JSON, JSONL และ DuckDB
