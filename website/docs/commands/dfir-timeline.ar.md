# أوامر مخطط زمني لـ DFIR

## الأمر `aws-ct-timeline`

إنشاء مخطط زمني لـ DFIR خاص بـ AWS CloudTrail استنادًا إلى قواعد Sigma الموجودة في مجلد `rules`.

## استخدام الأمر
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

### أمثلة على الأمر `aws-ct-timeline`

* إخراج التنبيهات إلى الشاشة: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* حفظ النتائج في ملف CSV: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* حفظ النتائج في ملفات CSV و JSONL: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### ملف تعريف الإخراج لـ `aws-ct-timeline`

سيقوم Suzaku بإخراج المعلومات استنادًا إلى ملف `config/default_profile.yaml`:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
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
RuleID: 'sigma.id'
```

* أي قيمة حقل تبدأ بـ `.` (مثال: `.eventTime`) سيتم أخذها من سجل CloudTrail.
* أي قيمة حقل تبدأ بـ `sigma.` (مثال: `sigma.title`) سيتم أخذها من قاعدة Sigma.
* حاليًا ندعم السلاسل النصية فقط ولكننا نخطط لدعم أنواع أخرى من قيم الحقول.

> ملاحظة: إذا كنت ترغب في إخراج بيانات JSON الأصلية والتأكد من عدم فقدان أي معلومات حقل، فقط أضف الخيار `-R, --raw-output` إلى الأمر `aws-ct-timeline`.
