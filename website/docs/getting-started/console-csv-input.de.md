# AWS-Console-CSV-Downloads lesen

Die AWS-Befehle von Suzaku (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) lesen mit `-f/--file` oder `-d/--directory` die CSV-Datei aus **CloudTrail → Event history → Download events → CSV** in der AWS Management Console.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

Die CSV ist eine praktische Alternative, wenn kein Trail Protokolle nach S3 schreibt. Sie enthält jedoch nur eine **Zusammenfassung** jedes Ereignisses und nicht den vollständigen CloudTrail-Datensatz. Verwenden Sie nach Möglichkeit die JSON-Protokolle eines Trails oder den JSON-Download aus Event history.

## Konvertierung

Jede Zeile wird als CloudTrail-Datensatz aufgebaut, z. B. `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]` und `Read-only` → boolesches `readOnly`. Leere Zellen werden als nicht vorhandene Felder behandelt, damit Regeln mit `errorCode|exists` Erfolg und Fehler korrekt unterscheiden. Für `root` wird außerdem `userIdentity.type: Root` gesetzt.

Nur englische Spaltennamen werden erkannt. CSV-Dateien, die nicht dem CloudTrail-Konsolenformat entsprechen, werden abgelehnt. Zeilen mit einer falschen Feldanzahl (meist fehlerhafte Anführungszeichen) oder ohne `Event time` werden übersprungen und mit Zeilennummer im Fehlerprotokoll erfasst.

## Einschränkungen

1. **Laden Sie jede benötigte Region einschließlich `us-east-1` herunter.** Die Konsole exportiert jeweils nur eine Region; Konsolenanmeldungen und IAM-Ereignisse werden in `us-east-1` protokolliert.
2. **Einige Regeln können nicht treffen.** Die CSV enthält kein `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` oder `additionalEventData`. Etwa 22 von 796 CloudTrail-Regeln, darunter fehlgeschlagene Anmeldungen und Anmeldungen ohne MFA, können deshalb nicht treffen.
3. **Einige Timeline-Spalten bleiben leer.** `UserARN`, `UserAccountID`, `UserPrincipalID` und `ErrorMessage` sind immer leer; `UserType` ebenfalls, außer bei root.
4. **`aws-ct-summary` gruppiert nach Benutzername.** Bei Rollen ist dies der Sitzungsname; verschiedene Rollen mit demselben Sitzungsnamen werden zusammengefasst. Zeilen ohne Benutzername fehlen in der Zusammenfassung, sodass die Anzahl von einem JSON-Export desselben Zeitraums abweichen kann.
5. **Google Workspace unterstützt kein CSV.** Für `gws-timeline` sind Reports-API-Dateien im Format JSON/JSONL/JSON.GZ erforderlich. Der Admin-console-CSV-Export verwendet ein anderes, verlustbehaftetes Schema.
