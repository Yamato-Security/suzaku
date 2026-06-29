# Fonctionnalités

* Prise en charge multiplateforme : Windows, Linux, macOS.
* Développé en Rust pour être sûr en mémoire, rapide et autonome.
* Analyse des fichiers `.json` ou des fichiers compressés `.json.gz` avec des performances multithread.
* Création de chronologies uniques et faciles à analyser pour les investigations forensiques et la réponse aux incidents.
* Excellente prise en charge native des signatures d'IoC écrites dans des règles [Sigma](https://github.com/SigmaHQ/sigma) basées sur YML, faciles à lire/créer/éditer. (Les règles de corrélation et tous les modificateurs de champ, à l'exception de [expand](https://sigmahq.io/docs/basics/modifiers.html#expand), sont pris en charge.)
* Création d'un résumé de toute l'utilisation des API, de métriques sur l'attaquant (adresses IP source, géolocalisation, régions utilisées, agents utilisateurs, etc.) afin de découvrir une activité anormale sans dépendre de signatures.
* Enregistrement des résultats au format CSV, JSON et JSONL.
