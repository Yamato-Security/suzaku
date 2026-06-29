# 네이티브 Sigma 지원

Suzaku는 Sigma 사양에 대한 매우 우수한 네이티브 지원을 제공하며, 일부 사전 구성이 필요한 [expand](https://sigmahq.io/docs/basics/modifiers.html#expand)를 제외한 모든 필드 수정자를 지원합니다.

버전 1.0.0부터 Suzaku는 클라우드 로그에서 공격을 탐지하는 데 중요한 상관관계 규칙(correlation rules)도 지원합니다.

> 참고: 현재는 상관관계 규칙을 단일 파일로 생성해야 합니다.

## 이벤트 개수 규칙(Event Count Rules)

특정 이벤트의 개수를 세어, 일정 기간 내에 이러한 이벤트가 너무 많거나 너무 적게 발생하면 경고하는 규칙입니다.
일정 기간 내에 많은 이벤트를 탐지하는 일반적인 예로는 패스워드 추측 공격, 패스워드 스프레이 공격, 서비스 거부 공격을 탐지하는 경우가 있습니다.
또한 이러한 규칙을 사용하여, 특정 이벤트가 일정 임계값 아래로 떨어지는 경우와 같은 로그 소스 신뢰성 문제를 탐지할 수도 있습니다.

### 이벤트 개수 규칙 예시

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

## 값 개수 규칙(Value Count Rules)

이러한 규칙은 일정 기간 내에서 주어진 필드의 **서로 다른** 값을 가지는 동일한 이벤트의 개수를 셉니다.

예시:
- 단일 출발지 IP 주소가 여러 다른 목적지 IP 주소 및/또는 포트에 연결을 시도하는 네트워크 스캔.
- 단일 출발지가 여러 다른 사용자로 인증에 실패하는 패스워드 스프레이 공격.
- 짧은 기간 내에 여러 고권한 AD 그룹을 열거하는 BloodHound와 같은 도구 탐지.

### 값 개수 규칙 예시

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

## 시간적 근접성 규칙(Temporal Proximity Rules)

rule 필드가 참조하는 규칙들에 정의된 모든 이벤트가 timespan에 정의된 기간 내에 발생해야 합니다.
`group-by`에 정의된 필드의 값은 모두 동일한 값(예: 동일한 호스트, 사용자 등)이어야 합니다.

한 가지 예로는 동일한 출발지 IP 주소에서 5분 이내에 임의의 순서로 호출되는, 세 개의 Sigma 규칙에 정의된 정찰 API 호출이 있습니다.

### 시간적 근접성 규칙 예시

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

## 시간적 순서 규칙(Temporal Ordered Rules)

`temporal_ordered` 상관관계 유형은 `temporal`과 유사하게 동작하며, 추가로 이벤트가 `rules` 속성에 제공된 순서대로 나타나야 합니다.

한 가지 예로는 여러 번의 로그인 실패 후 로그인 성공이 이어지는 경우가 있습니다.

### 시간적 순서 규칙 예시

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
