# MITRE ATT&CK 태그

Sigma 규칙에는 탐지 내용을 [MITRE ATT&CK®](https://attack.mitre.org/) 프레임워크(전술, 기법, 그룹)와 그 밖의 분류 체계에 매핑하는 [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) 필드를 지정할 수 있습니다. `tags`는 리스트이므로 `aws-ct-timeline` 및 `azure-timeline` 명령은 이를 하나의 **`Tags`** 열로 묶어 출력합니다. 각 항목은 (Hayabusa가 사용하는 것과 동일한 구분 문자인) ` ¦ ` 로 연결되며, 열이 간결하게 유지되도록 각 항목을 축약합니다.

## 예시

다음과 같이 태그가 지정된 규칙:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

는 `Tags` 열에서 다음과 같이 출력됩니다:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

JSON/JSONL 출력에서도 값은 동일한 평면 문자열로 유지되며(배열로 확장되지 **않습니다**), 따라서 CSV, JSON, 터미널에서 열의 내용이 동일합니다.

## 각 태그가 축약되는 방식

| 태그 종류 | 입력 예시 | 출력 | 규칙 |
| --- | --- | --- | --- |
| 전술 | `attack.credential-access` | `CredAccess` | `config/mitre_tactics.txt`에서 조회됩니다(아래 참조) |
| 기법 | `attack.t1562.001` | `T1562.001` | `attack.t` 접두사가 대문자 `T`로 바뀌며, 기법/하위 기법 번호는 그대로 유지됩니다 |
| 그룹 | `attack.g0035` | `G0035` | `attack.g` 접두사가 대문자 `G`로 바뀌며, 그룹 번호는 그대로 유지됩니다 |
| 그 밖의 모든 것 | `cve.2021.1234` | `cve.2021.1234` | 변경되지 않습니다 |

태그는 대소문자를 구분하지 않고 매칭되며, 하이픈과 밑줄 표기는 동일하게 취급되므로 `attack.credential-access`와 `attack.credential_access`는 모두 `CredAccess`가 됩니다.

## 전술 축약 테이블

전술 축약은 코드에 하드코딩되어 있지 **않으며**, [Hayabusa](https://github.com/Yamato-Security/hayabusa)가 사용하는 것과 동일한 테이블인 `config/mitre_tactics.txt`에서 런타임에 읽어들입니다. 각 행은 `<full tag>,<abbreviation>`이라는 단순한 쌍이므로, Suzaku를 다시 빌드하지 않고도 축약을 편집하거나 확장할 수 있습니다:

| 전체 태그 | 축약 |
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

> 참고: `config/mitre_tactics.txt`가 없는 경우 전술 태그는 변경 없이 그대로 전달됩니다(기법 및 그룹 축약은 계속 작동합니다).

## "Stealth"와 "Defense Evasion"에 대한 참고

[MITRE ATT&CK v19(2026년 4월)](https://attack.mitre.org/resources/updates/updates-april-2026/)부터 **Defense Evasion** 전술(`TA0005`)은 **Stealth**로 이름이 변경되었고, 여기에서 별도의 **Impair Defenses** 전술(`TA0112`)이 분리되었습니다. Suzaku는 새로운 명칭을 따릅니다:

- `attack.stealth`와 기존의 `attack.defense-evasion`은 모두 **`Stealth`**로 축약되므로, 여전히 `attack.defense-evasion` 태그를 사용하는 오래된 규칙도 현재의 전술 이름으로 표시됩니다.
- `attack.defense-impairment`는 **`DefImpair`**로 축약됩니다.

이전 라벨을 선호한다면 `config/mitre_tactics.txt`의 `attack.defense-evasion` 행을 변경하십시오(예: `Evas`로 되돌리기).
