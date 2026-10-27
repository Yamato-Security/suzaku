# AWS कंसोल CSV डाउनलोड पढ़ना

Suzaku के AWS कमांड (`aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-summary`, `aws-ct-search`) `-f/--file` या `-d/--directory` के साथ AWS Management Console के **CloudTrail → Event history → Download events → CSV** से डाउनलोड की गई फ़ाइल पढ़ सकते हैं।

```bash
./suzaku aws-ct-timeline -f event_history.csv
./suzaku aws-ct-summary -d ./console-downloads/ -o summary.csv
```

जब कोई Trail लॉग को S3 में नहीं भेजता तब यह CSV उपयोगी है, लेकिन यह पूरे CloudTrail रिकॉर्ड के बजाय हर इवेंट का केवल **सारांश** है। उपलब्ध होने पर Trail के JSON लॉग या Event history का JSON डाउनलोड प्रयोग करें।

## रूपांतरण

हर पंक्ति को CloudTrail रिकॉर्ड के रूप में बनाया जाता है: `Event time` → `eventTime`, `Event name` → `eventName`, `User name` → `userIdentity.userName`, `AWS access key` → `userIdentity.accessKeyId`, `Resources` → `resources[]`, और `Read-only` → boolean `readOnly`। खाली सेल को अनुपस्थित फ़ील्ड माना जाता है, जिससे `errorCode|exists` नियम सफल और विफल कॉल को सही तरह अलग कर सकें। `root` के लिए `userIdentity.type: Root` भी जोड़ा जाता है।

केवल अंग्रेज़ी कॉलम नाम पहचाने जाते हैं। जो CSV CloudTrail कंसोल डाउनलोड प्रारूप में नहीं है उसे अस्वीकार किया जाता है। गलत फ़ील्ड संख्या वाली पंक्तियाँ (आमतौर पर टूटी हुई quotes) या बिना `Event time` वाली पंक्तियाँ छोड़ दी जाती हैं और उनकी पंक्ति संख्या error log में लिखी जाती है।

## सीमाएँ

1. **`us-east-1` सहित हर आवश्यक region डाउनलोड करें।** कंसोल एक बार में केवल एक region निर्यात करता है; console sign-in और IAM इवेंट `us-east-1` में दर्ज होते हैं।
2. **कुछ नियम match नहीं कर सकते।** CSV में `userIdentity.arn`/`type`/`accountId`/`principalId`, `errorMessage`, `requestParameters`, `responseElements` या `additionalEventData` नहीं होते। लगभग 796 CloudTrail नियमों में से 22, जिनमें विफल login और MFA के बिना login शामिल हैं, match नहीं कर सकते।
3. **कुछ timeline कॉलम खाली रहते हैं।** `UserARN`, `UserAccountID`, `UserPrincipalID` और `ErrorMessage` हमेशा खाली रहते हैं; root को छोड़कर `UserType` भी खाली रहता है।
4. **`aws-ct-summary` user name से समूह बनाता है।** role के लिए यह session name होता है, इसलिए एक ही session name वाले अलग-अलग roles जुड़ जाते हैं। बिना user name वाली पंक्तियाँ summary में नहीं आतीं, इसलिए गिनती उसी अवधि के JSON से अलग हो सकती है।
5. **Google Workspace CSV स्वीकार नहीं करता।** `gws-timeline` के साथ Reports API JSON/JSONL/JSON.GZ प्रयोग करें। Admin console CSV एक अलग, अपूर्ण schema है।
