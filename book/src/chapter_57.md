# AI Cleanup and Status Effects

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

In the design document, we indicated that we'd like the AI to be smarter than the average rock. We've also added quite a few AI-related systems as we've worked through the chapters and (you may have noticed) some things like consistently applying *confusion* effects (and occasional problems hitting a mob that just moved) have slipped through the cracks!

As we add complexity to our monsters, it would be good to get all of this straightened out, make it easier to support new features, and handle commonalities like movement consistently.

## Chained Systems

Rather than try and do *everything* for an NPC in one system, we could break the process out into several steps. This is more typing, but has the advantage that each step is distinct, clear and does just one thing - making it *much* easier to debug. This is analogous to how we are handling `WantsToMelee` - we're indicating an intent and then handling it in its own step - which let us keep targeting and actually fighting separated.

Let's look at the steps, and see how they can be broken down:

* We determine that it is the NPC's turn.
* We check status effects - such as *Confusion* to determine if they can in fact go.
* The AI module for that AI type scans their surroundings, and determines if they want to move, attack or do nothing.
* Movement occurs, which updates various global statuses.
* Combat occurs, which can kill mobs or render them unable to act in the future.

We already have quite a few AI systems, and this is just going to add more. So let's move AI into a *module*. Create a new folder, `src/ai` - this will be the new AI module. Create a `mod.rs` file...

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-57-ai)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-57-ai)
---

Copyright (C) 2019, Herbert Wolverson.

---