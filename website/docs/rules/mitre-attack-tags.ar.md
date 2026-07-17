# وسوم MITRE ATT&CK

يمكن أن تحتوي قواعد Sigma على حقل [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) يصنّف الاكتشاف وفقًا لإطار عمل [MITRE ATT&CK®](https://attack.mitre.org/) (التكتيكات والتقنيات والمجموعات) وغيره من التصنيفات. ولأن `tags` عبارة عن قائمة، فإن الأمرين `aws-ct-timeline` و`azure-timeline` يعرضانها في عمود **`Tags`** واحد، حيث تُجمَع كل مدخلة باستخدام ` ¦ ` (نفس الفاصل الذي يستخدمه Hayabusa) ويُختصر كل منها لكي يبقى العمود موجزًا.

## مثال

قاعدة موسومة على النحو التالي:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

تُعرَض في عمود `Tags` على النحو التالي:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

في مخرجات JSON/JSONL، يُحتفَظ بالقيمة كنفس السلسلة النصية المسطّحة (فهي **لا** تُوسَّع إلى مصفوفة)، بحيث يكون العمود متطابقًا في CSV وJSON والطرفية.

## كيف يُختصر كل وسم

| نوع الوسم | مثال للإدخال | المخرجات | القاعدة |
| --- | --- | --- | --- |
| التكتيك | `attack.credential-access` | `CredAccess` | يُبحَث عنه في `config/mitre_tactics.txt` (انظر أدناه) |
| التقنية | `attack.t1562.001` | `T1562.001` | تتحوّل البادئة `attack.t` إلى حرف `T` كبير؛ ويُحتفَظ برقم التقنية/التقنية الفرعية كما هو |
| المجموعة | `attack.g0035` | `G0035` | تتحوّل البادئة `attack.g` إلى حرف `G` كبير؛ ويُحتفَظ برقم المجموعة كما هو |
| أي شيء آخر | `cve.2021.1234` | `cve.2021.1234` | يبقى دون تغيير |

تُطابَق الوسوم دون حساسية لحالة الأحرف، وتُعامَل صيغتا الكتابة بالشرطة والشرطة السفلية بالطريقة نفسها، بحيث يتحوّل كل من `attack.credential-access` و`attack.credential_access` إلى `CredAccess`.

## جدول اختصارات التكتيكات

اختصارات التكتيكات **ليست** مكتوبة بشكل ثابت في الشيفرة — بل تُقرأ أثناء التشغيل من `config/mitre_tactics.txt`، وهو الجدول نفسه الذي يستخدمه [Hayabusa](https://github.com/Yamato-Security/hayabusa). كل سطر هو زوج بسيط بالصيغة `<full tag>,<abbreviation>`، بحيث يمكنك تعديل الاختصارات أو توسيعها دون إعادة بناء Suzaku:

| الوسم الكامل | الاختصار |
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

> ملاحظة: إذا كان `config/mitre_tactics.txt` مفقودًا، تُمرَّر وسوم التكتيكات دون تغيير؛ وتظل اختصارات التقنيات والمجموعات تعمل.

## ملاحظة حول «Stealth» مقابل «Defense Evasion»

اعتبارًا من [MITRE ATT&CK v19 (أبريل 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/)، أُعيدت تسمية تكتيك **Defense Evasion** (`TA0005`) إلى **Stealth**، وفُصِل عنه تكتيك مستقل باسم **Impair Defenses** (`TA0112`). يتبع Suzaku التسمية الجديدة:

- يُختصر كل من `attack.stealth` و`attack.defense-evasion` القديم إلى **`Stealth`**، بحيث تُعرَض القواعد الأقدم التي لا تزال تستخدم الوسم `attack.defense-evasion` باسم التكتيك الحالي.
- يُختصر `attack.defense-impairment` إلى **`DefImpair`**.

إذا كنت تفضّل التسمية القديمة، فغيّر سطر `attack.defense-evasion` في `config/mitre_tactics.txt` (على سبيل المثال، بإعادته إلى `Evas`).
