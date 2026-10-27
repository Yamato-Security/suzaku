# विशेषताएँ

* क्रॉस-प्लेटफ़ॉर्म समर्थन: Windows, Linux, macOS।
* मेमोरी सुरक्षित, तेज़ और स्टैंडअलोन होने के लिए Rust में विकसित।
* मूल Sigma detection के साथ AWS CloudTrail, Azure/Entra ID/M365 और Google Workspace Reports API लॉग का विश्लेषण करें।
* `.json`, `.jsonl`, संपीड़ित `.json.gz`, Parquet और समर्थित [AWS Management Console CloudTrail CSV](../getting-started/console-csv-input.md) को multi-thread में स्कैन करें।
* फ़ोरेंसिक जाँच और घटना प्रतिक्रिया के लिए विश्लेषण में आसान एकल टाइमलाइन बनाएँ।
* पढ़ने/बनाने/संपादित करने में आसान YML-आधारित [Sigma](https://github.com/SigmaHQ/sigma) नियमों में लिखे गए IoC सिग्नेचर के लिए उत्कृष्ट मूल समर्थन। ([expand](https://sigmahq.io/docs/basics/modifiers.html#expand) को छोड़कर सहसंबंध नियम और सभी फ़ील्ड संशोधक समर्थित हैं।)
* सिग्नेचर पर निर्भर हुए बिना असामान्य गतिविधि की खोज करने के लिए सभी API उपयोग का सारांश, हमलावर के बारे में मेट्रिक्स (स्रोत IP पते, भू-स्थान, उपयोग किए गए क्षेत्र, उपयोगकर्ता एजेंट, आदि) बनाएँ।
* परिणामों को CSV, JSON, JSONL और DuckDB में सहेजें।
