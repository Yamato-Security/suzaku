# MITRE ATT&CK Étiquettes

Les règles Sigma peuvent comporter un champ [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) qui classe une détection selon le framework [MITRE ATT&CK®](https://attack.mitre.org/) (tactiques, techniques et groupes) ainsi que d'autres taxonomies. Comme `tags` est une liste, les commandes `aws-ct-timeline` et `azure-timeline` l'affichent dans une seule colonne **`Tags`**, en joignant les entrées avec ` ¦ ` (le même séparateur que celui utilisé par Hayabusa) et en abrégeant chaque entrée afin que la colonne reste compacte.

## Exemple

Une règle taguée comme ceci :

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

est affichée dans la colonne `Tags` comme suit :

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

Dans la sortie JSON/JSONL, la valeur est conservée sous la forme de la même chaîne plate (elle n'est **pas** développée en tableau), de sorte que la colonne est identique en CSV, en JSON et dans le terminal.

## Comment chaque étiquette est abrégée

| Type d'étiquette | Exemple d'entrée | Sortie | Règle |
| --- | --- | --- | --- |
| Tactique | `attack.credential-access` | `CredAccess` | Recherchée dans `config/mitre_tactics.txt` (voir ci-dessous) |
| Technique | `attack.t1562.001` | `T1562.001` | Le préfixe `attack.t` devient un `T` majuscule ; le numéro de technique/sous-technique est conservé tel quel |
| Groupe | `attack.g0035` | `G0035` | Le préfixe `attack.g` devient un `G` majuscule ; le numéro de groupe est conservé tel quel |
| Tout le reste | `cve.2021.1234` | `cve.2021.1234` | Inchangé |

Les étiquettes sont mises en correspondance sans tenir compte de la casse, et les orthographes avec trait d'union et avec tiret bas sont traitées de la même façon, de sorte que `attack.credential-access` et `attack.credential_access` deviennent tous deux `CredAccess`.

## La table des abréviations de tactiques

Les abréviations de tactiques ne sont **pas** codées en dur — elles sont lues au moment de l'exécution depuis `config/mitre_tactics.txt`, la même table que celle utilisée par [Hayabusa](https://github.com/Yamato-Security/hayabusa). Chaque ligne est un simple couple `<full tag>,<abbreviation>`, de sorte que vous pouvez modifier ou étendre les abréviations sans recompiler Suzaku :

| Étiquette complète | Abréviation |
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

> Note : si `config/mitre_tactics.txt` est absent, les étiquettes de tactiques sont transmises sans modification ; les abréviations de techniques et de groupes continuent de fonctionner.

## Une remarque sur « Stealth » et « Defense Evasion »

Depuis [MITRE ATT&CK v19 (avril 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/), la tactique **Defense Evasion** (`TA0005`) a été renommée en **Stealth**, et une tactique distincte **Impair Defenses** (`TA0112`) en a été séparée. Suzaku suit la nouvelle nomenclature :

- `attack.stealth` et l'ancien `attack.defense-evasion` sont tous deux abrégés en **`Stealth`**, de sorte que les anciennes règles qui utilisent encore l'étiquette `attack.defense-evasion` sont affichées avec le nom de tactique actuel.
- `attack.defense-impairment` est abrégé en **`DefImpair`**.

Si vous préférez l'ancien libellé, modifiez la ligne `attack.defense-evasion` dans `config/mitre_tactics.txt` (par exemple en revenant à `Evas`).
