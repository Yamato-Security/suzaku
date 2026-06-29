# Fitur

* Dukungan lintas platform: Windows, Linux, macOS.
* Dikembangkan dalam Rust agar aman dari segi memori, cepat, dan mandiri.
* Memindai file `.json` atau file terkompresi `.json.gz` dengan performa multi-thread.
* Membuat timeline tunggal yang mudah dianalisis untuk investigasi forensik dan respons insiden.
* Dukungan native yang sangat baik untuk signature IoC yang ditulis dalam aturan [Sigma](https://github.com/SigmaHQ/sigma) berbasis YML yang mudah dibaca/dibuat/diedit. (Aturan korelasi dan semua field modifier kecuali [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) didukung.)
* Membuat ringkasan dari semua penggunaan API, metrik tentang penyerang (alamat IP sumber, geo-lokasi, region yang digunakan, user agent, dll...) untuk menemukan aktivitas abnormal tanpa bergantung pada signature.
* Menyimpan hasil ke CSV, JSON, dan JSONL.
