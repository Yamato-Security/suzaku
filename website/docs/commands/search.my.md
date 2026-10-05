# ရှာဖွေရေး ညွှန်ကြားချက်များ (Search Commands)

## `aws-ct-search` command

Sigma rule မရေးဘဲ AWS CloudTrail log များထဲမှ လိုအပ်သော event များကို ထုတ်ယူရန် ဤ command ကို အသုံးပြုပါ။
ကိုက်ညီသော event များကို `aws-ct-timeline` နှင့် တူညီသော ပုံစံဖြင့် ပြသ သို့မဟုတ် သိမ်းဆည်းသောကြောင့် ရှာဖွေမှုမှ သိပြီးသား timeline column များသို့ တိုက်ရိုက် ကူးပြောင်းနိုင်သည်။

Event များကို ကျဉ်းမြောင်းစေရန် နည်းလမ်း သုံးမျိုးရှိပြီး ပေါင်းစပ်အသုံးပြုနိုင်သည်:

| Option | ကိုက်ညီစစ်ဆေးသည့်အရာ | တန်ဖိုးများစွာ |
|---|---|---|
| `-F, --filter FIELD:VALUE` | Field တစ်ခု၏ တန်ဖိုး၊ **အတိအကျကိုက်ညီမှု၊ စာလုံးအကြီးအသေး ခွဲခြားသည်** | `-F` ကို ထပ်ခါသုံးပါ။ filter **အားလုံး** ကိုက်ညီရမည် (AND) |
| `-k, --keyword KEYWORD` | Event JSON ထဲ မည်သည့်နေရာတွင်မဆို substring၊ စာလုံးအကြီးအသေး မခွဲခြား (`-c` ဖြင့် ခွဲခြားသည်) | `-k` ကို ထပ်ခါသုံးပါ။ keyword **တစ်ခုခု** ကိုက်ညီလျှင် လုံလောက်သည် (OR) |
| `-r, --regex REGEX` | Event JSON ထဲ မည်သည့်နေရာတွင်မဆို regular expression | Pattern တစ်ခု (ရွေးချယ်စရာများအတွက် escape မလုပ်ထားသော pipe character <code>&#124;</code> ကို သုံးပါ) |

Event တစ်ခုစီကို ဤအစီအစဉ်ဖြင့် စစ်ဆေးပြီး စစ်ဆေးမှုအားလုံး အောင်သော event များကိုသာ ထုတ်ပေးသည်: အချိန်အပိုင်းအခြား (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`။

တန်ဖိုးပါဝင်သော field ကို သိပါက (ဥပမာ `eventName:ConsoleLogin`) `-F` ကို၊ string ကိုသာသိပြီး မည်သည့်နေရာတွင် ပေါ်သည်ကို မသိပါက `-k` သို့မဟုတ် `-r` ကို အသုံးပြုပါ။

### `-F, --filter` အလုပ်လုပ်ပုံ

* ပုံစံမှာ `FIELD:VALUE` ဖြစ်သည်။ Argument ကို **ပထမဆုံး** `:` တွင် ခွဲသောကြောင့် ARN ကဲ့သို့ colon ပါသော တန်ဖိုးများကို ဒီအတိုင်း သုံးနိုင်သည်။
* Nested field များကို dot notation ဖြင့် ရေးသည် (`userIdentity.type`)။ ရှေ့ဆုံး `.` သည် မဖြစ်မနေ မလိုသောကြောင့် `.userIdentity.arn` လည်း အလုပ်လုပ်သည်။
* တန်ဖိုးကို ဝိုင်းထားသော quote (`"..."` သို့မဟုတ် `'...'`) များကို ဖယ်ရှားသည်။
* ဂဏန်းနှင့် boolean တန်ဖိုးများကို string အဖြစ် နှိုင်းယှဉ်သောကြောင့် `-F readOnly:false` နှင့် `-F responseElements.user.userId:12345` အလုပ်လုပ်သည်။
* ထို field မပါသော event သည် မကိုက်ညီပါ။
* Array အတွင်းရှိ element များကို ညွှန်းဆို၍ မရပါ (`items.0.name` ကဲ့သို့ syntax မရှိပါ)။
* `:` မပါသော filter ကို scan မစမီ ငြင်းပယ်သည်: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`။

> Note: `aws-ct-metrics` တွင် `-F` ၏ အဓိပ္ပာယ်မှာ ကွဲပြားပြီး `--field-name` ဖြစ်ကာ field အမည်ကိုသာ လက်ခံသည်။ `aws-ct-search` တွင်မူ `--filter` ဖြစ်ပြီး `FIELD:VALUE` ကို အမြဲလက်ခံသည်။

## Command usage
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

### `aws-ct-search` command ဥပမာများ

Field ဖြင့် filter လုပ်ခြင်း (`-F`):

* Console login များကို ရှာရန်: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Root user ၏ API call များကို ရှာရန် (nested field): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* သတ်မှတ်ထားသော IAM user တစ်ဦး၏ လုပ်ဆောင်ချက်အားလုံးကို ရှာရန် (တန်ဖိုးတွင် colon ပါသည်): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* ငြင်းပယ်ခံရသော call များကို ရှာရန်: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* `us-east-1` ရှိ read-only မဟုတ်သော IAM call များကို ရှာရန် (filter များစွာကို AND ဖြင့် ပေါင်းသည်): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* သတ်မှတ်ထားသော S3 bucket သို့ ဝင်ရောက်မှုကို ရှာရန် (API အလိုက် field): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Keyword (`-k`) နှင့် regular expression (`-r`) ဖြင့် ရှာဖွေခြင်း:

* IP address တစ်ခုကို မည်သည့်နေရာတွင်မဆို ဖော်ပြထားသော event များကို ရှာရန်: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* User နှစ်ဦးအနက် တစ်ဦးဦးကို ဖော်ပြထားသော event များကို ရှာရန် (keyword များစွာကို OR ဖြင့် ပေါင်းသည်): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* စာလုံးအကြီးအသေး ခွဲခြားသော keyword ရှာဖွေမှု: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* CloudTrail ကို ပိတ်ရန် ကြိုးပမ်းမှုများကို ရှာရန်: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

အခြေအနေများ ပေါင်းစပ်ခြင်းနှင့် ရလဒ်များ သိမ်းဆည်းခြင်း:

* Event source တစ်ခုတည်းသို့ ကန့်သတ်ထားသော keyword ရှာဖွေမှု: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* နောက်ဆုံး ၇ ရက်အတွင်း မအောင်မြင်သော console login များ: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* GeoIP အချက်အလက်ဖြင့် ရလဒ်များကို CSV နှင့် DuckDB သို့ သိမ်းရန်: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* ကိုက်ညီသော event များ၏ မူရင်း JSON ကို သိမ်းရန်: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search` output

Output column များသည် [`aws-ct-timeline`](dfir-timeline.md) နှင့် တူညီပြီး DuckDB output သည်လည်း တူညီသော [DuckDB output schema](dfir-timeline.md#duckdb-output-schema) ကို လိုက်နာသည်။
နောက်ဆုံးတွင် scan လုပ်ခဲ့သော event အရေအတွက်နှင့် ကိုက်ညီသော event အရေအတွက်ကို ပြသသည်:

```
Total events scanned: 209
Matching events: 1
```
