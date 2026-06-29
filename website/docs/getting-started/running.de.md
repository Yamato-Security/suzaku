# Suzaku ausführen

## Windows

Führen Sie in einer Eingabeaufforderung/PowerShell-Eingabeaufforderung oder im Windows Terminal einfach die passende 32-Bit- oder 64-Bit-Windows-Binärdatei aus.

### Fehler beim Versuch, eine Datei oder ein Verzeichnis mit einem Leerzeichen im Pfad zu scannen

Wenn Sie die integrierte Eingabeaufforderung oder PowerShell-Eingabeaufforderung in Windows verwenden, erhalten Sie möglicherweise einen Fehler, dass Suzaku keine Dateien laden konnte, falls sich ein Leerzeichen in Ihrem Datei- oder Verzeichnispfad befindet.
Um die Protokolldateien ordnungsgemäß zu laden, stellen Sie sicher, dass Sie Folgendes tun:
1. Schließen Sie den Datei- oder Verzeichnispfad in doppelte Anführungszeichen ein.
2. Wenn es sich um einen Verzeichnispfad handelt, stellen Sie sicher, dass Sie keinen Backslash als letztes Zeichen verwenden.

### Zeichen werden nicht korrekt angezeigt

Mit der Standardschriftart `Lucida Console` unter Windows werden verschiedene Zeichen, die im Logo und in den Tabellen verwendet werden, nicht korrekt angezeigt.
Sie sollten die Schriftart auf `Consalas` ändern, um dies zu beheben.

## Linux

Sie müssen die Binärdatei zunächst ausführbar machen.

```bash
chmod +x ./suzaku
```

Führen Sie sie dann aus dem Suzaku-Stammverzeichnis aus:

```bash
./suzaku
```

## macOS

Aus dem Terminal oder [iTerm2](https://iterm2.com/) müssen Sie die Binärdatei zunächst ausführbar machen.

```bash
chmod +x ./suzaku
```

Versuchen Sie dann, sie aus dem Suzaku-Stammverzeichnis auszuführen:

```bash
./suzaku
```

In der neuesten Version von macOS erhalten Sie möglicherweise einen Sicherheitsfehler, wenn Sie versuchen, sie auszuführen.
Klicken Sie auf "Abbrechen" und öffnen Sie dann unter Systemeinstellungen "Sicherheit & Datenschutz" und klicken Sie auf der Registerkarte Allgemein auf "Trotzdem zulassen".
Versuchen Sie danach erneut, sie auszuführen.

```bash
./suzaku
```

Es wird eine Warnung angezeigt, klicken Sie also einfach auf "Öffnen".
Sie sollten Suzaku nun ausführen können.
