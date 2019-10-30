# Game Stats

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Up until now, we've had *very* primitive stats: power and defense. This doesn't give a lot of room for variation, nor does it live up to the Roguelike ideal of drowning the player in numbers (ok, that's an overstatement). In the design document, we mentioned wanting a D&D-like approach to game stats. That gives a *lot* of room to play with things, allowing items with various bonuses (and penalties), and should feel familiar to most people likely to play a game in this genre. It will also require some UI work, but we'll push the bulk of it off until the next chapter.

## The Basic 6 Stats - Condensed to 4

Anyone who has played D&D will know that characters - and in later editions, everyone else - has six attributes:

* *Strength*, governing how much you can carry, how hard you bash things, and your general physical capability.
* *Dexterity*, governing how fast you dodge things, how well you leap around acrobatically, and things like picking locks and aiming your bow.
* *Constitution*, governing how physically fit and healthy you are, adjusting your hit point total and helping to resist disease.
* *Intelligence* for how smart you are, helping with spells, reading things.
* *Wisdom* for how much common sense you have, as well as helpful interactions with deities.
* *Charisma* for how well you interact with others.

This is overkill for the game we're making. Intelligence and Wisdom don't need to be separate (Wisdom would end up being the "dump stat" everyone ditches to get points elsewhere!), and Charisma is really only useful for interacting with vendors since we aren't doing a lot of social interaction in-game. So we'll opt for a condensed set of attributes for this game:

* *Might*, governing your general ability to hit things.
* *Fitness*, your general health.
* *Quickness*, your general dexterity replacement.
* *Intelligence*, which really combines Intelligence and Wisdom in D&D terms.

This is a pretty common mix for other games. Let's open `components.rs` and make a new component to hold them:

```rust

```

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-50-stats)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-50-stats)
---

Copyright (C) 2019, Herbert Wolverson.

---