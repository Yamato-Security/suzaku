# 讀取 AWS 主控台 CSV 下載檔

Suzaku 的 AWS 指令（`aws-ct-timeline`、`aws-ct-metrics`、`aws-ct-summary`、`aws-ct-search`）可透過 `-f/--file` 或 `-d/--directory`，讀取從 AWS Management Console 的 **CloudTrail → Event history → Download events → CSV** 下載的檔案。

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

CSV 適合在沒有 Trail 將日誌傳送到 S3 時作為替代方案，但它只是事件的**摘要**，不是完整的 CloudTrail 記錄。若有 Trail 的 JSON 日誌或 Event history 的 JSON 下載檔，請優先使用 JSON。

## 轉換方式

每一列會重建為 CloudTrail 記錄：`Event time` → `eventTime`、`Event name` → `eventName`、`User name` → `userIdentity.userName`、`AWS access key` → `userIdentity.accessKeyId`、`Resources` → `resources[]`，而 `Read-only` 會轉成布林值 `readOnly`。空白儲存格會視為欄位不存在，使使用 `errorCode|exists` 的規則能正確區分成功與失敗。`root` 也會補上 `userIdentity.type: Root`。

僅辨識英文欄名。不是 CloudTrail 主控台下載格式的 CSV 會被拒絕。欄位數不正確（通常是引號損壞）或沒有 `Event time` 的列會被略過，並連同行號寫入錯誤日誌。

## 限制

1. **必須下載所需的每個區域，包括 `us-east-1`。** 主控台一次只下載一個區域；主控台登入和 IAM 事件會記錄在 `us-east-1`。
2. **部分規則無法比對。** CSV 不包含 `userIdentity.arn`/`type`/`accountId`/`principalId`、`errorMessage`、`requestParameters`、`responseElements` 或 `additionalEventData`。約 796 條 CloudTrail 規則中的 22 條（包括登入失敗及未使用 MFA 的登入）因此無法比對。
3. **部分時間軸欄位會是空白。** `UserARN`、`UserAccountID`、`UserPrincipalID`、`ErrorMessage` 一律為空；除 root 外，`UserType` 也為空。
4. **`aws-ct-summary` 會依使用者名稱分組。** 角色的使用者名稱是工作階段名稱，因此使用相同工作階段名稱的不同角色會合併。沒有使用者名稱的列不會進入摘要，所以計數可能與相同期間的 JSON 不同。
5. **Google Workspace 不支援 CSV。** `gws-timeline` 必須使用 Reports API 的 JSON/JSONL/JSON.GZ；Admin console CSV 是另一種有資訊缺失的格式。
