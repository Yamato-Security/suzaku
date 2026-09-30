# DFIR टाइमलाइन कमांड

## `aws-ct-timeline` कमांड

`rules` फ़ोल्डर में मौजूद Sigma नियमों के आधार पर एक AWS CloudTrail DFIR टाइमलाइन बनाएँ।

## कमांड उपयोग
```
Usage: suzaku aws-ct-timeline [OPTIONS] <--directory <DIR>|--file <FILE>>

General Options:
  -r, --rules <DIR/FILE>  Specify a custom rule directory or file (default: ./rules)
  -h, --help              Show the help menu

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -C, --clobber                    Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>    Add GeoIP (ASN, city, country) info to IP addresses
  -m, --min-level <LEVEL>          Minimum level for rules to load (default: informational)
  -o, --output <FILE>              Save the results to a file
  -t, --output-type <OUTPUT_TYPE>  Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL [default: 1]
  -R, --raw-output                 Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>    Number of threads to use (default: same as CPU cores)

Display Settings:
  -K, --no-color               Disable color output
  -N, --no-summary             Do not display results summary
  -T, --no-frequency-timeline  Disable event frequency timeline (terminal needs to support Unicode)
  -q, --quiet                  Quiet mode: do not display the launch banner
```

### `aws-ct-timeline` कमांड उदाहरण

* स्क्रीन पर अलर्ट आउटपुट करें: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* परिणामों को एक CSV फ़ाइल में सहेजें: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* परिणामों को CSV और JSONL फ़ाइलों में सहेजें: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

### `aws-ct-timeline` आउटपुट प्रोफ़ाइल

