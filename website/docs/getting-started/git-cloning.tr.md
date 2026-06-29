# Git ile Klonlama

Depoyu aşağıdaki komutla `git clone` yapabilir ve ikili dosyayı kaynak koddan derleyebilirsiniz:

**Uyarı:** Deponun ana dalı geliştirme amaçlıdır, bu nedenle henüz resmi olarak yayımlanmamış yeni özelliklere erişebilirsiniz, ancak hatalar olabileceğinden kararsız olarak değerlendirin.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Not:** `--recursive` seçeneğini kullanmayı unutursanız, git alt modülü olarak yönetilen `rules` klasörü klonlanmayacaktır.

`rules` klasörünü senkronize edebilir ve en son Suzaku kurallarını `git pull --recurse-submodules` ile alabilir veya aşağıdaki komutu kullanabilirsiniz:

```bash
./suzaku update-rules
```

Güncelleme başarısız olursa, `rules` klasörünü yeniden adlandırıp tekrar denemeniz gerekebilir.

>> Dikkat: Güncelleme yapılırken, `rules` klasöründeki kurallar ve yapılandırma dosyaları, [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) deposundaki en son kurallar ve yapılandırma dosyalarıyla değiştirilir.
>> Mevcut dosyalarda yaptığınız değişiklikler üzerine yazılacağından, güncelleme yapmadan önce düzenlediğiniz dosyaların yedeklerini almanızı öneririz.
>> `rules` klasörünün içine **yeni** kurallar eklerseniz, güncelleme sırasında bunların üzerine **yazılmaz** veya silinmez.
