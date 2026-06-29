# DFIR အကျဉ်းချုပ် Commands

## `aws-ct-summary` command

ဤ command သည် အသုံးပြုသူ ARN များအပေါ် အခြေခံ၍ အောက်ပါအချက်အလက်များ၏ အကျဉ်းချုပ်ကို ဖန်တီးပေးသည်-
  - စုစုပေါင်း event အရေအတွက် (API calls များ) (`NumOfEvents`)
  - log များတွင် တွေ့ရှိသော ပထမဆုံး API call ၏ အချိန်တံဆိပ် (`FirstTimestamp`)
  - log များတွင် တွေ့ရှိသော နောက်ဆုံး API call ၏ အချိန်တံဆိပ် (`LastTimestamp`)
  - အောင်မြင်ခဲ့သော အများအားဖြင့် အလွဲသုံးစားခံရသည့် API calls များ (`AbusedAPIs-Success`)
  - ကြိုးပမ်းခဲ့သော်လည်း မအောင်မြင်ခဲ့သည့် အများအားဖြင့် အလွဲသုံးစားခံရသည့် API calls များ (`AbusedAPIs-Failed`)
  - အများအားဖြင့် အလွဲသုံးစားခံရသည့် API calls စာရင်းတွင် မပါဝင်ဘဲ အောင်မြင်ခဲ့သော အခြား API calls များ (`OtherAPIs-Success`)
  - အများအားဖြင့် အလွဲသုံးစားခံရသည့် API calls စာရင်းတွင် မပါဝင်ဘဲ ကြိုးပမ်းခဲ့သော်လည်း မအောင်မြင်ခဲ့သည့် အခြား API calls များ (`OtherAPIs-Failed`)
  - API calls များ ပြုလုပ်ခဲ့သည့် AWS regions များ (`AWS-Regions`)
  - API call ၏ source IP လိပ်စာများ (`SrcIPs`)
  - အသုံးပြုသူ အမျိုးအစားများ (`UserTypes`)
  - အသုံးပြုသူ access key ID များ (`UserAccessKeyIDs`)
  - API call ပြုလုပ်ခဲ့သည့် source ၏ user agent များ (`UserAgents`)

※ မှတ်ချက်- အများအားဖြင့် အလွဲသုံးစားခံရသည့် API calls များသည် [https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv](https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv) တွင် hosted လုပ်ထားသော config ဖိုင်မှ လာသည်။ ဤဖိုင်ကို အချိန်ကြာလာသည်နှင့်အမျှ update လုပ်သွားမည်ဖြစ်ပြီး `update-rules` command ကို run သည့်အခါတိုင်း ဒေသတွင်း sync လုပ်သွားမည်ဖြစ်သည်။

ဤရလဒ်များသည် သီးခြား signature များအပေါ် မမှီခိုဘဲ အေးဂျင့်များအား အပေးယူခံရသော အကောင့်များ သို့မဟုတ် တိုက်ခိုက်မှုများကို ရှာဖွေနိုင်ရန် အချက်အလက်များ ပေးအပ်ရန် ရည်ရွယ်ထားသည်။
ဥပမာအားဖြင့်၊ အချို့သော အသုံးပြုသူများသည် သူတို့ မ ခေါ်သင့်သော သံသယဖြစ်ဖွယ် API calls များကို ခေါ်နေခြင်း ရှိမရှိ၊ ပုံမှန်အသုံးမပြုသော regions များကို အသုံးပြုခြင်း၊ သံသယဖြစ်ဖွယ် source IP လိပ်စာများမှ သို့မဟုတ် သံသယဖြစ်ဖွယ် user agent များဖြင့် ဝင်ရောက်နေခြင်း စသည်တို့ကို စစ်ဆေးနိုင်သည်...
သံသယဖြစ်ဖွယ် API calls များ ခေါ်နေခြင်း၊ source IP လိပ်စာများ သို့မဟုတ် user agent များ တွေ့ရှိပြီးနောက်၊ ထိုအချိန်အတွင်း မည်သည့် access key များ အလွဲသုံးစားခံရနိုင်သည်ကို လျင်မြန်စွာ ဆုံးဖြတ်နိုင်ပြီး၊ ထို keyword များကို မူရင်း JSON log များတွင် pivot လုပ်၍ တိုက်ခိုက်သူ၏ လှုပ်ရှားမှု timeline တစ်ခု ဖန်တီးနိုင်သည်။

> သတိ- cell များတွင် data များစွာ ပါဝင်မည်ဖြစ်ပြီး Excel ကဲ့သို့သော program များတွင် ကောင်းစွာ ဖော်ပြနိုင်လိမ့်မည် မဟုတ်ပါ။ Mac တွင် Numbers၊ Windows တွင် Timeline Explorer စသည်တို့ကို အသုံးပြုပါ...

### `AbusedAPIs-Success` ဥပမာ-
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

### `AbusedAPIs-Failed` ဥပမာ-
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

### `OtherAPIs-Success` ဥပမာ-
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

### `AWS-Regions` ဥပမာ-
```
Total regions: 16
167,155 - us-west-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:14:53)
113,328 - us-east-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:04:14)
65,718 - ap-northeast-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:22:55)
64,787 - ap-northeast-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:34:57)
...
```

### `SrcIPs` ဥပမာ-
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

### `UserType` ဥပမာ-
```
IAMUser
```

### `UserAccessKeyIDs` ဥပမာ-
```
Total access key ids: 629
667,476 - AKIA01U43UX3RBRDXF4Q (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
218,544 - ASIARF55FBMFZBXLKDFW (2019-08-21 11:31:47 ~ 2019-08-23 13:00:28)
12,677 - AKIA1ZBTOEKWKVHP6GHZ (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
8,822 - ASIAGD2JRX0V6RJGWR59 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
4,940 - ASIAUNHV6EHIK5MNPEKF (2019-08-21 08:03:00 ~ 2019-09-30 06:36:17)
...
```

> AWS STS access key ID ယာယီများ အများအပြား ရှိမည်ဖြစ်သောကြောင့်၊ ၎င်းတို့ကို default အားဖြင့် ဖယ်ထုတ်ထားသည်ကို သတိပြုပါ။ ၎င်းတို့ကို ထည့်သွင်းလိုပါက `-s, --include-sts-keys` option ကို ထည့်ပါ။

### `UserAgents` ဥပမာ-
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

> `aws` client tool သည် user agent တွင် OS အချက်အလက်ကို ထည့်သွင်းမည်ဖြစ်သောကြောင့်၊ `kali` ကဲ့သို့သော တိုက်ခိုက်သူ OS များမှ API calls ပြုလုပ်ခဲ့ခြင်း ရှိမရှိကို ရှာဖွေနိုင်သည်ကို သတိပြုပါ။

## Command အသုံးပြုပုံ
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

### `aws-ct-metrics` command ဥပမာ

* ရလဒ်များကို CSV ဖိုင်တစ်ခုသို့ သိမ်းဆည်းရန်- `./suzaku aws-ct-summary -d ../suzaku-sample-data -o sample-summary.csv`
