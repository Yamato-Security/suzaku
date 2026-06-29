# Git Cloning

Sie können das Repository mit dem folgenden Befehl per `git clone` klonen und die Binärdatei aus dem Quellcode kompilieren:

**Warnung:** Der main-Branch des Repositorys dient Entwicklungszwecken, sodass Sie möglicherweise auf neue Funktionen zugreifen können, die noch nicht offiziell veröffentlicht wurden. Es können jedoch Fehler auftreten, betrachten Sie ihn daher als instabil.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Hinweis:** Wenn Sie die Option `--recursive` vergessen, wird der `rules`-Ordner, der als git-Submodul verwaltet wird, nicht geklont.

Sie können den `rules`-Ordner synchronisieren und die neuesten Suzaku-Regeln mit `git pull --recurse-submodules` abrufen oder den folgenden Befehl verwenden:

```bash
./suzaku update-rules
```

Wenn die Aktualisierung fehlschlägt, müssen Sie möglicherweise den `rules`-Ordner umbenennen und es erneut versuchen.

>> Vorsicht: Beim Aktualisieren werden die Regeln und Konfigurationsdateien im `rules`-Ordner durch die neuesten Regeln und Konfigurationsdateien aus dem [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules)-Repository ersetzt.
>> Alle Änderungen, die Sie an vorhandenen Dateien vornehmen, werden überschrieben. Wir empfehlen daher, Sicherungskopien aller Dateien anzulegen, die Sie bearbeiten, bevor Sie aktualisieren.
>> Wenn Sie **neue** Regeln innerhalb des `rules`-Ordners hinzufügen, werden diese beim Aktualisieren **nicht** überschrieben oder gelöscht.
