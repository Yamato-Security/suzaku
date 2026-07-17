# MITRE ATT&CK Etiketleri

Sigma kuralları, bir tespiti [MITRE ATT&CK®](https://attack.mitre.org/) çerçevesine (taktikler, teknikler ve gruplar) ve diğer sınıflandırma sistemlerine göre sınıflandıran bir [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) alanı içerebilir. `tags` bir liste olduğundan, `aws-ct-timeline` ve `azure-timeline` komutları bunu tek bir **`Tags`** sütununda gösterir; girdileri ` ¦ ` ile (Hayabusa'nın kullandığı ayırıcının aynısı) birleştirir ve sütunun kompakt kalması için her girdiyi kısaltır.

## Örnek

Şu şekilde etiketlenmiş bir kural:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

`Tags` sütununda şu şekilde gösterilir:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

JSON/JSONL çıktısında değer aynı düz metin olarak korunur (bir diziye **açılmaz**), böylece sütun CSV, JSON ve terminalde aynıdır.

## Her etiketin nasıl kısaltıldığı

| Etiket türü | Örnek girdi | Çıktı | Kural |
| --- | --- | --- | --- |
| Taktik | `attack.credential-access` | `CredAccess` | `config/mitre_tactics.txt` içinde aranır (aşağıya bakın) |
| Teknik | `attack.t1562.001` | `T1562.001` | `attack.t` ön eki büyük harf `T` olur; teknik/alt teknik numarası olduğu gibi korunur |
| Grup | `attack.g0035` | `G0035` | `attack.g` ön eki büyük harf `G` olur; grup numarası olduğu gibi korunur |
| Diğer her şey | `cve.2021.1234` | `cve.2021.1234` | Değiştirilmeden bırakılır |

Etiketler büyük/küçük harfe duyarlı olmadan eşleştirilir ve tire ile alt çizgi yazımları aynı şekilde ele alınır; bu nedenle `attack.credential-access` ve `attack.credential_access` her ikisi de `CredAccess` olur.

## Taktik kısaltma tablosu

Taktik kısaltmaları koda gömülü **değildir** — çalışma zamanında `config/mitre_tactics.txt` dosyasından, [Hayabusa](https://github.com/Yamato-Security/hayabusa)'nın kullandığı tablonun aynısından okunur. Her satır basit bir `<full tag>,<abbreviation>` çiftidir; böylece Suzaku'yu yeniden derlemeden kısaltmaları düzenleyebilir veya genişletebilirsiniz:

| Tam etiket | Kısaltma |
| --- | --- |
| `attack.reconnaissance` | Recon |
| `attack.resource-development` | ResDev |
| `attack.initial-access` | InitAccess |
| `attack.execution` | Exec |
| `attack.persistence` | Persis |
| `attack.privilege-escalation` | PrivEsc |
| `attack.stealth` | Stealth |
| `attack.defense-evasion` | Stealth |
| `attack.defense-impairment` | DefImpair |
| `attack.credential-access` | CredAccess |
| `attack.discovery` | Disc |
| `attack.lateral-movement` | LatMov |
| `attack.collection` | Collect |
| `attack.command-and-control` | C2 |
| `attack.exfiltration` | Exfil |
| `attack.impact` | Impact |

> Not: `config/mitre_tactics.txt` eksikse, taktik etiketleri değiştirilmeden geçirilir; teknik ve grup kısaltmaları çalışmaya devam eder.

## "Stealth" ve "Defense Evasion" hakkında bir not

[MITRE ATT&CK v19 (Nisan 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/) itibarıyla, **Defense Evasion** taktiği (`TA0005`) **Stealth** olarak yeniden adlandırıldı ve ondan ayrı bir **Impair Defenses** taktiği (`TA0112`) ayrıldı. Suzaku yeni adlandırmayı takip eder:

- `attack.stealth` ve eski `attack.defense-evasion` her ikisi de **`Stealth`** olarak kısaltılır; böylece hâlâ `attack.defense-evasion` etiketini kullanan eski kurallar güncel taktik adıyla görüntülenir.
- `attack.defense-impairment` **`DefImpair`** olarak kısaltılır.

Eski etiketi tercih ediyorsanız, `config/mitre_tactics.txt` dosyasındaki `attack.defense-evasion` satırını değiştirin (örneğin `Evas` olarak geri alın).
