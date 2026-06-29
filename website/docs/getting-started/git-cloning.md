# Git Cloning

You can `git clone` the repository with the following command and compile binary from source code:

**Warning:** The main branch of the repository is for development purposes so you may be able to access new features not yet officially released, however, there may be bugs so consider it unstable.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Note:** If you forget to use `--recursive` option, the `rules` folder, which is managed as a git submodule, will not be cloned.

You can sync the `rules` folder and get latest Suzaku rules with `git pull --recurse-submodules` or use the following command:

```bash
./suzaku update-rules
```

If the update fails, you may need to rename the `rules` folder and try again.

>> Caution: When updating, rules and config files in the `rules` folder are replaced with the latest rules and config files in the [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) repository.
>> Any changes you make to existing files will be overwritten, so we recommend that you make backups of any files that you edit before updating.
>> If you add **new** rules inside of the `rules` folder, they will **not** be overwritten or deleted when updating.
