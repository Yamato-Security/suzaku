# Ler downloads CSV do Console da AWS

Os comandos AWS do Suzaku (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) leem, com `-f/--file` ou `-d/--directory`, o arquivo obtido em **CloudTrail → Event history → Download events → CSV** no AWS Management Console.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

O CSV é uma alternativa conveniente quando nenhum Trail envia logs ao S3, mas ele contém apenas um **resumo** de cada evento, não o registro completo do CloudTrail. Prefira os logs JSON de um Trail ou o download JSON de Event history quando disponíveis.

## Conversão

Cada linha é reconstruída como um registro do CloudTrail: `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]` e `Read-only` → booleano `readOnly`. Uma célula vazia vira um campo ausente, permitindo que regras com `errorCode|exists` diferenciem corretamente chamadas bem-sucedidas e com falha. Para `root`, também é definido `userIdentity.type: Root`.

Apenas nomes de coluna em inglês são reconhecidos. Um CSV que não seja um download do CloudTrail pelo console é rejeitado. Linhas com quantidade incorreta de campos (normalmente por aspas quebradas) ou sem `Event time` são ignoradas e registradas com o número da linha.

## Limitações

1. **Baixe todas as regiões necessárias, incluindo `us-east-1`.** O console exporta uma região por vez; logins no console e eventos do IAM são registrados em `us-east-1`.
2. **Algumas regras não podem corresponder.** O CSV não possui `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` nem `additionalEventData`. Cerca de 22 das 796 regras do CloudTrail, incluindo falhas de login e login sem MFA, não podem corresponder.
3. **Algumas colunas da linha do tempo ficam vazias.** `UserARN`, `UserAccountID`, `UserPrincipalID` e `ErrorMessage` ficam sempre vazias; `UserType` também, exceto para root.
4. **`aws-ct-summary` agrupa pelo nome do usuário.** Para uma função, esse valor é o nome da sessão; funções diferentes com o mesmo nome são combinadas. Linhas sem usuário são omitidas, portanto as contagens podem divergir de um JSON do mesmo período.
5. **Google Workspace não aceita CSV.** Use JSON/JSONL/JSON.GZ da Reports API com `gws-timeline`. O CSV do Admin console usa outro esquema e perde informações.
