# Git Cloning

အောက်ပါ command ဖြင့် repository ကို `git clone` လုပ်ပြီး source code မှ binary ကို compile လုပ်နိုင်ပါသည်။

**သတိပေးချက်:** repository ၏ main branch သည် development အတွက် ဖြစ်သောကြောင့် တရားဝင် မထုတ်ပြန်ရသေးသော feature အသစ်များကို သုံးနိုင်ဖွယ်ရှိသော်လည်း bug များ ရှိနိုင်သဖြင့် တည်ငြိမ်မှု မရှိသည်ဟု မှတ်ယူပါ။

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Note:** `--recursive` option ကို သုံးရန် မေ့သွားပါက git submodule အဖြစ် စီမံခန့်ခွဲထားသော `rules` folder ကို clone လုပ်မည် မဟုတ်ပါ။

`rules` folder ကို sync လုပ်ပြီး နောက်ဆုံး Suzaku rules များ ရယူရန် `git pull --recurse-submodules` ကို သုံးနိုင်သည် သို့မဟုတ် အောက်ပါ command ကို သုံးပါ။

```bash
./suzaku update-rules
```

update လုပ်ခြင်း မအောင်မြင်ပါက `rules` folder ကို အမည်ပြောင်းပြီး ထပ်မံ ကြိုးစားရန် လိုအပ်နိုင်ပါသည်။

>> သတိပြုရန်: update လုပ်သည့်အခါ `rules` folder အတွင်းရှိ rules နှင့် config file များကို [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) repository အတွင်းရှိ နောက်ဆုံး rules နှင့် config file များဖြင့် အစားထိုးပါသည်။
>> ရှိပြီးသား file များတွင် သင်ပြုလုပ်ထားသော ပြောင်းလဲမှုများ overwrite လုပ်ခံရမည် ဖြစ်သောကြောင့် update မလုပ်မီ သင် edit လုပ်ထားသော file များ၏ backup ကို ပြုလုပ်ထားရန် အကြံပြုပါသည်။
>> `rules` folder အတွင်းတွင် rules **အသစ်** များ ထည့်ပါက update လုပ်သည့်အခါ ၎င်းတို့ကို overwrite သို့မဟုတ် ဖျက်မည် **မဟုတ်** ပါ။
