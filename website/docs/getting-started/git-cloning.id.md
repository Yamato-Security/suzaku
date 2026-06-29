# Git Cloning

Anda dapat melakukan `git clone` pada repositori dengan perintah berikut dan mengompilasi biner dari kode sumber:

**Peringatan:** Branch main dari repositori ini ditujukan untuk keperluan pengembangan sehingga Anda mungkin dapat mengakses fitur-fitur baru yang belum dirilis secara resmi, namun, mungkin terdapat bug sehingga anggaplah ini tidak stabil.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Catatan:** Jika Anda lupa menggunakan opsi `--recursive`, folder `rules`, yang dikelola sebagai git submodule, tidak akan di-clone.

Anda dapat menyinkronkan folder `rules` dan mendapatkan aturan Suzaku terbaru dengan `git pull --recurse-submodules` atau gunakan perintah berikut:

```bash
./suzaku update-rules
```

Jika pembaruan gagal, Anda mungkin perlu mengganti nama folder `rules` dan mencoba lagi.

>> Perhatian: Saat memperbarui, aturan dan file konfigurasi di folder `rules` akan diganti dengan aturan dan file konfigurasi terbaru di repositori [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules).
>> Perubahan apa pun yang Anda buat pada file yang sudah ada akan ditimpa, jadi kami menyarankan Anda membuat cadangan dari file apa pun yang Anda edit sebelum memperbarui.
>> Jika Anda menambahkan aturan **baru** di dalam folder `rules`, aturan tersebut **tidak** akan ditimpa atau dihapus saat memperbarui.
