# การรองรับ Sigma แบบเนทีฟ

Suzaku รองรับข้อกำหนดของ Sigma แบบเนทีฟได้เป็นอย่างดี และรองรับตัวปรับแต่งฟิลด์ทั้งหมดยกเว้น [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) ซึ่งต้องมีการกำหนดค่าล่วงหน้าบางอย่าง

ตั้งแต่เวอร์ชัน 1.0.0 เป็นต้นไป Suzaku ยังรองรับกฎความสัมพันธ์ (correlation rules) ซึ่งมีความสำคัญต่อการตรวจจับการโจมตีในบันทึกของคลาวด์

> หมายเหตุ: ในขณะนี้ คุณต้องสร้างกฎความสัมพันธ์ในไฟล์เดียว

## กฎการนับจำนวนเหตุการณ์ (Event Count Rules)

กฎเหล่านี้คือกฎที่นับเหตุการณ์บางอย่างและแจ้งเตือนหากมีเหตุการณ์เหล่านี้เกิดขึ้นมากเกินไปหรือน้อยเกินไปภายในกรอบเวลาที่กำหนด
ตัวอย่างทั่วไปของการตรวจจับเหตุการณ์จำนวนมากภายในช่วงเวลาหนึ่งคือการตรวจจับการโจมตีแบบเดารหัสผ่าน การโจมตีแบบ password spray และการโจมตีแบบปฏิเสธการให้บริการ
คุณยังสามารถใช้กฎเหล่านี้เพื่อตรวจจับปัญหาความน่าเชื่อถือของแหล่งบันทึก เช่น เมื่อเหตุการณ์บางอย่างลดต่ำกว่าค่าเกณฑ์ที่กำหนด

### ตัวอย่างกฎการนับจำนวนเหตุการณ์

```yml
title: Correlation Test
id: 49d15187-4203-4e11-8acd-8736f25b6609
status: test
author: TEST
correlation:
    type: event_count
    rules:
        - Console Login With MFA
    group-by:
        - sourceIPAddress
    timespan: 3d
    condition:
        gte: 3
        field: sourceIPAddress
    generate: true 
level: high
---
title: Console Login With MFA
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'Yes'
    condition: selection
level: informational
```

## กฎการนับจำนวนค่า (Value Count Rules)

กฎเหล่านี้นับเหตุการณ์เดียวกันภายในกรอบเวลาที่มีค่าของฟิลด์ที่กำหนด **แตกต่างกัน**

ตัวอย่าง:
- การสแกนเครือข่ายซึ่งที่อยู่ IP ต้นทางเพียงแห่งเดียวพยายามเชื่อมต่อกับที่อยู่ IP ปลายทางและ/หรือพอร์ตที่แตกต่างกันจำนวนมาก
- การโจมตีแบบ password spraying ซึ่งต้นทางเพียงแห่งเดียวพยายามยืนยันตัวตนล้มเหลวกับผู้ใช้ที่แตกต่างกันจำนวนมาก
- ตรวจจับเครื่องมืออย่าง BloodHound ที่ทำการแจกแจงกลุ่ม AD ที่มีสิทธิ์สูงจำนวนมากภายในกรอบเวลาสั้น ๆ

### ตัวอย่างกฎการนับจำนวนค่า

```yml
title: Correlation value_count Test
id: 49d15187-4203-4e11-8acd-8736f25b66xx
status: test
author: TEST
correlation:
    type: value_count
    rules:
        - Console Login Without MFA
    group-by:
        - sourceIPAddress
    timespan: 3d
    condition:
        gte: 2
        field: sourceIPAddress
    generate: true 
level: high
---
title: Console Login Without MFA
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'No'
    condition: selection
level: medium
```

## กฎความใกล้เคียงเชิงเวลา (Temporal Proximity Rules)

เหตุการณ์ทั้งหมดที่กำหนดโดยกฎที่อ้างถึงในฟิลด์ rule ต้องเกิดขึ้นในกรอบเวลาที่กำหนดโดย timespan
ค่าของฟิลด์ที่กำหนดใน `group-by` ต้องมีค่าเดียวกันทั้งหมด (เช่น โฮสต์เดียวกัน ผู้ใช้เดียวกัน ฯลฯ)

ตัวอย่างหนึ่งคือการเรียก API เพื่อสำรวจข้อมูลซึ่งกำหนดไว้ในกฎ Sigma สามกฎที่ถูกเรียกตามลำดับใดก็ได้ภายใน 5 นาทีจากที่อยู่ IP ต้นทางเดียวกัน

### ตัวอย่างกฎความใกล้เคียงเชิงเวลา

```yml
title: Correlation temporal Test
id: 49d15187-4203-4e11-8acd-8736f25b66xx
status: test
author: TEST
correlation:
    type: temporal
    rules:
        - CloudTrail Log Settings Modified
        - Console Login Without MFA
        - Role Enumeration
    timespan: 3d
    generate: true
level: high
---
title: CloudTrail Log Settings Modified
author: Zach Mathis (@yamatosecurity)
date: 2025-04-23
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'cloudtrail.amazonaws.com'
        eventName: 'UpdateTrail'
    filter:
        errorCode: 'AccessDenied'
    condition: selection and not filter
level: high
---
title: Console Login Without MFA
author: Zach Mathis (@yamatosecurity)
date: 2025-04-13
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'No'
    condition: selection
level: medium
---
title: Role Enumeration 
author: Zach Mathis (@yamatosecurity)
date: 2025-04-24
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'iam.amazonaws.com'
        eventName: 'ListRoles'
    condition: selection
falsepositives:
level: low
```

## กฎเชิงเวลาแบบมีลำดับ (Temporal Ordered Rules)

ประเภทความสัมพันธ์ `temporal_ordered` ทำงานเหมือน `temporal` และยังกำหนดเพิ่มเติมว่าเหตุการณ์ต้องปรากฏตามลำดับที่ให้ไว้ในแอตทริบิวต์ `rules`

ตัวอย่างหนึ่งคือการล็อกอินล้มเหลวหลายครั้งตามด้วยการล็อกอินสำเร็จ

### ตัวอย่างกฎเชิงเวลาแบบมีลำดับ

```yml
title: Correlation temporal_ordered Test
id: 49d15187-4203-4e11-8acd-8736f25b66xx
status: test
author: TEST
correlation:
    type: temporal_ordered
    rules:
        - Console Login Without MFA
        - Role Enumeration
        - CloudTrail Log Settings Modified
    timespan: 1d
    generate: true
level: high
---
title: CloudTrail Log Settings Modified
author: Zach Mathis (@yamatosecurity)
date: 2025-04-23
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'cloudtrail.amazonaws.com'
        eventName: 'UpdateTrail'
    filter:
        errorCode: 'AccessDenied'
    condition: selection and not filter
level: high
---
title: Console Login Without MFA
author: Zach Mathis (@yamatosecurity)
date: 2025-04-13
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'No'
    condition: selection
level: medium
---
title: Role Enumeration 
author: Zach Mathis (@yamatosecurity)
date: 2025-04-24
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'iam.amazonaws.com'
        eventName: 'ListRoles'
    condition: selection
falsepositives:
level: low
```
