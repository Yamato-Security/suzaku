# Native Sigma Support

Suzaku သည် Sigma သတ်မှတ်ချက်အတွက် အလွန်ကောင်းမွန်သော native support ရှိပြီး [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) (ကြိုတင်ပြင်ဆင်မှု အချို့ လိုအပ်သည်) မှလွဲ၍ field modifier အားလုံးကို ပံ့ပိုးပေးသည်။

ဗားရှင်း 1.0.0 မှစ၍ Suzaku သည် cloud log များတွင် တိုက်ခိုက်မှုများကို ရှာဖွေဖော်ထုတ်ရန် အရေးကြီးသော correlation rule များကိုလည်း ပံ့ပိုးပေးသည်။

> မှတ်ချက်- လက်ရှိတွင် correlation rule များကို single file များအဖြစ် ဖန်တီးရန် လိုအပ်ပါသည်။

## Event Count Rules

ဤ rule များသည် အချို့သော event များကို ရေတွက်ပြီး၊ ၎င်း event အရေအတွက် များလွန်းခြင်း သို့မဟုတ် နည်းလွန်းခြင်းဖြစ်ပါက သတ်မှတ်ထားသော အချိန်ကာလအတွင်း သတိပေးချက်ထုတ်ပေးသော rule များ ဖြစ်သည်။
သတ်မှတ်အချိန်ကာလအတွင်း event များစွာကို ရှာဖွေဖော်ထုတ်ခြင်း၏ ဘုံဥပမာများမှာ password ခန့်မှန်းတိုက်ခိုက်မှုများ၊ password spray တိုက်ခိုက်မှုများနှင့် denial of service တိုက်ခိုက်မှုများကို ရှာဖွေဖော်ထုတ်ခြင်းတို့ ဖြစ်သည်။
အချို့သော event များသည် သတ်မှတ်ထားသော threshold အောက်သို့ ကျဆင်းသွားသည့်အခါကဲ့သို့သော log source ၏ ယုံကြည်စိတ်ချရမှု ပြဿနာများကို ရှာဖွေဖော်ထုတ်ရန်လည်း ဤ rule များကို သုံးနိုင်သည်။

### Event Count Rule Example

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

## Value Count Rules

ဤ rule များသည် သတ်မှတ်အချိန်ကာလအတွင်း တူညီသော event များကို ပေးထားသော field ၏ **မတူညီသော** တန်ဖိုးများဖြင့် ရေတွက်သည်။

ဥပမာများ-
- single source IP address တစ်ခုက မတူညီသော destination IP address များ နှင့်/သို့မဟုတ် port များစွာသို့ ချိတ်ဆက်ရန် ကြိုးစားသော network scan များ။
- single source တစ်ခုက မတူညီသော user များစွာဖြင့် authenticate လုပ်ရာတွင် ကျရှုံးသော password spraying တိုက်ခိုက်မှုများ။
- တိုတောင်းသော အချိန်ကာလအတွင်း high-privilege AD group များစွာကို စာရင်းကောက်ယူသော BloodHound ကဲ့သို့သော tool များကို ရှာဖွေဖော်ထုတ်ခြင်း။

### Value Count Rule Example

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

## Temporal Proximity Rules

rule field မှ ရည်ညွှန်းထားသော rule များက သတ်မှတ်ထားသော event အားလုံးသည် timespan က သတ်မှတ်ထားသော အချိန်ကာလအတွင်း ဖြစ်ပေါ်ရမည်။
`group-by` တွင် သတ်မှတ်ထားသော field များ၏ တန်ဖိုးများသည် တူညီသော တန်ဖိုး ဖြစ်ရမည် (ဥပမာ- တူညီသော host၊ user စသည်...)။

ဥပမာတစ်ခုမှာ တူညီသော source IP address တစ်ခုမှ မိနစ် 5 အတွင်း ကြိုက်နှစ်သက်ရာ အစီအစဉ်ဖြင့် ခေါ်ဆိုထားသော Sigma rule သုံးခုတွင် သတ်မှတ်ထားသော reconnaissance API call များ ဖြစ်သည်။

### Temporal Proximity Rule Example

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

## Temporal Ordered Rules

`temporal_ordered` correlation အမျိုးအစားသည် `temporal` ကဲ့သို့ ပြုမူပြီး၊ ထို့အပြင် event များသည် `rules` attribute တွင် ပေးထားသော အစီအစဉ်အတိုင်း ပေါ်လာရန် လိုအပ်သည်။

ဥပမာတစ်ခုမှာ login ကျရှုံးမှုများစွာ ပြီးနောက် login အောင်မြင်မှုတစ်ခု ဖြစ်သည်။

### Temporal Ordered Rule Example

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
