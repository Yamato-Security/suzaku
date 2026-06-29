# विश्लेषण कमांड

## `aws-ct-metrics` कमांड

इस कमांड का उपयोग AWS CloudTrail लॉग के अंदर के फ़ील्ड पर मेट्रिक्स बनाने के लिए करें।
डिफ़ॉल्ट रूप से, यह `eventName` फ़ील्ड को स्कैन करेगा।
हम वर्तमान में इस कमांड का उपयोग यह पता लगाने के लिए कर रहे हैं कि कौन से API कॉल सबसे आम हैं, ताकि डिटेक्शन नियम लिखने को प्राथमिकता दी जा सके।

## कमांड का उपयोग
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
      --timeline-start <DATE>  Start time of the events to load (ex: "2022-02-22T23:59:59Z)
      --timeline-end <DATE>    End time of the events to load (ex: "2020-02-22T00:00:00Z")
      --time-offset <OFFSET>   Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)

Output:
  -F, --field-name <FIELD_NAME>  The field to generate metrics for [default: eventName]
  -o, --output <FILE>            Output CSV

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
```

### `aws-ct-metrics` कमांड के उदाहरण

* स्क्रीन पर `eventName` API कॉल की एक तालिका आउटपुट करें: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* एक CSV फ़ाइल में सहेजें: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`
