# 功能

* 跨平台支援：Windows、Linux、macOS。
* 以 Rust 開發，具備記憶體安全、快速且獨立執行的特性。
* 以多執行緒效能掃描 `.json` 或壓縮的 `.json.gz` 檔案。
* 為數位鑑識調查與事件回應建立單一且易於分析的時間軸。
* 對於以易讀／易建立／易編輯的 YML 為基礎的 [Sigma](https://github.com/SigmaHQ/sigma) 規則所撰寫的 IoC 特徵碼，提供優異的原生支援。（支援關聯規則以及除了 [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) 以外的所有欄位修飾子。）
* 建立所有 API 使用情形的摘要，以及關於攻擊者的各項指標（來源 IP 位址、地理位置、使用的區域、使用者代理程式等等），以便在不依賴特徵碼的情況下發現異常活動。
* 將結果儲存為 CSV、JSON 與 JSONL。
