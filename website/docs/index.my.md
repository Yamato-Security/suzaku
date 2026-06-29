---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) သည် <a href="https://github.com/Yamato-Security">Yamato
Security</a> မှ ဖန်တီးပြီး <a href="https://www.rust-lang.org/">Rust</a> ဖြင့် ရေးသားထားသော <strong>cloud log များအတွက် Sigma အခြေပြု threat hunting နှင့် မြန်ဆန်သော forensics timeline
generator</strong> တစ်ခု ဖြစ်သည်။ <a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a> ကို စိတ်ကူးကြည့်ပါ၊ သို့သော် Windows event log များအစား cloud log များအတွက် ဖြစ်ပြီး — AWS CloudTrail အတွက် native <a href="https://github.com/SigmaHQ/sigma">Sigma</a> ပံ့ပိုးမှုဖြင့် (Azure နှင့် GCP ကို စီစဉ်ထားသည်)။
</p>

<div class="hb-cta" markdown>
[Get Started :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Command Reference :material-console:](commands/index.md){ .md-button }
[View on GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## Suzaku ကို ဘာကြောင့်?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Cloud-native Sigma__

    ---

    Cloud log များအတွက် native [Sigma](https://github.com/SigmaHQ/sigma) detection — ယနေ့ AWS CloudTrail အတွက်၊
    Azure နှင့် GCP ကို စီစဉ်ထားသည်။ Correlation rule များနှင့် field modifier အားလုံးနီးပါးကို ပံ့ပိုးထားသည်။

-   :material-timeline-clock:{ .lg .middle } __Fast forensics timelines__

    ---

    ဆူညံသော cloud API call ထောင်ပေါင်းများစွာကို သင်လိုအပ်သော event များသာပါဝင်သည့်
    အလွယ်တကူ ခွဲခြမ်းစိတ်ဖြာနိုင်သော **DFIR timeline** တစ်ခုတည်းအဖြစ် ပြောင်းလဲပေးသည်။

-   :material-flash:{ .lg .middle } __Blazing fast in Rust__

    ---

    Memory-safe၊ multi-threaded နှင့် standalone ဖြစ်သည်။ Windows၊ Linux နှင့် macOS ပေါ်တွင် `.json` နှင့် compressed `.json.gz` log များကို scan လုပ်သည်။

-   :material-chart-box:{ .lg .middle } __Attacker summaries__

    ---

    API အသုံးပြုမှုနှင့် တိုက်ခိုက်သူ၏ metric များ — source IP များ၊ geo-location၊ region များ၊ user agent များ — ကို အကျဉ်းချုပ်ပြီး
    လျင်မြန်စွာ pivot လုပ်ပါ။

-   :material-shield-search:{ .lg .middle } __Behavior detection__

    ---

    **signature များကို အားကိုးခြင်းမရှိဘဲ** ပုံမှန်မဟုတ်သော လှုပ်ရှားမှုများကို ဖော်ထုတ်ပေးသည်၊ ထို့ကြောင့် အသစ်အဆန်းသော တိုက်ခိုက်မှုများကို သင် လွတ်သွားမည်မဟုတ်ပါ။

-   :material-export:{ .lg .middle } __Flexible output__

    ---

    သင်ရွေးချယ်သော tool တွင် ခွဲခြမ်းစိတ်ဖြာရန် ရလဒ်များကို **CSV, JSON နှင့် JSONL** အဖြစ် သိမ်းဆည်းပါ။

</div>

## Quick links

<div class="grid cards" markdown>

-   __:material-book-open-variant: ဒီမှာ အသစ်လား?__

    [Overview](overview/index.md) ဖြင့် စတင်ပါ၊ ထို့နောက် Suzaku ကို download ဆွဲပြီး run ရန်
    [Getting Started](getting-started/index.md) သို့ သွားပါ။

-   __:material-console-line: CLI ဖြင့် အလုပ်လုပ်နေပါသလား?__

    [Command List](commands/index.md) နှင့် [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md) နှင့်
    [DFIR Timeline](commands/dfir-timeline.md) command များအတွက် တစ်ခုချင်းစီ၏ reference ကို ကြည့်ရှုပါ။

-   __:material-puzzle: ပိုမိုဆက်လက်လိုပါသလား?__

    [Native Sigma Support](rules/index.md)၊
    [Companion Projects](resources/companion-projects.md)၊ နှင့်
    [contribute](resources/contributing.md) လုပ်ပုံကို လေ့လာပါ။

</div>
