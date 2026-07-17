# MITRE ATT&CK टैग

Sigma नियम एक [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) फ़ील्ड रख सकते हैं, जो किसी डिटेक्शन को [MITRE ATT&CK®](https://attack.mitre.org/) फ़्रेमवर्क (टैक्टिक्स, तकनीकें और समूह) तथा अन्य वर्गीकरण-प्रणालियों के अनुसार वर्गीकृत करता है। चूँकि `tags` एक सूची है, इसलिए `aws-ct-timeline` और `azure-timeline` कमांड इसे एक ही **`Tags`** कॉलम में प्रस्तुत करते हैं। इसमें प्रविष्टियों को ` ¦ ` (वही विभाजक जो Hayabusa उपयोग करता है) से जोड़ा जाता है और प्रत्येक प्रविष्टि को संक्षिप्त किया जाता है ताकि कॉलम संहत बना रहे।

## उदाहरण

इस प्रकार टैग किया गया एक नियम:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

`Tags` कॉलम में इस प्रकार प्रस्तुत होता है:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

JSON/JSONL आउटपुट में यह मान उसी सपाट स्ट्रिंग के रूप में रखा जाता है (इसे किसी ऐरे में **विस्तारित नहीं** किया जाता), इसलिए CSV, JSON और टर्मिनल में कॉलम एक समान रहता है।

## प्रत्येक टैग को कैसे संक्षिप्त किया जाता है

| टैग का प्रकार | इनपुट उदाहरण | आउटपुट | नियम |
| --- | --- | --- | --- |
| टैक्टिक | `attack.credential-access` | `CredAccess` | `config/mitre_tactics.txt` में देखा जाता है (नीचे देखें) |
| तकनीक | `attack.t1562.001` | `T1562.001` | `attack.t` उपसर्ग एक बड़े अक्षर `T` में बदल जाता है; तकनीक/उप-तकनीक संख्या ज्यों-की-त्यों रखी जाती है |
| समूह | `attack.g0035` | `G0035` | `attack.g` उपसर्ग एक बड़े अक्षर `G` में बदल जाता है; समूह संख्या ज्यों-की-त्यों रखी जाती है |
| अन्य कुछ भी | `cve.2021.1234` | `cve.2021.1234` | अपरिवर्तित रखा जाता है |

टैग बड़े-छोटे अक्षरों के भेद के बिना मिलाए जाते हैं, और हाइफ़न व अंडरस्कोर वाली वर्तनी को एक समान माना जाता है, इसलिए `attack.credential-access` और `attack.credential_access` दोनों `CredAccess` बन जाते हैं।

## टैक्टिक संक्षेपण तालिका

टैक्टिक संक्षेपण कोड में हार्ड-कोडेड **नहीं** हैं — इन्हें रनटाइम पर `config/mitre_tactics.txt` से पढ़ा जाता है, वही तालिका जिसका उपयोग [Hayabusa](https://github.com/Yamato-Security/hayabusa) करता है। प्रत्येक पंक्ति एक साधारण `<full tag>,<abbreviation>` जोड़ा है, इसलिए आप Suzaku को दोबारा बिल्ड किए बिना संक्षेपण को संपादित या विस्तारित कर सकते हैं:

| पूर्ण टैग | संक्षेपण |
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

> नोट: यदि `config/mitre_tactics.txt` अनुपस्थित है, तो टैक्टिक टैग अपरिवर्तित रूप से आगे भेज दिए जाते हैं; तकनीक और समूह के संक्षेपण फिर भी काम करते हैं।

## "Stealth" बनाम "Defense Evasion" पर एक टिप्पणी

[MITRE ATT&CK v19 (April 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/) से, **Defense Evasion** टैक्टिक (`TA0005`) का नाम बदलकर **Stealth** कर दिया गया, और उसमें से एक अलग **Impair Defenses** टैक्टिक (`TA0112`) को विभाजित कर दिया गया। Suzaku नए नामकरण का अनुसरण करता है:

- `attack.stealth` और पुराना `attack.defense-evasion` दोनों **`Stealth`** में संक्षिप्त होते हैं, इसलिए वे पुराने नियम जो अभी भी `attack.defense-evasion` टैग का उपयोग करते हैं, वर्तमान टैक्टिक नाम के साथ प्रदर्शित होते हैं।
- `attack.defense-impairment` **`DefImpair`** में संक्षिप्त होता है।

यदि आप पुराना लेबल पसंद करते हैं, तो `config/mitre_tactics.txt` में `attack.defense-evasion` वाली पंक्ति को बदलें (उदाहरण के लिए वापस `Evas` में)।
