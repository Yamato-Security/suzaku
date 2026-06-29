# Prise en charge native de Sigma

Suzaku offre une très bonne prise en charge native de la spécification Sigma et prend en charge tous les modificateurs de champ à l'exception de [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) qui nécessite une configuration préalable.

Depuis la version 1.0.0, Suzaku prend également en charge les règles de corrélation, qui sont importantes pour détecter les attaques dans les journaux cloud.

> Note : Actuellement, vous devez créer les règles de corrélation dans des fichiers uniques.

## Règles de comptage d'événements

Ce sont des règles qui comptent certains événements et alertent si un nombre trop élevé ou trop faible de ces événements survient dans une fenêtre temporelle.
Des exemples courants de détection de nombreux événements sur une certaine période sont la détection d'attaques de devinette de mot de passe, d'attaques par pulvérisation de mots de passe et d'attaques par déni de service.
Vous pourriez également utiliser ces règles pour détecter des problèmes de fiabilité de la source de journaux, par exemple lorsque certains événements tombent en dessous d'un certain seuil.

### Exemple de règle de comptage d'événements

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

## Règles de comptage de valeurs

Ces règles comptent les mêmes événements dans une fenêtre temporelle avec des valeurs **différentes** d'un champ donné.

Exemples :
- Balayages réseau où une seule adresse IP source tente de se connecter à de nombreuses adresses IP et/ou ports de destination différents.
- Attaques par pulvérisation de mots de passe où une seule source échoue à s'authentifier avec de nombreux utilisateurs différents.
- Détecter des outils comme BloodHound qui énumèrent de nombreux groupes AD à privilèges élevés dans un court laps de temps.

### Exemple de règle de comptage de valeurs

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

## Règles de proximité temporelle

Tous les événements définis par les règles référencées par le champ rule doivent survenir dans la fenêtre temporelle définie par timespan.
Les valeurs des champs définis dans `group-by` doivent toutes avoir la même valeur (ex : même hôte, même utilisateur, etc.).

Un exemple est constitué d'appels d'API de reconnaissance définis dans trois règles Sigma invoquées dans un ordre arbitraire en moins de 5 minutes depuis la même adresse IP source.

### Exemple de règle de proximité temporelle

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

## Règles temporelles ordonnées

Le type de corrélation `temporal_ordered` se comporte comme `temporal` et exige en plus que les événements apparaissent dans l'ordre fourni dans l'attribut `rules`.

Un exemple est constitué de nombreuses connexions échouées suivies d'une connexion réussie.

### Exemple de règle temporelle ordonnée

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
