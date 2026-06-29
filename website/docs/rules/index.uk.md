# Нативна підтримка Sigma

Suzaku має дуже хорошу нативну підтримку специфікації Sigma і підтримує всі модифікатори полів, окрім [expand](https://sigmahq.io/docs/basics/modifiers.html#expand), який потребує деякої попередньої конфігурації.

Починаючи з версії 1.0.0, Suzaku також підтримує кореляційні правила, які є важливими для виявлення атак у хмарних логах.

> Примітка: Наразі вам потрібно створювати кореляційні правила в окремих файлах.

## Правила підрахунку подій

Це правила, які підраховують певні події та сповіщають, якщо протягом певного проміжку часу відбувається занадто багато або недостатньо таких подій.
Поширеними прикладами виявлення великої кількості подій протягом певного періоду часу є виявлення атак підбору паролів, атак розпилення паролів та атак на відмову в обслуговуванні.
Ви також можете використовувати ці правила для виявлення проблем з надійністю джерела логів, наприклад, коли певні події опускаються нижче певного порогу.

### Приклад правила підрахунку подій

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

## Правила підрахунку значень

Ці правила підраховують однакові події протягом проміжку часу з **різними** значеннями заданого поля.

Приклади:
- Сканування мережі, коли одна вихідна IP-адреса намагається підключитися до багатьох різних адрес призначення та/або портів.
- Атаки розпилення паролів, коли одне джерело не може автентифікуватися з багатьма різними користувачами.
- Виявлення інструментів на кшталт BloodHound, які перераховують багато груп AD з високими привілеями за короткий проміжок часу.

### Приклад правила підрахунку значень

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

## Правила часової близькості

Усі події, визначені правилами, на які посилається поле rule, повинні відбутися у проміжку часу, визначеному timespan.
Значення полів, визначених у `group-by`, повинні всі мати однакове значення (наприклад: той самий хост, користувач тощо).

Прикладом є розвідувальні виклики API, визначені у трьох правилах Sigma, викликані у довільному порядку протягом 5 хвилин з однієї вихідної IP-адреси.

### Приклад правила часової близькості

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

## Правила часової впорядкованості

Кореляційний тип `temporal_ordered` поводиться як `temporal` і додатково вимагає, щоб події з'являлися у порядку, наданому в атрибуті `rules`.

Прикладом є багато невдалих входів, за якими слідує успішний вхід.

### Приклад правила часової впорядкованості

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
