# ခွဲခြမ်းစိတ်ဖြာ ညွှန်ကြားချက်များ (Analysis Commands)

## `aws-ct-metrics` command

AWS CloudTrail logs များအတွင်းရှိ fields များအပေါ် metrics များ ဖန်တီးရန် ဤ command ကို အသုံးပြုပါ။
ပုံသေအားဖြင့် ၎င်းသည် `eventName` field ကို scan လုပ်ပါမည်။
မည်သည့် API calls များ အများဆုံးဖြစ်သည်ကို သိရှိနိုင်ရန်နှင့် detection rules ရေးသားခြင်းကို ဦးစားပေးနိုင်ရန်အတွက် ဤ command ကို ကျွန်ုပ်တို့ လက်ရှိ အသုံးပြုနေပါသည်။

## Command usage
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

### `aws-ct-metrics` command ဥပမာများ

* `eventName` API calls များ၏ ဇယားကို မျက်နှာပြင်ပေါ်တွင် ထုတ်ပြရန်: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* CSV file တစ်ခုသို့ သိမ်းဆည်းရန်: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
