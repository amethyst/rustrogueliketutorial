# Experience and Levelling

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

The design document talks about using *Town Portal* to return to town, which implies that *backtracking* is possible - that is, it's possible to return to levels. This is quite a common feature of games such as Dungeon Crawl: Stone Soup (in which it is standard procedure to leave items in a "stash" where hopefully the monsters won't find them).

If we're going to support going back and forth between levels (either via entrance/exit pairs, or through mechanisms such as teleports/portals), we need to adjust the way we handle levels altogether.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-55-backtrack)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-55-backtrack)
---

Copyright (C) 2019, Herbert Wolverson.

---