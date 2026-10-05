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

سيقوم Suzaku بإخراج المعلومات استنادًا إلى ملف `config/aws_profile.yaml`:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
Level: 'sigma.level'
EventSource: '.eventSource'
EventName: '.eventName'
ErrorCode: '.errorCode'
UserName: '.userIdentity.userName'
UserType: '.userIdentity.type'
SrcIP: '.sourceIPAddress'
AWS-Region: '.awsRegion'
UserAccountID: '.userIdentity.accountId'
UserARN: '.userIdentity.arn'
UserAccessKeyID: '.userIdentity.accessKeyId'
UserPrincipalID: '.userIdentity.principalId'
UserAgent: '.userAgent'
ErrorMessage: '.errorMessage'
EventID: '.eventID'
RuleAuthor: 'sigma.author'
Tags: 'sigma.tags'
RuleID: 'sigma.id'
```

* أي قيمة حقل تبدأ بـ `.` (مثال: `.eventTime`) سيتم أخذها من سجل CloudTrail.
* أي قيمة حقل تبدأ بـ `sigma.` (مثال: `sigma.title`) سيتم أخذها من قاعدة Sigma.
* حاليًا ندعم السلاسل النصية فقط ولكننا نخطط لدعم أنواع أخرى من قيم الحقول.

> ملاحظة: إذا كنت ترغب في إخراج بيانات JSON الأصلية والتأكد من عدم فقدان أي معلومات حقل، فقط أضف الخيار `-R, --raw-output` إلى الأمر `aws-ct-timeline`.

### مخطط إخراج DuckDB {#duckdb-output-schema}

مخرجات CSV وJSON هي *عرض* لملف التعريف أعلاه؛ أما مخرجات DuckDB فهي *واجهة بيانات*، لذا فهي محددة الأنواع وذاتية الوصف. هذه الاختلافات مقصودة وتنطبق على `aws-ct-timeline` و`azure-timeline` و`gws-timeline` و`aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| قيمة مفقودة | `-` (أو فارغة) | `NULL` |
| `Timestamp` | نص منسق | `TIMESTAMP` |
| `Level` | نص | `suzaku_level` (`ENUM` مرتب حسب الخطورة) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (لا حاجة لعلامات الاقتباس في SQL) |
| `Tags` | سلسلة واحدة مدمجة بـ ` ¦ ` | `Tactics`, `TechniqueIDs`, `OtherTags` (كل منها `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | تُضاف فقط مع `-G, --geo-ip` | موجودة دائمًا (عندما يحتوي ملف التعريف على `SrcIP`)، و`NULL` عند عدم استخدام `-G` |
| الصفوف المكررة | تُحتفظ بها | تُزال التكرارات المطابقة تمامًا، ويُسجَّل عددها في `suzaku_meta` |

يحتوي كل ملف أيضًا على جدول `suzaku_meta` من صف واحد، حتى يعرف القارئ ما الذي أنتجه دون تخمين:

| العمود | المعنى |
|---|---|
| `schema_version` | إصدار البنية. تحقق منه قبل قراءة الجداول الأخرى. |
| `suzaku_version`, `command`, `command_line` | أي إصدار من Suzaku، وأي أمر فرعي، وأي استدعاء بالضبط. |
| `generated_at` | وقت كتابة الملف. |
| `timestamp_tz` | المنطقة الزمنية لعمود `Timestamp` — `UTC`، أو الإزاحة المحلية مع `-l, --localtime`. |
| `rules_version`, `rules_count` | مراجعة مجموعة القواعد (عندما يكون مجلد rules نسخة git) وعدد القواعد المحمّلة. |
| `geoip_enabled` | ما إذا تم تشغيل `-G, --geo-ip`. يميّز بين `SrcCountry` الذي كله `NULL` ("كان الإثراء متوقفًا") وخلية `NULL` في ملف مُثرى ("هذه القيمة ليست عنوان IP"). |
| `scanned_files`, `scanned_events` | نطاق التشغيل. |
| `output_rows`, `duplicate_rows_removed` | الصفوف المكتوبة، والتكرارات المطابقة تمامًا التي أُزيلت أثناء الكتابة. |

بالنسبة إلى أوامر المخطط الزمني المستندة إلى القواعد، يمثل صف `timeline` الواحد **حدثًا واحدًا × تطابق قاعدة واحدًا**: الحدث الذي يطابق عدة قواعد ينتج صفًا لكل تطابق، لذا فإن `EventID` *ليس* فريدًا. في `aws-ct-search`، ينتج كل حدث مطابق صفًا واحدًا.

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

يُجري Suzaku نقطة تفتيش (checkpoint) لقاعدة البيانات قبل الخروج، لذا يكون ملف `.duckdb` مكتملًا ويمكن فتحه للقراءة فقط (انسخه بعد انتهاء الأمر، وليس أثناء تشغيله).
