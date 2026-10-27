# Membaca Unduhan CSV Konsol AWS

Perintah AWS Suzaku (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) dapat membaca dengan `-f/--file` atau `-d/--directory` berkas yang diunduh dari **CloudTrail → Event history → Download events → CSV** di AWS Management Console.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

CSV ini berguna saat tidak ada Trail yang mengirim log ke S3, tetapi isinya hanya **ringkasan** setiap peristiwa, bukan catatan CloudTrail lengkap. Gunakan log JSON dari Trail atau unduhan JSON Event history jika tersedia.

## Konversi

Setiap baris dibangun kembali sebagai catatan CloudTrail: `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]`, dan `Read-only` → boolean `readOnly`. Sel kosong menjadi field yang tidak ada sehingga aturan `errorCode|exists` dapat membedakan panggilan berhasil dan gagal. Untuk `root`, Suzaku juga menambahkan `userIdentity.type: Root`.

Hanya nama kolom bahasa Inggris yang dikenali. CSV yang bukan unduhan CloudTrail dari konsol akan ditolak. Baris dengan jumlah field yang salah (biasanya karena tanda kutip rusak) atau tanpa `Event time` dilewati dan dicatat bersama nomor barisnya dalam log kesalahan.

## Keterbatasan

1. **Unduh setiap region yang diperlukan, termasuk `us-east-1`.** Konsol mengekspor satu region setiap kali; login konsol dan peristiwa IAM dicatat di `us-east-1`.
2. **Sebagian aturan tidak dapat cocok.** CSV tidak memiliki `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements`, atau `additionalEventData`. Sekitar 22 dari 796 aturan CloudTrail, termasuk login gagal dan login tanpa MFA, tidak dapat cocok.
3. **Sebagian kolom linimasa kosong.** `UserARN`, `UserAccountID`, `UserPrincipalID`, dan `ErrorMessage` selalu kosong; `UserType` juga kosong kecuali untuk root.
4. **`aws-ct-summary` mengelompokkan berdasarkan nama pengguna.** Untuk role, nilainya adalah nama sesi; role berbeda dengan nama sesi yang sama akan digabung. Baris tanpa nama pengguna tidak masuk ringkasan, sehingga hitungan dapat berbeda dari JSON untuk periode yang sama.
5. **Google Workspace tidak menerima CSV.** Gunakan JSON/JSONL/JSON.GZ Reports API dengan `gws-timeline`. CSV Admin console memakai skema lain yang kehilangan informasi.
