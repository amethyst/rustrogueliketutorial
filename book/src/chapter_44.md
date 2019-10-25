# Design Document

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

If you plan to *finish* a game, it's important to set out your objectives ahead of time! Traditionally, this has taken the form of a *design document* - a master document outlining the game, and smaller sections detailing what you want to accomplish. In this case, it also forms the skeleton of writing the section. There are thousands of [online references to writing game design documents](https://www.gamasutra.com/blogs/LeandroGonzalez/20160726/277928/How_to_Write_a_Game_Design_Document.php). The format really doesn't matter so long as it acts as a guiding light for development and gives you criteria for which you can say "this is done!"

Because this is a tutorial, we're going to make the game design document a *skeleton* for now, and flesh it out as we progress. That leaves some flexibility in writing the guide on my end! So until this section is approaching complete, consider this to be a *living document* - a perpetual work in progress, being expanded as we go. That's *really* not how one should write a design document, but I have two luxuries that most teams don't: no time limit, and no team members to direct!

## Rusty Roguelike

Rusty Roguelike is a 2D traditional roguelike that attempts to capture the essentials of the genre as it has developed since Rogue's release in 1980. Turn-based, tile-based and centered on an adventurer's descent into a dungeon to retrieve the Amulet of Yala (Yet Another Lost Amulet). The adventurer battles through numerous procedurally generated levels to retrieve the amulet, and then must fight their way back to town to win the game.

### Characters

The player controls one major character, *Hero Protagonist* as he/she/it battles through the dungeon. Human NPCs will range from shop-keepers to fantasy RPG staples such as bandits, brigands, sorcerers, etc. Other characters in the game will largely be fantasy RPG staples: elves, dwarves, gnomes, halflings, orcs, goblins, trolls, ogres, dragons, etc.

(Description of all NPCs should go here)

A stretch goal is to have NPCs belong to factions, and allow the clever player to "faction farm" and adjust loyalties.

Ideally, NPC AI should be more intelligent than a rock.

### Story

