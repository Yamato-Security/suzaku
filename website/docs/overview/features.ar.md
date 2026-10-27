# الميزات

* دعم متعدد المنصات: Windows، Linux، macOS.
* مطوّر بلغة Rust ليكون آمناً للذاكرة وسريعاً ومستقلاً.
* تحليل سجلات AWS CloudTrail وAzure/Entra ID/M365 وGoogle Workspace Reports API عبر اكتشاف Sigma الأصلي.
* فحص `.json` و`.jsonl` و`.json.gz` المضغوط وParquet و[ملفات CloudTrail CSV المدعومة من AWS Management Console](../getting-started/console-csv-input.md) بأداء متعدد الخيوط.
* إنشاء جداول زمنية مفردة سهلة التحليل للتحقيقات الجنائية الرقمية والاستجابة للحوادث.
* دعم أصلي ممتاز لتوقيعات مؤشرات الاختراق (IoC) المكتوبة بصيغة قواعد [Sigma](https://github.com/SigmaHQ/sigma) المبنية على YML والسهلة القراءة/الإنشاء/التحرير. (قواعد الارتباط وجميع معدّلات الحقول مدعومة باستثناء [expand](https://sigmahq.io/docs/basics/modifiers.html#expand).)
* إنشاء ملخص لجميع استخدامات الـ API، ومقاييس حول المهاجم (عناوين IP المصدر، الموقع الجغرافي، المناطق المستخدمة، وكلاء المستخدم، إلخ...) لاكتشاف النشاط غير الطبيعي دون الاعتماد على التوقيعات.
* حفظ النتائج بصيغ CSV وJSON وJSONL وDuckDB.
