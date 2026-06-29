# دعم Sigma الأصلي

يتمتع Suzaku بدعم أصلي جيد جدًا لمواصفات Sigma ويدعم جميع معدِّلات الحقول باستثناء [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) الذي يتطلب بعض الإعداد المسبق.

اعتبارًا من الإصدار 1.0.0، يدعم Suzaku أيضًا قواعد الارتباط (correlation rules) التي تُعد مهمة لاكتشاف الهجمات في سجلات السحابة.

> ملاحظة: حاليًا، تحتاج إلى إنشاء قواعد الارتباط في ملفات منفردة.

## قواعد عدّ الأحداث

هذه قواعد تقوم بعدّ أحداث معينة وتُطلق تنبيهًا إذا وقع عدد كبير جدًا أو غير كافٍ من هذه الأحداث ضمن إطار زمني محدد.
أمثلة شائعة لاكتشاف العديد من الأحداث ضمن فترة زمنية معينة هي اكتشاف هجمات تخمين كلمات المرور، وهجمات رش كلمات المرور (password spray)، وهجمات حجب الخدمة.
يمكنك أيضًا استخدام هذه القواعد لاكتشاف مشكلات موثوقية مصدر السجل، مثل عندما تنخفض أحداث معينة دون عتبة معينة.

### مثال على قاعدة عدّ الأحداث

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

## قواعد عدّ القيم

تقوم هذه القواعد بعدّ نفس الأحداث ضمن إطار زمني بقيم **مختلفة** لحقل معين.

أمثلة:
- عمليات مسح الشبكة حيث يحاول عنوان IP مصدر واحد الاتصال بالعديد من عناوين IP و/أو المنافذ الوجهة المختلفة.
- هجمات رش كلمات المرور حيث يفشل مصدر واحد في المصادقة مع العديد من المستخدمين المختلفين.
- اكتشاف أدوات مثل BloodHound التي تعدّد العديد من مجموعات AD ذات الامتيازات العالية ضمن إطار زمني قصير.

### مثال على قاعدة عدّ القيم

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

## قواعد التقارب الزمني

يجب أن تقع جميع الأحداث المحددة بواسطة القواعد المشار إليها بحقل القاعدة ضمن الإطار الزمني المحدد بواسطة timespan.
يجب أن تكون قيم الحقول المحددة في `group-by` كلها بنفس القيمة (مثل: نفس المضيف، المستخدم، إلخ...).

من الأمثلة على ذلك استدعاءات API الاستطلاعية المحددة في ثلاث قواعد Sigma المستدعاة بترتيب عشوائي خلال 5 دقائق من نفس عنوان IP المصدر.

### مثال على قاعدة التقارب الزمني

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

## قواعد الترتيب الزمني

يتصرف نوع الارتباط `temporal_ordered` مثل `temporal` ويتطلب بالإضافة إلى ذلك أن تظهر الأحداث بالترتيب المُقدَّم في خاصية `rules`.

من الأمثلة على ذلك العديد من محاولات تسجيل الدخول الفاشلة متبوعة بتسجيل دخول ناجح.

### مثال على قاعدة الترتيب الزمني

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
