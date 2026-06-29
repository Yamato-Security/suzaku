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

Suzaku виводить інформацію на основі файлу `config/default_profile.yaml`:
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

* Будь-яке значення поля, що починається з `.` (наприклад, `.eventTime`), буде взято з логу CloudTrail.
* Будь-яке значення поля, що починається з `sigma.` (наприклад, `sigma.title`), буде взято з правила Sigma.
* Наразі ми підтримуємо лише рядки, але плануємо підтримку інших типів значень полів.

> Примітка: Якщо ви хочете вивести оригінальні дані JSON і переконатися, що не втратите жодної інформації про поля, просто додайте опцію `-R, --raw-output` до команди `aws-ct-timeline`.
