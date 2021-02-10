# DuGuo
[![docs: 0.1.0](https://img.shields.io/badge/Docs-0.1.0-blue)](https://duguo-app.com/static/doc/duguo/index.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
## Overview

DuGuo is a web application that allows users to read Chinese text in an interactive learning environment including pinyin support, speech-to-text, and a lookup dictionary. Building from existing solutions, DuGuo aims to provide the best UX for contextual learning while remaining open-source. This app is designed in particular for L2 (second-language) learners, though hopefully it is useful for all levels of Chinese learning! 

### Deployment

The app is currently available at [duguo.app](https://duguo.app) (redirects to [duguo-app.com](https://duguo-app.com)). The production deployment is hosted on AWS. This repository contains all code and configuration to run an instance locally using docker-compose (from the root directory, run `docker-compose up`).

### Tech Stack

The app is made of 3 components:
1. A web server written in Rust using [Rocket](https://rocket.rs/) and other assorted libraries (see the [Cargo.toml](app/Cargo.toml) file). 
2. An NLP tokenization service written in Python using [spaCy's Chinese module](https://spacy.io/models/zh) (which builds on top of [jieba](https://github.com/fxsjy/jieba)). [OpenCC](https://github.com/BYVoid/OpenCC) and [pypinyin](https://github.com/mozillazg/python-pinyin) are used during processing. 
3. Data persistance via a database ([mongoDB](https://www.mongodb.com/)) and a cache ([Redis](https://redis.io/)).

Tokenized words are looked-up in the [CC-CEDICT](https://cc-cedict.org/wiki/) which is generously available for use under a Creative Commons license. Radical information (for saved vocab) is sourced from [this web API](http://ccdb.hemiola.com/) and can be quickly accessed using the accompanying [Hemiola Chinese Character Browser](http://hanzi.hemiola.com/).

## Motivation

Learning Chinese as a second language is hard for many reasons: it is logographic (whereas English is alphabetic) which necessitates extensive memorization, and starting out requires learning technical phonetic syntax (pinyin) which is quickly deprecated and virtually non-existant in practice.

While there are many more nuanced approaches to Chinese learning (e.g. the [HSK framework](https://en.wikipedia.org/wiki/Hanyu_Shuiping_Kaoshi)), in my simple opinion there are really 3 "tiers" that need to be mastered for Chinese reading:

1. All-pinyin (for absolute beginners)
2. Some-pinyin (roughly grade-school level for native Chinese speakers)
3. No-pinyin (adult level)

Below are images to provide a visual reference. While for natives the jump from tier 1 to 3 is trivial, for L2 learners it can feel insurmountable!

[<img src="design/images/textbook-beginner.jpg" alt="A beginner-level Chinese textbook with pinyin included for all words ('Tier 1')." width="250">](design/images/textbook-beginner.jpg)
[<img src="design/images/textbook-intermediate.jpg" alt="An intermediate-level Chinese textbook with pinyin for some words ('Tier 2'). In practice, this is grade-school level for natives!" width="250">](design/images/textbook-intermediate.jpg)
[<img src="design/images/newspaper-hard.jpg" alt="A native-level article from a Chinese newspaper ('Tier 3'). No pinyin is used at all, since natives don't really need it!" width="325">](design/images/newspaper-hard.jpg)

### Contextual Learning 

Contextual learning is arguably the best way to learn a language. People remember things that are linked to experiences or assorted significant pieces of information. For natives, learning Chinese is essential. However for L2 learners, finding the urgency to learn is uniquely difficult without an external driving force (e.g. living in a Chinese-speaking country).

Barring the ability to live in a foreign country, DuGuo hopes to offer the next-best thing by allowing users to pick what they want to read (improving contextual relevance) and saving contextual references for "learned" phrases (adding contextual triggers).

## Existing Tools

There are several existing tools that provide similar functionality, including (but not limited to): [Zhongwen Chrome Extension](https://chrome.google.com/webstore/detail/zhongwen-chinese-english/kkmlkkjojmombglmlpbpapmhcaljjkde?hl=en), [Purple Culture Pinyin Converter](https://www.purpleculture.net/chinese-pinyin-converter/), [Du Chinese (mobile)](https://www.duchinese.net/), [mdbg.net](https://www.mdbg.net/chinese/dictionary), [Hànzì Analyzer](http://hemiola.com/), [pin1yin1](https://www.pin1yin1.com/), etc.

The main differentiator I hope to provide with this project is improved UX (pinyin toggling, contextual saving) and the ability to persist documents to a database (allows building a long-term knowledge base). Ultimately this is provided as an additional tool to help users learn Chinese, so definitely use the combination of tools that best supplements your learning experience.

## Acknowledgements

This project was adopted from Martin Kess's previous CS6460 final project, the Chinese Reading Machine (中文读机). He provided the starter code (in Python Flask) and a strong existing framework to build on. 感谢!