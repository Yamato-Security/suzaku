---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) هو <strong>مولّد للجداول الزمنية للتحقيقات الجنائية الرقمية السريعة وتصيّد التهديدات قائم على Sigma لسجلات السحابة</strong>، أنشأته <a href="https://github.com/Yamato-Security">Yamato
Security</a> وكُتب بلغة <a href="https://www.rust-lang.org/">Rust</a>. تخيّل
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a>، لكن لسجلات السحابة بدلاً من
سجلات أحداث Windows — مع دعم أصلي لـ <a href="https://github.com/SigmaHQ/sigma">Sigma</a> لخدمة
AWS CloudTrail (مع التخطيط لدعم Azure وGCP).
</p>

<div class="hb-cta" markdown>
[ابدأ الآن :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[مرجع الأوامر :material-console:](commands/index.md){ .md-button }
[عرض على GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## لماذا Suzaku؟

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Sigma سحابي أصلي__

    ---

    كشف [Sigma](https://github.com/SigmaHQ/sigma) أصلي لسجلات السحابة — AWS CloudTrail اليوم،
    مع التخطيط لدعم Azure وGCP. قواعد الارتباط ومعظم معدِّلات الحقول مدعومة.

-   :material-timeline-clock:{ .lg .middle } __جداول زمنية للتحقيقات الجنائية السريعة__

    ---

    حوّل آلاف استدعاءات واجهات برمجة التطبيقات السحابية المزعجة إلى **جدول زمني للتحقيقات الجنائية الرقمية (DFIR)** واحد وسهل التحليل يحتوي فقط على
    الأحداث التي تحتاجها.

-   :material-flash:{ .lg .middle } __سرعة فائقة بفضل Rust__

    ---

    آمن للذاكرة، متعدد الخيوط ومستقل. يفحص سجلات `.json` والسجلات المضغوطة `.json.gz` على
    Windows وLinux وmacOS.

-   :material-chart-box:{ .lg .middle } __ملخصات المهاجمين__

    ---

    لخّص استخدام واجهات برمجة التطبيقات ومقاييس المهاجمين — عناوين IP المصدرية، الموقع الجغرافي، المناطق، وكلاء المستخدم — من أجل
    التمحور بسرعة.

-   :material-shield-search:{ .lg .middle } __كشف السلوك__

    ---

    اكشف عن النشاط غير الطبيعي **دون الاعتماد على التواقيع**، حتى لا تفوتك الهجمات الجديدة.

-   :material-export:{ .lg .middle } __إخراج مرن__

    ---

    احفظ النتائج بصيغة **CSV وJSON وJSONL** لتحليلها بالأداة التي تختارها.

</div>

## روابط سريعة

<div class="grid cards" markdown>

-   __:material-book-open-variant: جديد هنا؟__

    ابدأ بـ [نظرة عامة](overview/index.md)، ثم توجّه إلى
    [البدء](getting-started/index.md) لتنزيل Suzaku وتشغيله.

-   __:material-console-line: تعمل مع واجهة سطر الأوامر؟__

    تصفّح [قائمة الأوامر](commands/index.md) والمرجع المخصص لكل أمر لأوامر
    [التحليل](commands/analysis.md) و[ملخص DFIR](commands/dfir-summary.md) و
    [الجدول الزمني لـ DFIR](commands/dfir-timeline.md).

-   __:material-puzzle: تمضي أبعد من ذلك؟__

    استكشف [الدعم الأصلي لـ Sigma](rules/index.md)، و
    [المشاريع المرافقة](resources/companion-projects.md)، وكيفية
    [المساهمة](resources/contributing.md).

</div>
