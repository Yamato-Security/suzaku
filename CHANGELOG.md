# Changes

## 1.0.1 [xxxx/xx/xx] - Black Hat Arsenal USA 2025 Release

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