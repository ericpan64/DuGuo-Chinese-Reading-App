# DuGuo
[![docs: 0.2.0](https://img.shields.io/badge/Docs-0.2.0-blue)](https://duguo.app/static/doc/duguo/index.html)
[![License: AGPL](https://img.shields.io/badge/License-AGPL-yellow.svg)](https://www.gnu.org/licenses/agpl-3.0.en.html)

## Overview
DuGuo is an open-source web application that allows users to read Chinese text in an interactive learning environment. The main features include:
- Phonetic support (Pinyin + Zhuyin) and phrase lookup via [CC-CEDICT](https://cc-cedict.org/wiki/)
- Phrase tokenization via [spaCy](https://spacy.io/)
- Text-to-speech via the [SpeechSynthesis API](https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesis)
- Transposition between Simplified + Traditional Chinese text
- ... other ideas tbd - view + contribute in the [Issues](https://github.com/ericpan64/DuGuo-Chinese-Reading-App/issues) tab!

This app is designed in particular for L2 (second-language) learners, though hopefully it is useful for all levels of Chinese learning! 

Check it out at [duguo.app](https://duguo.app)! "Production" is hosted on GCP while this repository contains all code and configuration to run an instance locally using docker-compose (from the root directory, run `docker-compose up`).

### Tech Stack
The app has 2 microservices:
1. A web server written in Rust using [Rocket](https://rocket.rs/)
2. An NLP tokenization service written in Python primarily using [spaCy's Chinese module](https://spacy.io/models/zh) (which builds on top of [jieba](https://github.com/fxsjy/jieba))
    - [OpenCC](https://github.com/BYVoid/OpenCC) and [pypinyin](https://github.com/mozillazg/python-pinyin) are used during processing. 

For data persistance, [mongoDB](https://www.mongodb.com/) and [Redis](https://redis.io/) are used.

Tokenized words are looked-up in the [CC-CEDICT](https://cc-cedict.org/wiki/) which is generously available under a Creative Commons license. Radical information (for saved vocab) is sourced from [this web API](http://ccdb.hemiola.com/) and can be quickly accessed using the accompanying [Hemiola Chinese Character Browser](http://hanzi.hemiola.com/).

## Motivation
Learning Chinese as a second language is hard for many reasons. To start, Chinese characters are logographic whereas English characters are alphabetic - this necessitates a fundamentally different approach to phrase memorization. Additionally, phrase pronunciation requires learning technical phonetic syntax (e.g. pinyin) which is rarely used by natives and virtually non-existant in practice.

While there are many more nuanced approaches to Chinese learning (e.g. the [HSK framework](https://en.wikipedia.org/wiki/Hanyu_Shuiping_Kaoshi)), one simplified view is that there are 3 levels of Chinese reading mastery:
1. Almost entirely pinyin-dependent (for beginners and L2 learners that can speak but can't read, like myself...)
2. Some pinyin needed (roughly grade-school level for native Chinese speakers)
3. Almost no pinyin needed (adult level - phrases are either memorized or able to be intuited based on the context)

Below are images to provide a visual reference. While for natives the jump from tier 1 to 3 is trivial, for L2 learners it can feel insurmountable!

1. [<img src="design/images/textbook-beginner.jpg" alt="A beginner-level Chinese textbook with pinyin included for all words ('Tier 1')." width="350">](design/images/textbook-beginner.jpg)
2. [<img src="design/images/textbook-intermediate.jpg" alt="An intermediate-level Chinese textbook with pinyin for some words ('Tier 2'). In practice, this is grade-school level for natives!" width="350">](design/images/textbook-intermediate.jpg)
3. [<img src="design/images/newspaper-hard.jpg" alt="A native-level article from a Chinese newspaper ('Tier 3'). No pinyin is used at all, since natives don't really need it!" width="350">](design/images/newspaper-hard.jpg)

### Contextual Learning 
Contextual learning is arguably the best way to learn a language. People remember things that are linked to experiences or assorted significant pieces of information. For natives, learning Chinese is essential. However for L2 learners, finding the urgency to learn is uniquely difficult without an external driving force (e.g. living in a Chinese-speaking country).

Barring the ability to live in a foreign country, DuGuo hopes to offer the next-best thing by allowing users to pick what they want to read (improving contextual relevance) and saving contextual references for "learned" phrases (adding contextual triggers).

## Other Existing Tools
There are several existing tools that provide similar functionality, including (but not limited to): [Zhongwen Chrome Extension](https://chrome.google.com/webstore/detail/zhongwen-chinese-english/kkmlkkjojmombglmlpbpapmhcaljjkde?hl=en), [Purple Culture Pinyin Converter](https://www.purpleculture.net/chinese-pinyin-converter/), [Du Chinese (mobile)](https://www.duchinese.net/), [mdbg.net](https://www.mdbg.net/chinese/dictionary), [Hànzì Analyzer](http://hemiola.com/), [pin1yin1](https://www.pin1yin1.com/), etc.

The main differentiators DuGuo hopes to provide with this project are improved UX, progress persistance (via accounts), document difficulty scoring (in progress), and [Duey](app/static/img/duey/duey_extra_happy.png)! Ultimately this is provided as an additional tool to help users learn Chinese, so definitely use the combination of tools that best supplements your learning experience.

## Acknowledgement

This project was adopted from Martin Kess's previous CS6460 final project, the Chinese Reading Machine (中文读机). He provided the starter code (in Python Flask) and a strong existing framework to build on. The images for Duey came from Dzaky Taufik (his Upwork linked [here](https://www.upwork.com/freelancers/~013f8e6de5a2a64421)). 感谢 and 大家加油!

[<img src="app/static/img/duey/duey_base_normal.png" alt="Duey!" width="150">](app/static/img/duey/duey_base_normal.png)
[<img src="app/static/img/duey/duey_base_confused.png" alt="Confused Duey?" width="150">](app/static/img/duey/duey_base_confused.png)
[<img src="app/static/img/duey/duey_base_surprised.png" alt="Surprised Duey" width="150">](app/static/img/duey/duey_base_surprised.png)
[<img src="app/static/img/duey/duey_base_worried.png" alt="Worried Duey :-(" width="150">](app/static/img/duey/duey_base_worried.png)
[<img src="app/static/img/duey/duey_base_happy.png" alt="Happy Duey!" width="150">](app/static/img/duey/duey_base_happy.png)