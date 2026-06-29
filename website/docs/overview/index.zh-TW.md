# 關於 Suzaku

Suzaku（朱雀）意指[「朱雀」](https://en.wikipedia.org/wiki/Vermilion_Bird)，在[中國神話](https://en.wikipedia.org/wiki/Four_Holy_Beasts)中，祂是一位翱翔於雲端之上、統御南方天界的神祇。

Suzaku 是一款針對雲端日誌的威脅獵捕與快速鑑識時間軸產生器。
（可以想像成是 [Hayabusa](https://github.com/Yamato-Security/hayabusa)，但處理的是雲端日誌而非 Windows 事件日誌。）
它目前正積極開發中，原生支援針對 AWS CloudTrail 日誌的 [Sigma](https://github.com/SigmaHQ/sigma) 偵測功能。
我們也計畫支援 Azure 與 GCP 日誌。

在雲端日誌中，存在數千種不同的 API 呼叫，以及多到任何人都無法以人工逐一篩查的事件。
Suzaku 的設計目標不僅是要在雜訊中找出攻擊，還要為您提供一份 DFIR 時間軸，其中只包含您執行快速鑑識調查所需的事件與資料。
您也可以建立摘要，以便快速從宏觀層面了解發生了什麼事、發現不依賴特徵碼的異常行為，並輕鬆找出諸如 IP 位址、使用者代理（user agent）、區域、地理位置等關鍵字，以便進行樞紐分析，避免遺漏攻擊者在被您發現後所執行的任何事件。
