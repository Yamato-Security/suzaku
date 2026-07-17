# MITRE ATT&CK Etiquetas

Las reglas de Sigma pueden incluir un campo [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) que clasifica una detección según el framework [MITRE ATT&CK®](https://attack.mitre.org/) (tácticas, técnicas y grupos), así como otras taxonomías. Dado que `tags` es una lista, los comandos `aws-ct-timeline` y `azure-timeline` la representan en una única columna **`Tags`**, uniendo las entradas con ` ¦ ` (el mismo separador que utiliza Hayabusa) y abreviando cada entrada para que la columna se mantenga compacta.

## Ejemplo

Una regla etiquetada de esta forma:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

se representa en la columna `Tags` de la siguiente forma:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

En la salida JSON/JSONL, el valor se mantiene como la misma cadena plana (**no** se expande en un array), por lo que la columna es idéntica en CSV, JSON y en la terminal.

## Cómo se abrevia cada etiqueta

| Tipo de etiqueta | Entrada de ejemplo | Salida | Regla |
| --- | --- | --- | --- |
| Táctica | `attack.credential-access` | `CredAccess` | Se busca en `config/mitre_tactics.txt` (véase más abajo) |
| Técnica | `attack.t1562.001` | `T1562.001` | El prefijo `attack.t` se convierte en una `T` mayúscula; el número de técnica/subtécnica se mantiene tal cual |
| Grupo | `attack.g0035` | `G0035` | El prefijo `attack.g` se convierte en una `G` mayúscula; el número de grupo se mantiene tal cual |
| Cualquier otra cosa | `cve.2021.1234` | `cve.2021.1234` | Se deja sin cambios |

Las etiquetas se comparan sin distinguir entre mayúsculas y minúsculas, y las grafías con guion y con guion bajo se tratan igual, por lo que tanto `attack.credential-access` como `attack.credential_access` se convierten en `CredAccess`.

## La tabla de abreviaturas de tácticas

Las abreviaturas de tácticas **no** están codificadas de forma fija en el código: se leen en tiempo de ejecución desde `config/mitre_tactics.txt`, la misma tabla que utiliza [Hayabusa](https://github.com/Yamato-Security/hayabusa). Cada línea es un simple par `<full tag>,<abbreviation>`, por lo que puede editar o ampliar las abreviaturas sin recompilar Suzaku:

| Etiqueta completa | Abreviatura |
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

> Nota: Si falta `config/mitre_tactics.txt`, las etiquetas de tácticas se pasan sin cambios; las abreviaturas de técnicas y grupos siguen funcionando.

## Una nota sobre «Stealth» frente a «Defense Evasion»

A partir de [MITRE ATT&CK v19 (abril de 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/), la táctica **Defense Evasion** (`TA0005`) pasó a llamarse **Stealth**, y de ella se separó una táctica independiente **Impair Defenses** (`TA0112`). Suzaku sigue la nueva denominación:

- `attack.stealth` y el antiguo `attack.defense-evasion` se abrevian ambos como **`Stealth`**, por lo que las reglas más antiguas que todavía usan la etiqueta `attack.defense-evasion` se muestran con el nombre de táctica actual.
- `attack.defense-impairment` se abrevia como **`DefImpair`**.

Si prefiere la etiqueta antigua, cambie la línea `attack.defense-evasion` en `config/mitre_tactics.txt` (por ejemplo, de vuelta a `Evas`).
