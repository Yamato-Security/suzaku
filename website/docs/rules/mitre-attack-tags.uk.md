# MITRE ATT&CK теги

Правила Sigma можуть містити поле [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags), яке класифікує виявлення за фреймворком [MITRE ATT&CK®](https://attack.mitre.org/) (тактики, техніки та групи), а також за іншими таксономіями. Оскільки `tags` є списком, команди `aws-ct-timeline` та `azure-timeline` виводять його в одному стовпці **`Tags`**, з'єднуючи записи за допомогою ` ¦ ` (той самий роздільник, який використовує Hayabusa) і скорочуючи кожен запис, щоб стовпець залишався компактним.

## Приклад

Правило з такими тегами:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

виводиться у стовпці `Tags` так:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

У виводі JSON/JSONL значення зберігається як той самий плоский рядок (воно **не** розгортається в масив), тому стовпець є однаковим у CSV, JSON та терміналі.

## Як скорочується кожен тег

| Тип тегу | Приклад вводу | Вивід | Правило |
| --- | --- | --- | --- |
| Тактика | `attack.credential-access` | `CredAccess` | Шукається у `config/mitre_tactics.txt` (див. нижче) |
| Техніка | `attack.t1562.001` | `T1562.001` | Префікс `attack.t` стає великою літерою `T`; номер техніки/субтехніки зберігається без змін |
| Група | `attack.g0035` | `G0035` | Префікс `attack.g` стає великою літерою `G`; номер групи зберігається без змін |
| Будь-що інше | `cve.2021.1234` | `cve.2021.1234` | Залишається без змін |

Теги зіставляються без урахування регістру, а написання з дефісом і підкресленням трактуються однаково, тому `attack.credential-access` і `attack.credential_access` обидва стають `CredAccess`.

## Таблиця скорочень тактик

Скорочення тактик **не** зашиті у код — вони зчитуються під час виконання з `config/mitre_tactics.txt`, тієї самої таблиці, яку використовує [Hayabusa](https://github.com/Yamato-Security/hayabusa). Кожен рядок є простою парою `<full tag>,<abbreviation>`, тож ви можете редагувати або розширювати скорочення без повторного складання Suzaku:

| Повний тег | Скорочення |
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

> Примітка: Якщо `config/mitre_tactics.txt` відсутній, теги тактик передаються без змін; скорочення технік і груп продовжують працювати.

## Примітка щодо "Stealth" та "Defense Evasion"

Починаючи з [MITRE ATT&CK v19 (квітень 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/), тактику **Defense Evasion** (`TA0005`) було перейменовано на **Stealth**, а окрему тактику **Impair Defenses** (`TA0112`) було виділено з неї. Suzaku дотримується нового найменування:

- `attack.stealth` і застарілий `attack.defense-evasion` обидва скорочуються до **`Stealth`**, тому старіші правила, які все ще використовують тег `attack.defense-evasion`, відображаються з поточною назвою тактики.
- `attack.defense-impairment` скорочується до **`DefImpair`**.

Якщо ви віддаєте перевагу старій назві, змініть рядок `attack.defense-evasion` у `config/mitre_tactics.txt` (наприклад, назад на `Evas`).
