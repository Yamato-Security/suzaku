---
hide:
  - navigation
  - toc
---

<div class="hb-hero" markdown>

![Suzaku](assets/logo.jpeg){ .hb-logo }

<p class="hb-tagline">
<strong>Suzaku</strong> (朱雀) คือ <strong>เครื่องมือสร้างไทม์ไลน์สำหรับการล่าภัยคุกคามแบบ Sigma และการพิสูจน์หลักฐานอย่างรวดเร็วสำหรับล็อกบนคลาวด์</strong> สร้างโดย <a href="https://github.com/Yamato-Security">Yamato
Security</a> และเขียนด้วย <a href="https://www.rust-lang.org/">Rust</a> ลองนึกถึง
<a href="https://github.com/Yamato-Security/hayabusa">Hayabusa</a> แต่สำหรับล็อกบนคลาวด์แทนที่จะเป็น
ล็อกเหตุการณ์ของ Windows — พร้อมการรองรับ <a href="https://github.com/SigmaHQ/sigma">Sigma</a> โดยกำเนิดสำหรับ
AWS CloudTrail (มีแผนรองรับ Azure และ GCP)
</p>

<div class="hb-cta" markdown>
[เริ่มต้นใช้งาน :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[เอกสารอ้างอิงคำสั่ง :material-console:](commands/index.md){ .md-button }
[ดูบน GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/suzaku){ .md-button }
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

## ทำไมต้อง Suzaku?

<div class="grid cards" markdown>

-   :material-cloud-search:{ .lg .middle } __Sigma แบบ Cloud-native__

    ---

    การตรวจจับด้วย [Sigma](https://github.com/SigmaHQ/sigma) โดยกำเนิดสำหรับล็อกบนคลาวด์ — AWS CloudTrail ในวันนี้
    มีแผนรองรับ Azure และ GCP รองรับกฎแบบ correlation และ field modifier เกือบทั้งหมด

-   :material-timeline-clock:{ .lg .middle } __ไทม์ไลน์การพิสูจน์หลักฐานที่รวดเร็ว__

    ---

    เปลี่ยนการเรียก API บนคลาวด์ที่สับสนนับพันครั้งให้เป็น **ไทม์ไลน์ DFIR** เดียวที่วิเคราะห์ได้ง่าย โดยมีเฉพาะ
    เหตุการณ์ที่คุณต้องการ

-   :material-flash:{ .lg .middle } __รวดเร็วสุดขีดด้วย Rust__

    ---

    ปลอดภัยด้านหน่วยความจำ ทำงานแบบหลายเธรด และทำงานได้อย่างอิสระ สแกนล็อก `.json` และล็อกบีบอัด `.json.gz` บน
    Windows, Linux และ macOS

-   :material-chart-box:{ .lg .middle } __สรุปข้อมูลผู้โจมตี__

    ---

    สรุปการใช้งาน API และเมตริกของผู้โจมตี — IP ต้นทาง ตำแหน่งทางภูมิศาสตร์ ภูมิภาค user agent — เพื่อ
    pivot ได้อย่างรวดเร็ว

-   :material-shield-search:{ .lg .middle } __การตรวจจับพฤติกรรม__

    ---

    เผยกิจกรรมที่ผิดปกติ **โดยไม่ต้องพึ่งพา signature** เพื่อให้คุณไม่พลาดการโจมตีรูปแบบใหม่

-   :material-export:{ .lg .middle } __เอาต์พุตที่ยืดหยุ่น__

    ---

    บันทึกผลลัพธ์เป็น **CSV, JSON และ JSONL** เพื่อวิเคราะห์ด้วยเครื่องมือที่คุณเลือก

</div>

## ลิงก์ด่วน

<div class="grid cards" markdown>

-   __:material-book-open-variant: เพิ่งเข้ามาใหม่?__

    เริ่มต้นด้วย [ภาพรวม](overview/index.md) จากนั้นไปที่
    [การเริ่มต้นใช้งาน](getting-started/index.md) เพื่อดาวน์โหลดและรัน Suzaku

-   __:material-console-line: ทำงานกับ CLI?__

    เรียกดู [รายการคำสั่ง](commands/index.md) และเอกสารอ้างอิงรายคำสั่งสำหรับคำสั่ง
    [Analysis](commands/analysis.md), [DFIR Summary](commands/dfir-summary.md) และ
    [DFIR Timeline](commands/dfir-timeline.md)

-   __:material-puzzle: ต้องการเจาะลึกกว่านี้?__

    สำรวจ [การรองรับ Sigma โดยกำเนิด](rules/index.md),
    [โครงการที่เกี่ยวข้อง](resources/companion-projects.md), และวิธีการ
    [มีส่วนร่วม](resources/contributing.md)

</div>
