# Команди пошуку

## Команда `aws-ct-search`

Використовуйте цю команду, щоб отримати потрібні події з журналів AWS CloudTrail без написання правила Sigma.
Знайдені події виводяться або зберігаються в тому ж форматі, що й у `aws-ct-timeline`, тож можна одразу перейти від пошуку до знайомих стовпців хронології.

Є три способи звузити коло подій, і їх можна поєднувати:

| Параметр | З чим порівнюється | Кілька значень |
|---|---|---|
| `-F, --filter FIELD:VALUE` | Значення одного поля, **точний збіг із урахуванням регістру** | Повторіть `-F`; мають збігтися **всі** фільтри (І) |
| `-k, --keyword KEYWORD` | Підрядок будь-де в JSON події без урахування регістру (з урахуванням — з `-c`) | Повторіть `-k`; достатньо збігу з **будь-яким** ключовим словом (АБО) |
| `-r, --regex REGEX` | Регулярний вираз будь-де в JSON події | Один шаблон (для альтернатив використовуйте неекранований символ вертикальної риски <code>&#124;</code>) |

Кожна подія перевіряється в такому порядку, і виводяться лише ті, що пройшли всі перевірки: часовий діапазон (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`.

Використовуйте `-F`, якщо знаєте, у якому полі міститься значення (наприклад `eventName:ConsoleLogin`), і `-k` або `-r`, якщо знаєте лише рядок, але не знаєте, де він трапляється.

### Як працює `-F, --filter`

* Формат — `FIELD:VALUE`. Аргумент розділяється за **першим** `:`, тому значення з двокрапками, як-от ARN, працюють без змін.
* Вкладені поля записуються через крапку (`userIdentity.type`). Початкова `.` необов'язкова, тому `.userIdentity.arn` теж працює.
* Лапки навколо значення (`"..."` або `'...'`) видаляються.
* Числа й булеві значення порівнюються як рядки, тому `-F readOnly:false` і `-F responseElements.user.userId:12345` працюють.
* Подія, що не має такого поля, не збігається.
* Звернутися до елементів масиву не можна (синтаксису на кшталт `items.0.name` немає).
* Фільтр без `:` відхиляється ще до початку сканування: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`.

> Примітка: у `aws-ct-metrics` `-F` означає інше — це `--field-name`, і він приймає лише назву поля. В `aws-ct-search` це `--filter`, і він завжди приймає `FIELD:VALUE`.

## Використання команди
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

### Приклади команди `aws-ct-search`

Фільтрування за полем (`-F`):

* Знайти входи в консоль: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* Знайти виклики API від користувача root (вкладене поле): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* Знайти всі дії певного користувача IAM (значення містить двокрапки): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* Знайти відхилені виклики: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* Знайти виклики IAM на запис (не лише для читання) в `us-east-1` (кілька фільтрів поєднуються через І): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* Знайти доступ до певного бакета S3 (поле, специфічне для API): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

Пошук за ключовим словом (`-k`) і регулярним виразом (`-r`):

* Знайти події, що будь-де згадують IP-адресу: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* Знайти події, що згадують одного з двох користувачів (кілька ключових слів поєднуються через АБО): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* Пошук за ключовим словом з урахуванням регістру: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* Знайти спроби вимкнути CloudTrail: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

Поєднання умов і збереження результатів:

* Пошук за ключовим словом, обмежений одним джерелом подій: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* Невдалі входи в консоль за останні 7 днів: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* Зберегти результати в CSV і DuckDB з інформацією GeoIP: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* Зберегти оригінальний JSON знайдених подій: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### Вивід `aws-ct-search`

Стовпці виводу такі самі, як у [`aws-ct-timeline`](dfir-timeline.md), а вивід DuckDB відповідає тій самій [схемі виводу DuckDB](dfir-timeline.md#duckdb-output-schema).
Наприкінці виводиться кількість просканованих і знайдених подій:

```
Total events scanned: 209
Matching events: 1
```
