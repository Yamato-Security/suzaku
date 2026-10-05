# Команди DFIR Timeline

## Команда `aws-ct-timeline`

Створення DFIR-таймлайну AWS CloudTrail на основі правил Sigma в теці `rules`.

## Використання команди
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

### Приклади команди `aws-ct-timeline`

* Вивід сповіщень на екран: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Збереження результатів у файл CSV: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Збереження результатів у файли CSV та JSONL: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### Профіль виводу команди `aws-ct-timeline`

Suzaku виводить інформацію на основі файлу `config/aws_profile.yaml`:
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

* Будь-яке значення поля, що починається з `.` (наприклад, `.eventTime`), буде взято з логу CloudTrail.
* Будь-яке значення поля, що починається з `sigma.` (наприклад, `sigma.title`), буде взято з правила Sigma.
* Наразі ми підтримуємо лише рядки, але плануємо підтримку інших типів значень полів.

> Примітка: Якщо ви хочете вивести оригінальні дані JSON і переконатися, що не втратите жодної інформації про поля, просто додайте опцію `-R, --raw-output` до команди `aws-ct-timeline`.

### Схема виводу DuckDB {#duckdb-output-schema}

Вивід CSV і JSON — це *відображення* наведеного вище профілю; вивід DuckDB — це *інтерфейс даних*, тому він типізований і самоописовий. Ці відмінності навмисні й стосуються `aws-ct-timeline`, `azure-timeline`, `gws-timeline` та `aws-ct-search`:

| | CSV / JSON | DuckDB |
|---|---|---|
| Відсутнє значення | `-` (або порожньо) | `NULL` |
| `Timestamp` | відформатований текст | `TIMESTAMP` |
| `Level` | текст | `suzaku_level` (`ENUM`, упорядкований за рівнем серйозності) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (у SQL не потрібні лапки) |
| `Tags` | один рядок, об'єднаний через ` ¦ ` | `Tactics`, `TechniqueIDs`, `OtherTags` (кожен типу `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | додаються лише з `-G, --geo-ip` | присутні завжди (якщо профіль містить `SrcIP`), `NULL`, якщо `-G` не використовувався |
| Дублікати рядків | зберігаються | точні дублікати видаляються, їх кількість вказується в `suzaku_meta` |

Кожен файл також містить однорядкову таблицю `suzaku_meta`, щоб без здогадок можна було визначити, що його створило:

| Стовпець | Значення |
|---|---|
| `schema_version` | Версія структури. Перевірте її, перш ніж читати інші таблиці. |
| `suzaku_version`, `command`, `command_line` | Яка версія Suzaku, яка підкоманда, який саме виклик. |
| `generated_at` | Коли файл було записано. |
| `timestamp_tz` | Часовий пояс стовпця `Timestamp` — `UTC` або локальне зміщення з `-l, --localtime`. |
| `rules_version`, `rules_count` | Ревізія набору правил (якщо тека rules є git-checkout) і кількість завантажених правил. |
| `geoip_enabled` | Чи запускався `-G, --geo-ip`. Дозволяє відрізнити повністю `NULL` стовпець `SrcCountry` («збагачення було вимкнено») від `NULL`-клітинки у збагаченому файлі («це значення не є IP-адресою»). |
| `scanned_files`, `scanned_events` | Охоплення запуску. |
| `output_rows`, `duplicate_rows_removed` | Записані рядки та точні дублікати, видалені під час запису. |

Для команд часової шкали на основі правил рядок `timeline` — це **одна подія × один збіг правила**: подія, що збігається з кількома правилами, дає по рядку на кожен збіг, тому `EventID` *не* є унікальним. У `aws-ct-search` кожна відповідна подія дає один рядок.

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

Перед завершенням Suzaku виконує checkpoint бази даних, тому файл `.duckdb` повний і його можна відкрити лише для читання (копіюйте його після завершення команди, а не під час її виконання).
