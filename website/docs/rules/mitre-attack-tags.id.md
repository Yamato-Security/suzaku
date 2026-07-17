# MITRE ATT&CK Tag

Aturan Sigma dapat memuat field [`tags`](https://github.com/SigmaHQ/sigma-specification/blob/main/specification/sigma-rules-specification.md#tags) yang mengklasifikasikan suatu deteksi terhadap framework [MITRE ATT&CK®](https://attack.mitre.org/) (taktik, teknik, dan grup) serta taksonomi lainnya. Karena `tags` merupakan sebuah list, perintah `aws-ct-timeline` dan `azure-timeline` menampilkannya dalam satu kolom **`Tags`**, menggabungkan setiap entri dengan ` ¦ ` (pemisah yang sama dengan yang digunakan Hayabusa) dan menyingkat setiap entri agar kolom tetap ringkas.

## Contoh

Sebuah aturan yang diberi tag seperti ini:

```yaml
tags:
    - attack.g0035
    - attack.credential-access
    - attack.discovery
    - attack.t1110
    - attack.t1087
```

akan ditampilkan pada kolom `Tags` seperti berikut:

```
G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087
```

Pada output JSON/JSONL, nilainya tetap dipertahankan sebagai string datar yang sama (**tidak** diuraikan menjadi sebuah array), sehingga kolomnya identik pada CSV, JSON, dan terminal.

## Bagaimana setiap tag disingkat

| Jenis tag | Contoh input | Keluaran | Aturan |
| --- | --- | --- | --- |
| Taktik | `attack.credential-access` | `CredAccess` | Dicari dalam `config/mitre_tactics.txt` (lihat di bawah) |
| Teknik | `attack.t1562.001` | `T1562.001` | Prefiks `attack.t` diubah menjadi huruf kapital `T`; nomor teknik/sub-teknik dipertahankan apa adanya |
| Grup | `attack.g0035` | `G0035` | Prefiks `attack.g` diubah menjadi huruf kapital `G`; nomor grup dipertahankan apa adanya |
| Lainnya | `cve.2021.1234` | `cve.2021.1234` | Tidak diubah |

Tag dicocokkan tanpa membedakan huruf besar/kecil, dan penulisan dengan tanda hubung maupun garis bawah diperlakukan sama, sehingga `attack.credential-access` dan `attack.credential_access` sama-sama menjadi `CredAccess`.

## Tabel singkatan taktik

Singkatan taktik **tidak** ditulis secara hard-coded — melainkan dibaca saat runtime dari `config/mitre_tactics.txt`, tabel yang sama dengan yang digunakan [Hayabusa](https://github.com/Yamato-Security/hayabusa). Setiap baris merupakan pasangan sederhana `<tag lengkap>,<singkatan>`, sehingga Anda dapat menyunting atau menambah singkatan tanpa perlu membangun ulang Suzaku:

| Tag lengkap | Singkatan |
| --- | --- |
| `attack.reconnaissance` | Recon |
| `attack.resource-development` | ResDev |
| `attack.initial-access` | InitAccess |
| `attack.execution` | Exec |
| `attack.persistence` | Persis |
| `attack.privilege-escalation` | PrivEsc |
| `attack.stealth` | Stealth |
| `attack.defense-evasion` | Stealth |
| `attack.defense-impairment` | DefImpair |
| `attack.credential-access` | CredAccess |
| `attack.discovery` | Disc |
| `attack.lateral-movement` | LatMov |
| `attack.collection` | Collect |
| `attack.command-and-control` | C2 |
| `attack.exfiltration` | Exfil |
| `attack.impact` | Impact |

> Catatan: Jika `config/mitre_tactics.txt` tidak ada, tag taktik akan diteruskan tanpa perubahan; singkatan teknik dan grup tetap berfungsi.

## Catatan tentang "Stealth" vs "Defense Evasion"

Sejak [MITRE ATT&CK v19 (April 2026)](https://attack.mitre.org/resources/updates/updates-april-2026/), taktik **Defense Evasion** (`TA0005`) diganti namanya menjadi **Stealth**, dan sebuah taktik terpisah **Impair Defenses** (`TA0112`) dipisahkan darinya. Suzaku mengikuti penamaan baru ini:

- `attack.stealth` dan `attack.defense-evasion` versi lama sama-sama disingkat menjadi **`Stealth`**, sehingga aturan lama yang masih menggunakan tag `attack.defense-evasion` ditampilkan dengan nama taktik saat ini.
- `attack.defense-impairment` disingkat menjadi **`DefImpair`**.

Jika Anda lebih menyukai label lama, ubah baris `attack.defense-evasion` pada `config/mitre_tactics.txt` (misalnya kembali ke `Evas`).
