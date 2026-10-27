# Lire les téléchargements CSV de la console AWS

Les commandes AWS de Suzaku (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) lisent avec `-f/--file` ou `-d/--directory` le fichier téléchargé depuis **CloudTrail → Event history → Download events → CSV** dans AWS Management Console.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

Ce CSV est utile lorsqu'aucun Trail n'envoie les journaux vers S3, mais il ne contient qu'un **résumé** de chaque événement, pas l'enregistrement CloudTrail complet. Préférez les journaux JSON d'un Trail ou le téléchargement JSON d'Event history lorsqu'ils sont disponibles.

## Conversion

Chaque ligne est reconstruite comme un enregistrement CloudTrail : `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]` et `Read-only` → booléen `readOnly`. Une cellule vide devient un champ absent afin que les règles utilisant `errorCode|exists` distinguent correctement les appels réussis et échoués. Pour `root`, Suzaku ajoute aussi `userIdentity.type: Root`.

Seuls les noms de colonnes anglais sont reconnus. Un CSV qui n'est pas un téléchargement CloudTrail de la console est refusé. Les lignes ayant un nombre de champs incorrect (souvent des guillemets cassés) ou sans `Event time` sont ignorées et consignées avec leur numéro de ligne.

## Limites

1. **Téléchargez chaque région nécessaire, y compris `us-east-1`.** La console exporte une seule région à la fois ; les connexions à la console et les événements IAM sont enregistrés dans `us-east-1`.
2. **Certaines règles ne peuvent pas correspondre.** Le CSV ne contient pas `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` ni `additionalEventData`. Environ 22 des 796 règles CloudTrail, notamment les échecs de connexion et les connexions sans MFA, ne peuvent donc pas correspondre.
3. **Certaines colonnes de la chronologie restent vides.** `UserARN`, `UserAccountID`, `UserPrincipalID` et `ErrorMessage` sont toujours vides ; `UserType` l'est aussi sauf pour root.
4. **`aws-ct-summary` regroupe par nom d'utilisateur.** Pour un rôle, il s'agit du nom de session ; des rôles différents utilisant le même nom sont fusionnés. Les lignes sans nom d'utilisateur sont omises, les nombres peuvent donc différer d'un export JSON de la même période.
5. **Google Workspace n'accepte pas le CSV.** Utilisez les fichiers JSON/JSONL/JSON.GZ de Reports API avec `gws-timeline`. Le CSV d'Admin console utilise un schéma différent et incomplet.
