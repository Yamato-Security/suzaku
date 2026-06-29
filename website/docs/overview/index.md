# About Suzaku

Suzaku (朱雀) means ["Vermilion Bird"](https://en.wikipedia.org/wiki/Vermilion_Bird) who is a god who flies above the clouds ruling over the southern heavens in [Chinese mythology](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku is a threat hunting and fast forensics timeline generator for cloud logs.
(Imagine [Hayabusa](https://github.com/Yamato-Security/hayabusa) but for cloud logs instead of Windows event logs.)
It is currently under active development with native [Sigma](https://github.com/SigmaHQ/sigma) detection support for AWS CloudTrail logs.
We plan on supporting Azure and GCP logs as well.

With cloud logs, there are thousands of different API calls and more events then anyone could sift through manually.
Suzaku is designed to not just find the attacks amongst the noise, but also to provide you with a DFIR timeline that contains only the events and data you need to perform a fast-forensics investigation.
You can also create summaries in order to quickly discover what happened at a high level, discover abnormal behavior not relying on signatures and easily find keywords such as IP addresses, user agents, regions, geo-location, etc... to pivot on and not miss any events that an attacker performed after you discover them.
