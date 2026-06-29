# DFIR 요약 명령어

## `aws-ct-summary` 명령어

이 명령어는 사용자 ARN을 기준으로 다음 정보의 요약을 생성합니다:
  - 전체 이벤트(API 호출) 수 (`NumOfEvents`)
  - 로그에서 발견된 첫 번째 API 호출의 타임스탬프 (`FirstTimestamp`)
  - 로그에서 발견된 마지막 API 호출의 타임스탬프 (`LastTimestamp`)
  - 성공한, 일반적으로 악용되는 API 호출 (`AbusedAPIs-Success`)
  - 시도되었으나 실패한, 일반적으로 악용되는 API 호출 (`AbusedAPIs-Failed`)
  - 일반적으로 악용되는 API 호출 목록에 없으면서 성공한 기타 API 호출 (`OtherAPIs-Success`)
  - 일반적으로 악용되는 API 호출 목록에 없으면서 시도되었으나 실패한 기타 API 호출 (`OtherAPIs-Failed`)
  - API 호출이 이루어진 AWS 리전 (`AWS-Regions`)
  - API 호출의 출발지 IP 주소 (`SrcIPs`)
  - 사용자 유형 (`UserTypes`)
  - 사용자 액세스 키 ID (`UserAccessKeyIDs`)
  - API 호출을 수행한 출발지의 사용자 에이전트 (`UserAgents`)

※ 참고: 일반적으로 악용되는 API 호출은 [https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv](https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv)에 호스팅된 설정 파일에서 가져옵니다. 이 파일은 시간이 지남에 따라 업데이트되며 `update-rules` 명령어를 실행할 때마다 로컬에서 동기화됩니다.

이 결과는 특정 시그니처에 의존하지 않고 침해된 계정이나 공격을 발견할 수 있도록 분석가에게 정보를 제공하기 위한 것입니다.
예를 들어, 특정 사용자가 호출해서는 안 되는 의심스러운 API 호출을 하고 있는지, 일반적으로 사용되지 않는 리전을 사용하고 있는지, 의심스러운 출발지 IP 주소나 의심스러운 사용자 에이전트로 로그인하고 있는지 등을 확인할 수 있습니다...
의심스러운 API 호출, 출발지 IP 주소 또는 사용자 에이전트가 발견되면, 해당 기간 동안 어떤 액세스 키가 악용되었을 수 있는지 신속하게 판단하고, 원본 JSON 로그에서 해당 키워드를 기준으로 피벗하여 공격자 활동의 타임라인을 생성할 수 있습니다.

> 경고: 셀에 많은 데이터가 들어가게 되며 Excel과 같은 프로그램에서는 제대로 표시되지 않을 가능성이 높습니다. Mac에서는 Numbers, Windows에서는 Timeline Explorer 등을 사용하세요...

### `AbusedAPIs-Success` 예시:
```
Unique APIs: 11 | Total APIs 477,373
415,552 - RunInstances (ec2.amazonaws.com) - Spin up EC2 instances (crypto mining, tools) (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
28,907 - GetBucketAcl (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-10-21 13:59:40)
10,095 - GetCallerIdentity (sts.amazonaws.com) - Current credentials recon (2019-08-23 06:00:07 ~ 2019-08-23 06:04:14)
9,936 - ListBuckets (s3.amazonaws.com) - S3 recon (2019-08-23 06:00:07 ~ 2019-08-23 06:14:53)
9,168 - DescribeInstances (ec2.amazonaws.com) - EC2 and network layout recon (2019-08-23 06:00:07 ~ 2019-08-23 06:04:20)
3,658 - DescribeVpcs (ec2.amazonaws.com) - EC2 and network layout recon (2019-08-21 08:03:03 ~ 2019-09-12 20:00:44)
19 - ListGroups (greengrass.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-10-19 23:49:25)
14 - DescribeInstances (opsworks.amazonaws.com) - EC2 and network layout recon (2019-08-21 08:03:03 ~ 2019-10-19 23:49:22)
12 - GetBucketPolicy (s3.amazonaws.com) - S3 recon (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
7 - ListGroups (resource-groups.amazonaws.com) - IAM enumeration (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
5 - StartInstances (ec2.amazonaws.com) - Spin up EC2 instances (crypto mining, tools) (2019-08-21 08:03:03 ~ 2019-12-12 07:07:31)
```

### `AbusedAPIs-Failed` 예시:
```
Unique APIs: 23 | Total APIs 20,464
11,603 - AssumeRole (sts.amazonaws.com) - Lateral movement via roles (2019-08-21 08:03:03 ~ 2019-09-18 07:04:12)
7,279 - GetBucketAcl (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-09-09 09:01:26)
515 - GetBucketPolicy (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-10-01 19:11:07)
331 - ListUsers (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-08-29 14:53:14)
210 - ListSecrets (secretsmanager.amazonaws.com) - Find credential storage locations (2019-08-21 08:03:03 ~ 2019-10-19 23:49:30)
153 - ListGroups (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-09-12 15:24:39)
148 - ListRoles (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-09-12 15:20:56)
112 - ListAccessKeys (iam.amazonaws.com) - Enumerates keys on IAM users (2019-08-21 08:03:03 ~ 2019-09-16 14:28:15)
31 - ListGroups (greengrass.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2020-02-25 14:41:24)
...
```

