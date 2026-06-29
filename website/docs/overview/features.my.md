# အင်္ဂါရပ်များ

* Cross-platform ပံ့ပိုးမှု - Windows, Linux, macOS။
* memory safe ဖြစ်ပြီး မြန်ဆန်ကာ standalone ဖြစ်စေရန် Rust ဖြင့် ဖန်တီးထားသည်။
* `.json` သို့မဟုတ် ချုံ့ထားသော `.json.gz` ဖိုင်များကို multi-thread စွမ်းဆောင်ရည်ဖြင့် scan လုပ်ပါ။
* forensic စုံစမ်းစစ်ဆေးမှုများနှင့် incident response အတွက် ခွဲခြမ်းစိတ်ဖြာရလွယ်ကူသော timeline တစ်ခုတည်းကို ဖန်တီးပါ။
* ဖတ်ရ/ဖန်တီးရ/တည်းဖြတ်ရ လွယ်ကူသော YML-based [Sigma](https://github.com/SigmaHQ/sigma) rules များဖြင့် ရေးသားထားသည့် IoC signatures များအတွက် ထူးချွန်သော native ပံ့ပိုးမှု။ (Correlation rules နှင့် [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) မှလွဲ၍ field modifier အားလုံးကို ပံ့ပိုးထားသည်။)
* signatures များအပေါ် မှီခိုစရာမလိုဘဲ ပုံမှန်မဟုတ်သော လှုပ်ရှားမှုများကို ရှာဖွေတွေ့ရှိနိုင်ရန် API အသုံးပြုမှုအားလုံး၏ အကျဉ်းချုပ်နှင့် တိုက်ခိုက်သူအကြောင်း metrics များ (source IP addresses, geo-location, အသုံးပြုခဲ့သော regions, user agents စသည်...) ကို ဖန်တီးပါ။
* ရလဒ်များကို CSV, JSON နှင့် JSONL အဖြစ် သိမ်းဆည်းပါ။
