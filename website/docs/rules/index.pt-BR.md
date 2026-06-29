# Suporte Nativo ao Sigma

O Suzaku tem um suporte nativo muito bom à especificação Sigma e oferece suporte a todos os modificadores de campo, exceto [expand](https://sigmahq.io/docs/basics/modifiers.html#expand), que requer alguma configuração prévia.

A partir da versão 1.0.0, o Suzaku também oferece suporte a regras de correlação, que são importantes para detectar ataques em logs de nuvem.

> Nota: Atualmente, é necessário criar as regras de correlação em arquivos únicos.

## Regras de Contagem de Eventos

Estas são regras que contam determinados eventos e alertam caso ocorram eventos demais ou de menos dentro de um intervalo de tempo.
Exemplos comuns de detecção de muitos eventos dentro de um determinado período de tempo são a detecção de ataques de adivinhação de senha, ataques de password spray e ataques de negação de serviço.
Você também poderia usar essas regras para detectar problemas de confiabilidade da fonte de logs, como quando determinados eventos ficam abaixo de um certo limiar.

### Exemplo de Regra de Contagem de Eventos

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

## Regras de Contagem de Valores

Estas regras contam os mesmos eventos dentro de um intervalo de tempo com valores **diferentes** de um determinado campo.

Exemplos:
- Varreduras de rede em que um único endereço IP de origem tenta se conectar a muitos endereços IP de destino e/ou portas diferentes.
- Ataques de password spraying em que uma única origem falha na autenticação com muitos usuários diferentes.
- Detectar ferramentas como o BloodHound, que enumeram muitos grupos AD de alto privilégio em um curto intervalo de tempo.

### Exemplo de Regra de Contagem de Valores

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

## Regras de Proximidade Temporal

Todos os eventos definidos pelas regras referenciadas pelo campo rule devem ocorrer dentro do intervalo de tempo definido por timespan.
Os valores dos campos definidos em `group-by` devem ter todos o mesmo valor (ex: mesmo host, usuário, etc...).

Um exemplo são chamadas de API de reconhecimento definidas em três regras Sigma invocadas em ordem arbitrária dentro de 5 minutos a partir do mesmo endereço IP de origem.

### Exemplo de Regra de Proximidade Temporal

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

## Regras Temporais Ordenadas

O tipo de correlação `temporal_ordered` comporta-se como `temporal` e exige, adicionalmente, que os eventos apareçam na ordem fornecida no atributo `rules`.

Um exemplo são muitas tentativas de login malsucedidas seguidas de um login bem-sucedido.

### Exemplo de Regra Temporal Ordenada

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
