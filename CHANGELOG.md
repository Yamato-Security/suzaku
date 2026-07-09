# Changes

## 2.0.0 [xxxx/xx/xx]

**New Features:**

- Added the `azure-timeline` command to create a DFIR timeline for Azure logs. (#109) (@fukusuket)
- New `aws-ct-search` command to search through CloudTrail logs. (#117) (@fukusuket)

**Enhancements:**

- Code refactored for easier handling of different log sources. (@fukusuket)
- Added support for Microsoft Graph API JSON format for Azure logs. (#113) (@fukusuket)
- `azure-timeline` now unwraps the `{ "records": [...] }` batch envelope used by Azure Monitor diagnostic-settings blobs and Event Hub messages (both whole-file and per-line), so those exports are read record-by-record instead of as a single event, and it now loads/matches the `identity_protection` (`riskdetection`) and `privileged_identity_management` (`pim`) rule types, which were previously dropped at load. (#130) (@YamatoSecurity)
- Added support for the M365 Unified Audit Log to `azure-timeline`: reads `Search-UnifiedAuditLog` CSV exports (and JSON) by unwrapping the `AuditData` column/wrapper, folds UAL Name/Value property bags (`ExtendedProperties`/`Parameters`/…) into objects so rules can match nested values (e.g. `ExtendedProperties.UserAgent`), parses single/pretty-printed record objects, no longer drops events when no time filter is set, parses the `CreationTime` timestamp, and adds an `m365` log-source service. The Azure output profile now surfaces DFIR-relevant M365 fields (`Workload`, `Operation`, `Result`, `User`, `SrcIP`, `TargetObject`, `UserAgent`, `AppId`, `LogonError`, and a `Details` summary of the change's `Parameters`/`ModifiedProperties`) instead of the previously empty Azure-Monitor-only columns. (#129) (@YamatoSecurity)
- Added `--file-date-from/--file-date-to` options that filter objects by their S3 key date prefix, distinct from the existing `--timeline-start/--timeline-end` options, which operates on in-file event timestamps. (#118) (@fukusuket)
- Added `-output-type` option for the `aws-ct-summary` command to output in JSON. (#123) (@fukusuket)

**Bug Fixes:**

- `-T, --no-frequency-timeline` option was not working so we removed it. Also fixed a logic bug in the authors display. (#110) (@fukusuket)
- Output file would get saved even if there were no results. (#114) (@fukusuket)
- `aws-ct-summary` would panic when processing a corrupt or imcomplete log file. (#119) (@fukusuket)

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