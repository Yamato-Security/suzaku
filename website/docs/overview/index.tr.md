# Suzaku Hakkında

Suzaku (朱雀), [Çin mitolojisinde](https://en.wikipedia.org/wiki/Four_Holy_Beasts) güney göklerine hükmeden, bulutların üzerinde uçan bir tanrı olan ["Kızıl Kuş"](https://en.wikipedia.org/wiki/Vermilion_Bird) anlamına gelir.

Suzaku, bulut günlükleri için bir tehdit avcılığı ve hızlı adli bilişim zaman çizelgesi oluşturucusudur.
([Hayabusa](https://github.com/Yamato-Security/hayabusa) düşünün ama Windows olay günlükleri yerine bulut günlükleri için.)
Şu anda AWS CloudTrail günlükleri için yerel [Sigma](https://github.com/SigmaHQ/sigma) tespit desteğiyle aktif olarak geliştirilmektedir.
Azure ve GCP günlüklerini de desteklemeyi planlıyoruz.

Bulut günlüklerinde, binlerce farklı API çağrısı ve herhangi birinin manuel olarak elden geçirebileceğinden daha fazla olay vardır.
Suzaku, yalnızca gürültü arasındaki saldırıları bulmak için değil, aynı zamanda size hızlı adli bilişim incelemesi yapmak için ihtiyaç duyduğunuz yalnızca olayları ve verileri içeren bir DFIR zaman çizelgesi sağlamak için tasarlanmıştır.
Ayrıca yüksek düzeyde neler olduğunu hızlıca keşfetmek, imzalara dayanmadan anormal davranışları ortaya çıkarmak ve IP adresleri, kullanıcı aracıları, bölgeler, coğrafi konum vb. gibi anahtar kelimeleri kolayca bulup üzerinde pivot yapmak ve saldırganın siz onları keşfettikten sonra gerçekleştirdiği hiçbir olayı kaçırmamak için özetler oluşturabilirsiniz.
