# การอ่านไฟล์ CSV ที่ดาวน์โหลดจากคอนโซล AWS

คำสั่ง AWS ของ Suzaku (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) สามารถอ่านไฟล์ที่ดาวน์โหลดจาก **CloudTrail → Event history → Download events → CSV** ใน AWS Management Console ด้วย `-f/--file` หรือ `-d/--directory`

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

CSV มีประโยชน์เมื่อไม่มี Trail ส่ง log ไปยัง S3 แต่เป็นเพียง **ข้อมูลสรุป** ของแต่ละ event ไม่ใช่ CloudTrail record แบบเต็ม หากมี Trail JSON log หรือ JSON download จาก Event history ควรใช้ JSON ก่อน

## วิธีแปลงข้อมูล

แต่ละแถวจะถูกสร้างใหม่เป็น CloudTrail record เช่น `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]` และ `Read-only` → boolean `readOnly` ช่องว่างจะถูกมองว่าไม่มี field เพื่อให้ rule ที่ใช้ `errorCode|exists` แยก call ที่สำเร็จและล้มเหลวได้ถูกต้อง และสำหรับ `root` จะเพิ่ม `userIdentity.type: Root`

รองรับเฉพาะชื่อคอลัมน์ภาษาอังกฤษ CSV ที่ไม่ใช่รูปแบบดาวน์โหลด CloudTrail จาก console จะถูกปฏิเสธ แถวที่จำนวน field ไม่ถูกต้อง (มักเกิดจาก quote เสีย) หรือไม่มี `Event time` จะถูกข้ามและบันทึกหมายเลขแถวไว้ใน error log

## ข้อจำกัด

1. **ดาวน์โหลดทุก region ที่ต้องใช้ รวมถึง `us-east-1`** Console ส่งออกได้ครั้งละหนึ่ง region และ console sign-in กับ IAM event ถูกบันทึกใน `us-east-1`
2. **บาง rule ไม่สามารถ match ได้** CSV ไม่มี `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` หรือ `additionalEventData` ทำให้ประมาณ 22 จาก 796 CloudTrail rule รวมถึง failed login และ login ที่ไม่มี MFA ไม่สามารถ match ได้
3. **บางคอลัมน์ใน timeline จะว่าง** `UserARN`, `UserAccountID`, `UserPrincipalID` และ `ErrorMessage` ว่างเสมอ ส่วน `UserType` จะว่างยกเว้น root
4. **`aws-ct-summary` จัดกลุ่มตามชื่อผู้ใช้** สำหรับ role ค่านี้คือ session name ดังนั้น role ต่างกันที่ใช้ชื่อเดียวกันจะถูกรวม แถวที่ไม่มีชื่อผู้ใช้จะไม่อยู่ใน summary ทำให้จำนวนอาจต่างจาก JSON ในช่วงเวลาเดียวกัน
5. **Google Workspace ไม่รองรับ CSV** ใช้ Reports API JSON/JSONL/JSON.GZ กับ `gws-timeline` แทน เพราะ Admin console CSV เป็น schema อื่นที่สูญเสียข้อมูล
