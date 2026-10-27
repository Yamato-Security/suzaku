# Funktionen

* Plattformübergreifende Unterstützung: Windows, Linux, macOS.
* In Rust entwickelt, um speichersicher, schnell und eigenständig zu sein.
* Analyse von AWS CloudTrail-, Azure/Entra ID/M365- und Google Workspace Reports API-Protokollen mit nativer Sigma-Erkennung.
* Multithread-Scan von `.json`, `.jsonl`, komprimiertem `.json.gz`, Parquet und unterstützten [AWS-Management-Console-CloudTrail-CSV-Dateien](../getting-started/console-csv-input.md).
* Erstellen einzelner, einfach zu analysierender Zeitleisten für forensische Untersuchungen und die Reaktion auf Sicherheitsvorfälle.
* Hervorragende native Unterstützung für IoC-Signaturen, die in leicht lesbaren/erstellbaren/bearbeitbaren YML-basierten [Sigma](https://github.com/SigmaHQ/sigma)-Regeln geschrieben sind. (Korrelationsregeln und alle Feld-Modifikatoren außer [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) werden unterstützt.)
* Erstellen einer Zusammenfassung der gesamten API-Nutzung sowie Metriken über den Angreifer (Quell-IP-Adressen, geografische Lage, verwendete Regionen, User-Agents usw.), um abnormale Aktivitäten ohne Abhängigkeit von Signaturen zu erkennen.
* Speichern der Ergebnisse als CSV, JSON, JSONL und DuckDB.
