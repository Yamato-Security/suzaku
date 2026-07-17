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
