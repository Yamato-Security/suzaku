# 검색 명령어

## `aws-ct-search` 명령어

Sigma 룰을 작성하지 않고 AWS CloudTrail 로그에서 원하는 이벤트를 추출할 때 이 명령어를 사용합니다.
일치한 이벤트는 `aws-ct-timeline`과 같은 형식으로 표시되거나 저장되므로, 검색 결과를 익숙한 타임라인 열로 바로 확인할 수 있습니다.

이벤트를 좁히는 방법은 다음 세 가지이며 함께 사용할 수 있습니다:

| 옵션 | 일치 대상 | 여러 값 |
|---|---|---|
| `-F, --filter FIELD:VALUE` | 하나의 필드 값, **완전 일치, 대소문자 구분** | `-F`를 반복 지정. **모든** 조건과 일치해야 함(AND) |
| `-k, --keyword KEYWORD` | 이벤트 JSON 내 임의 위치의 부분 문자열, 대소문자 구분 안 함(`-c` 지정 시 구분) | `-k`를 반복 지정. **어느 하나**와 일치하면 됨(OR) |
| `-r, --regex REGEX` | 이벤트 JSON 내 임의 위치의 정규 표현식 | 하나의 패턴(여러 후보는 이스케이프하지 않은 파이프 문자 <code>&#124;</code>로 지정) |

각 이벤트는 다음 순서로 판정되며, 모든 검사를 통과한 이벤트만 출력됩니다: 시간 범위(`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

값이 들어 있는 필드를 알고 있다면(예: `eventName:ConsoleLogin`) `-F`를, 문자열만 알고 어디에 나타나는지 모른다면 `-k` 또는 `-r`을 사용하십시오.

### `-F, --filter` 동작 방식

* 형식은 `FIELD:VALUE`입니다. 인수는 **첫 번째** `:`에서 분할되므로 ARN처럼 콜론을 포함한 값도 그대로 지정할 수 있습니다.
* 중첩 필드는 점 표기법으로 지정합니다(`userIdentity.type`). 앞의 `.`는 생략 가능하므로 `.userIdentity.arn`도 사용할 수 있습니다.
* 값을 감싸는 따옴표(`"..."` 또는 `'...'`)는 제거됩니다.
* 숫자와 불리언은 문자열로 비교되므로 `-F readOnly:false`와 `-F responseElements.user.userId:12345`를 사용할 수 있습니다.
* 해당 필드가 없는 이벤트는 일치하지 않습니다.
* 배열 내 요소는 지정할 수 없습니다(`items.0.name` 같은 구문은 없습니다).
* `:`가 없는 필터는 스캔 시작 전에 거부됩니다: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> 참고: `aws-ct-metrics`의 `-F`는 `--field-name`이며 필드 이름만 받습니다. `aws-ct-search`의 `-F`는 `--filter`이며 항상 `FIELD:VALUE`를 받습니다.

## 명령어 사용법
```
Usage: suzaku aws-ct-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory of multiple gz/json/parquet files
  -f, --file <FILE>      File path to one gz/json/parquet file

Filtering:
  -F, --filter <FILTER...>     Filter by specific field(s)
  -c, --preserve-case          Case-sensitive keyword search
  -k, --keyword <KEYWORD...>   Search by keyword(s)
  -r, --regex <REGEX>          Search by regular expression
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
      --file-date-to <DATE>    Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")

Output:
  -C, --clobber                   Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>   Add GeoIP (ASN, city, country) info to IP addresses
  -o, --output <FILE>             Save the results to a file
  -t, --output-type <FORMAT,...>  Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [default: csv] [possible values: csv, json, jsonl, duckdb]
      --raw-output                Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>   Number of threads to use (default: same as CPU cores)

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### `aws-ct-search` 명령어 예시

필드로 필터링(`-F`):

* 콘솔 로그인 찾기: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* root 사용자의 API 호출 찾기(중첩 필드): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* 특정 IAM 사용자의 모든 작업 찾기(값에 콜론 포함): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* 거부된 호출 찾기: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* `us-east-1`의 읽기 전용이 아닌 IAM 호출 찾기(여러 필터는 AND): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* 특정 S3 버킷에 대한 접근 찾기(API별 필드): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

키워드(`-k`)와 정규 표현식(`-r`)으로 검색:

* IP 주소가 어디든 포함된 이벤트 찾기: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* 두 사용자 중 하나가 포함된 이벤트 찾기(여러 키워드는 OR): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* 대소문자를 구분하는 키워드 검색: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* CloudTrail 비활성화 시도 찾기: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

조건 조합과 결과 저장:

* 하나의 이벤트 소스로 제한한 키워드 검색: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* 최근 7일간 실패한 콘솔 로그인: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* GeoIP 정보를 추가해 CSV와 DuckDB로 저장: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* 일치한 이벤트의 원본 JSON 저장: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search` 출력

출력 열은 [`aws-ct-timeline`](dfir-timeline.md)과 같으며, DuckDB 출력도 같은 [DuckDB 출력 스키마](dfir-timeline.md#duckdb-output-schema)를 따릅니다.
마지막에 스캔한 이벤트 수와 일치한 이벤트 수가 표시됩니다:

```
Total events scanned: 209
Matching events: 1
```
