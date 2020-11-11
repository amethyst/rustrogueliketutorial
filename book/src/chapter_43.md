# Let's Make a Game!

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

[![Hands-On Rust](./beta-webBanner.jpg)](https://pragprog.com/titles/hwrust/hands-on-rust/)

---

So far, the tutorial has followed three sections:

1. Make a skeletal game, showing how to make a very minimalistic roguelike.
2. Add some essential genre features to the game, making it more fun to play.
3. Building lots of maps, a very important part of making fun roguelikes.

Now we're going to start a series of articles that actually makes a cohesive game from our framework. It won't be huge, and it's unlikely to challenge for "best Roguelike ever!" status - but it will explore the trials and tribulations that go with turning a tech demo into a cohesive game.

## The Berlin Interpretation

This game will stick closely to the genre, with very little ground-breaking innovation. So we'll have a fantasy setting, dungeon diving, and limited progression. If you're familiar with the [Berlin Interpretation](http://www.roguebasin.com/index.php?title=Berlin_Interpretation) (an attempt at codifying what counts as a roguelike in a world of games using the name!), we'll try to stick closely to the important aspects:

*High-value targets*

* *Random Environment Generation* is essential, and we've already covered a lot of interesting ways to do it!
* *Permadeath* defines the genre, so we'll go with it. We'll probably sneak in game saving/loading, and look at how to handle non-permadeath if that's what you want - but we'll stick to the principle, and its implication that you should be able to beat a roguelike without dying.
* *Turn-based* - we'll definitely stick to a turn-based setup, but will introduce varying speeds and initiative.
* *Grid-based* - we'll definitely stick to a grid-based system.
* *Non-modal* - we'll probably break this one, by having systems that take you out of the regular "all on one screen" play system.
* *Complexity* - we'll strive for complexity, but try to keep the game playable without being a Master's thesis topic!
* *Resource management* - we've already got some of that with the hunger clock and consumable items, but we'll definitely want to retain this as a defining trait.
* *Hack'n'slash* - definitely!
* *Exploration and discovery* - absolutely!

*Low-value targets*

* *Single player character* - we're unlikely to introduce groups in this section, but we might introduce friendly NPCs.
* *Monsters are similar to players* - the ECS helps with this, since we're simulating the player in the same way as NPCs. We'll stick to the basic principle.
* *Tactical challenge* - always something to strive for; what good is a game without challenge?
* *ASCII Display* - we'll be sticking with this, but may find time to introduce graphical tiles later.
* *Dungeons* - of course! They don't *have* to be rooms and corridors, but we've worked hard to have *good* rooms and corridors!
* *Numbers* - this one is a little more controversial; not everyone wants to see a giant wall of math every time they punch a goblin. We'll try for some balance - so there are plenty of numbers, mostly visible, but they aren't essential to playing the game.

So it seems pretty likely that with this constraints we will be making a *real roguelike* - one that checks almost all of the boxes!

## Setting

We've already decided on a fantasy-faux-medieval setting, but that doesn't mean it has to be *just* like D&D or Tolkien! We'll try and introduce some fun and unique elements in our setting.

## Narrative

In the next chapter, we'll work on outlining our overall objective in a *design document*. This will necessarily include some narrative, although roguelikes aren't really known for *deep* stories!

...

---

Copyright (C) 2019, Herbert Wolverson.

---