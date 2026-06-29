# Native Sigma-Unterstützung

Suzaku bietet eine sehr gute native Unterstützung für die Sigma-Spezifikation und unterstützt alle Feld-Modifikatoren außer [expand](https://sigmahq.io/docs/basics/modifiers.html#expand), der eine gewisse Vorkonfiguration erfordert.

Ab Version 1.0.0 unterstützt Suzaku auch Korrelationsregeln, die für die Erkennung von Angriffen in Cloud-Logs wichtig sind.

> Hinweis: Derzeit müssen Sie Korrelationsregeln in einzelnen Dateien erstellen.

## Ereigniszählungsregeln

Dies sind Regeln, die bestimmte Ereignisse zählen und einen Alarm auslösen, wenn zu viele oder zu wenige dieser Ereignisse innerhalb eines Zeitraums auftreten.
Häufige Beispiele für die Erkennung vieler Ereignisse innerhalb eines bestimmten Zeitraums sind die Erkennung von Passwort-Rate-Angriffen, Password-Spray-Angriffen und Denial-of-Service-Angriffen.
Sie könnten diese Regeln auch verwenden, um Zuverlässigkeitsprobleme von Log-Quellen zu erkennen, etwa wenn bestimmte Ereignisse unter einen bestimmten Schwellenwert fallen.

### Beispiel einer Ereigniszählungsregel

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

## Wertzählungsregeln

Diese Regeln zählen dieselben Ereignisse innerhalb eines Zeitraums mit **unterschiedlichen** Werten eines bestimmten Feldes.

Beispiele:
- Netzwerk-Scans, bei denen eine einzelne Quell-IP-Adresse versucht, sich mit vielen verschiedenen Ziel-IP-Adressen und/oder Ports zu verbinden.
- Password-Spraying-Angriffe, bei denen eine einzelne Quelle sich bei vielen verschiedenen Benutzern nicht authentifizieren kann.
- Erkennung von Tools wie BloodHound, die viele hochprivilegierte AD-Gruppen innerhalb eines kurzen Zeitraums aufzählen.

### Beispiel einer Wertzählungsregel

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

## Regeln für zeitliche Nähe

Alle durch die im Feld rule referenzierten Regeln definierten Ereignisse müssen innerhalb des durch timespan definierten Zeitraums auftreten.
Die Werte der in `group-by` definierten Felder müssen alle denselben Wert haben (z. B. derselbe Host, Benutzer usw.).

Ein Beispiel sind Reconnaissance-API-Aufrufe, die in drei Sigma-Regeln definiert sind und in beliebiger Reihenfolge innerhalb von 5 Minuten von derselben Quell-IP-Adresse aufgerufen werden.

### Beispiel einer Regel für zeitliche Nähe

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

## Regeln mit zeitlicher Reihenfolge

Der Korrelationstyp `temporal_ordered` verhält sich wie `temporal` und erfordert zusätzlich, dass die Ereignisse in der im Attribut `rules` angegebenen Reihenfolge auftreten.

Ein Beispiel sind viele fehlgeschlagene Anmeldungen, gefolgt von einer erfolgreichen Anmeldung.

### Beispiel einer Regel mit zeitlicher Reihenfolge

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
