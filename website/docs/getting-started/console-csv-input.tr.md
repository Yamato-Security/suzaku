# AWS Konsolu CSV İndirmelerini Okuma

Suzaku'nun AWS komutları (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`), AWS Management Console'daki **CloudTrail → Event history → Download events → CSV** yolundan indirilen dosyayı `-f/--file` veya `-d/--directory` ile okuyabilir.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

Bu CSV, günlükleri S3'e ileten bir Trail olmadığında kullanışlıdır; ancak tam CloudTrail kaydı değil, her olayın bir **özetidir**. Varsa Trail JSON günlüklerini veya Event history JSON indirmesini tercih edin.

## Dönüştürme biçimi

Her satır bir CloudTrail kaydı olarak yeniden oluşturulur: `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]` ve `Read-only` → boolean `readOnly`. Boş hücreler eksik alan olarak değerlendirilir; böylece `errorCode|exists` kullanan kurallar başarılı ve başarısız çağrıları doğru ayırır. `root` için ayrıca `userIdentity.type: Root` eklenir.

Yalnızca İngilizce sütun adları tanınır. CloudTrail konsol indirme biçiminde olmayan CSV dosyaları reddedilir. Yanlış alan sayısına sahip (genellikle bozuk tırnak işaretleri) veya `Event time` içermeyen satırlar atlanır ve satır numarasıyla hata günlüğüne yazılır.

## Sınırlamalar

1. **Gereken her bölgeyi, `us-east-1` dahil, indirin.** Konsol her seferinde tek bir bölge indirir; konsol oturum açma ve IAM olayları `us-east-1` içinde kaydedilir.
2. **Bazı kurallar eşleşemez.** CSV'de `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` veya `additionalEventData` bulunmaz. Yaklaşık 796 CloudTrail kuralının 22'si, başarısız girişler ve MFA'sız girişler dahil, eşleşemez.
3. **Bazı zaman çizelgesi sütunları boş kalır.** `UserARN`, `UserAccountID`, `UserPrincipalID` ve `ErrorMessage` her zaman boştur; root dışında `UserType` da boştur.
4. **`aws-ct-summary` kullanıcı adına göre gruplar.** Bir rol için bu değer oturum adıdır; aynı oturum adını kullanan farklı roller birleştirilir. Kullanıcı adı olmayan satırlar özete girmez; bu nedenle sayılar aynı dönemin JSON dışa aktarımından farklı olabilir.
5. **Google Workspace CSV kabul etmez.** `gws-timeline` için Reports API JSON/JSONL/JSON.GZ kullanın. Admin console CSV, bilgi kaybına uğratan farklı bir şemadır.
