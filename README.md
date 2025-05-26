<div align="center">
 <p>
    <img alt="Suzaku Logo" src="logo.jpeg" width="60%">
 </p>
 [ <b>English</b> ] | [<a href="README-Japanese.md">日本語</a>]
</div>

---

<p align="center">
    <a href="https://gist.github.com/cheerfulstoic/d107229326a01ff0f333a1d3476e068d"><img src="https://img.shields.io/badge/Maintenance%20Level-Actively%20Developed-brightgreen.svg" /></a>
    <a href="https://github.com/Yamato-Security/suzaku/commits/main/"><img src="https://img.shields.io/github/commit-activity/t/Yamato-Security/suzaku/main" /></a>
    <a href="https://twitter.com/SecurityYamato"><img src="https://img.shields.io/twitter/follow/SecurityYamato?style=social"/></a>
</p>


# About Suzaku

Suzaku (朱雀) means ["Vermilion Bird"](https://en.wikipedia.org/wiki/Vermilion_Bird) who is a god who flies above the clouds ruling over the southern heavens in [Chinese mythology](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku is a threat hunting and fast forensics timeline generator for cloud logs.
(Imagine [Hayabusa](https://github.com/Yamato-Security/hayabusa) but for cloud logs instead of Windows event logs.)
It is currently under active development with basic native [Sigma](https://github.com/SigmaHQ/sigma) detection support for AWS CloudTrail logs.
After AWS, we plan on supporting Azure and GCP logs.

With cloud logs, there are thousands of different API calls and more events then anyone could sift through manually.
Suzaku is designed to not just find the attacks amongst the noise, but also to provide you with a DFIR timeline that contains only the events and data you need to perform a fast-forensics investigation.
We also plan on creating summaries, search capabilities, etc... in order to quickly discover what happened at a high level as well as not miss any events that an attacker performed after you discover them. 

# Companion Projects

* [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) - our repository of Sigma rules. New upstream Sigma rules are added automatically. We also host our own rules here.
* [suzaku-sample-data](https://github.com/Yamato-Security/suzaku-sample-data) - a repository of various cloud logs with attack evidence inside them used for testing Suzaku as well as for writing new detection rules.

# Table of Contents


- [About Suzaku](#about-suzaku)
- [Companion Projects](#companion-projects)
- [Table of Contents](#table-of-contents)
- [Screenshots](#screenshots)
  - [Startup](#startup)
  - [DFIR Timeline Terminal Output](#dfir-timeline-terminal-output)
  - [Detection Frequency Timeline](#detection-frequency-timeline)
  - [Results Summary](#results-summary)
- [Features](#features)
- [Downloads](#downloads)
- [Git Cloning](#git-cloning)
- [Advanced: Compiling From Source (Optional)](#advanced-compiling-from-source-optional)
  - [Updating Rust Packages](#updating-rust-packages)
  - [macOS Compiling Notes](#macos-compiling-notes)
  - [Linux Compiling Notes](#linux-compiling-notes)
  - [Cross-compiling Linux MUSL Binaries](#cross-compiling-linux-musl-binaries)
- [Running Suzaku](#running-suzaku)
  - [Windows](#windows)
    - [Error when trying to scan a file or directory with a space in the path](#error-when-trying-to-scan-a-file-or-directory-with-a-space-in-the-path)
    - [Characters not being displayed correctly](#characters-not-being-displayed-correctly)
  - [Linux](#linux)
  - [macOS](#macos)
- [Command List](#command-list)
  - [Analysis Commands:](#analysis-commands)
  - [Summary Commands:](#summary-commands)
  - [DFIR Timeline Commands:](#dfir-timeline-commands)
  - [General Commands:](#general-commands)
- [Command Details](#command-details)
  - [Analysis Commands](#analysis-commands-1)
    - [`aws-ct-metrics` command](#aws-ct-metrics-command)
    - [Command usage](#command-usage)
      - [`aws-ct-metrics` command examples](#aws-ct-metrics-command-examples)
  - [DFIR Summary Commands](#dfir-summary-commands)
    - [`aws-ct-summary` command](#aws-ct-summary-command)
      - [`AbusedAPIs-Success` example:](#abusedapis-success-example)
      - [`AbusedAPIs-Failed` example:](#abusedapis-failed-example)
      - [`OtherAPIs-Success` example:](#otherapis-success-example)
      - [`AWS-Regions` example:](#aws-regions-example)
      - [`SrcIPs` example:](#srcips-example)
      - [`UserType` example:](#usertype-example)
      - [`UserAccessKeyIDs` example:](#useraccesskeyids-example)
      - [`UserAgents` example:](#useragents-example)
    - [Command usage](#command-usage-1)
      - [`aws-ct-metrics` command example](#aws-ct-metrics-command-example)
  - [DFIR Timeline Commands](#dfir-timeline-commands-1)
    - [`aws-ct-timeline` command](#aws-ct-timeline-command)
    - [Command usage](#command-usage-2)
      - [`aws-ct-timeline` command examples](#aws-ct-timeline-command-examples)
      - [`aws-ct-timeline` output profile](#aws-ct-timeline-output-profile)
- [Contribution](#contribution)
- [Bug Submission](#bug-submission)
- [License](#license)
- [Contributors](#contributors)
- [Acknowledgements](#acknowledgements)
- [Twitter](#twitter)

# Screenshots

## Startup

![Suzaku Startup](screenshots/Startup.png)

## DFIR Timeline Terminal Output

![Terminal Output](screenshots/TerminalOutput.png)

## Detection Frequency Timeline

![Detection Frequency Timeline](screenshots/DetectionFrequencyTimeline.png)

## Results Summary

![Results Summary](screenshots/ResultsSummary.png)

# Features

* Cross-platform support: Windows, Linux, macOS.
* Developed in Rust to be memory safe and fast.
* Scans `.json` or compressed `.json.gz` files.
* Creates single easy-to-analyze timelines for forensic investigations and incident response.
* Threat hunting based on IoC signatures written in easy to read/create/edit YML-based [Sigma](https://github.com/SigmaHQ/sigma) rules.
* Save results to CSV, JSON and JSONL.

# Downloads

Please download the latest stable version of Suzaku with compiled binaries or compile the source code from the [Releases](https://github.com/Yamato-Security/suzaku/releases) page.

We provide binaries for the following architectures:
- Linux ARM 64-bit GNU (`suzaku-x.x.x-lin-aarch64-gnu`)
- Linux Intel 64-bit GNU (`suzaku-x.x.x-lin-x64-gnu`)
- Linux Intel 64-bit MUSL (`suzaku-x.x.x-lin-x64-musl`)
- macOS ARM 64-bit (`suzaku-x.x.x-mac-aarch64`)
- macOS Intel 64-bit (`suzaku-x.x.x-mac-x64`)
- Windows ARM 64-bit (`suzaku-x.x.x-win-aarch64.exe`)
- Windows Intel 64-bit (`suzaku-x.x.x-win-x64.exe`)
- Windows Intel 32-bit (`suzaku-x.x.x-win-x86.exe`)

> [For some reason the Linux ARM MUSL binary does not run properly](https://github.com/Yamato-Security/hayabusa/issues/1332) so we do not provide that binary. It is out of our control, so we plan on providing it in the future when it gets fixed.

# Git Cloning

You can `git clone` the repository with the following command and compile binary from source code:

**Warning:** The main branch of the repository is for development purposes so you may be able to access new features not yet officially released, however, there may be bugs so consider it unstable.

```bash
git clone https://github.com/Yamato-Security/suzaku.git --recursive
```

> **Note:** If you forget to use --recursive option, the `rules` folder, which is managed as a git submodule, will not be cloned.

You can sync the `rules` folder and get latest Suzaku rules with `git pull --recurse-submodules` or use the following command:

```bash
./suzaku update-rules
```

If the update fails, you may need to rename the `rules` folder and try again.

>> Caution: When updating, rules and config files in the `rules` folder are replaced with the latest rules and config files in the [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) repository.
>> Any changes you make to existing files will be overwritten, so we recommend that you make backups of any files that you edit before updating.
>> If you add **new** rules inside of the `rules` folder, they will **not** be overwritten or deleted when updating.

# Advanced: Compiling From Source (Optional)

If you have Rust installed, you can compile from source with the following command:

Note: To compile, you usually need the latest version of Rust.

```bash
cargo build --release
```

You can download the latest unstable version from the main branch or the latest stable version from the [Releases](https://github.com/Yamato-Security/suzaku/releases) page.

Be sure to periodically update Rust with:

```bash
rustup update stable
```

The compiled binary will be outputted in the `./target/release` folder.

## Updating Rust Packages

You can update to the latest Rust crates before compiling:

```bash
cargo update
```

> Please let us know if anything breaks after you update.

## macOS Compiling Notes

If you receive compile errors about openssl, you will need to install [Homebrew](https://brew.sh/) and then install the following packages:

```bash
brew install pkg-config
brew install openssl
```

## Linux Compiling Notes

If you receive compile errors about openssl, you will need to install the following package.

Ubuntu-based distros:

```bash
sudo apt install libssl-dev
```

Fedora-based distros:

```bash
sudo yum install openssl-devel
```

## Cross-compiling Linux MUSL Binaries

On a Linux OS, first install the target.

```bash
rustup install stable-x86_64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl
```

Compile with:

```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

> **Warning: Be sure to run `rustup install stable-x86_64-unknown-linux-musl` whenever there is a new stable version of Rust as `rustup update stable` will not update the compiler for cross compiling and you may receive build errors.**

The MUSL binary will be created in the `./target/x86_64-unknown-linux-musl/release/` directory.
MUSL binaries are are about 15% slower than the GNU binaries, however, they are more portable accross different versions and distributions of linux.

# Running Suzaku

## Windows

In a Command/PowerShell Prompt or Windows Terminal, just run the appropriate 32-bit or 64-bit Windows binary.

### Error when trying to scan a file or directory with a space in the path

When using the built-in Command or PowerShell prompt in Windows, you may receive an error that Suzaku was not able to load any files if there is a space in your file or directory path.
In order to load the log files properly, be sure to do the following:
1. Enclose the file or directory path with double quotes.
2. If it is a directory path, make sure that you do not include a backslash for the last character.

### Characters not being displayed correctly

With the default font `Lucida Console` on Windows, various characters used in the logo and tables will not be displayed properly.
You should change the font to `Consalas` to fix this.

## Linux

You first need to make the binary executable.

```bash
chmod +x ./suzaku
```

Then run it from the Suzaku root directory:

```bash
./suzaku
```

## macOS

From Terminal or [iTerm2](https://iterm2.com/), you first need to make the binary executable.

```bash
chmod +x ./suzaku
```

Then, try to run it from the Suzaku root directory:

```bash
./suzaku
```

On the latest version of macOS, you may receive a security error when you try to run it.
Click "Cancel" and then from System Preferences, open "Security & Privacy" and from the General tab, click "Allow Anyway".
After that, try to run it again.

```bash
./suzaku
```

A warning will pop up so just click "Open".
You should now be able to run suzaku.

# Command List

## Analysis Commands:
* `aws-ct-metrics`: Generates metrics from AWS CloudTrail logs
  
## Summary Commands:
* `aws-ct-summary`: Generates a summary of activity from user ARNs

## DFIR Timeline Commands:
* `aws-ct-timeline`: Creates an AWS CloudTrail DFIR timeline
* `update-rules`: Updates the Sigma detection rules

## General Commands:
* `help`: Print the help menu for commands

# Command Details

## Analysis Commands

### `aws-ct-metrics` command

Use this command to create metrics on fields inside AWS CloudTrail logs.
By default, it will scan the `eventName` field.
We are currently using this command to figure out which API calls are the most common in order to prioritize writing detection rules.

### Command usage
```
Usage: suzaku aws-ct-metrics [OPTIONS] <--directory <DIR>|--file <FILE>>

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Output:
  -F, --field-name <FIELD_NAME>  The field to generate metrics for [default: eventName]
  -o, --output <FILE>            Output CSV

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
  ```

#### `aws-ct-metrics` command examples

* Output a table of `eventName` API calls to screen: `./suzaku aws-ct-metrics -d ../suzaku-sample-data`
* Save to a CSV file: `./suzaku aws-ct-metrics -d ../suzaku-sample-data -o sample-metrics.csv`

## DFIR Summary Commands

### `aws-ct-summary` command

This command creates a summary of the following information based on user ARNs:
  - Total number of events (API calls) (`NumOfEvents`)
  - Timestamp of the first API call found in the logs (`FirstTimestamp`)
  - Timestamp of the last API call found in the logs (`LastTimestamp`)
  - Commonly abused API calls that were successful (`AbusedAPIs-Success`)
  - Commonly abused API calls that were attempted but failed (`AbusedAPIs-Failed`)
  - Other API calls that are not on the list of commonly abused API calls that were successful (`OtherAPIs-Success`)
  - Other API calls that are not on the list of commonly abused API calls that were attempted but failed (`OtherAPIs-Failed`)
  - AWS regions of where the API calls were made (`AWS-Regions`)
  - Source IP addresses of the API call (`SrcIPs`)
  - User types (`UserTypes`)
  - User access key IDs (`UserAccessKeyIDs`)
  - User agents of the source who made the API call (`UserAgents`)

※ Note: the API calls that are commonly abused come from the config file hosted at [https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv](https://github.com/Yamato-Security/suzaku-rules/blob/main/config/abused_aws_api_calls.csv). This file will be updated over time and will be locally synced every time you run the `update-rules` command.

These results are intended to provide analysts with information to discover compromised accounts or attacks that does not rely on specific signatures.
For example, you can check to see if certain users are calling suspicious API calls that they shouldn't be calling, using regions that are not typically used, logging in from suspicious source IP addresses or with suspicious user agents, etc...
After you found any suspicious API calls being called, source IP addresses or user agents, you can quickly determine which access keys might have been abused during that timeframe and pivot on those keywords in the original JSON logs to create a timeline of attacker activity.

> Warning: there will be a lot of data in the cells and will most likely not display well in programs like Excel. Please use Numbers on a Mac, Timeline Explorer on Windows, etc...

#### `AbusedAPIs-Success` example:
```
Unique APIs: 11 | Total APIs 477,373
415,552 - RunInstances (ec2.amazonaws.com) - Spin up EC2 instances (crypto mining, tools) (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
28,907 - GetBucketAcl (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-10-21 13:59:40)
10,095 - GetCallerIdentity (sts.amazonaws.com) - Current credentials recon (2019-08-23 06:00:07 ~ 2019-08-23 06:04:14)
9,936 - ListBuckets (s3.amazonaws.com) - S3 recon (2019-08-23 06:00:07 ~ 2019-08-23 06:14:53)
9,168 - DescribeInstances (ec2.amazonaws.com) - EC2 and network layout recon (2019-08-23 06:00:07 ~ 2019-08-23 06:04:20)
3,658 - DescribeVpcs (ec2.amazonaws.com) - EC2 and network layout recon (2019-08-21 08:03:03 ~ 2019-09-12 20:00:44)
19 - ListGroups (greengrass.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-10-19 23:49:25)
14 - DescribeInstances (opsworks.amazonaws.com) - EC2 and network layout recon (2019-08-21 08:03:03 ~ 2019-10-19 23:49:22)
12 - GetBucketPolicy (s3.amazonaws.com) - S3 recon (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
7 - ListGroups (resource-groups.amazonaws.com) - IAM enumeration (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
5 - StartInstances (ec2.amazonaws.com) - Spin up EC2 instances (crypto mining, tools) (2019-08-21 08:03:03 ~ 2019-12-12 07:07:31)
```

#### `AbusedAPIs-Failed` example:
```
Unique APIs: 23 | Total APIs 20,464
11,603 - AssumeRole (sts.amazonaws.com) - Lateral movement via roles (2019-08-21 08:03:03 ~ 2019-09-18 07:04:12)
7,279 - GetBucketAcl (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-09-09 09:01:26)
515 - GetBucketPolicy (s3.amazonaws.com) - S3 recon (2019-08-21 08:03:03 ~ 2019-10-01 19:11:07)
331 - ListUsers (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-08-29 14:53:14)
210 - ListSecrets (secretsmanager.amazonaws.com) - Find credential storage locations (2019-08-21 08:03:03 ~ 2019-10-19 23:49:30)
153 - ListGroups (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-09-12 15:24:39)
148 - ListRoles (iam.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2019-09-12 15:20:56)
112 - ListAccessKeys (iam.amazonaws.com) - Enumerates keys on IAM users (2019-08-21 08:03:03 ~ 2019-09-16 14:28:15)
31 - ListGroups (greengrass.amazonaws.com) - IAM enumeration (2019-08-21 08:03:03 ~ 2020-02-25 14:41:24)
...
```

#### `OtherAPIs-Success` example:
```
Unique APIs: 289 | Total APIs 143,759
98,689 - DescribeSnapshots (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:50:59)
10,679 - DescribeSpotPriceHistory (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:32)
3,740 - DescribeReservedInstancesOfferings (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:30)
2,372 - DescribeSnapshotAttribute (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-08-24 12:38:34)
2,307 - CreateDefaultVpc (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:17)
1,532 - DescribeKeyPairs (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:16)
1,504 - DescribeSecurityGroups (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:00:40)
1,495 - DescribeImages (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:07:26)
1,438 - CreateKeyPair (ec2.amazonaws.com) (2019-08-23 06:00:07 ~ 2019-08-23 06:04:16)
1,402 - DescribeVolumes (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-03 12:06:20)
1,217 - DescribeSubnets (ec2.amazonaws.com) (2019-08-21 08:03:03 ~ 2019-09-12 20:00:43)
...
```

#### `AWS-Regions` example:
```
Total regions: 16
167,155 - us-west-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:14:53)
113,328 - us-east-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:04:14)
65,718 - ap-northeast-2 (2019-08-23 06:00:07 ~ 2019-08-23 06:22:55)
64,787 - ap-northeast-1 (2019-08-23 06:00:07 ~ 2019-08-23 06:34:57)
...
```

#### `SrcIPs` example:
```
Total source IPs: 5,293
634,454 - 5.205.62.253 (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
23,498 - 193.29.252.218 (2019-08-21 08:03:00 ~ 2019-10-17 09:11:22)
15,925 - 155.63.17.217 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
9,067 - 253.0.255.253 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
7,078 - 163.21.250.220 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
6,575 - 236.9.245.88 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
5,138 - 84.252.252.117 (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
4,946 - 24.251.252.2 (2019-08-21 08:03:00 ~ 2019-09-30 06:36:13)
4,225 - 211.111.151.81 (2019-08-21 08:03:00 ~ 2019-09-12 19:53:35)
...
```

#### `UserType` example:
```
IAMUser
```

#### `UserAccessKeyIDs` example:
```
Total access key ids: 629
667,476 - AKIA01U43UX3RBRDXF4Q (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
218,544 - ASIARF55FBMFZBXLKDFW (2019-08-21 11:31:47 ~ 2019-08-23 13:00:28)
12,677 - AKIA1ZBTOEKWKVHP6GHZ (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
8,822 - ASIAGD2JRX0V6RJGWR59 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
4,940 - ASIAUNHV6EHIK5MNPEKF (2019-08-21 08:03:00 ~ 2019-09-30 06:36:17)
...
```

> Note that since there will be a large amount of temporary AWS STS access key IDs, these are filtered out by default. Add the `-s, --include-sts-keys` option if you want to include these.

#### `UserAgents` example:
```
Total user agents: 7,760
351,022 - Boto3/1.9.201 Python/2.7.12 Linux/4.4.0-159-generic Botocore/1.12.201 (2019-08-23 06:00:07 ~ 2019-08-23 06:00:07)
283,430 - Boto3/1.9.201 Python/2.7.12 Linux/4.4.0-157-generic Botocore/1.12.201 (2019-08-21 11:31:47 ~ 2019-08-23 13:00:28)
23,467 - [Boto3/1.15.13 Python/3.8.5 Darwin/19.6.0 Botocore/1.18.13 Resource] (2017-02-12 21:15:12 ~ 2020-10-07 16:05:52)
15,924 - Boto3/1.7.4 Python/2.7.12 Linux/4.4.0-119-generic Botocore/1.10.4 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
9,400 - aws-sdk-java/1.11.301 Linux/4.9.93-linuxkit-aufs Java_HotSpot(TM)_64-Bit_Server_VM/25.172-b11 java/1.8.0_172 (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
6,372 - [Boto3/1.7.65 Python/3.5.2 Linux/4.13.0-37-generic Botocore/1.10.65 Resource] (2018-04-17 10:09:00 ~ 2020-09-21 21:06:22)
5,352 - AWSPowerShell/3.3.365.0 .NET_Runtime/4.0 .NET_Framework/4.0 OS/Microsoft_Windows_NT_10.0.01985.0 WindowsPowerShell/5.0 ClientSync (2019-01-08 20:30:01 ~ 2020-03-29 09:06:56)
4,945 - Boto3/1.9.231 Python/2.7.15+ Linux/4.15.0-64-generic Botocore/1.12.231 (2019-08-21 08:03:00 ~ 2019-09-30 06:36:13)
4,599 - [aws-cli/1.16.301 Python/3.7.6 Linux/5.4.0-kali3-amd64 botocore/1.13.37] (2019-08-21 08:03:00 ~ 2020-02-09 22:00:32)
3,909 - Boto3/1.14.28 Python/3.8.5 Linux/5.7.0-kali1-amd64 Botocore/1.17.28 (2019-01-08 20:30:01 ~ 2020-09-11 17:35:39)
3,450 - Boto3/1.4.2 Python/2.7.13+ Linux/4.9.0-3-amd64 Botocore/1.5.19 (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
3,198 - Boto3/1.4.2 Python/2.7.14 Linux/4.13.0-1-amd64 Botocore/1.5.19 (2017-02-12 21:15:12 ~ 2020-09-21 21:06:22)
...
```

> Note that the `aws` client tool will include the OS information in the user agent so it is possible to detect if there were API calls made from attacker OSes like `kali`.


### Command usage
```
Usage:
  suzaku aws-ct-summary <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Filtering:
  -s, --include-sts-keys  Include temporary AWS STS access key IDs

Output:
  -o, --output <FILE>           Output results to a CSV file
  -D, --hide-descriptions       Hide description of the commonly abused API calls
  -G, --GeoIP <MAXMIND-DB-DIR>  Add GeoIP (ASN, city, country) info to IP addresses

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner

General Options:
  -h, --help  Show the help menu
  ```

#### `aws-ct-metrics` command example

* Save results to a CSV file: `./suzaku aws-ct-summary -d ../suzaku-sample-data -o sample-summary.csv`

## DFIR Timeline Commands

### `aws-ct-timeline` command

Create an AWS CloudTrail DFIR timeline based on Sigma rules in the `rules` folder.

### Command usage
```
Usage: suzaku aws-ct-timeline [OPTIONS] <--directory <DIR>|--file <FILE>>

General Options:
  -r, --rules <DIR/FILE>  Specify a custom rule directory or file (default: ./rules)
  -h, --help              Show the help menu

Input:
  -d, --directory <DIR>  Directory of multiple gz/json files
  -f, --file <FILE>      File path to one gz/json file

Output:
  -o, --output <FILE>              Save the results to a file
  -t, --output-type <OUTPUT_TYPE>  Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL [default: 1]
  -C, --clobber                    Overwrite files when saving
  -G, --GeoIP <MAXMIND-DB-DIR>     Add GeoIP (ASN, city, country) info to IP addresses
  -R, --raw-output                 Output the original JSON logs (only available in JSON formats)

Display Settings:
  -K, --no-color               Disable color output
  -N, --no-summary             Do not display results summary
  -T, --no-frequency-timeline  Disable event frequency timeline (terminal needs to support Unicode)
  -q, --quiet                  Quiet mode: do not display the launch banner
  ```

#### `aws-ct-timeline` command examples

* Output alerts to screen: `./suzaku aws-ct-timeline -d ../suzaku-sample-data`
* Save results to a CSV file: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline.csv`
* Save results to CSV and JSONL files: `./suzaku aws-ct-timeline -d ../suzaku-sample-data -o sample-timeline -t 5`

#### `aws-ct-timeline` output profile

Suzaku will output information based on the `config/default_profile.yaml` file:
```yaml
Timestamp: '.eventTime'
RuleTitle: 'sigma.title'
RuleAuthor: 'sigma.author'
Level: 'sigma.level'
EventName: '.eventName'
EventSource: '.eventSource'
AWS-Region: '.awsRegion'
SrcIP: '.sourceIPAddress'
UserAgent: '.userAgent'
UserName: '.userIdentity.userName'
UserType: '.userIdentity.type'
UserAccountID: '.userIdentity.accountId'
UserARN: '.userIdentity.arn'
UserPrincipalID: '.userIdentity.principalId'
UserAccessKeyID: '.userIdentity.accessKeyId'
EventID: '.eventID'
RuleID: 'sigma.id'
```

* Any field value that starts with `.` (ex: `.eventTime`) will be taken from the CloudTrail log.
* Any field value that starts with `sigma.` (ex: `sigma.title`) will be taken from the Sigma rule.
* Currently we only support strings but plan on supporting other types of field values.

# Contribution

We would love any form of contribution.
Pull requests, rule creation and sample logs are the best but feature requests, notifying us of bugs, etc... are also very welcome.

At the least, **if you like our tools and resources then please give us a star on GitHub and show your support!**

# Bug Submission

* Please submit any bugs you find [here.](https://github.com/Yamato-Security/suzaku/issues/new?assignees=&labels=bug&template=bug_report.md&title=%5Bbug%5D)
* This project is currently actively maintained and we are happy to fix any bugs reported.
* If you find any issues (false positives, bugs, etc...) with Suzaku rules, please report them to the suzaku-rules GitHub issues page [here](https://github.com/Yamato-Security/suzaku-rules/issues/new).
* If you find any issues (false positives, bugs, etc...) with Sigma rules, please report them to the upstream SigmaHQ GitHub issues page [here](https://github.com/SigmaHQ/sigma/issues).

# License

* Suzaku is released under [AGPLv3](https://www.gnu.org/licenses/agpl-3.0.en.html) and all rules are released under the [Detection Rule License (DRL) 1.1](https://github.com/SigmaHQ/sigma/blob/master/LICENSE.Detection.Rules.md).
* You may freely use Suzaku internally, SaaS solutions, for consulting work, etc...
However, if you use Suzaku in a type of SaaS solution and make improvements to it, we ask you to open-source those improvements and give back to the project.

# Contributors

* DustInDark (core developer)
* Fukusuke Takahashi (core developer)
* Zach Mathis (project leader, tool design, rules, testing, etc...) (@yamatosecurity)

# Acknowledgements

* [Flaws.cloud](http://flaws.cloud/)
* [Invictus-ir](https://www.invictus-ir.com/)
* [Sigma](https://github.com/SigmaHQ/sigma)
* [sigma-rust](https://github.com/jopohl/sigma-rust)
* [Stratus Red Team](https://stratus-red-team.cloud/)
* [Traildiscover.cloud](https://traildiscover.cloud/)

# Twitter

You can receive the latest news about Suzaku, rule updates, other Yamato Security tools, etc... by following us on Twitter at [@SecurityYamato](https://twitter.com/SecurityYamato).