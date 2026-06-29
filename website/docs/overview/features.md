# Features

* Cross-platform support: Windows, Linux, macOS.
* Developed in Rust to be memory safe, fast and standalone.
* Scan `.json` or compressed `.json.gz` files with multi-thread performance.
* Create single easy-to-analyze timelines for forensic investigations and incident response.
* Execllent native support for IoC signatures written in easy to read/create/edit YML-based [Sigma](https://github.com/SigmaHQ/sigma) rules. (Correlation rules and all field modifiers except [expand](https://sigmahq.io/docs/basics/modifiers.html#expand) are supported.)
* Create a summary of all the API usage, metrics about the attacker (source IP addresses, geo-location, regions used, user agents, etc...) to discover abnormal activity without relying on signatures.
* Save results to CSV, JSON and JSONL.
