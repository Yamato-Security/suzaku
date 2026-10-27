# Leer descargas CSV de la consola de AWS

Los comandos AWS de Suzaku (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) pueden leer con `-f/--file` o `-d/--directory` el archivo descargado desde **CloudTrail → Event history → Download events → CSV** en AWS Management Console.

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

El CSV es una alternativa útil cuando no hay un Trail que entregue registros a S3, pero es solo un **resumen** de cada evento, no el registro completo de CloudTrail. Use preferentemente los registros JSON de un Trail o la descarga JSON de Event history.

## Conversión

Cada fila se reconstruye como un registro de CloudTrail: `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]` y `Read-only` → booleano `readOnly`. Una celda vacía se trata como un campo ausente para que las reglas con `errorCode|exists` distingan correctamente llamadas exitosas y fallidas. Para `root` también se agrega `userIdentity.type: Root`.

Solo se reconocen los nombres de columna en inglés. Se rechaza cualquier CSV que no tenga el formato de descarga de CloudTrail de la consola. Las filas con un número incorrecto de campos (normalmente por comillas dañadas) o sin `Event time` se omiten y se registran con su número de línea.

## Limitaciones

1. **Descargue cada región necesaria, incluida `us-east-1`.** La consola exporta una sola región por vez; los inicios de sesión en la consola y los eventos de IAM se registran en `us-east-1`.
2. **Algunas reglas no pueden coincidir.** El CSV no contiene `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` ni `additionalEventData`. Por ello, unas 22 de las 796 reglas de CloudTrail, incluidas las de inicios fallidos y accesos sin MFA, no pueden coincidir.
3. **Algunas columnas de la línea de tiempo quedan vacías.** `UserARN`, `UserAccountID`, `UserPrincipalID` y `ErrorMessage` siempre están vacías; `UserType` también, salvo para root.
4. **`aws-ct-summary` agrupa por nombre de usuario.** En un rol este valor es el nombre de sesión, por lo que roles distintos con el mismo nombre se combinan. Las filas sin nombre de usuario se omiten y los totales pueden diferir de un JSON del mismo período.
5. **Google Workspace no admite CSV.** Use JSON/JSONL/JSON.GZ de Reports API con `gws-timeline`. El CSV de Admin console usa otro esquema y pierde información.