### `OtherAPIs-Success` 예시:
```
Unique APIs: 289 | Total APIs 143,759
98,689 - DescribeSnapshots (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:50:59)
10,679 - DescribeSpotPriceHistory (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:32)
3,740 - DescribeReservedInstancesOfferings (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:30)
2,372 - DescribeSnapshotAttribute (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-08-24 12:38:34)
2,307 - CreateDefaultVpc (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:17)
1,532 - DescribeKeyPairs (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:16)
1,504 - DescribeSecurityGroups (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:00:40)
1,495 - DescribeImages (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:26)
1,438 - CreateKeyPair (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:16)
1,402 - DescribeVolumes (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-03 12:06:20)
1,217 - DescribeSubnets (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:00:43)
...
```

### `AWS-Regions` 예시:
```
Total regions: 16
167,155 - us-west-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:14:53)
113,328 - us-east-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:04:14)
65,718 - ap-northeast-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:22:55)
64,787 - ap-northeast-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:34:57)
...
```

### `SrcIPs` 예시:
```
Total source IPs: 5,293
634,454 - 5.205.62.253 (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
23,498 - 193.29.252.218 (2019-08-21 08:03:00 ~ 2019-10-17 09:11:22)
15,925 - 155.63.17.217 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
9,067 - 253.0.255.253 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
7,078 - 163.21.250.220 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
6,575 - 236.9.245.88 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
5,138 - 84.252.252.117 (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
4,946 - 24.251.252.2 (2019-08-21 08:03:00 ~ 2019-09-30 06:36:13)
4,225 - 211.111.151.81 (2019-08-21 08:03:00 ~ 2019-09-12 19:53:35)
...
```

### `UserType` 예시:
```
IAMUser
```

### `UserAccessKeyIDs` 예시:
```
Total access key ids: 629
667,476 - AKIA01U43UX3RBRDXF4Q (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
218,544 - ASIARF55FBMFZBXLKDFW (2019-08-21 11:31:47 ~ 2019-08-23 13:00:28)
12,677 - AKIA1ZBTOEKWKVHP6GHZ (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
8,822 - ASIAGD2JRX0V6RJGWR59 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
4,940 - ASIAUNHV6EHIK5MNPEKF (2019-08-21 08:03:00 ~ 2019-09-30 06:36:17)
...
```

> AWS STS 임시 액세스 키 ID가 대량으로 존재하므로 기본적으로 이러한 키는 필터링되어 제외됩니다. 이를 포함하려면 `-s, --include-sts-keys` 옵션을 추가하세요.

### `UserAgents` 예시:
```
Total user agents: 7,760
351,022 - Boto3/1.9.201 Python/2.7.12 Linux/4.4.0-159-generic Botocore/1.12.201 (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
283,430 - Boto3/1.9.201 Python/2.7.12 Linux/4.4.0-157-generic Botocore/1.12.201 (2019-08-21 11:31:47 ~ 2019-08-23 13:00:28)
23,467 - [Boto3/1.15.13 Python/3.8.5 Darwin/19.6.0 Botocore/1.18.13 Resource] (2017-02-12 21:15:12 ~ 2020-10-07 16:05:52)
15,924 - Boto3/1.7.4 Python/2.7.12 Linux/4.4.0-119-generic Botocore/1.10.4 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
9,400 - aws-sdk-java/1.11.301 Linux/4.9.93-linuxkit-aufs Java_HotSpot(TM)_64-Bit_Server_VM/25.172-b11 java/1.8.0_172 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
6,372 - [Boto3/1.7.65 Python/3.5.2 Linux/4.13.0-37-generic Botocore/1.10.65 Resource] (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
5,352 - AWSPowerShell/3.3.365.0 .NET_Runtime/4.0 .NET_Framework/4.0 OS/Microsoft_Windows_NT_10.0.01985.0 WindowsPowerShell/5.0 ClientSync (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
4,945 - Boto3/1.9.231 Python/2.7.15+ Linux/4.15.0-64-generic Botocore/1.12.231 (2019-08-21 08:03:00 ~ 2019-09-30 06:36:13)
4,599 - [aws-cli/1.16.301 Python/3.7.6 Linux/5.4.0-kali3-amd64 botocore/1.13.37] (2019-08-21 08:03:00 ~ 2020-02-09 22:00:32)
3,909 - Boto3/1.14.28 Python/3.8.5 Linux/5.7.0-kali1-amd64 Botocore/1.17.28 (2019-01-08 20:30:01 ~ 2020-09-11 17:35:39)
3,450 - Boto3/1.4.2 Python/2.7.13+ Linux/4.9.0-3-amd64 Botocore/1.5.19 (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
3,198 - Boto3/1.4.2 Python/2.7.14 Linux/4.13.0-1-amd64 Botocore/1.5.19 (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
...
```

> `aws` 클라이언트 도구는 사용자 에이전트에 OS 정보를 포함하므로 `kali`와 같은 공격자 OS에서 이루어진 API 호출이 있었는지 탐지할 수 있습니다.

## 명령어 사용법
```
Usage:
  suzaku aws-ct-summary <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
  -s, --include-sts-keys       Include temporary AWS STS access key IDs

Output:
  -o, --output <FILE>           Output results to a CSV file
  -D, --hide-descriptions       Hide description of the commonly abused API calls
  -G, --GeoIP <MAXMIND-DB-DIR>  Add GeoIP (ASN, city, country) info to IP addresses

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
```

### `aws-ct-metrics` 명령어 예시

* 결과를 CSV 파일로 저장: `./suzaku aws-ct-summary -d ../suzaku-sample-data -o sample-summary.csv`
