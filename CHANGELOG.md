# Changes

## 0.2.0 [2025/05/22] - AUSCERT Release

**New Features:**

- `aws-ct-summary`: for each unique ARN, creates a summary of total events, regions used, user types, access keys, user agents, etc...  (#53) (@fukusuket)

## 0.1.1 [2025/04/24] - AlphaOne Release

**Bug Fixes:**

- Some Sigma fields were not being outputted properly. (#61) (@fukusuket)

# Initial Release

## 0.1.0 [2025/04/20] - AlphaOne Release

**New Features:**

- `aws-ct-metrics`: create metrics for AWS CloudTrail events
- `aws-ct-timeline`: perform sigma detection on AWS CloudTrail logs
- `update-rules`: update sigma rules