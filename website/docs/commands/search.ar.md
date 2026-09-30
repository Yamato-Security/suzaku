# أوامر البحث

## أمر `aws-ct-search`

استخدم هذا الأمر لاستخراج الأحداث التي تهمك من سجلات AWS CloudTrail دون كتابة قاعدة Sigma.
تُعرض الأحداث المطابقة أو تُحفظ بنفس تنسيق `aws-ct-timeline`، لذا يمكنك الانتقال مباشرة من البحث إلى أعمدة الجدول الزمني التي تعرفها.

هناك ثلاث طرق لتضييق نطاق الأحداث، ويمكن الجمع بينها:

| الخيار | ما تتم مطابقته | قيم متعددة |
|---|---|---|
| `-F, --filter FIELD:VALUE` | قيمة حقل واحد، **مطابقة تامة، مع التمييز بين الأحرف الكبيرة والصغيرة** | كرّر `-F`؛ يجب أن تتطابق **جميع** المرشحات (AND) |
| `-k, --keyword KEYWORD` | سلسلة فرعية في أي مكان من JSON الحدث، دون تمييز بين الأحرف الكبيرة والصغيرة (مع التمييز عند `-c`) | كرّر `-k`؛ يكفي تطابق **أي** كلمة مفتاحية (OR) |
| `-r, --regex REGEX` | تعبير نمطي في أي مكان من JSON الحدث | نمط واحد (استخدم حرف الأنبوب غير المُهَرَّب <code>&#124;</code> للبدائل) |

يُفحص كل حدث بهذا الترتيب، ولا تُخرج إلا الأحداث التي تجتاز جميع الفحوص: النطاق الزمني (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

استخدم `-F` عندما تعرف الحقل الذي يحتوي على القيمة (مثل `eventName:ConsoleLogin`)، واستخدم `-k` أو `-r` عندما تعرف السلسلة فقط دون مكان ظهورها.

### كيف يعمل `-F, --filter`

* التنسيق هو `FIELD:VALUE`. يُقسَّم الوسيط عند أول `:`، لذا تعمل القيم التي تحتوي على نقطتين، مثل ARN، كما هي.
* تُكتب الحقول المتداخلة بصيغة النقاط (`userIdentity.type`). النقطة `.` في البداية اختيارية، لذا يعمل `.userIdentity.arn` أيضًا.
* تُزال علامات الاقتباس حول القيمة (`"..."` أو `'...'`).
* تُقارن الأرقام والقيم المنطقية كسلاسل نصية، لذا يعمل `-F readOnly:false` و`-F responseElements.user.userId:12345`.
* الحدث الذي لا يحتوي على الحقل لا يتطابق.
* لا يمكن الإشارة إلى العناصر داخل المصفوفات (لا توجد صيغة مثل `items.0.name`).
* يُرفض المرشح الذي لا يحتوي على `:` قبل بدء الفحص: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> ملاحظة: في `aws-ct-metrics` يعني `-F` شيئًا مختلفًا، فهو `--field-name` ويقبل اسم حقل فقط. أما في `aws-ct-search` فهو `--filter` ويأخذ دائمًا `FIELD:VALUE`.

## استخدام الأمر
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

### أمثلة على أمر `aws-ct-search`

التصفية حسب الحقل (`-F`):

* البحث عن عمليات تسجيل الدخول إلى وحدة التحكم: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* البحث عن استدعاءات API التي أجراها المستخدم root (حقل متداخل): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* البحث عن كل ما فعله مستخدم IAM معيّن (القيمة تحتوي على نقطتين): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* البحث عن الاستدعاءات المرفوضة: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* البحث عن استدعاءات IAM الكتابية (غير المقتصرة على القراءة) في `us-east-1` (تُجمع المرشحات المتعددة بـ AND): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* البحث عن الوصول إلى حاوية S3 معيّنة (حقل خاص بالـ API): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

البحث بالكلمة المفتاحية (`-k`) والتعبير النمطي (`-r`):

* البحث عن الأحداث التي تذكر عنوان IP في أي مكان: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* البحث عن الأحداث التي تذكر أيًّا من مستخدمَين (تُجمع الكلمات المفتاحية المتعددة بـ OR): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* بحث بالكلمة المفتاحية مع التمييز بين الأحرف الكبيرة والصغيرة: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* البحث عن محاولات تعطيل CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

الجمع بين الشروط وحفظ النتائج:

* بحث بالكلمة المفتاحية مقصور على مصدر أحداث واحد: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* عمليات تسجيل الدخول الفاشلة إلى وحدة التحكم خلال آخر 7 أيام: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* حفظ النتائج بصيغتي CSV وDuckDB مع معلومات GeoIP: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* حفظ JSON الأصلي للأحداث المطابقة: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### مخرجات `aws-ct-search`

أعمدة المخرجات هي نفسها في [`aws-ct-timeline`](dfir-timeline.md)، وتتبع مخرجات DuckDB نفس [مخطط إخراج DuckDB](dfir-timeline.md#duckdb-output-schema).
يُطبع في النهاية عدد الأحداث التي تم فحصها وعدد الأحداث المطابقة:

```
Total events scanned: 209
Matching events: 1
```
