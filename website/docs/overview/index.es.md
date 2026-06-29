# Acerca de Suzaku

Suzaku (朱雀) significa ["Pájaro Bermellón"](https://en.wikipedia.org/wiki/Vermilion_Bird) que es un dios que vuela sobre las nubes gobernando los cielos del sur en la [mitología china](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku es un generador de líneas de tiempo de caza de amenazas y análisis forense rápido para registros en la nube.
(Imagina [Hayabusa](https://github.com/Yamato-Security/hayabusa) pero para registros en la nube en lugar de registros de eventos de Windows.)
Actualmente está en desarrollo activo con soporte nativo de detección [Sigma](https://github.com/SigmaHQ/sigma) para registros de AWS CloudTrail.
Planeamos dar soporte también a los registros de Azure y GCP.

Con los registros en la nube, hay miles de llamadas a la API diferentes y más eventos de los que cualquiera podría examinar manualmente.
Suzaku está diseñado no solo para encontrar los ataques en medio del ruido, sino también para proporcionarte una línea de tiempo DFIR que contiene únicamente los eventos y datos que necesitas para realizar una investigación forense rápida.
También puedes crear resúmenes con el fin de descubrir rápidamente qué ocurrió a alto nivel, detectar comportamientos anómalos sin depender de firmas y encontrar fácilmente palabras clave como direcciones IP, agentes de usuario, regiones, geolocalización, etc... sobre las cuales pivotar y no perder ningún evento que un atacante haya realizado después de descubrirlos.
