# Tentang Suzaku

Suzaku (朱雀) berarti ["Burung Vermilion"](https://en.wikipedia.org/wiki/Vermilion_Bird) yaitu dewa yang terbang di atas awan dan menguasai langit selatan dalam [mitologi Tiongkok](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku adalah generator timeline untuk threat hunting dan forensik cepat bagi log cloud.
(Bayangkan [Hayabusa](https://github.com/Yamato-Security/hayabusa) tetapi untuk log cloud alih-alih log peristiwa Windows.)
Saat ini sedang dalam pengembangan aktif dengan dukungan deteksi [Sigma](https://github.com/SigmaHQ/sigma) native untuk log AWS CloudTrail.
Kami berencana untuk mendukung log Azure dan GCP juga.

Dengan log cloud, terdapat ribuan panggilan API yang berbeda dan lebih banyak peristiwa daripada yang dapat ditelusuri siapa pun secara manual.
Suzaku dirancang tidak hanya untuk menemukan serangan di tengah kebisingan, tetapi juga untuk menyediakan timeline DFIR yang hanya berisi peristiwa dan data yang Anda perlukan untuk melakukan investigasi forensik cepat.
Anda juga dapat membuat ringkasan untuk dengan cepat mengetahui apa yang terjadi pada tingkat tinggi, menemukan perilaku abnormal tanpa bergantung pada signature, dan dengan mudah menemukan kata kunci seperti alamat IP, user agent, region, geo-location, dll... untuk dijadikan titik pivot agar tidak melewatkan peristiwa apa pun yang dilakukan penyerang setelah Anda menemukannya.
