# À propos de Suzaku

Suzaku (朱雀) signifie ["Oiseau vermillon"](https://en.wikipedia.org/wiki/Vermilion_Bird), un dieu qui vole au-dessus des nuages et règne sur les cieux du sud dans la [mythologie chinoise](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku est un générateur de chronologie pour la chasse aux menaces et l'analyse forensique rapide des journaux cloud.
(Imaginez [Hayabusa](https://github.com/Yamato-Security/hayabusa), mais pour les journaux cloud au lieu des journaux d'événements Windows.)
Il est actuellement en cours de développement actif avec une prise en charge native de la détection [Sigma](https://github.com/SigmaHQ/sigma) pour les journaux AWS CloudTrail.
Nous prévoyons de prendre également en charge les journaux Azure et GCP.

Avec les journaux cloud, il existe des milliers d'appels d'API différents et plus d'événements que quiconque ne pourrait passer en revue manuellement.
Suzaku est conçu non seulement pour repérer les attaques au milieu du bruit, mais aussi pour vous fournir une chronologie DFIR ne contenant que les événements et les données dont vous avez besoin pour mener une investigation forensique rapide.
Vous pouvez également créer des résumés afin de découvrir rapidement ce qui s'est passé à un haut niveau, détecter des comportements anormaux sans recourir à des signatures et trouver facilement des mots-clés tels que des adresses IP, des agents utilisateurs, des régions, des géolocalisations, etc. pour pivoter et ne manquer aucun des événements qu'un attaquant a réalisés après que vous les avez découverts.
