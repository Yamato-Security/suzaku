# AWS 콘솔 CSV 다운로드 읽기

Suzaku의 AWS 명령(`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`)은 AWS Management Console의 **CloudTrail → Event history → Download events → CSV**에서 받은 파일을 `-f/--file` 또는 `-d/--directory`로 읽을 수 있습니다.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

Trail이 S3로 로그를 전송하지 않는 환경에서는 유용하지만, 이 CSV는 전체 CloudTrail 레코드가 아니라 이벤트의 **요약**입니다. Trail JSON 로그나 Event history의 JSON 다운로드가 있다면 JSON을 우선 사용하십시오.

## 변환 방식

각 행은 CloudTrail 레코드로 재구성됩니다. 예: `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]`, `Read-only` → 불리언 `readOnly`. 빈 셀은 필드가 없는 것으로 처리되어 `errorCode|exists` 규칙이 성공과 실패를 올바르게 구분합니다. `root`에는 `userIdentity.type: Root`도 추가됩니다.

영문 열 이름만 인식합니다. CloudTrail 콘솔 다운로드 형식이 아닌 CSV는 거부됩니다. 필드 수가 잘못된 행(주로 깨진 따옴표)이나 `Event time`이 없는 행은 건너뛰고, 행 번호와 함께 오류 로그에 기록합니다.

## 제한 사항

1. **필요한 모든 리전과 `us-east-1`을 다운로드해야 합니다.** 콘솔은 한 번에 한 리전만 다운로드하며, 콘솔 로그인과 IAM 이벤트는 `us-east-1`에 기록됩니다.
2. **일부 규칙은 일치할 수 없습니다.** CSV에는 `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements`, `additionalEventData`가 없습니다. 약 796개 CloudTrail 규칙 중 22개(로그인 실패 및 MFA 없는 로그인 포함)는 일치하지 않습니다.
3. **일부 타임라인 열은 비어 있습니다.** `UserARN`, `UserAccountID`, `UserPrincipalID`, `ErrorMessage`는 항상 비어 있고, root를 제외하면 `UserType`도 비어 있습니다.
4. **`aws-ct-summary`는 사용자 이름으로 그룹화합니다.** 역할의 사용자 이름은 세션 이름이므로 같은 세션 이름을 쓴 서로 다른 역할이 하나로 합쳐집니다. 사용자 이름이 없는 행은 요약에서 제외되어 같은 기간의 JSON과 건수가 다를 수 있습니다.
5. **Google Workspace는 CSV를 지원하지 않습니다.** `gws-timeline`에는 Reports API JSON/JSONL/JSON.GZ를 사용해야 합니다. Admin console CSV는 정보가 손실된 별도 스키마입니다.
