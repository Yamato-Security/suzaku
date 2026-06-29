# Menjalankan Suzaku

## Windows

Di Command/PowerShell Prompt atau Windows Terminal, cukup jalankan biner Windows 32-bit atau 64-bit yang sesuai.

### Error saat mencoba memindai berkas atau direktori dengan spasi pada path-nya

Saat menggunakan Command atau PowerShell prompt bawaan di Windows, Anda mungkin menerima error bahwa Suzaku tidak dapat memuat berkas apa pun jika ada spasi pada path berkas atau direktori Anda.
Agar dapat memuat berkas log dengan benar, pastikan untuk melakukan hal berikut:
1. Apit path berkas atau direktori dengan tanda kutip ganda.
2. Jika berupa path direktori, pastikan Anda tidak menyertakan backslash sebagai karakter terakhir.

### Karakter tidak ditampilkan dengan benar

Dengan font bawaan `Lucida Console` di Windows, berbagai karakter yang digunakan pada logo dan tabel tidak akan ditampilkan dengan benar.
Anda sebaiknya mengganti font menjadi `Consalas` untuk memperbaiki hal ini.

## Linux

Anda terlebih dahulu perlu membuat biner tersebut dapat dieksekusi.

```bash
chmod +x ./suzaku
```

Kemudian jalankan dari direktori root Suzaku:

```bash
./suzaku
```

## macOS

Dari Terminal atau [iTerm2](https://iterm2.com/), Anda terlebih dahulu perlu membuat biner tersebut dapat dieksekusi.

```bash
chmod +x ./suzaku
```

Kemudian, coba jalankan dari direktori root Suzaku:

```bash
./suzaku
```

Pada versi macOS terbaru, Anda mungkin menerima error keamanan saat mencoba menjalankannya.
Klik "Cancel" lalu dari System Preferences, buka "Security & Privacy" dan dari tab General, klik "Allow Anyway".
Setelah itu, coba jalankan kembali.

```bash
./suzaku
```

Sebuah peringatan akan muncul, jadi cukup klik "Open".
Sekarang Anda seharusnya dapat menjalankan suzaku.
