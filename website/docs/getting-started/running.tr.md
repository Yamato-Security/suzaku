# Suzaku'yu Çalıştırma

## Windows

Bir Komut/PowerShell İstemi veya Windows Terminal içinde, uygun 32 bit veya 64 bit Windows ikili dosyasını çalıştırmanız yeterlidir.

### Yolunda boşluk bulunan bir dosya veya dizini taramaya çalışırken oluşan hata

Windows'taki yerleşik Komut veya PowerShell istemini kullanırken, dosya veya dizin yolunuzda bir boşluk varsa Suzaku'nun hiçbir dosya yükleyemediğine dair bir hata alabilirsiniz.
Günlük dosyalarını düzgün şekilde yüklemek için aşağıdakileri yaptığınızdan emin olun:
1. Dosya veya dizin yolunu çift tırnak içine alın.
2. Eğer bir dizin yoluysa, son karakter olarak bir ters eğik çizgi eklemediğinizden emin olun.

### Karakterlerin doğru görüntülenmemesi

Windows'taki varsayılan `Lucida Console` yazı tipiyle, logoda ve tablolarda kullanılan çeşitli karakterler düzgün görüntülenmeyecektir.
Bunu düzeltmek için yazı tipini `Consalas` olarak değiştirmelisiniz.

## Linux

Önce ikili dosyayı çalıştırılabilir hale getirmeniz gerekir.

```bash
chmod +x ./suzaku
```

Ardından Suzaku kök dizininden çalıştırın:

```bash
./suzaku
```

## macOS

Terminal'den veya [iTerm2](https://iterm2.com/) üzerinden, önce ikili dosyayı çalıştırılabilir hale getirmeniz gerekir.

```bash
chmod +x ./suzaku
```

Ardından, Suzaku kök dizininden çalıştırmayı deneyin:

```bash
./suzaku
```

macOS'un en son sürümünde, çalıştırmaya çalıştığınızda bir güvenlik hatası alabilirsiniz.
"Cancel" (İptal) düğmesine tıklayın ve ardından Sistem Tercihleri'nden "Security & Privacy" (Güvenlik ve Gizlilik) bölümünü açın, General (Genel) sekmesinden "Allow Anyway" (Yine de İzin Ver) düğmesine tıklayın.
Bundan sonra tekrar çalıştırmayı deneyin.

```bash
./suzaku
```

Bir uyarı belirecektir, sadece "Open" (Aç) düğmesine tıklayın.
Artık suzaku'yu çalıştırabilmeniz gerekir.
