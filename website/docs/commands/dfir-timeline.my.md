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

### DuckDB output schema {#duckdb-output-schema}

CSV နှင့် JSON output များသည် အထက်ပါ profile ကို *ပြသရန် ပုံဖော်ထားခြင်း* ဖြစ်ပြီး DuckDB output မှာမူ *data interface* ဖြစ်သောကြောင့် type သတ်မှတ်ထားပြီး မိမိကိုယ်ကို ဖော်ပြနိုင်သော ပုံစံဖြစ်သည်။ ဤကွာခြားချက်များကို ရည်ရွယ်ချက်ရှိရှိ ပြုလုပ်ထားခြင်းဖြစ်ပြီး `aws-ct-timeline`, `azure-timeline`, `gws-timeline` နှင့် `aws-ct-search` တို့တွင် အကျုံးဝင်သည်:

| | CSV / JSON | DuckDB |
|---|---|---|
| တန်ဖိုးမရှိသောအခါ | `-` (သို့မဟုတ် အလွတ်) | `NULL` |
| `Timestamp` | ပုံဖော်ထားသော စာသား | `TIMESTAMP` |
| `Level` | စာသား | `suzaku_level` (ပြင်းထန်မှုအလိုက် စီထားသော `ENUM`) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (SQL တွင် quote မလိုအပ်) |
| `Tags` | ` ¦ ` ဖြင့် ဆက်ထားသော string တစ်ခု | `Tactics`, `TechniqueIDs`, `OtherTags` (တစ်ခုစီသည် `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | `-G, --geo-ip` သုံးမှသာ ထည့်သည် | အမြဲရှိသည် (profile တွင် `SrcIP` ရှိလျှင်)၊ `-G` မသုံးပါက `NULL` |
| ထပ်နေသော row များ | ထားရှိသည် | အတိအကျ ထပ်နေသည်များကို ဖယ်ရှားပြီး အရေအတွက်ကို `suzaku_meta` တွင် မှတ်တမ်းတင်သည် |

ဖိုင်တိုင်းတွင် row တစ်ကြောင်းပါသော `suzaku_meta` table ပါဝင်သောကြောင့် ဖိုင်ကို မည်သည့်အရာက ထုတ်ပေးခဲ့သည်ကို ခန့်မှန်းစရာမလိုဘဲ သိနိုင်သည်:

| Column | အဓိပ္ပာယ် |
|---|---|
| `schema_version` | Layout version။ အခြား table များကို မဖတ်မီ စစ်ဆေးပါ။ |
| `suzaku_version`, `command`, `command_line` | မည်သည့် Suzaku၊ မည်သည့် subcommand၊ မည်သည့် command line အတိအကျ။ |
| `generated_at` | ဖိုင်ကို ရေးသားခဲ့သည့်အချိန်။ |
| `timestamp_tz` | `Timestamp` column ၏ time zone — `UTC` သို့မဟုတ် `-l, --localtime` သုံးပါက local offset။ |
| `rules_version`, `rules_count` | Ruleset revision (rules folder သည် git checkout ဖြစ်ပါက) နှင့် load လုပ်ခဲ့သော rule အရေအတွက်။ |
| `geoip_enabled` | `-G, --geo-ip` ကို run ခဲ့သလား။ `SrcCountry` အားလုံး `NULL` ဖြစ်ခြင်း ("enrichment ပိတ်ထားသည်") နှင့် enrich လုပ်ထားသောဖိုင်ရှိ `NULL` cell ("ဤတန်ဖိုးသည် IP address မဟုတ်") ကို ခွဲခြားနိုင်သည်။ |
| `scanned_files`, `scanned_events` | Run ၏ scan လုပ်ခဲ့သော အတိုင်းအတာ။ |
| `output_rows`, `duplicate_rows_removed` | ရေးခဲ့သော row များနှင့် ရေးစဉ် ဖယ်ရှားခဲ့သော အတိအကျထပ်နေသည်များ။ |

Rule အခြေပြု timeline command များတွင် `timeline` row တစ်ကြောင်းသည် **event တစ်ခု × rule match တစ်ခု** ဖြစ်သည်။ rule အများအပြားနှင့် ကိုက်ညီသော event သည် match တစ်ခုစီအတွက် row တစ်ကြောင်းစီ ထုတ်ပေးသောကြောင့် `EventID` သည် unique *မဟုတ်ပါ*။ `aws-ct-search` တွင် ကိုက်ညီသော event တစ်ခုစီအတွက် row တစ်ကြောင်း ထုတ်ပေးသည်။

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

Suzaku သည် မထွက်မီ database ကို checkpoint လုပ်သောကြောင့် `.duckdb` ဖိုင်သည် ပြည့်စုံပြီး read-only ဖြင့် ဖွင့်နိုင်သည် (command run နေစဉ်မဟုတ်ဘဲ ပြီးဆုံးပြီးမှ copy ကူးပါ)။
