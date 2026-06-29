# Características

* Soporte multiplataforma: Windows, Linux, macOS.
* Desarrollado en Rust para ser seguro en memoria, rápido y autónomo.
* Escanea archivos `.json` o comprimidos `.json.gz` con rendimiento multihilo.
* Crea cronologías únicas y fáciles de analizar para investigaciones forenses y respuesta a incidentes.
* Excelente soporte nativo para firmas de IoC escritas en reglas [Sigma](https://github.com/SigmaHQ/sigma) basadas en YML fáciles de leer/crear/editar. (Se admiten las reglas de correlación y todos los modificadores de campo excepto [expand](https://sigmahq.io/docs/basics/modifiers.html#expand).)
* Crea un resumen de todo el uso de la API, métricas sobre el atacante (direcciones IP de origen, geolocalización, regiones utilizadas, agentes de usuario, etc...) para descubrir actividad anómala sin depender de firmas.
* Guarda los resultados en CSV, JSON y JSONL.
