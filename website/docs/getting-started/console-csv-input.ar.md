# قراءة تنزيلات CSV من وحدة تحكم AWS

تستطيع أوامر AWS في Suzaku (`aws-ct-timeline` و`aws-ct-metrics` و`aws-ct-summary` و`aws-ct-search`) قراءة الملف الذي يتم تنزيله من **CloudTrail → Event history → Download events → CSV** في AWS Management Console باستخدام `-f/--file` أو `-d/--directory`.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

يفيد CSV عندما لا يرسل أي Trail السجلات إلى S3، لكنه ليس سجل CloudTrail الكامل؛ بل هو **ملخص** لكل حدث. استخدم سجلات JSON الخاصة بـ Trail أو تنزيل JSON من Event history متى كانت متاحة.

## التحويل

يُعاد بناء كل صف كسجل CloudTrail:‏ `Event time` ← `eventTime`، و`Event name` ← `eventName`، و`User name` ← `userIdentity.userName`، و`AWS access key` ← `userIdentity.accessKeyId`، و`Resources` ← `resources[]`، وتتحول `Read-only` إلى قيمة منطقية `readOnly`. تُعامل الخلية الفارغة كحقل غير موجود كي تميز قواعد `errorCode|exists` بصورة صحيحة بين الاستدعاءات الناجحة والفاشلة. ويُضاف أيضًا `userIdentity.type: Root` للمستخدم `root`.

لا تُعرَف إلا أسماء الأعمدة الإنجليزية. يُرفض أي CSV لا يطابق تنزيل CloudTrail من وحدة التحكم. تُتجاوز الصفوف ذات عدد الحقول غير الصحيح (غالبًا بسبب علامات اقتباس تالفة) أو التي لا تحتوي على `Event time`، وتُسجل مع رقم السطر في سجل الأخطاء.

## القيود

1. **نزّل كل منطقة مطلوبة، بما فيها `us-east-1`.** تصدّر وحدة التحكم منطقة واحدة في كل مرة، وتُسجل عمليات الدخول إلى وحدة التحكم وأحداث IAM في `us-east-1`.
2. **لا يمكن لبعض القواعد أن تتطابق.** لا يحتوي CSV على `userIdentity.arn`/`type`/`accountId`/`principalId` أو `errorMessage` أو `requestParameters` أو `responseElements` أو `additionalEventData`. لذلك لا يمكن لنحو 22 قاعدة من أصل 796 قاعدة CloudTrail، ومنها فشل تسجيل الدخول والدخول دون MFA، أن تتطابق.
3. **تبقى بعض أعمدة الخط الزمني فارغة.** تكون `UserARN` و`UserAccountID` و`UserPrincipalID` و`ErrorMessage` فارغة دائمًا، وكذلك `UserType` باستثناء root.
4. **يجمع `aws-ct-summary` حسب اسم المستخدم.** في حالة الدور تكون القيمة اسم الجلسة، لذلك تُدمج الأدوار المختلفة التي تستخدم الاسم نفسه. تُحذف الصفوف التي لا تحتوي على اسم مستخدم، وقد تختلف الأعداد عن JSON للفترة نفسها.
5. **لا يدعم Google Workspace صيغة CSV.** استخدم JSON/JSONL/JSON.GZ من Reports API مع `gws-timeline`. يستخدم CSV من Admin console مخططًا مختلفًا يفقد بعض المعلومات.
