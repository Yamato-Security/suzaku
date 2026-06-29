# Über Suzaku

Suzaku (朱雀) bedeutet ["Zinnoberroter Vogel"](https://en.wikipedia.org/wiki/Vermilion_Bird), ein Gott, der über den Wolken fliegt und in der [chinesischen Mythologie](https://en.wikipedia.org/wiki/Four_Holy_Beasts) über die südlichen Himmel herrscht.

Suzaku ist ein Generator für Threat Hunting und schnelle forensische Zeitleisten für Cloud-Logs.
(Stellen Sie sich [Hayabusa](https://github.com/Yamato-Security/hayabusa) vor, aber für Cloud-Logs anstelle von Windows-Ereignisprotokollen.)
Es befindet sich derzeit in aktiver Entwicklung mit nativer [Sigma](https://github.com/SigmaHQ/sigma)-Erkennungsunterstützung für AWS CloudTrail-Logs.
Wir planen, auch Azure- und GCP-Logs zu unterstützen.

Bei Cloud-Logs gibt es Tausende verschiedener API-Aufrufe und mehr Ereignisse, als irgendjemand manuell durchsehen könnte.
Suzaku ist darauf ausgelegt, nicht nur die Angriffe im Rauschen zu finden, sondern Ihnen auch eine DFIR-Zeitleiste bereitzustellen, die nur die Ereignisse und Daten enthält, die Sie für eine schnelle forensische Untersuchung benötigen.
Sie können auch Zusammenfassungen erstellen, um schnell auf hohem Niveau herauszufinden, was passiert ist, abnormales Verhalten ohne Abhängigkeit von Signaturen zu entdecken und einfach Schlüsselwörter wie IP-Adressen, User-Agents, Regionen, Geolokalisierung usw. zu finden, um daran anzuknüpfen und keine Ereignisse zu verpassen, die ein Angreifer durchgeführt hat, nachdem Sie ihn entdeckt haben.
