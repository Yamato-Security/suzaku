# Changes

## 2.0.0 [xxxx/xx/xx]

**New Features:**

- Added the `azure-timeline` command to create a DFIR timeline for Azure logs. (#109) (@fukusuket)
- New `aws-ct-search` command to search through CloudTrail logs. (#117) (@fukusuket)
- Added support for an ignore-list file (`config/aws_ignore_rule_list.txt`) to skip loading rules by UUID, so superseded/duplicate rules can stay in the repo without being loaded. (#136) (@YamatoSecurity)

**Enhancements:**

- Added a `Tags` column to the `aws-ct-timeline` and `azure-timeline` output that renders a rule's Sigma `tags` list as a single ` ¦ `-separated string (like Hayabusa) instead of dropping the list. ATT&CK tactics are abbreviated using the editable `config/mitre_tactics.txt` table shared with Hayabusa (e.g. `attack.credential-access` becomes `CredAccess`), while techniques and groups are shortened (`attack.t1562.001` becomes `T1562.001`, `attack.g0035` becomes `G0035`); both the hyphen and underscore tactic spellings are handled. JSON output keeps the value as a flat string. (#62) (@YamatoSecurity)
- Added a `-l, --localtime` option to the `aws-ct-timeline` and `azure-timeline` commands that outputs event timestamps in the machine's local timezone with an explicit UTC offset (e.g. `2023-07-10 12:27:45` becomes `2023-07-10 21:27:45+09:00` in JST) instead of UTC. Unparseable timestamps fall back to the existing UTC rendering. (#34) (@YamatoSecurity)
- Bumped `sigma-rust` to the released `v0.7.1` and updated all other dependencies to their latest versions. `sigma-rust` v0.7.1 keeps the Sigma correlation support suzaku relies on while moving its YAML backend off the deprecated `serde_yml`/`noyalib` (which parsed large unsigned 64-bit values in rules/events as lossy floats) to the actively maintained `yaml_serde`, restoring correct `u64` parsing. (@YamatoSecurity)
- Code refactored for easier handling of different log sources. (@fukusuket)
- Added support for Microsoft Graph API JSON format for Azure logs. (#113) (@fukusuket)
- `azure-timeline` now unwraps the `{ "records": [...] }` batch envelope used by Azure Monitor diagnostic-settings blobs and Event Hub messages (both whole-file and per-line), so those exports are read record-by-record instead of as a single event, and it now loads/matches the `identity_protection` (`riskdetection`) and `privileged_identity_management` (`pim`) rule types, which were previously dropped at load. (#130) (@YamatoSecurity)
- `azure-timeline` now loads and matches SigmaHQ's Microsoft 365 rules, which declare `logsource.service` as `audit`/`exchange`/`threat_detection`/`threat_management` — only `m365` was recognized before, so every upstream m365 rule was dropped at load. These services are routed through the same `Workload`/`RecordType` Unified Audit Log discriminator. (#137) (@YamatoSecurity)
- Added support for the M365 Unified Audit Log to `azure-timeline`: reads `Search-UnifiedAuditLog` CSV exports (and JSON) by unwrapping the `AuditData` column/wrapper, folds UAL Name/Value property bags (`ExtendedProperties`/`Parameters`/…) into objects so rules can match nested values (e.g. `ExtendedProperties.UserAgent`), parses single/pretty-printed record objects, no longer drops events when no time filter is set, parses the `CreationTime` timestamp, and adds an `m365` log-source service. The Azure output profile now surfaces DFIR-relevant M365 fields (`Workload`, `Operation`, `Result`, `User`, `SrcIP`, `TargetObject`, `UserAgent`, `AppId`, `LogonError`, and a `Details` summary of the change's `Parameters`/`ModifiedProperties`) instead of the previously empty Azure-Monitor-only columns. (#129) (@YamatoSecurity)
- Added `--file-date-from/--file-date-to` options that filter objects by their S3 key date prefix, distinct from the existing `--timeline-start/--timeline-end` options, which operates on in-file event timestamps. (#118) (@fukusuket)
- Added `-output-type` option for the `aws-ct-summary` command to output in JSON. (#123) (@fukusuket)

**Bug Fixes:**

- Malformed `--timeline-start` / `--timeline-end` / `--time-offset` values are now rejected up front with a clear error, instead of being parsed per-event and silently dropping **every** event (empty timeline, no warning) — e.g. a plain `--timeline-start 2024-01-01` instead of full RFC 3339. Also fixed a `parse_offset` panic on an empty offset, trailing whitespace, or a multibyte trailing character (the split index was taken from the untrimmed length). (#150) (@YamatoSecurity)
- Fixed a panic (`byte index 24 is not a char boundary`) when the end-of-scan "Rule Authors" summary truncated an author name longer than 27 bytes whose 24th byte fell mid-codepoint — routine for Japanese/CJK and other non-ASCII author names common in Sigma rule packs. Truncation now counts and cuts by characters, not bytes, so the completed run's output is no longer discarded. (#148) (@YamatoSecurity)
- Neutralized CSV/spreadsheet formula injection (CWE-1236) in report output. CSV cells come from attacker-influenceable cloud-log fields (`userAgent`, principal ARNs, error strings, …); a value beginning with `=`, `+`, `-`, `@`, tab, or CR would be evaluated as a formula when the report is opened in Excel/LibreOffice/Sheets. Such values are now prefixed with an apostrophe (spreadsheets treat it as a force-text marker) at all CSV sinks; JSON/JSONL and stdout are unchanged. (#146) (@YamatoSecurity)
- The `aws-ct-timeline`, `aws-ct-metrics`, `aws-ct-search`, and `aws-ct-summary` commands silently dropped JSONL input (one CloudTrail event, or a `{ "Records": [...] }` batch, per line): the parsers read the whole file as a single JSON document and returned no events when that failed. They now fall back to per-line JSONL parsing, and `.jsonl` files are discovered and read. (#139) (@YamatoSecurity)
- `-T, --no-frequency-timeline` option was not working so we removed it. Also fixed a logic bug in the authors display. (#110) (@fukusuket)
- Output file would get saved even if there were no results. (#114) (@fukusuket)
- `aws-ct-summary` would panic when processing a corrupt or imcomplete log file. (#119) (@fukusuket)
- `--geo-ip` would panic at startup (`invalid IP address syntax`) because the abbreviated CIDR strings used for the private-IP check (e.g. `10/8`, `172.16/12`, `2000::/3`) are no longer accepted by the `cidr` crate. Dropped the `cidr-utils` dependency and check private ranges directly with `std`'s `Ipv4Addr::is_private()` and a manual IPv6 prefix match; also populated the previously unused GeoIP country/city caches. (#132) (@fukusuket)

## 1.1.0 [2025/08/14] - Obon Release

**Enhancements:**

- `-R, --raw-output` now outputs raw logs to the terminal when `-o` is not specified. (#101) (@fukusuket)

## 1.0.1 [2025/08/07] - Black Hat Arsenal USA 2025 Release

**Bug Fixes:**

- Better error handling for invalid file and directory input. (#99) (@fukusuket)

## 1.0.0 [2025/07/31] - Black Hat Arsenal USA 2025 Release

**New Features:**

- Added support for correlation rules (`event_count`, `value_count`, `temporal`, `temporal_order`) for the `aws-ct-timeline` command. (#97) (@fukusuket)

**Enhancements:**

- Level names are now abbreviated in `aws-ct-timeline`. (#68) (@fukusuket)
- Error message output when no rules are found. (#76) (@fukusuket)
- Added `--timeline-offset`, `--timeline-start` and `--timeline-end` options to the `aws-ct-timeline` command. (#58) (@fukusuket)
- `aws-ct-timeline` now runs with multi-threading. (#32, #93) (@hach1yon)
 
## 0.2.1 [2025/05/25] - AUSCERT/SINCON Release 2

- Fixed the release name and updated the readme. (@yamatosecurity)

## 0.2.0 [2025/05/22] - AUSCERT/SINCON Release

**New Features:**

- `aws-ct-summary`: for each unique ARN, creates a summary of total events, regions used, user types, access keys, user agents, etc...  (#53) (@fukusuket)

**Enhancements:**

- Added Maxmind geolocation information to source IP addresses for the `aws-ct-timeline` and `aws-ct-summary` commands. (#16) (@fukusuket)
- Added a `-R, --raw-output` option to the `aws-ct-timeline` command to output the original JSON data when there is a detection. (#67) (@fukusuket)

**Bug Fixes:**

- The CSV headers for the `aws-ct-metrics` command were incorrect. (#72) (@fukusuket)

## 0.1.1 [2025/04/24] - AlphaOne Release

**Bug Fixes:**

- Some Sigma fields were not being outputted properly. (#61) (@fukusuket)

# Initial Release

## 0.1.0 [2025/04/20] - AlphaOne Release

**New Features:**

- `aws-ct-metrics`: create metrics for AWS CloudTrail events
- `aws-ct-timeline`: perform sigma detection on AWS CloudTrail logs
- `update-rules`: update sigma rules