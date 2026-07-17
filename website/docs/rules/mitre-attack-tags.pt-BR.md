# MITRE ATT&CK Etiquetas

As regras do Sigma podem incluir um campo [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) que classifica uma detecção segundo o framework [MITRE ATT&CK®](https://attack.mitre.org/) (táticas, técnicas e grupos), bem como outras taxonomias. Como `tags` é uma lista, os comandos `aws-ct-timeline` e `azure-timeline` a renderizam em uma única coluna **`Tags`**, unindo as entradas com ` ¦ ` (o mesmo separador que o Hayabusa usa) e abreviando cada entrada para que a coluna permaneça compacta.

## Exemplo

Uma regra marcada assim:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

é renderizada na coluna `Tags` da seguinte forma:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

Na saída JSON/JSONL, o valor é mantido como a mesma string plana (**não** é expandido em um array), portanto a coluna é idêntica no CSV, no JSON e no terminal.

## Como cada etiqueta é abreviada

| Tipo de etiqueta | Entrada de exemplo | Saída | Regra |
| --- | --- | --- | --- |
| Tática | `attack.credential-access` | `CredAccess` | Consultado em `config/mitre_tactics.txt` (veja abaixo) |
| Técnica | `attack.t1562.001` | `T1562.001` | O prefixo `attack.t` vira um `T` maiúsculo; o número da técnica/subtécnica é mantido como está |
| Grupo | `attack.g0035` | `G0035` | O prefixo `attack.g` vira um `G` maiúsculo; o número do grupo é mantido como está |
| Qualquer outra coisa | `cve.2021.1234` | `cve.2021.1234` | Mantido sem alterações |

As etiquetas são comparadas sem diferenciar maiúsculas de minúsculas, e as grafias com hífen e com sublinhado são tratadas da mesma forma, portanto tanto `attack.credential-access` quanto `attack.credential_access` se tornam `CredAccess`.

## A tabela de abreviações de táticas

As abreviações de táticas **não** são fixas no código — elas são lidas em tempo de execução a partir de `config/mitre_tactics.txt`, a mesma tabela que o [Hayabusa](https://github.com/Yamato-Security/hayabusa) usa. Cada linha é um par simples `<etiqueta completa>,<abreviação>`, então você pode editar ou estender as abreviações sem recompilar o Suzaku:

| Etiqueta completa | Abreviação |
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

> Observação: Se `config/mitre_tactics.txt` estiver ausente, as etiquetas de táticas são repassadas sem alterações; as abreviações de técnicas e grupos continuam funcionando.

## Uma nota sobre "Stealth" vs "Defense Evasion"

A partir do [MITRE ATT&CK v19 (abril de 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/), a tática **Defense Evasion** (`TA0005`) foi renomeada para **Stealth**, e uma tática **Impair Defenses** (`TA0112`) separada foi desmembrada dela. O Suzaku segue a nova nomenclatura:

- `attack.stealth` e o legado `attack.defense-evasion` são ambos abreviados para **`Stealth`**, portanto regras mais antigas que ainda usam a etiqueta `attack.defense-evasion` são exibidas com o nome atual da tática.
- `attack.defense-impairment` é abreviado para **`DefImpair`**.

Se você preferir o rótulo antigo, altere a linha `attack.defense-evasion` em `config/mitre_tactics.txt` (por exemplo, de volta para `Evas`).