This is not a story heavy game (Roguelikes are frequently shorter in story than traditional RPGs, because you die and restart a lot and won't generally spend a lot of time reading story/lore).

*In the dark ages of yore, the sorcerer kings crafted the Amulet of Yala to bind the demons of the Abyss - and end their reign of terror. A Golden Age followed, and the good races flourished. Now dark times have fallen upon the land once more, demons stir, and the forces of darkness once again ravage the land. The Amulet of Yala may be the good folk's last hope. After a long night in the pub, you realize that maybe it is your destiny to recover it and restore tranquility to the land. Only slightly hungover, you set forth into the dungeons beneath your home town - sure that you can be the one to set things right.*

### Theme

We'll aim for a traditional D&D style dungeon bash, with traps, monsters, the occasional puzzle and "replayability". The game should be different every time. A light-hearted approach is preferred, with humor sprinkled liberally (another staple of the genre). A "kitchen sink" approach is preferred to strictly focused realism - this is a tutorial project, and it's better to have lots of themes (from which to learn) than a single cohesive one in this case.

### Story Progression

There is no *horizontal progression* - you don't keep any benefits from previous runs through the game. So you always start in the same place as a new character, and gain benefits for a *single run* only. You can go both *up* and *down* in the dungeon, returning to town to sell items and goods. Progression on levels is preserved until you find the Amulet of Yala - at which point the universe truly is out to get you until you return home.

As a starting guide, consider the following progression. It will evolve and become more random as we work on the game.

1. The game starts in town. In town, there are only minimal enemies (pickpockets, thugs). You start in the to-be-named pub (tavern), armed only with a meager purse, minimal starting equipment, a stein of beer, a dried sausage, a backpack and a hangover. Town lets you visit various vendors.
2. You spelunk into the caves next to town, and fight your way through natural limestone caverns.
3. The limestone caverns give way to a ruined dwarven fortress, now occupied by vile beasts - and a black dragon (thanks Mr. Sveller!).
4. Beneath the dwarven fortress lies a vast mushroom forest.
5. The mushroom forest gives way to a dark elven city.
6. The depths contain a citadel with a portal to the Abyss.
7. The Abyss is a nasty fight against high-level demonic monsters. Here you find the Amulet of Yala.
8. You fight your way back up to town.

Travel should be facilitated with an equivalent of *Town Portal* scrolls from Diablo.

### Gameplay

Gameplay should be a very traditional turn-based dungeon crawl, but with an emphasis on making mechanics easy to use. At the base level, this is the "murder hobo experience": you start with very little, subsist off of what you find, kill (or evade) monsters you encounter, and take their stuff! This should be sprinkled with staples of the genre: item identification, interesting magical items, stats and plenty of ways to modify them, and multiple "valid" ways to play and beat the game. The game should be difficult but not impossible. Nothing that requires quick reflexes is permitted!

In a real game design document, we'd painstakingly describe each element here. For the purposes of the tutorial, we'll add to the list as we write more.

### Goals

* *Overall*: The ultimate goal is to retrieve the Amulet of Yala - and return to town (town portal spells stop working once you have it). 
* *Short-term*: Defeat enemies on each level.
* Navigate each level of the dungeon, avoiding traps and reaching the exit.
* Obtain lots of cool loot.
* Earn bragging rights for your score.

### User Skills

* Navigating different dungeons.
* Tactical combat, learning AI behavior and terrain to maximize the chances of survival.
* Item identification should be more than just "identify spell" - there should be some hints/system that the user can use to better understand the odds.
* Stat management - equip to improve your chances of survival for different threats.
* Long and short-term resource management.
* Ideally we want enough depth to spur "build" discussions.

### Game Mechanics

We'll go with the tried and tested "sort of D&D" mechanics used by so many games (and licensed under the Open Gaming License), but without being *tied* to a D&D-like game. We'll expand upon this as we develop the tutorial.

### Items and Power-Ups

The game should include a good variety of items. Broadly, items are divided as:

* Wearables (armor, clothes, etc.)
* Wearable specials (amulets, rings, etc.)
* Defense items (shields and similar)
* Melee weapons
* Ranged weapons
* Consumables (potions, scrolls, anything consumed by use)
* Charged items (items that can only be used `x` times unless recharged)
* Loot/junk to sell/scrap.
* Food.

Other notes:
* Eventually, items should have weight and inventory management becomes a skill. Until then, it can be quite loose/ready.
* Magical items shouldn't immediately reveal what they do, beyond being magical.
* Items should be drawn from loot tables that at least sort-of make sense.
* "Props" are a special form of item that doesn't move, but can be interacted with.

### Progression and challenge

* As you defeat enemies, you earn experience points and can level up. This improves your general abilities and grants access to better ways to defeat more enemies!
* The levels should increase in difficulty as you descend. "Out of level" enemies are possible but *very rare* - to keep it fair.
* Try to avoid capriciously killing the player with no hope of circumventing it.
* Once the Amulet of Yala has been claimed, difficulty ramps up on *all levels* as you fight your way back up to town. Certain perks (like town portal) no longer work.
* There is no progression between runs - it's entirely self-contained.

### Losing

*Losing is fun!* In fact, a fair portion of the appeal of traditional roguelikes is that you have one life - and it's "game over" when you succumb to your wounds/traps/being turned into a banana. The game will feature [permadeath](http://www.roguebasin.com/index.php?title=Permadeath) - once you've died, your run is over and you start afresh.

As a stretch goal, we may introduce some ways to mitigate/soften this.

### Art Style

We'll aim for beautiful ASCII, and may introduce tiles.

### Music and Sound

None! It would be nice to have once tiles are done, but fully voicing a modern RPG is far beyond my resources.

### Technical Description

The game will be written in Rust, using [RLTK_RS](https://github.com/thebracket/rltk_rs) for its back-end. It will support all the platforms on which Rust can compile and link to OpenGL, including Web Assembly for browser-based play.

### Marketing and Funding

This is a free tutorial, so the budget is approximately $0. If anyone wants to donate to [my Patreon](https://www.patreon.com/blackfuture) I can promise eternal gratitude, a monster in your honor, and not a lot else!

### Localization

I'm hopeless at languages, so English it is.

### Other Ideas

Anyone who has great ideas should send them to me. :-)


## Wrap-Up

So there we have it: a very skeletal design document, with lots of holes in it. It's a good idea to write one of these, especially when making a time-constrained game such as a "7-day roguelike challenge". This chapter will keep improving in quality as more features are implemented. For now, it's intended to serve as a baseline.


...

---

Copyright (C) 2019, Herbert Wolverson.

---