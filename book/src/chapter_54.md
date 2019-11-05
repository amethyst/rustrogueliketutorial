# Experience and Levelling

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

So far, we've delved into the dungeon and really only improved ourselves by finding better gear. Swords, shields and armor with better stats - giving us a chance to survive stronger enemies. That's good, but only half of the equation typically found in roguelikes and RPGs; defeating enemies typically grants *experience points* - and those can be used to better your character.

The type of game we are making implies some guidelines:

* With permadeath, you can expect to die *a lot*. So managing your character's progression needs to be *simple* - so you don't spend a huge amount of time on it, only to have to do it all again.
* *Vertical progression* is a good thing: as you delve, you get stronger (allowing us to make stronger mobs). *Horizontal* progression largely defeats the point of permadeath; if you keep benefits between games, then the "each run is unique" aspect of roguelikes is compromised, and you can expect the fine fellows of `/r/roguelikes` on Reddit to complain!

## Gaining experience points

When you defeat something, you should gain XP. We'll go with a simple progression for now: you earn `100 XP * enemy level` each time you defeat something. This gives a bigger benefit to killing something tough - and a smaller (relative) benefit to hunting things down once you have out-levelled them. Additionally, we'll decide that you need `1,000 XP * Current Level` to advance to the next one.

We already have `level` and `xp` in our `Pools` component (you'd almost think that we were planning this chapter!). Let's start by modifying our GUI to display level progression.




**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-54-xp)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-54-xp)
---

Copyright (C) 2019, Herbert Wolverson.

---