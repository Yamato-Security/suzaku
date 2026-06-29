# Yerel Sigma Desteği

Suzaku, Sigma spesifikasyonu için çok iyi bir yerel desteğe sahiptir ve bir miktar ön yapılandırma gerektiren [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) dışındaki tüm alan değiştiricilerini destekler.

Sürüm 1.0.0 itibarıyla Suzaku, bulut günlüklerindeki saldırıları tespit etmek için önemli olan korelasyon kurallarını da destekler.

> Not: Şu anda korelasyon kurallarını tek dosyalar halinde oluşturmanız gerekmektedir.

## Olay Sayısı Kuralları

Bunlar, belirli olayları sayan ve bu olaylardan çok fazla veya yeterince olmayan sayıda bir zaman dilimi içinde meydana gelmesi durumunda uyarı veren kurallardır.
Belirli bir zaman diliminde çok sayıda olayın tespit edilmesine ilişkin yaygın örnekler, parola tahmin saldırılarını, parola püskürtme saldırılarını ve hizmet reddi saldırılarını tespit etmeye yöneliktir.
Bu kuralları, belirli olayların belirli bir eşiğin altına düşmesi gibi günlük kaynağı güvenilirliği sorunlarını tespit etmek için de kullanabilirsiniz.

### Olay Sayısı Kuralı Örneği

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

## Değer Sayısı Kuralları

Bu kurallar, belirli bir alanın **farklı** değerleriyle aynı olayları bir zaman dilimi içinde sayar.

Örnekler:
- Tek bir kaynak IP adresinin birçok farklı hedef IP adresine ve/veya bağlantı noktasına bağlanmaya çalıştığı ağ taramaları.
- Tek bir kaynağın birçok farklı kullanıcıyla kimlik doğrulamasını başaramadığı parola püskürtme saldırıları.
- Kısa bir zaman dilimi içinde birçok yüksek ayrıcalıklı AD grubunu numaralandıran BloodHound gibi araçların tespit edilmesi.

### Değer Sayısı Kuralı Örneği

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

## Zamansal Yakınlık Kuralları

Kural alanı tarafından atıfta bulunulan kurallarca tanımlanan tüm olaylar, timespan tarafından tanımlanan zaman diliminde meydana gelmelidir.
`group-by` içinde tanımlanan alanların değerlerinin tümü aynı değere sahip olmalıdır (örneğin: aynı ana bilgisayar, kullanıcı vb.).

Bir örnek, aynı kaynak IP adresinden 5 dakika içinde rastgele sırayla çağrılan üç Sigma kuralında tanımlanan keşif API çağrılarıdır.

### Zamansal Yakınlık Kuralı Örneği

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

## Zamansal Sıralı Kurallar

`temporal_ordered` korelasyon türü `temporal` gibi davranır ve buna ek olarak olayların `rules` özniteliğinde verilen sırada görünmesini gerektirir.

Bir örnek, birçok başarısız oturum açma denemesinin ardından başarılı bir oturum açmadır.

### Zamansal Sıralı Kural Örneği

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
