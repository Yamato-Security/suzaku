# Читання CSV-завантажень із консолі AWS

Команди AWS у Suzaku (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) можуть через `-f/--file` або `-d/--directory` читати файл, завантажений у AWS Management Console за шляхом **CloudTrail → Event history → Download events → CSV**.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

CSV зручний, коли немає Trail, що доставляє журнали до S3, однак це лише **короткий опис** кожної події, а не повний запис CloudTrail. Якщо доступні JSON-журнали Trail або JSON-завантаження Event history, використовуйте їх.

## Перетворення

Кожен рядок перебудовується як запис CloudTrail: `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]`, а `Read-only` → логічне `readOnly`. Порожня клітинка стає відсутнім полем, тому правила з `errorCode|exists` правильно розрізняють успішні та невдалі виклики. Для `root` також додається `userIdentity.type: Root`.

Розпізнаються лише англійські назви стовпців. CSV, який не є консольним завантаженням CloudTrail, відхиляється. Рядки з неправильною кількістю полів (зазвичай через пошкоджені лапки) або без `Event time` пропускаються й записуються до журналу помилок із номером рядка.

## Обмеження

1. **Завантажте всі потрібні регіони, включно з `us-east-1`.** Консоль експортує лише один регіон за раз; входи до консолі та події IAM записуються в `us-east-1`.
2. **Деякі правила не можуть спрацювати.** У CSV немає `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` чи `additionalEventData`. Приблизно 22 з 796 правил CloudTrail, зокрема для невдалих входів і входів без MFA, не можуть спрацювати.
3. **Деякі стовпці хронології порожні.** `UserARN`, `UserAccountID`, `UserPrincipalID` та `ErrorMessage` завжди порожні; `UserType` також, крім root.
4. **`aws-ct-summary` групує за ім'ям користувача.** Для ролі це ім'я сеансу, тому різні ролі з однаковим ім'ям об'єднуються. Рядки без імені користувача не входять до підсумку, тому кількість може відрізнятися від JSON за той самий період.
5. **Google Workspace не підтримує CSV.** Для `gws-timeline` використовуйте JSON/JSONL/JSON.GZ із Reports API. CSV з Admin console має іншу схему з втратою даних.
