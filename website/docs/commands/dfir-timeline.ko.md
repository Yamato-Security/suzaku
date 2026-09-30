# DFIR 타임라인 명령어

## `aws-ct-timeline` 명령어

`rules` 폴더에 있는 Sigma 규칙을 기반으로 AWS CloudTrail DFIR 타임라인을 생성합니다.

## 명령어 사용법
```
Usage: suzaku aws-ct-timeline [OPTIONS] <--directory <DIR>|--file <FILE>>

General Options:
  -r, --rules <DIR/FILE>  Specify a custom rule directory or file (default: ./rules)
  -h, --help              Show the help menu

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -C, --clobber                    Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>    Add GeoIP (ASN, city, country) info to IP addresses
  -m, --min-level <LEVEL>          Minimum level for rules to load (default: informational)
  -o, --output <FILE>              Save the results to a file
  -t, --output-type <OUTPUT_TYPE>  Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL [default: 1]
  -R, --raw-output                 Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>    Number of threads to use (default: same as CPU cores)

Display Settings:
  -K, --no-color               Disable color output
  -N, --no-summary             Do not display results summary
  -T, --no-frequency-timeline  Disable event frequency timeline (terminal needs to support Unicode)
  -q, --quiet                  Quiet mode: do not display the launch banner
```

### `aws-ct-timeline` 명령어 예시

* 화면에 경고 출력: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* 결과를 CSV 파일로 저장: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* 결과를 CSV 및 JSONL 파일로 저장: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### `aws-ct-timeline` 출력 프로파일

Suzaku는 `config/aws_profile.yaml` 파일을 기반으로 정보를 출력합니다:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
ErrorCode: '.errorCode'
ErrorMessage: '.errorMessage'
EventSource: '.eventSource'
AWS-Region: '.awsRegion'
SrcIP: '.sourceIPAddress'
UserAgent: '.userAgent'
UserName: '.userIdentity.userName'
UserType: '.userIdentity.type'
UserAccountID: '.userIdentity.accountId'
UserARN: '.userIdentity.arn'
UserPrincipalID: '.userIdentity.principalId'
UserAccessKeyID: '.userIdentity.accessKeyId'
EventID: '.eventID'
Tags: 'sigma.tags'
RuleID: 'sigma.id'
```

* `.`으로 시작하는 모든 필드 값(예: `.eventTime`)은 CloudTrail 로그에서 가져옵니다.
* `sigma.`으로 시작하는 모든 필드 값(예: `sigma.title`)은 Sigma 규칙에서 가져옵니다.
* 현재는 문자열만 지원하지만 다른 유형의 필드 값도 지원할 계획입니다.

> 참고: 원본 JSON 데이터를 출력하고 필드 정보가 손실되지 않도록 하려면 `aws-ct-timeline` 명령어에 `-R, --raw-output` 옵션을 추가하기만 하면 됩니다.

### DuckDB 출력 스키마 {#duckdb-output-schema}

CSV와 JSON 출력은 위 프로파일을 *표시용으로 렌더링*한 것이지만, DuckDB 출력은 *데이터 인터페이스*이므로 타입이 지정되고 자기 기술적인 형식입니다. 이 차이는 의도된 것이며 `aws-ct-timeline`, `azure-timeline`, `gws-timeline`, `aws-ct-search`에 공통으로 적용됩니다:

| | CSV / JSON | DuckDB |
|---|---|---|
| 값이 없는 경우 | `-` (또는 빈 값) | `NULL` |
| `Timestamp` | 렌더링된 텍스트 | `TIMESTAMP` |
| `Level` | 텍스트 | `suzaku_level` (심각도 순으로 정렬된 `ENUM`) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (SQL에서 따옴표 불필요) |
| `Tags` | ` ¦ `로 연결된 하나의 문자열 | `Tactics`, `TechniqueIDs`, `OtherTags` (각각 `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | `-G, --geo-ip` 사용 시에만 추가 | 항상 존재(프로파일에 `SrcIP`가 있는 경우), `-G`를 사용하지 않으면 `NULL` |
| 중복 행 | 유지 | 완전히 동일한 중복은 제거되며 개수는 `suzaku_meta`에 기록 |

모든 파일에는 한 행짜리 `suzaku_meta` 테이블도 포함되어 있어, 파일을 무엇이 생성했는지 추측하지 않고 확인할 수 있습니다:

| 열 | 의미 |
|---|---|
| `schema_version` | 레이아웃 버전. 다른 테이블을 읽기 전에 확인하십시오. |
| `suzaku_version`, `command`, `command_line` | Suzaku 버전, 하위 명령, 실행한 정확한 명령줄. |
| `generated_at` | 파일이 작성된 시각. |
| `timestamp_tz` | `Timestamp` 열의 시간대. `UTC` 또는 `-l, --localtime` 사용 시 로컬 오프셋. |
| `rules_version`, `rules_count` | 룰셋 리비전(rules 폴더가 git 체크아웃인 경우)과 로드된 룰 수. |
| `geoip_enabled` | `-G, --geo-ip` 실행 여부. `SrcCountry`가 모두 `NULL`인 경우(보강이 꺼져 있음)와 보강된 파일의 `NULL` 셀(해당 값이 IP 주소가 아님)을 구별할 수 있습니다. |
| `scanned_files`, `scanned_events` | 실행 시 스캔한 범위. |
| `output_rows`, `duplicate_rows_removed` | 작성된 행 수와 작성 시 제거된 완전 중복 행 수. |

룰 기반 타임라인 명령에서 `timeline`의 한 행은 **이벤트 × 룰 매치 1건**입니다. 여러 룰에 매치된 이벤트는 매치마다 한 행이 생성되므로 `EventID`는 고유하지 *않습니다*. `aws-ct-search`에서는 매치된 이벤트마다 한 행이 생성됩니다.

```sql
-- Critical and high alerts in a time range, with their ATT&CK techniques.
-- `Level` is an ENUM, so cast the literal to compare by severity rather than alphabetically.
SELECT Timestamp, RuleTitle, EventName, SrcIP, TechniqueIDs
FROM timeline
WHERE Level >= 'high'::suzaku_level
  AND Timestamp BETWEEN TIMESTAMP '2024-01-01' AND TIMESTAMP '2024-02-01'
  AND ErrorCode IS NULL          -- the call succeeded
ORDER BY Timestamp;

-- ATT&CK technique coverage, no string parsing required
SELECT technique, count(*) AS hits
FROM (SELECT unnest(TechniqueIDs) AS technique FROM timeline)
GROUP BY 1 ORDER BY hits DESC;
```

Suzaku는 종료 전에 데이터베이스를 체크포인트하므로 `.duckdb` 파일은 완전한 상태이며 읽기 전용으로 열 수 있습니다(명령 실행 중이 아니라 종료 후에 복사하십시오).
