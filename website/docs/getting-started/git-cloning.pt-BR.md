# Clonagem com Git

Você pode usar `git clone` no repositório com o comando a seguir e compilar o binário a partir do código-fonte:

**Aviso:** A branch main do repositório é destinada a fins de desenvolvimento, portanto você poderá acessar novos recursos ainda não lançados oficialmente; no entanto, pode haver bugs, então considere-a instável.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Nota:** Se você esquecer de usar a opção `--recursive`, a pasta `rules`, que é gerenciada como um submódulo git, não será clonada.

Você pode sincronizar a pasta `rules` e obter as regras mais recentes do Suzaku com `git pull --recurse-submodules` ou usar o comando a seguir:

```bash
./suzaku update-rules
```

Se a atualização falhar, talvez seja necessário renomear a pasta `rules` e tentar novamente.

>> Cuidado: Ao atualizar, as regras e os arquivos de configuração na pasta `rules` são substituídos pelas regras e arquivos de configuração mais recentes do repositório [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules).
>> Quaisquer alterações que você fizer em arquivos existentes serão sobrescritas, por isso recomendamos que você faça backups de quaisquer arquivos que editar antes de atualizar.
>> Se você adicionar **novas** regras dentro da pasta `rules`, elas **não** serão sobrescritas nem excluídas durante a atualização.