Suzaku `config/aws_profile.yaml` फ़ाइल के आधार पर जानकारी आउटपुट करेगा:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
ErrorCode: '.errorCode'
ErrorMessage: '.errorMessage'
EventSource: '.eventSource'
AWS-Region: '.awsRegion'
SrcIP: '.sourceIPAddress'
UserAgent: '.userAgent'
UserName: '.userIdentity.userName'
UserType: '.userIdentity.type'
UserAccountID: '.userIdentity.accountId'
UserARN: '.userIdentity.arn'
UserPrincipalID: '.userIdentity.principalId'
UserAccessKeyID: '.userIdentity.accessKeyId'
EventID: '.eventID'
Tags: 'sigma.tags'
RuleID: 'sigma.id'
```

* `.` से शुरू होने वाला कोई भी फ़ील्ड मान (उदा: `.eventTime`) CloudTrail लॉग से लिया जाएगा।
* `sigma.` से शुरू होने वाला कोई भी फ़ील्ड मान (उदा: `sigma.title`) Sigma नियम से लिया जाएगा।
* वर्तमान में हम केवल स्ट्रिंग्स का समर्थन करते हैं लेकिन अन्य प्रकार के फ़ील्ड मानों का समर्थन करने की योजना है।

> नोट: यदि आप मूल JSON डेटा आउटपुट करना चाहते हैं और सुनिश्चित करना चाहते हैं कि आप कोई फ़ील्ड जानकारी न खोएँ, तो बस `aws-ct-timeline` कमांड में `-R, --raw-output` विकल्प जोड़ें।

### DuckDB आउटपुट स्कीमा {#duckdb-output-schema}

CSV और JSON आउटपुट ऊपर दी गई प्रोफ़ाइल का *रेंडरिंग* हैं; DuckDB आउटपुट एक *डेटा इंटरफ़ेस* है, इसलिए यह टाइप्ड और स्व-वर्णनात्मक है। ये अंतर जानबूझकर हैं और `aws-ct-timeline`, `azure-timeline`, `gws-timeline` और `aws-ct-search` पर लागू होते हैं:

| | CSV / JSON | DuckDB |
|---|---|---|
| अनुपस्थित मान | `-` (या खाली) | `NULL` |
| `Timestamp` | रेंडर किया गया टेक्स्ट | `TIMESTAMP` |
| `Level` | टेक्स्ट | `suzaku_level` (गंभीरता के क्रम में `ENUM`) |
| `AWS-Region` | `AWS-Region` | `AwsRegion` (SQL में उद्धरण चिह्न आवश्यक नहीं) |
| `Tags` | ` ¦ ` से जुड़ी एक स्ट्रिंग | `Tactics`, `TechniqueIDs`, `OtherTags` (प्रत्येक `VARCHAR[]`) |
| `SrcASN` / `SrcCity` / `SrcCountry` | केवल `-G, --geo-ip` के साथ जोड़े जाते हैं | हमेशा मौजूद (जब प्रोफ़ाइल में `SrcIP` हो), `-G` का उपयोग न होने पर `NULL` |
| डुप्लिकेट पंक्तियाँ | रखी जाती हैं | सटीक डुप्लिकेट हटा दिए जाते हैं, संख्या `suzaku_meta` में दर्ज होती है |

प्रत्येक फ़ाइल में एक पंक्ति वाली `suzaku_meta` तालिका भी होती है, ताकि बिना अनुमान लगाए पता चल सके कि इसे किसने बनाया:

| कॉलम | अर्थ |
|---|---|
| `schema_version` | लेआउट संस्करण। अन्य तालिकाएँ पढ़ने से पहले इसे जाँचें। |
| `suzaku_version`, `command`, `command_line` | कौन सा Suzaku, कौन सा सबकमांड, कौन सा सटीक आह्वान। |
| `generated_at` | फ़ाइल कब लिखी गई। |
| `timestamp_tz` | `Timestamp` कॉलम का समय क्षेत्र — `UTC`, या `-l, --localtime` के साथ स्थानीय ऑफ़सेट। |
| `rules_version`, `rules_count` | नियम-सेट का संशोधन (जब rules फ़ोल्डर एक git checkout हो) और लोड किए गए नियमों की संख्या। |
| `geoip_enabled` | क्या `-G, --geo-ip` चलाया गया था। पूरी तरह `NULL` `SrcCountry` ("संवर्धन बंद था") को संवर्धित फ़ाइल की `NULL` सेल ("यह मान IP पता नहीं है") से अलग करता है। |
| `scanned_files`, `scanned_events` | रन की कवरेज। |
| `output_rows`, `duplicate_rows_removed` | लिखी गई पंक्तियाँ, और लिखते समय हटाए गए सटीक डुप्लिकेट। |

नियम-आधारित टाइमलाइन कमांड में एक `timeline` पंक्ति **एक इवेंट × एक नियम मिलान** है: कई नियमों से मेल खाने वाला इवेंट प्रत्येक मिलान के लिए एक पंक्ति बनाता है, इसलिए `EventID` अद्वितीय *नहीं* है। `aws-ct-search` में प्रत्येक मेल खाने वाला इवेंट एक पंक्ति बनाता है।

```sql
-- Critical and high alerts in a time range, with their ATT&CK techniques.
-- `Level` is an ENUM, so cast the literal to compare by severity rather than alphabetically.
SELECT Timestamp, RuleTitle, EventName, SrcIP, TechniqueIDs
FROM timeline
WHERE Level >= 'high'::suzaku_level
  AND Timestamp BETWEEN TIMESTAMP '2024-01-01' AND TIMESTAMP '2024-02-01'
  AND ErrorCode IS NULL          -- the call succeeded
ORDER BY Timestamp;

-- ATT&CK technique coverage, no string parsing required
SELECT technique, count(*) AS hits
FROM (SELECT unnest(TechniqueIDs) AS technique FROM timeline)
GROUP BY 1 ORDER BY hits DESC;
```

Suzaku बंद होने से पहले डेटाबेस का checkpoint करता है, इसलिए `.duckdb` फ़ाइल पूर्ण होती है और केवल-पढ़ने के लिए खोली जा सकती है (कमांड समाप्त होने के बाद इसे कॉपी करें, चलते समय नहीं)।
