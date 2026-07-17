# DFIR Timeline Commands

## `aws-ct-timeline` command

`rules` ဖိုလ်ဒါအတွင်းရှိ Sigma rules များအပေါ်အခြေခံ၍ AWS CloudTrail DFIR အချိန်ဇယားတစ်ခုကို ဖန်တီးပါ။

## Command usage
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

### `aws-ct-timeline` command examples

* မျက်နှာပြင်ပေါ်သို့ alerts များ ထုတ်ရန်: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* ရလဒ်များကို CSV ဖိုင်တစ်ခုသို့ သိမ်းဆည်းရန်: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* ရလဒ်များကို CSV နှင့် JSONL ဖိုင်များသို့ သိမ်းဆည်းရန်: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### `aws-ct-timeline` output profile

Suzaku သည် `config/aws_profile.yaml` ဖိုင်ကို အခြေခံ၍ အချက်အလက်များကို ထုတ်ပေးပါမည်:
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

* `.` ဖြင့်စတင်သော မည်သည့် field value မဆို (ဥပမာ: `.eventTime`) CloudTrail log မှ ရယူပါမည်။
* `sigma.` ဖြင့်စတင်သော မည်သည့် field value မဆို (ဥပမာ: `sigma.title`) Sigma rule မှ ရယူပါမည်။
* လက်ရှိတွင် strings များကိုသာ ပံ့ပိုးပေးသော်လည်း အခြား field value အမျိုးအစားများကိုလည်း ပံ့ပိုးရန် စီစဉ်ထားပါသည်။

> Note: မူရင်း JSON data ကို ထုတ်လိုပြီး မည်သည့် field အချက်အလက်ကိုမျှ မဆုံးရှုံးစေရန် သေချာစေလိုပါက `aws-ct-timeline` command သို့ `-R, --raw-output` option ကို ထည့်လိုက်ပါ။
