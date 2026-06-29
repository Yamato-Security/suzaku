# 原生 Sigma 支援

Suzaku 對 Sigma 規範提供了非常好的原生支援，並支援所有欄位修飾子，除了 [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) 之外，因為它需要一些事前設定。

從 1.0.0 版開始，Suzaku 也支援關聯規則，這對於偵測雲端日誌中的攻擊非常重要。

> 注意：目前你需要在單一檔案中建立關聯規則。

## 事件計數規則

這些規則會計算特定事件的數量，並在某個時間範圍內這些事件發生太多或不足時發出警示。
偵測某個時間範圍內大量事件的常見範例，包括偵測密碼猜測攻擊、密碼噴灑攻擊以及阻斷服務攻擊。
你也可以使用這些規則來偵測日誌來源的可靠性問題，例如當某些事件低於特定門檻時。

### 事件計數規則範例

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

## 值計數規則

這些規則會在一個時間範圍內，計算同一事件中某個指定欄位的**不同**值的數量。

範例：
- 網路掃描，其中單一來源 IP 位址嘗試連線到許多不同的目的地 IP 位址與／或連接埠。
- 密碼噴灑攻擊，其中單一來源以許多不同的使用者進行驗證失敗。
- 偵測像 BloodHound 這類在短時間範圍內列舉許多高權限 AD 群組的工具。

### 值計數規則範例

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

## 時間鄰近性規則

由 rule 欄位所參照的規則所定義的所有事件，都必須在 timespan 所定義的時間範圍內發生。
在 `group-by` 中定義的欄位值必須全部具有相同的值（例如：相同的主機、使用者等等）。

一個範例是在三個 Sigma 規則中定義的偵察 API 呼叫，在 5 分鐘內以任意順序從相同的來源 IP 位址被呼叫。

### 時間鄰近性規則範例

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

## 時間順序規則

`temporal_ordered` 關聯類型的行為與 `temporal` 相同，但額外要求事件必須依照 `rules` 屬性中所提供的順序出現。

一個範例是多次登入失敗之後接著一次成功登入。

### 時間順序規則範例

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
