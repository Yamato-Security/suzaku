# Clonación con Git

Puede ejecutar `git clone` del repositorio con el siguiente comando y compilar el binario desde el código fuente:

**Advertencia:** La rama main del repositorio es para fines de desarrollo, por lo que es posible que pueda acceder a nuevas funciones que aún no se han publicado oficialmente; sin embargo, puede haber errores, así que considérela inestable.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Nota:** Si olvida usar la opción `--recursive`, la carpeta `rules`, que se gestiona como un submódulo de git, no se clonará.

Puede sincronizar la carpeta `rules` y obtener las reglas más recientes de Suzaku con `git pull --recurse-submodules` o usar el siguiente comando:

```bash
./suzaku update-rules
```

Si la actualización falla, es posible que deba cambiar el nombre de la carpeta `rules` e intentarlo de nuevo.

>> Precaución: Al actualizar, las reglas y los archivos de configuración de la carpeta `rules` se reemplazan con las reglas y los archivos de configuración más recientes del repositorio [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules).
>> Cualquier cambio que realice en los archivos existentes se sobrescribirá, por lo que le recomendamos que haga copias de seguridad de cualquier archivo que edite antes de actualizar.
>> Si agrega reglas **nuevas** dentro de la carpeta `rules`, **no** se sobrescribirán ni se eliminarán al actualizar.
