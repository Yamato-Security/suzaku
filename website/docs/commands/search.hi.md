# खोज कमांड

## `aws-ct-search` कमांड

Sigma नियम लिखे बिना AWS CloudTrail लॉग से अपनी ज़रूरत के इवेंट निकालने के लिए इस कमांड का उपयोग करें।
मेल खाने वाले इवेंट `aws-ct-timeline` के समान प्रारूप में दिखाए या सहेजे जाते हैं, इसलिए आप खोज से सीधे परिचित टाइमलाइन कॉलम पर जा सकते हैं।

इवेंट को सीमित करने के तीन तरीके हैं, और इन्हें एक साथ उपयोग किया जा सकता है:

| विकल्प | किससे मिलान होता है | एकाधिक मान |
|---|---|---|
| `-F, --filter FIELD:VALUE` | एक फ़ील्ड का मान, **सटीक मिलान, केस-संवेदी** | `-F` दोहराएँ; **सभी** फ़िल्टर मेल खाने चाहिए (AND) |
| `-k, --keyword KEYWORD` | इवेंट के JSON में कहीं भी एक सबस्ट्रिंग, केस-असंवेदी (`-c` के साथ केस-संवेदी) | `-k` दोहराएँ; **कोई भी** कीवर्ड मेल खाना पर्याप्त है (OR) |
| `-r, --regex REGEX` | इवेंट के JSON में कहीं भी एक रेगुलर एक्सप्रेशन | एक पैटर्न (विकल्पों के लिए बिना एस्केप किया पाइप वर्ण <code>&#124;</code> उपयोग करें) |

प्रत्येक इवेंट की जाँच इस क्रम में होती है, और केवल सभी जाँच पास करने वाले इवेंट आउटपुट होते हैं: समय सीमा (`--timeline-start`, `--timeline-end`, `--time-offset`) → `-F` → `-k` → `-r`।

जब आप जानते हों कि मान किस फ़ील्ड में है (जैसे `eventName:ConsoleLogin`) तो `-F` का, और जब केवल स्ट्रिंग पता हो पर यह न पता हो कि वह कहाँ है तो `-k` या `-r` का उपयोग करें।

### `-F, --filter` कैसे काम करता है

* प्रारूप `FIELD:VALUE` है। तर्क को **पहले** `:` पर विभाजित किया जाता है, इसलिए ARN जैसे कोलन वाले मान ज्यों के त्यों काम करते हैं।
* नेस्टेड फ़ील्ड डॉट नोटेशन में लिखे जाते हैं (`userIdentity.type`)। शुरुआती `.` वैकल्पिक है, इसलिए `.userIdentity.arn` भी काम करता है।
* मान के चारों ओर के उद्धरण चिह्न (`"..."` या `'...'`) हटा दिए जाते हैं।
* संख्याओं और बूलियन की तुलना स्ट्रिंग के रूप में होती है, इसलिए `-F readOnly:false` और `-F responseElements.user.userId:12345` काम करते हैं।
* जिस इवेंट में वह फ़ील्ड नहीं है, वह मेल नहीं खाता।
* ऐरे के अंदर के तत्वों को संबोधित नहीं किया जा सकता (`items.0.name` जैसा कोई सिंटैक्स नहीं है)।
* `:` के बिना फ़िल्टर स्कैन शुरू होने से पहले अस्वीकार कर दिया जाता है: `Invalid --filter 'eventName': expected FIELD:VALUE (e.g. eventName:ConsoleLogin)`।

> नोट: `aws-ct-metrics` में `-F` का अर्थ अलग है, वहाँ यह `--field-name` है और केवल फ़ील्ड नाम लेता है। `aws-ct-search` में यह `--filter` है और हमेशा `FIELD:VALUE` लेता है।

## कमांड का उपयोग
```
Usage: suzaku aws-ct-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory of multiple gz/json/parquet files
  -f, --file <FILE>      File path to one gz/json/parquet file

Filtering:
  -F, --filter <FILTER...>     Filter by specific field(s)
  -c, --preserve-case          Case-sensitive keyword search
  -k, --keyword <KEYWORD...>   Search by keyword(s)
  -r, --regex <REGEX>          Search by regular expression
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
      --file-date-from <DATE>  Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
      --file-date-to <DATE>    Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")

Output:
  -C, --clobber                   Overwrite files when saving
  -G, --geo-ip <MAXMIND-DB-DIR>   Add GeoIP (ASN, city, country) info to IP addresses
  -o, --output <FILE>             Save the results to a file
  -t, --output-type <FORMAT,...>  Output format(s) (only used with -o): csv (default), json, jsonl, duckdb. Comma-separate or repeat to write several at once, e.g. -t csv,duckdb [default: csv] [possible values: csv, json, jsonl, duckdb]
      --raw-output                Output the original JSON logs (only available in JSON formats or stdout)
      --threads <THREAD NUMBER>   Number of threads to use (default: same as CPU cores)

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### `aws-ct-search` कमांड के उदाहरण

फ़ील्ड द्वारा फ़िल्टर करना (`-F`):

* कंसोल लॉगिन खोजें: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin`
* root उपयोगकर्ता द्वारा किए गए API कॉल खोजें (नेस्टेड फ़ील्ड): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root`
* किसी विशिष्ट IAM उपयोगकर्ता की सभी गतिविधियाँ खोजें (मान में कोलन है): `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.arn:arn:aws:iam::123456789012:user/alice`
* अस्वीकृत कॉल खोजें: `./suzaku aws-ct-search -d ../suzaku-sample-data -F errorCode:AccessDenied`
* `us-east-1` में लिखने वाले (केवल-पढ़ने वाले नहीं) IAM कॉल खोजें (कई फ़िल्टर AND होते हैं): `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:iam.amazonaws.com -F awsRegion:us-east-1 -F readOnly:false`
* किसी विशिष्ट S3 बकेट तक पहुँच खोजें (API-विशिष्ट फ़ील्ड): `./suzaku aws-ct-search -d ../suzaku-sample-data -F requestParameters.bucketName:my-bucket`

कीवर्ड (`-k`) और रेगुलर एक्सप्रेशन (`-r`) से खोज:

* कहीं भी IP पते का उल्लेख करने वाले इवेंट खोजें: `./suzaku aws-ct-search -d ../suzaku-sample-data -k 203.0.113.10`
* दो उपयोगकर्ताओं में से किसी का उल्लेख करने वाले इवेंट खोजें (कई कीवर्ड OR होते हैं): `./suzaku aws-ct-search -d ../suzaku-sample-data -k alice -k bob`
* केस-संवेदी कीवर्ड खोज: `./suzaku aws-ct-search -d ../suzaku-sample-data -k AKIAIOSFODNN7EXAMPLE -c`
* CloudTrail को अक्षम करने के प्रयास खोजें: `./suzaku aws-ct-search -d ../suzaku-sample-data -r "(StopLogging|DeleteTrail|UpdateTrail)"`

शर्तों को मिलाना और परिणाम सहेजना:

* एक इवेंट स्रोत तक सीमित कीवर्ड खोज: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventSource:s3.amazonaws.com -k delete`
* पिछले 7 दिनों में विफल कंसोल लॉगिन: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -k Failure --time-offset 7d`
* GeoIP जानकारी के साथ परिणाम CSV और DuckDB में सहेजें: `./suzaku aws-ct-search -d ../suzaku-sample-data -F eventName:ConsoleLogin -G ../GeoLite2-DBs -o console-logins -t csv,duckdb`
* मेल खाने वाले इवेंट का मूल JSON सहेजें: `./suzaku aws-ct-search -d ../suzaku-sample-data -F userIdentity.type:Root -o root-events.jsonl -t jsonl --raw-output`

### `aws-ct-search` आउटपुट

आउटपुट कॉलम [`aws-ct-timeline`](dfir-timeline.md) के समान हैं, और DuckDB आउटपुट भी उसी [DuckDB आउटपुट स्कीमा](dfir-timeline.md#duckdb-output-schema) का पालन करता है।
अंत में स्कैन किए गए और मेल खाने वाले इवेंट की संख्या दिखाई जाती है:

```
Total events scanned: 209
Matching events: 1
```
