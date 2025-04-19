<div align="center">
 <p>
    <img alt="Suzaku Logo" src="logo.jpeg" width="60%">
 </p>
 [ <b>English</b> ] | [<a href="README-Japanese.md">日本語</a>]
</div>

---

<p align="center">
    <a href="Maintenance Level"><img src="https://img.shields.io/badge/Maintenance%20Level-Actively%20Developed-brightgreen.svg" /></a>
    <a href="Total Commits"><img src="https://img.shields.io/github/commit-activity/t/Yamato-Security/suzaku/main" /></a>
    <a href="https://twitter.com/SecurityYamato"><img src="https://img.shields.io/twitter/follow/SecurityYamato?style=social"/></a>
</p>


# About Suzaku

Suzaku (朱雀) means ["Vermilion Bird"](https://en.wikipedia.org/wiki/Vermilion_Bird) who is a god who flies above the clouds ruling over the southern heavens in [Chinese mythology](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku is a threat hunting and fast forensics timeline generator for cloud logs.
(Imagine [Hayabusa](https://github.com/Yamato-Security/hayabusa) but for cloud logs instead of Windows event logs.)
It is currently under active development with basic native [sigma](https://github.com/SigmaHQ/sigma) detection support for AWS CloudTrail logs.
After AWS, we plan on supporting Azure and GCP logs.

With cloud logs, there are thousands of different API calls and more events then anyone could sift through manually.
Suzaku is designed to not just find the attacks amongst the noise, but also to provide you with a DFIR timeline that contains only the events and data you need to perform a fast-forensics investigation.
We also plan on creating summaries, search capabilities, etc... in order to quickly discover what happened at a high level as well as not miss any events that an attacker performed after you discover them. 

# Companion Projects

* [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) - our repository of sigma rules. New upstream sigma rules are added automatically. We also host our own rules here.
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
  - [DFIR Timeline Commands:](#dfir-timeline-commands)
  - [General Commands:](#general-commands)
- [Command Usage](#command-usage)
  - [Analysis Commands](#analysis-commands-1)
    - [`aws-ct-metrics` command](#aws-ct-metrics-command)
      - [`aws-ct-metrics` command examples](#aws-ct-metrics-command-examples)
    - [`aws-ct-timeline` command](#aws-ct-timeline-command)
      - [`aws-ct-timeline` command examples](#aws-ct-timeline-command-examples)
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
  
## DFIR Timeline Commands:
* `aws-ct-metrics`: Creates an AWS CloudTrail DFIR timeline
* `update-rules`: Updates the Sigma detection rules

## General Commands:
* `help`: Print the help menu for commands

# Command Usage

## Analysis Commands

### `aws-ct-metrics` command

Use this command to create metrics on fields inside AWS CloudTrail logs.
By default, it will scan the `eventName` field.
We are currently using this command to figure out which API calls are the most common in order to prioritize writing detection rules.

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

### `aws-ct-timeline` command

Create an AWS CloudTrail DFIR timeline based on sigma rules in the `rules` folder.

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

You can recieve the latest news about Suzaku, rule updates, other Yamato Security tools, etc... by following us on Twitter at [@SecurityYamato](https://twitter.com/SecurityYamato).