# नेटिव Sigma समर्थन

Suzaku में Sigma विनिर्देश के लिए बहुत अच्छा नेटिव समर्थन है और यह [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) को छोड़कर सभी फ़ील्ड मॉडिफायर का समर्थन करता है, जिसके लिए कुछ पूर्व-कॉन्फ़िगरेशन की आवश्यकता होती है।

संस्करण 1.0.0 से, Suzaku correlation नियमों का भी समर्थन करता है जो क्लाउड लॉग में हमलों का पता लगाने के लिए महत्वपूर्ण हैं।

> नोट: वर्तमान में, आपको correlation नियमों को एकल फ़ाइलों में बनाने की आवश्यकता है।

## इवेंट गणना नियम

ये ऐसे नियम हैं जो कुछ इवेंट्स की गणना करते हैं और चेतावनी देते हैं यदि एक समय-सीमा के भीतर इनमें से बहुत अधिक या बहुत कम संख्या में इवेंट होते हैं।
एक निश्चित समयावधि के भीतर कई इवेंट्स का पता लगाने के सामान्य उदाहरण पासवर्ड अनुमान लगाने वाले हमलों, पासवर्ड स्प्रे हमलों और डिनायल ऑफ़ सर्विस हमलों का पता लगाने के लिए हैं।
आप इन नियमों का उपयोग लॉग स्रोत विश्वसनीयता संबंधी समस्याओं का पता लगाने के लिए भी कर सकते हैं, जैसे कि जब कुछ इवेंट एक निश्चित सीमा से नीचे आ जाते हैं।

### इवेंट गणना नियम उदाहरण

```yml
title: Correlation Test
id: 49d15187-4203-4e11-8acd-8736f25b6609
status: test
author: TEST
correlation:
    type: event_count
    rules:
        - Console Login With MFA
    group-by:
        - sourceIPAddress
    timespan: 3d
    condition:
        gte: 3
        field: sourceIPAddress
    generate: true 
level: high
---
title: Console Login With MFA
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'Yes'
    condition: selection
level: informational
```

## मान गणना नियम

ये नियम किसी दिए गए फ़ील्ड के **अलग-अलग** मानों के साथ एक समय-सीमा के भीतर समान इवेंट्स की गणना करते हैं।

उदाहरण:
- नेटवर्क स्कैन जहां एक एकल स्रोत IP पता कई अलग-अलग गंतव्य IP पतों और/या पोर्ट्स से कनेक्ट करने का प्रयास करता है।
- पासवर्ड स्प्रेइंग हमले जहां एक एकल स्रोत कई अलग-अलग उपयोगकर्ताओं के साथ प्रमाणीकरण करने में विफल रहता है।
- BloodHound जैसे टूल का पता लगाएं जो कम समय-सीमा के भीतर कई उच्च-विशेषाधिकार वाले AD समूहों की गणना करते हैं।

### मान गणना नियम उदाहरण

```yml
title: Correlation value_count Test
id: 49d15187-4203-4e11-8acd-8736f25b66xx
status: test
author: TEST
correlation:
    type: value_count
    rules:
        - Console Login Without MFA
    group-by:
        - sourceIPAddress
    timespan: 3d
    condition:
        gte: 2
        field: sourceIPAddress
    generate: true 
level: high
---
title: Console Login Without MFA
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'No'
    condition: selection
level: medium
```

## टेम्पोरल प्रॉक्सिमिटी नियम

rule फ़ील्ड द्वारा संदर्भित नियमों द्वारा परिभाषित सभी इवेंट्स को timespan द्वारा परिभाषित समय-सीमा में होना चाहिए।
`group-by` में परिभाषित फ़ील्ड के मानों का एक ही मान होना चाहिए (उदा: समान होस्ट, उपयोगकर्ता, आदि...)।

एक उदाहरण है तीन Sigma नियमों में परिभाषित टोही API कॉल जो समान स्रोत IP पते से 5 मिनट के भीतर मनमाने क्रम में आह्वानित होते हैं।

### टेम्पोरल प्रॉक्सिमिटी नियम उदाहरण

```yml
title: Correlation temporal Test
id: 49d15187-4203-4e11-8acd-8736f25b66xx
status: test
author: TEST
correlation:
    type: temporal
    rules:
        - CloudTrail Log Settings Modified
        - Console Login Without MFA
        - Role Enumeration
    timespan: 3d
    generate: true
level: high
---
title: CloudTrail Log Settings Modified
author: Zach Mathis (@yamatosecurity)
date: 2025-04-23
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'cloudtrail.amazonaws.com'
        eventName: 'UpdateTrail'
    filter:
        errorCode: 'AccessDenied'
    condition: selection and not filter
level: high
---
title: Console Login Without MFA
author: Zach Mathis (@yamatosecurity)
date: 2025-04-13
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'No'
    condition: selection
level: medium
---
title: Role Enumeration 
author: Zach Mathis (@yamatosecurity)
date: 2025-04-24
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'iam.amazonaws.com'
        eventName: 'ListRoles'
    condition: selection
falsepositives:
level: low
```

## टेम्पोरल ऑर्डर्ड नियम

`temporal_ordered` correlation प्रकार `temporal` की तरह व्यवहार करता है और इसके अतिरिक्त यह आवश्यक करता है कि इवेंट्स `rules` विशेषता में प्रदान किए गए क्रम में दिखाई दें।

एक उदाहरण है कई विफल लॉगिन के बाद एक सफल लॉगिन।

### टेम्पोरल ऑर्डर्ड नियम उदाहरण

```yml
title: Correlation temporal_ordered Test
id: 49d15187-4203-4e11-8acd-8736f25b66xx
status: test
author: TEST
correlation:
    type: temporal_ordered
    rules:
        - Console Login Without MFA
        - Role Enumeration
        - CloudTrail Log Settings Modified
    timespan: 1d
    generate: true
level: high
---
title: CloudTrail Log Settings Modified
author: Zach Mathis (@yamatosecurity)
date: 2025-04-23
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'cloudtrail.amazonaws.com'
        eventName: 'UpdateTrail'
    filter:
        errorCode: 'AccessDenied'
    condition: selection and not filter
level: high
---
title: Console Login Without MFA
author: Zach Mathis (@yamatosecurity)
date: 2025-04-13
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: signin.amazonaws.com
        eventName: 'ConsoleLogin'
        additionalEventData.MFAUsed: 'No'
    condition: selection
level: medium
---
title: Role Enumeration 
author: Zach Mathis (@yamatosecurity)
date: 2025-04-24
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventSource: 'iam.amazonaws.com'
        eventName: 'ListRoles'
    condition: selection
falsepositives:
level: low
```
