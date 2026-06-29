# Ejecutar Suzaku

## Windows

En un Símbolo del sistema/PowerShell o en la Terminal de Windows, simplemente ejecute el binario de Windows de 32 bits o 64 bits que corresponda.

### Error al intentar analizar un archivo o directorio con un espacio en la ruta

Al usar el Símbolo del sistema o PowerShell integrado en Windows, es posible que reciba un error indicando que Suzaku no pudo cargar ningún archivo si hay un espacio en la ruta de su archivo o directorio.
Para cargar los archivos de registro correctamente, asegúrese de hacer lo siguiente:
1. Encierre la ruta del archivo o directorio entre comillas dobles.
2. Si es una ruta de directorio, asegúrese de no incluir una barra invertida como último carácter.

### Los caracteres no se muestran correctamente

Con la fuente predeterminada `Lucida Console` en Windows, varios caracteres usados en el logotipo y las tablas no se mostrarán correctamente.
Debe cambiar la fuente a `Consalas` para solucionar esto.

## Linux

Primero necesita hacer que el binario sea ejecutable.

```bash
chmod +x ./suzaku
```

Luego ejecútelo desde el directorio raíz de Suzaku:

```bash
./suzaku
```

## macOS

Desde Terminal o [iTerm2](https://iterm2.com/), primero necesita hacer que el binario sea ejecutable.

```bash
chmod +x ./suzaku
```

Luego, intente ejecutarlo desde el directorio raíz de Suzaku:

```bash
./suzaku
```

En la última versión de macOS, es posible que reciba un error de seguridad al intentar ejecutarlo.
Haga clic en "Cancelar" y luego, desde Preferencias del Sistema, abra "Seguridad y privacidad" y, en la pestaña General, haga clic en "Permitir de todos modos".
Después de eso, intente ejecutarlo nuevamente.

```bash
./suzaku
```

Aparecerá una advertencia, así que simplemente haga clic en "Abrir".
Ahora debería poder ejecutar suzaku.
