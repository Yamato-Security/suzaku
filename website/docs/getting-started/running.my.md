# Suzaku ကို အသုံးပြုခြင်း

## Windows

Command/PowerShell Prompt သို့မဟုတ် Windows Terminal တွင် သင့်လျော်သော 32-bit သို့မဟုတ် 64-bit Windows binary ကို ရိုးရှင်းစွာ run လိုက်ပါ။

### path တွင် space ပါသော ဖိုင် သို့မဟုတ် directory ကို scan ဖတ်ရန် ကြိုးစားသည့်အခါ error ဖြစ်ခြင်း

Windows တွင် ပါဝင်ပြီးသား Command သို့မဟုတ် PowerShell prompt ကို အသုံးပြုသည့်အခါ၊ သင်၏ ဖိုင် သို့မဟုတ် directory path တွင် space ပါဝင်ပါက Suzaku သည် မည်သည့်ဖိုင်ကိုမှ load မလုပ်နိုင်ကြောင်း error တစ်ခု ရရှိနိုင်ပါသည်။
log ဖိုင်များကို မှန်ကန်စွာ load လုပ်နိုင်ရန် အောက်ပါတို့ကို ဆောင်ရွက်ရန် သေချာပါစေ။
1. ဖိုင် သို့မဟုတ် directory path ကို double quotes ဖြင့် ဝိုင်းရံပါ။
2. directory path ဖြစ်ပါက နောက်ဆုံးအက္ခရာအဖြစ် backslash ကို မထည့်မိစေရန် သေချာပါစေ။

### အက္ခရာများ မှန်ကန်စွာ မပြသခြင်း

Windows ပေါ်ရှိ default font `Lucida Console` ဖြင့်ဆိုပါက logo နှင့် ဇယားများတွင် အသုံးပြုထားသော အက္ခရာအမျိုးမျိုးသည် မှန်ကန်စွာ ပြသမည် မဟုတ်ပါ။
ဤပြဿနာကို ဖြေရှင်းရန် font ကို `Consalas` သို့ ပြောင်းသင့်ပါသည်။

## Linux

သင်သည် ဦးစွာ binary ကို executable ဖြစ်အောင် လုပ်ရန် လိုအပ်ပါသည်။

```bash
chmod +x ./suzaku
```

ထို့နောက် Suzaku root directory မှ ၎င်းကို run ပါ။

```bash
./suzaku
```

## macOS

Terminal သို့မဟုတ် [iTerm2](https://iterm2.com/) မှ၊ သင်သည် ဦးစွာ binary ကို executable ဖြစ်အောင် လုပ်ရန် လိုအပ်ပါသည်။

```bash
chmod +x ./suzaku
```

ထို့နောက် Suzaku root directory မှ ၎င်းကို run ရန် ကြိုးစားပါ။

```bash
./suzaku
```

macOS ၏ နောက်ဆုံးထွက် version တွင် ၎င်းကို run ရန် ကြိုးစားသည့်အခါ security error တစ်ခု ရရှိနိုင်ပါသည်။
"Cancel" ကို နှိပ်ပြီးနောက် System Preferences မှ "Security & Privacy" ကို ဖွင့်ကာ General tab မှ "Allow Anyway" ကို နှိပ်ပါ။
ထို့နောက် ၎င်းကို ပြန်လည် run ရန် ကြိုးစားပါ။

```bash
./suzaku
```

သတိပေးချက်တစ်ခု ပေါ်လာမည်ဖြစ်၍ "Open" ကို ရိုးရှင်းစွာ နှိပ်ပါ။
ယခု သင်သည် suzaku ကို run နိုင်ပြီ ဖြစ်ပါသည်။
