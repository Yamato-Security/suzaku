# Soporte nativo de Sigma

Suzaku tiene un soporte nativo muy bueno para la especificación Sigma y admite todos los modificadores de campo excepto [expand](https://sigmahq.io/docs/basics/modifiers.html#expand), que requiere cierta configuración previa.

A partir de la versión 1.0.0, Suzaku también admite reglas de correlación, que son importantes para detectar ataques en los registros de la nube.

> Nota: Actualmente, es necesario crear las reglas de correlación en archivos individuales.

## Reglas de conteo de eventos

Estas son reglas que cuentan ciertos eventos y alertan si ocurren demasiados o muy pocos de estos eventos dentro de un periodo de tiempo.
Ejemplos comunes de detección de muchos eventos dentro de un periodo de tiempo determinado son la detección de ataques de adivinación de contraseñas, ataques de pulverización de contraseñas y ataques de denegación de servicio.
También podría usar estas reglas para detectar problemas de fiabilidad de la fuente de registros, como cuando ciertos eventos caen por debajo de un umbral determinado.

### Ejemplo de regla de conteo de eventos

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

## Reglas de conteo de valores

Estas reglas cuentan los mismos eventos dentro de un periodo de tiempo con valores **diferentes** de un campo determinado.

Ejemplos:
- Escaneos de red donde una única dirección IP de origen intenta conectarse a muchas direcciones IP de destino y/o puertos diferentes.
- Ataques de pulverización de contraseñas donde un único origen falla al autenticarse con muchos usuarios diferentes.
- Detección de herramientas como BloodHound que enumeran muchos grupos de AD con privilegios elevados en un periodo de tiempo corto.

### Ejemplo de regla de conteo de valores

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

## Reglas de proximidad temporal

Todos los eventos definidos por las reglas referidas por el campo rule deben ocurrir en el periodo de tiempo definido por timespan.
Los valores de los campos definidos en `group-by` deben tener todos el mismo valor (ej.: mismo host, usuario, etc...).

Un ejemplo son las llamadas a la API de reconocimiento definidas en tres reglas Sigma invocadas en orden arbitrario dentro de un lapso de 5 minutos desde la misma dirección IP de origen.

### Ejemplo de regla de proximidad temporal

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

## Reglas temporales ordenadas

El tipo de correlación `temporal_ordered` se comporta como `temporal` y requiere además que los eventos aparezcan en el orden proporcionado en el atributo `rules`.

Un ejemplo son muchos inicios de sesión fallidos seguidos de un inicio de sesión exitoso.

### Ejemplo de regla temporal ordenada

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
