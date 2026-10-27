# AWS Console CSV ဒေါင်းလုဒ်များကို ဖတ်ခြင်း

Suzaku ၏ AWS command များ (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) သည် AWS Management Console ရှိ **CloudTrail → Event history → Download events → CSV** မှ ဒေါင်းလုဒ်ထားသောဖိုင်ကို `-f/--file` သို့မဟုတ် `-d/--directory` ဖြင့် ဖတ်နိုင်သည်။

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

S3 သို့ log ပို့သည့် Trail မရှိပါက CSV သည် အသုံးဝင်သော်လည်း၊ CloudTrail record အပြည့်အစုံမဟုတ်ဘဲ event တစ်ခုချင်း၏ **အကျဉ်းချုပ်** သာဖြစ်သည်။ ရနိုင်ပါက Trail JSON log သို့မဟုတ် Event history JSON download ကို ဦးစားပေးအသုံးပြုပါ။

## ပြောင်းလဲပုံ

Row တစ်ခုချင်းကို CloudTrail record အဖြစ် ပြန်လည်တည်ဆောက်သည်။ ဥပမာ `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]`, `Read-only` → boolean `readOnly` ဖြစ်သည်။ Cell အလွတ်ကို field မရှိဟုယူဆသဖြင့် `errorCode|exists` rule များက အောင်မြင်/မအောင်မြင်သော call များကို မှန်ကန်စွာခွဲခြားနိုင်သည်။ `root` အတွက် `userIdentity.type: Root` ကိုလည်း ထည့်သည်။

အင်္ဂလိပ် column name များကိုသာ အသိအမှတ်ပြုသည်။ CloudTrail console download format မဟုတ်သော CSV ကို ပယ်ချသည်။ Field အရေအတွက်မမှန်သော row (အများအားဖြင့် quote ပျက်ခြင်း) သို့မဟုတ် `Event time` မရှိသော row ကို ကျော်ပြီး line number နှင့်အတူ error log တွင် မှတ်တမ်းတင်သည်။

## ကန့်သတ်ချက်များ

1. **`us-east-1` အပါအဝင် လိုအပ်သော region အားလုံးကို ဒေါင်းလုဒ်ပါ။** Console သည် တစ်ကြိမ်လျှင် region တစ်ခုသာ export လုပ်ပြီး console sign-in နှင့် IAM event များကို `us-east-1` တွင် မှတ်တမ်းတင်သည်။
2. **Rule အချို့ match မဖြစ်နိုင်ပါ။** CSV တွင် `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements`, `additionalEventData` မပါဝင်ပါ။ CloudTrail rule 796 ခုအနက် 22 ခုခန့်၊ failed login နှင့် MFA မပါသော login rule များအပါအဝင်၊ match မဖြစ်နိုင်ပါ။
3. **Timeline column အချို့ အလွတ်ဖြစ်သည်။** `UserARN`, `UserAccountID`, `UserPrincipalID`, `ErrorMessage` သည် အမြဲအလွတ်ဖြစ်ပြီး root မဟုတ်လျှင် `UserType` လည်း အလွတ်ဖြစ်သည်။
4. **`aws-ct-summary` သည် user name အလိုက် စုစည်းသည်။** Role အတွက် session name ဖြစ်သောကြောင့် session name တူသည့် role များ ပေါင်းသွားနိုင်သည်။ User name မရှိသော row များကို summary မှဖယ်ထားသဖြင့် JSON နှင့် count မတူနိုင်ပါ။
5. **Google Workspace သည် CSV ကို မလက်ခံပါ။** `gws-timeline` နှင့် Reports API JSON/JSONL/JSON.GZ ကိုသုံးပါ။ Admin console CSV သည် အချက်အလက်ဆုံးရှုံးသော အခြား schema ဖြစ်သည်။
