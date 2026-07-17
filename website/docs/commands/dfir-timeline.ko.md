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
