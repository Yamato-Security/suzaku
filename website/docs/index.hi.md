---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) एक <strong>Sigma-आधारित खतरा शिकार और क्लाउड लॉग के लिए तेज़ फ़ॉरेंसिक टाइमलाइन
जनरेटर</strong> है, जिसे <a href="https://github.com/Yamato-Security">Yamato
Security</a> द्वारा बनाया गया है और <a href="https://www.rust-lang.org/">Rust</a> में लिखा गया है। कल्पना कीजिए
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a> की, लेकिन Windows इवेंट लॉग के बजाय क्लाउड लॉग के लिए — 
AWS CloudTrail के लिए मूल <a href="https://github.com/SigmaHQ/sigma">Sigma</a> समर्थन के साथ (Azure और GCP की योजना है)।
</p>

<div class="hb-cta" markdown>
[शुरू करें :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[कमांड संदर्भ :material-console:](commands/index.md){ .md-button }
[GitHub पर देखें :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
</div>

<p class="hb-badges">
<a href="https://github.com/Yamato-Security/suzaku/releases"><img src="https://img.shields.io/github/v/release/Yamato-Security/suzaku?color=blue&label=Stable%20Version&style=flat"/></a>
<a href="https://github.com/Yamato-Security/suzaku/releases"><img src="https://img.shields.io/github/downloads/Yamato-Security/suzaku/total?style=flat&label=GitHub%F0%9F%A6%85Downloads&color=blue"/></a>
<a href="https://github.com/Yamato-Security/suzaku/stargazers"><img src="https://img.shields.io/github/stars/Yamato-Security/suzaku?style=flat&label=GitHub%F0%9F%A6%85Stars"/></a>
<a href="https://github.com/Yamato-Security/suzaku/blob/main/LICENSE.txt"><img src="https://img.shields.io/badge/License-AGPLv3-blue.svg?style=flat"/></a>
<a href="https://www.blackhat.com/us-25/arsenal/schedule/index.html#cloud-log-fast-forensics-with-yamato-securitys-suzaku-45630"><img src="https://img.shields.io/badge/Black%20Hat%20Arsenal%20USA-2025-blue"></a>
<a href="https://twitter.com/SecurityYamato"><img src="https://img.shields.io/twitter/follow/SecurityYamato?style=social"/></a>
</p>

</div>

---

## Suzaku क्यों?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __क्लाउड-नेटिव Sigma__

    ---

    क्लाउड लॉग के लिए मूल [Sigma](https://github.com/SigmaHQ/sigma) पहचान — आज AWS CloudTrail,
    Azure और GCP की योजना है। सहसंबंध नियम और लगभग सभी फ़ील्ड संशोधक समर्थित हैं।

-   :material-timeline-clock:{ .lg .middle } __तेज़ फ़ॉरेंसिक टाइमलाइन__

    ---

    हजारों शोरगुल वाली क्लाउड API कॉल को एक एकल, विश्लेषण में आसान **DFIR टाइमलाइन** में बदलें जिसमें केवल
    वे ही इवेंट हों जिनकी आपको आवश्यकता है।

-   :material-flash:{ .lg .middle } __Rust में अत्यंत तेज़__

    ---

    मेमोरी-सुरक्षित, मल्टी-थ्रेडेड और स्टैंडअलोन। Windows, Linux और macOS पर `.json` और संपीड़ित `.json.gz` लॉग
    स्कैन करता है।

-   :material-chart-box:{ .lg .middle } __हमलावर सारांश__

    ---

    API उपयोग और हमलावर मेट्रिक्स का सारांश दें — स्रोत IP, भू-स्थान, क्षेत्र, उपयोगकर्ता एजेंट — ताकि
    जल्दी से पिवट किया जा सके।

-   :material-shield-search:{ .lg .middle } __व्यवहार पहचान__

    ---

    असामान्य गतिविधि को **हस्ताक्षरों पर निर्भर हुए बिना** सामने लाएं, ताकि आप नए हमलों से न चूकें।

-   :material-export:{ .lg .middle } __लचीला आउटपुट__

    ---

    अपनी पसंद के टूल में विश्लेषण के लिए परिणामों को **CSV, JSON और JSONL** में सहेजें।

</div>

## त्वरित लिंक

<div class="grid cards" markdown>

-   __:material-book-open-variant: यहाँ नए हैं?__

    [अवलोकन](overview/index.md) से शुरू करें, फिर Suzaku डाउनलोड करने और चलाने के लिए
    [शुरुआत करना](getting-started/index.md) पर जाएं।

-   __:material-console-line: CLI के साथ काम कर रहे हैं?__

    [कमांड सूची](commands/index.md) और
    [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md) और
    [DFIR Timeline](commands/dfir-timeline.md) कमांड के लिए प्रति-कमांड संदर्भ ब्राउज़ करें।

-   __:material-puzzle: आगे बढ़ रहे हैं?__

    [मूल Sigma समर्थन](rules/index.md),
    [संगी परियोजनाएं](resources/companion-projects.md), और कैसे
    [योगदान करें](resources/contributing.md) का अन्वेषण करें।

</div>
