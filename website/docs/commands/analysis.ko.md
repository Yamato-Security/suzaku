# 분석 명령어

## `aws-ct-metrics` 명령어

이 명령어를 사용하여 AWS CloudTrail 로그 내 필드에 대한 메트릭을 생성합니다.
기본적으로 `eventName` 필드를 스캔합니다.
현재 우리는 탐지 규칙 작성의 우선순위를 정하기 위해 어떤 API 호출이 가장 흔한지 파악하는 데 이 명령어를 사용하고 있습니다.

## 명령어 사용법
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -F, --field-name <FIELD_NAME>  The field to generate metrics for [default: eventName]
  -o, --output <FILE>            Output CSV

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
```

### `aws-ct-metrics` 명령어 예시

* `eventName` API 호출 테이블을 화면에 출력: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* CSV 파일로 저장: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
