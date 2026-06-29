# Dukungan Sigma Native

Suzaku memiliki dukungan native yang sangat baik untuk spesifikasi Sigma dan mendukung semua modifier field kecuali [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) yang memerlukan beberapa pra-konfigurasi.

Mulai dari versi 1.0.0, Suzaku juga mendukung correlation rules yang penting untuk mendeteksi serangan pada log cloud.

> Catatan: Saat ini, Anda perlu membuat correlation rules dalam file tunggal.

## Event Count Rules

Ini adalah rules yang menghitung event tertentu dan memberikan peringatan jika terlalu banyak atau terlalu sedikit jumlah event ini terjadi dalam suatu jangka waktu.
Contoh umum mendeteksi banyak event dalam periode waktu tertentu adalah untuk mendeteksi serangan tebakan kata sandi, serangan password spray, dan serangan denial of service.
Anda juga dapat menggunakan rules ini untuk mendeteksi masalah keandalan sumber log, seperti ketika event tertentu turun di bawah ambang batas tertentu.

### Contoh Event Count Rule

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

Rules ini menghitung event yang sama dalam suatu jangka waktu dengan nilai yang **berbeda** dari suatu field tertentu.

Contoh:
- Pemindaian jaringan di mana satu alamat IP sumber mencoba terhubung ke banyak alamat IP tujuan dan/atau port yang berbeda.
- Serangan password spraying di mana satu sumber gagal mengautentikasi dengan banyak pengguna yang berbeda.
- Mendeteksi tool seperti BloodHound yang mengenumerasi banyak grup AD dengan hak istimewa tinggi dalam jangka waktu singkat.

### Contoh Value Count Rule

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

Semua event yang didefinisikan oleh rules yang dirujuk oleh field rule harus terjadi dalam jangka waktu yang didefinisikan oleh timespan.
Nilai dari field yang didefinisikan dalam `group-by` semuanya harus memiliki nilai yang sama (mis: host yang sama, pengguna yang sama, dll...).

Sebuah contoh adalah panggilan API pengintaian yang didefinisikan dalam tiga Sigma rules yang dipanggil dalam urutan sembarang dalam 5 menit dari alamat IP sumber yang sama.

### Contoh Temporal Proximity Rule

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

Tipe korelasi `temporal_ordered` berperilaku seperti `temporal` dan selain itu mensyaratkan agar event muncul dalam urutan yang diberikan dalam atribut `rules`.

Sebuah contoh adalah banyak login yang gagal diikuti oleh login yang berhasil.

### Contoh Temporal Ordered Rule

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
