# Clonage Git

Vous pouvez exécuter `git clone` sur le dépôt avec la commande suivante et compiler le binaire à partir du code source :

**Avertissement :** La branche main du dépôt est destinée au développement, vous pourrez donc accéder à de nouvelles fonctionnalités qui ne sont pas encore officiellement publiées, mais il peut y avoir des bugs, considérez-la donc comme instable.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Remarque :** Si vous oubliez d'utiliser l'option `--recursive`, le dossier `rules`, qui est géré comme un sous-module git, ne sera pas cloné.

Vous pouvez synchroniser le dossier `rules` et obtenir les dernières règles Suzaku avec `git pull --recurse-submodules` ou utiliser la commande suivante :

```bash
./suzaku update-rules
```

Si la mise à jour échoue, vous devrez peut-être renommer le dossier `rules` et réessayer.

>> Attention : Lors de la mise à jour, les règles et les fichiers de configuration du dossier `rules` sont remplacés par les dernières règles et fichiers de configuration du dépôt [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules).
>> Toute modification que vous apportez aux fichiers existants sera écrasée, nous vous recommandons donc de faire des sauvegardes de tous les fichiers que vous modifiez avant de mettre à jour.
>> Si vous ajoutez de **nouvelles** règles dans le dossier `rules`, elles ne seront **pas** écrasées ou supprimées lors de la mise à jour.
