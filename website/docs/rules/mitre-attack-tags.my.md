# MITRE ATT&CK တဂ်များ

Sigma rule များတွင် detection တစ်ခုကို [MITRE ATT&CK®](https://attack.mitre.org/) framework (tactics၊ techniques နှင့် groups) အပြင် အခြား taxonomy များနှင့်ပါ ဆက်စပ်သတ်မှတ်ပေးသည့် [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) field ပါဝင်နိုင်ပါသည်။ `tags` သည် list ဖြစ်သောကြောင့် `aws-ct-timeline` နှင့် `azure-timeline` command များသည် ၎င်းကို **`Tags`** column တစ်ခုတည်းအဖြစ် ဖော်ပြပါသည်။ entry များကို ` ¦ ` (Hayabusa အသုံးပြုသည့် separator အတူတူ) ဖြင့် ချိတ်ဆက်ပြီး column ကို ကျစ်လစ်စေရန် entry တစ်ခုစီကို အတိုကောက်ပြုလုပ်ပါသည်။

## ဥပမာ

အောက်ပါအတိုင်း တဂ်သတ်မှတ်ထားသော rule တစ်ခု:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

သည် `Tags` column တွင် အောက်ပါအတိုင်း ဖော်ပြပါသည်:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

JSON/JSONL output တွင် တန်ဖိုးကို တူညီသော flat string အဖြစ် ဆက်လက်ထားရှိပါသည် (array အဖြစ် **မ**ချဲ့ထွင်ပါ)။ ထို့ကြောင့် column သည် CSV၊ JSON နှင့် terminal တစ်လျှောက် တူညီပါသည်။

## တဂ်တစ်ခုစီကို မည်သို့ အတိုကောက်ပြုလုပ်သည်

| တဂ်အမျိုးအစား | input ဥပမာ | အထွက် | စည်းမျဉ်း |
| --- | --- | --- | --- |
| Tactic | `attack.credential-access` | `CredAccess` | `config/mitre_tactics.txt` တွင် ရှာဖွေသည် (အောက်တွင်ကြည့်ပါ) |
| Technique | `attack.t1562.001` | `T1562.001` | `attack.t` prefix ကို စာလုံးကြီး `T` အဖြစ် ပြောင်းသည်။ technique/sub-technique နံပါတ်ကို ရှိသည်အတိုင်း ထားသည် |
| Group | `attack.g0035` | `G0035` | `attack.g` prefix ကို စာလုံးကြီး `G` အဖြစ် ပြောင်းသည်။ group နံပါတ်ကို ရှိသည်အတိုင်း ထားသည် |
| အခြားအရာများ | `cve.2021.1234` | `cve.2021.1234` | မပြောင်းလဲဘဲ ထားသည် |

တဂ်များကို စာလုံးအကြီးအသေး ခွဲခြားခြင်းမရှိဘဲ တိုက်ဆိုင်စစ်ဆေးပြီး၊ hyphen နှင့် underscore စာလုံးပေါင်းများကို တူညီအဖြစ် သဘောထားသောကြောင့် `attack.credential-access` နှင့် `attack.credential_access` နှစ်ခုစလုံးသည် `CredAccess` ဖြစ်လာပါသည်။

## Tactic အတိုကောက် ဇယား

Tactic အတိုကောက်များကို hard-code လုပ်ထားခြင်း **မဟုတ်ပါ** — ၎င်းတို့ကို [Hayabusa](https://github.com/Yamato-Security/hayabusa) အသုံးပြုသည့် ဇယားတူတူဖြစ်သော `config/mitre_tactics.txt` မှ runtime တွင် ဖတ်ယူပါသည်။ လိုင်းတစ်ခုစီသည် ရိုးရှင်းသော `<full tag>,<abbreviation>` အတွဲဖြစ်သောကြောင့် Suzaku ကို ပြန်လည်တည်ဆောက်စရာမလိုဘဲ အတိုကောက်များကို တည်းဖြတ်ခြင်း သို့မဟုတ် ဖြည့်စွက်ခြင်း ပြုလုပ်နိုင်ပါသည်:

| တဂ်အပြည့်အစုံ | အတိုကောက် |
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

> မှတ်ချက်: `config/mitre_tactics.txt` မရှိပါက tactic တဂ်များကို မပြောင်းလဲဘဲ ဖြတ်သန်းထုတ်ပေးပါသည်။ technique နှင့် group အတိုကောက်များမူ ဆက်လက်အလုပ်လုပ်ပါသည်။

## "Stealth" နှင့် "Defense Evasion" အကြောင်း မှတ်ချက်

[MITRE ATT&CK v19 (April 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/) မှစ၍ **Defense Evasion** tactic (`TA0005`) ကို **Stealth** အဖြစ် အမည်ပြောင်းလဲခဲ့ပြီး၊ ၎င်းမှ သီးခြား **Impair Defenses** tactic (`TA0112`) ကို ခွဲထုတ်ခဲ့ပါသည်။ Suzaku သည် အမည်သစ်ကို လိုက်နာပါသည်:

- `attack.stealth` နှင့် အဟောင်း `attack.defense-evasion` နှစ်ခုစလုံးကို **`Stealth`** အဖြစ် အတိုကောက်ပြုလုပ်သောကြောင့် `attack.defense-evasion` တဂ်ကို ဆက်လက်အသုံးပြုနေသည့် rule အဟောင်းများသည်လည်း လက်ရှိ tactic အမည်ဖြင့် ပြသပါသည်။
- `attack.defense-impairment` ကို **`DefImpair`** အဖြစ် အတိုကောက်ပြုလုပ်ပါသည်။

အညွှန်းအဟောင်းကို ပိုနှစ်သက်ပါက `config/mitre_tactics.txt` ရှိ `attack.defense-evasion` လိုင်းကို ပြောင်းလဲပါ (ဥပမာ `Evas` သို့ ပြန်ပြောင်းခြင်း)။
