# MITRE ATT&CK Tags

Sigma rules can carry a [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) field that classifies a detection against the [MITRE ATT&CK®](https://attack.mitre.org/) framework (tactics, techniques and groups) as well as other taxonomies. Because `tags` is a list, the `aws-ct-timeline` and `azure-timeline` commands render it in a single **`Tags`** column, joining the entries with ` ¦ ` (the same separator Hayabusa uses) and abbreviating each entry so the column stays compact.

## Example

A rule tagged like this:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

is rendered in the `Tags` column as:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

In JSON/JSONL output the value is kept as the same flat string (it is **not** expanded into an array), so the column is identical across CSV, JSON and the terminal.

## How each tag is abbreviated

| Tag type | Example input | Output | Rule |
| --- | --- | --- | --- |
| Tactic | `attack.credential-access` | `CredAccess` | Looked up in `config/mitre_tactics.txt` (see below) |
| Technique | `attack.t1562.001` | `T1562.001` | The `attack.t` prefix becomes an upper-case `T`; the technique/sub-technique number is kept as-is |
| Group | `attack.g0035` | `G0035` | The `attack.g` prefix becomes an upper-case `G`; the group number is kept as-is |
| Anything else | `cve.2021.1234` | `cve.2021.1234` | Left unchanged |

Tags are matched case-insensitively, and the hyphen and underscore spellings are treated the same, so `attack.credential-access` and `attack.credential_access` both become `CredAccess`.

## The tactic abbreviation table

Tactic abbreviations are **not** hard-coded — they are read at runtime from `config/mitre_tactics.txt`, the same table [Hayabusa](https://github.com/Yamato-Security/hayabusa) uses. Each line is a simple `<full tag>,<abbreviation>` pair, so you can edit or extend the abbreviations without rebuilding Suzaku:

| Full tag | Abbreviation |
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

> Note: If `config/mitre_tactics.txt` is missing, tactic tags are passed through unchanged; technique and group abbreviations still work.

## A note on "Stealth" vs "Defense Evasion"

As of [MITRE ATT&CK v19 (April 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/), the **Defense Evasion** tactic (`TA0005`) was renamed to **Stealth**, and a separate **Impair Defenses** tactic (`TA0112`) was split out from it. Suzaku follows the new naming:

- `attack.stealth` and the legacy `attack.defense-evasion` both abbreviate to **`Stealth`**, so older rules that still use the `attack.defense-evasion` tag are displayed with the current tactic name.
- `attack.defense-impairment` abbreviates to **`DefImpair`**.

If you prefer the old label, change the `attack.defense-evasion` line in `config/mitre_tactics.txt` (for example back to `Evas`).
