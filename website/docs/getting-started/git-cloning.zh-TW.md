# Git Cloning

您可以使用以下指令 `git clone` 此儲存庫，並從原始碼編譯出二進位檔：

**警告：** 此儲存庫的 main 分支用於開發目的，因此您可能可以使用尚未正式發布的新功能，但其中可能存在錯誤，請將其視為不穩定版本。

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **注意：** 如果您忘記使用 `--recursive` 選項，作為 git 子模組管理的 `rules` 資料夾將不會被複製下來。

您可以使用 `git pull --recurse-submodules` 同步 `rules` 資料夾並取得最新的 Suzaku 規則，或使用以下指令：

```bash
./suzaku update-rules
```

如果更新失敗，您可能需要重新命名 `rules` 資料夾後再試一次。

>> 注意：更新時，`rules` 資料夾中的規則與設定檔會被替換為 [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) 儲存庫中最新的規則與設定檔。
>> 您對現有檔案所做的任何變更都將被覆寫，因此我們建議您在更新前先備份任何您編輯過的檔案。
>> 如果您在 `rules` 資料夾內新增**新的**規則，更新時這些規則**不會**被覆寫或刪除。
