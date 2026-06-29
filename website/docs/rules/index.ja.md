# ネイティブSigmaサポート

SuzakuはSigma仕様に対して非常に優れたネイティブサポートを提供しており、[expand](https://sigmahq.io/docs/basics/modifiers.html#expand) を除くすべてのフィールド修飾子に対応しています。

バージョン1.0.0以降では、Suzakuはクラウドログにおける攻撃検出に重要な相関ルール（Correlation Rules）にも対応しています。

> 注意：現在のところ、相関ルールは1つのファイル内に作成する必要があります。

## EventCountルール

これは、特定のイベントをカウントし、一定の期間内にその数が多すぎる、または少なすぎる場合にアラートを出すルールです。
一定時間内に多数のイベントを検出する一般的な例としては、パスワード推測攻撃、パスワードスプレー攻撃、サービス拒否（DoS）攻撃の検出があります。
また、これらのルールは、特定のイベントの発生数がしきい値を下回る場合など、ログソースの信頼性に関する問題の検出にも使用できます。

### EventCountルールの例

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

## ValueCountルール

これらのルールは、指定された時間枠内で **異なる**値 を持つ特定のフィールドに基づいて、同じ種類のイベントをカウントします。

例：
- 単一の送信元IPアドレスが多数の異なる宛先IPアドレスやポートに接続を試みるネットワークスキャン
- 単一の送信元が多数の異なるユーザーに対して認証に失敗するパスワードスプレー攻撃
- 短時間で多くの高権限のActive Directoryグループを列挙するBloodHoundのようなツールの検出

### ValueCountルールの例

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

## Temporalルール

`rule`フィールドで参照されるルールによって定義されたすべてのイベントは、timespan で定義された時間枠内に発生している必要があります。
また、`group-by`で定義されたフィールドの値はすべて同じでなければなりません（例：同じホスト、同じユーザーなど）。

例としては、同一の送信元IPアドレスから、3つのSigmaルールで定義された偵察用API呼び出しが5分以内に任意の順序で実行されたケースが挙げられます。

### Temporalルールの例

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

## Temporal Orderedルール

`temporal_ordered`相関タイプは`temporal`と同様に動作しますが、加えて`rules`属性で指定された順序でイベントが発生する必要があります。

例としては、複数回のログイン失敗の後にログイン成功が続く場合などが挙げられます。

### Temporal Orderedルールの例

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
