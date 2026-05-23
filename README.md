# Unofficial Cthulhu Bot

This bot emulates dice rolling. It is tuned for CoC, but you can use this for general purposes.

**Note that  Chaosium Inc. owns the copyright of Call of Cthulhu.**

## Commands available

|Command|Frequently used|Description|
|:---|:---:|:---|
|`/choose`||Makes a random choice.|
|`/cs`|:star:|Creates a character sheet.|
|`/op7`||Does an opposed roll following the Call of Cthulhu 7th Edition.|
|`/roll`|:star:|Rolls designated dices. Expressions supported by [Tyche](https://github.com/Gawdl3y/tyche-rs) can be used.|
|`/skill`|:star:|Does a skill roll. Alias for `/sk6`.|
|`/sk6`||Does a skill roll following the Call of Cthulhu 6th Edition.|
|`/sk7`|:star:|Does a skill roll following the Call of Cthulhu 7th Edition.|
|`/skdg`||Does a skill roll following the Delta Green.|
|`/skbrp`||Does a skill roll following the BRP 2023.|

### Roll dices

Command: `/roll` dice:`3d5 + 2d4`

### Attempts a skill roll.

Command: `/skill` value:`50` comment:`Listen`

### Create a character sheet

Command: `/cs` 근력:`60` 건강:`60` 크기:`65` 민첩:`70` 외모:`50` 지능:`65` 정신:`50` 교육:`75`

## Memo

Inspired by [Caphorsa](https://github.com/caphosra/cthulhu_bot).
