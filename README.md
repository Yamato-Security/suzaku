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


## Main Goals

# About Suzaku

Suzaku (朱雀) means ["Vermilion Bird"](https://en.wikipedia.org/wiki/Vermilion_Bird) who is a god who flies above the clouds ruling over the southern heavens in [Chinese mythology](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku is a threat hunting and fast forensics timeline generator for cloud logs.
(Imagine [Hayabusa](https://github.com/Yamato-Security/hayabusa) but for cloud logs instead of Windows event logs.)
It is currently under active development with basic native [sigma](https://github.com/SigmaHQ/sigma) detection support AWS CloudTrail logs.
After AWS, we plan on supporting Azure and GCP logs.

With cloud logs, there are thousands of different API calls and more events then anyone could sift through manually.
Suzaku is designed to not just find the attacks amongst the noise, but also to provide you with a DFIR timeline that contains only the events and data you need to perform a forensics investigation.
We also plan on creating summaries, search capabilities, etc... in order to quickly discover what happened at a high level and not miss any events that an attacker performed after you discover them. 

# Companion Projects

* [suzaku-sample-data](https://github.com/Yamato-Security/suzaku-sample-data) - a repository of various cloud logs with attack evidence inside them used for testing Suzaku as well as for writing new detection rules.
* [suzaku-rules](https://github.com/Yamato-Security/suzaku-rules) - our repository of sigma rules. New upstream sigma rules are added automatically. We also host our own rules here.

# Table of Contents


- [About Suzaku](#about-suzaku)
- [Companion Projects](#companion-projects)
- [Table of Contents](#table-of-contents)
- [Contributors](#contributors)
- [Acknowledgements](#acknowledgements)


# Contributors

* DustInDark (core developer)
* Fukusuke Takahashi (core developer)
* Zach Mathis (tool design, rules, testing, etc...) (@yamatosecurity)

# Acknowledgements

* The [Sigma](https://github.com/SigmaHQ/sigma) project
* [Traildiscover.cloud](https://traildiscover.cloud/)
* [sigma-rust](https://github.com/jopohl/sigma-rust)