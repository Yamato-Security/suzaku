# MITRE ATT&CK Tags

Sigma-Regeln können ein [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags)-Feld enthalten, das eine Erkennung anhand des [MITRE ATT&CK®](https://attack.mitre.org/)-Frameworks (Taktiken, Techniken und Gruppen) sowie anderer Taxonomien klassifiziert. Da `tags` eine Liste ist, geben die Befehle `aws-ct-timeline` und `azure-timeline` sie in einer einzigen **`Tags`**-Spalte aus. Dabei werden die Einträge mit ` ¦ ` (demselben Trennzeichen, das auch Hayabusa verwendet) verbunden und jeweils abgekürzt, damit die Spalte kompakt bleibt.

## Beispiel

Eine Regel, die wie folgt getaggt ist:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

wird in der `Tags`-Spalte wie folgt ausgegeben:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

In der JSON/JSONL-Ausgabe wird der Wert als dieselbe flache Zeichenkette beibehalten (er wird **nicht** in ein Array expandiert), sodass die Spalte in CSV, JSON und im Terminal identisch ist.

## Wie jeder Tag abgekürzt wird

| Tag-Typ | Beispieleingabe | Ausgabe | Regel |
| --- | --- | --- | --- |
| Taktik | `attack.credential-access` | `CredAccess` | Wird in `config/mitre_tactics.txt` nachgeschlagen (siehe unten) |
| Technik | `attack.t1562.001` | `T1562.001` | Das Präfix `attack.t` wird zu einem großgeschriebenen `T`; die Technik-/Subtechnik-Nummer bleibt unverändert |
| Gruppe | `attack.g0035` | `G0035` | Das Präfix `attack.g` wird zu einem großgeschriebenen `G`; die Gruppennummer bleibt unverändert |
| Alles andere | `cve.2021.1234` | `cve.2021.1234` | Bleibt unverändert |

Tags werden ohne Berücksichtigung der Groß-/Kleinschreibung abgeglichen, und die Schreibweisen mit Bindestrich und Unterstrich werden gleich behandelt, sodass sowohl `attack.credential-access` als auch `attack.credential_access` zu `CredAccess` werden.

## Die Tabelle der Taktik-Abkürzungen

Taktik-Abkürzungen sind **nicht** fest im Code hinterlegt – sie werden zur Laufzeit aus `config/mitre_tactics.txt` gelesen, derselben Tabelle, die auch [Hayabusa](https://github.com/Yamato-Security/hayabusa) verwendet. Jede Zeile ist ein einfaches Paar `<full tag>,<abbreviation>`, sodass Sie die Abkürzungen bearbeiten oder erweitern können, ohne Suzaku neu zu bauen:

| Vollständiger Tag | Abkürzung |
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

> Hinweis: Wenn `config/mitre_tactics.txt` fehlt, werden Taktik-Tags unverändert durchgereicht; Technik- und Gruppen-Abkürzungen funktionieren weiterhin.

## Ein Hinweis zu „Stealth“ vs. „Defense Evasion“

Seit [MITRE ATT&CK v19 (April 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/) wurde die Taktik **Defense Evasion** (`TA0005`) in **Stealth** umbenannt, und eine separate Taktik **Impair Defenses** (`TA0112`) wurde daraus ausgegliedert. Suzaku folgt der neuen Benennung:

- `attack.stealth` und das veraltete `attack.defense-evasion` werden beide zu **`Stealth`** abgekürzt, sodass ältere Regeln, die noch den Tag `attack.defense-evasion` verwenden, mit dem aktuellen Taktiknamen angezeigt werden.
- `attack.defense-impairment` wird zu **`DefImpair`** abgekürzt.

Wenn Sie die alte Bezeichnung bevorzugen, ändern Sie die Zeile `attack.defense-evasion` in `config/mitre_tactics.txt` (zum Beispiel zurück zu `Evas`).
