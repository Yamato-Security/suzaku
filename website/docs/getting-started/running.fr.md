# Exécuter Suzaku

## Windows

Dans une invite de commande/PowerShell ou dans Windows Terminal, exécutez simplement le binaire Windows 32 bits ou 64 bits approprié.

### Erreur lors de la tentative d'analyse d'un fichier ou d'un répertoire dont le chemin contient un espace

Lorsque vous utilisez l'invite de commande ou PowerShell intégrée à Windows, il se peut que vous receviez une erreur indiquant que Suzaku n'a pas pu charger de fichiers si votre chemin de fichier ou de répertoire contient un espace.
Afin de charger correctement les fichiers journaux, veillez à effectuer les opérations suivantes :
1. Entourez le chemin du fichier ou du répertoire de guillemets doubles.
2. S'il s'agit d'un chemin de répertoire, assurez-vous de ne pas inclure de barre oblique inverse comme dernier caractère.

### Caractères affichés incorrectement

Avec la police par défaut `Lucida Console` sous Windows, divers caractères utilisés dans le logo et les tableaux ne s'afficheront pas correctement.
Vous devriez changer la police pour `Consalas` afin de corriger ce problème.

## Linux

Vous devez d'abord rendre le binaire exécutable.

```bash
chmod +x ./suzaku
```

Puis exécutez-le depuis le répertoire racine de Suzaku :

```bash
./suzaku
```

## macOS

Depuis Terminal ou [iTerm2](https://iterm2.com/), vous devez d'abord rendre le binaire exécutable.

```bash
chmod +x ./suzaku
```

Ensuite, essayez de l'exécuter depuis le répertoire racine de Suzaku :

```bash
./suzaku
```

Sur la dernière version de macOS, il se peut que vous receviez une erreur de sécurité lorsque vous essayez de l'exécuter.
Cliquez sur « Annuler », puis depuis les Préférences Système, ouvrez « Sécurité et confidentialité » et, dans l'onglet Général, cliquez sur « Autoriser quand même ».
Ensuite, essayez de l'exécuter à nouveau.

```bash
./suzaku
```

Un avertissement apparaîtra, il vous suffit alors de cliquer sur « Ouvrir ».
Vous devriez maintenant pouvoir exécuter suzaku.
