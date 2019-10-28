# Making the starting town

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

## What is the town for?

Back in the [Design Document](./chapter_44.md) we decided: *The game starts in town. In town, there are only minimal enemies (pickpockets, thugs). You start in the to-be-named pub (tavern), armed only with a meager purse, minimal starting equipment, a stein of beer, a dried sausage, a backpack and a hangover. Town lets you visit various vendors.*

From a development point of view, this tells us a few things:

* The town has a *story* aspect, in that you start there and it ground the story - giving a starting point, a destiny (in this case a drunken promise to save the world). So the town implies a certain *cozy* starting point, implies some communication to help you understand *why* you are embarking on the life of an adventurer, and so on.
* The town has vendors. That won't make sense at this point, because we don't have a value/currency system - but we know that we need somewhere to put them.
* The town has a tavern/inn/pub - it's a starting location, but it's obviously important enough that it needs to *do* something!
* Elsewhere in the design document, we mention that you can *town portal* back to the settlement. This again implies a certain coziness/safety, and also implies that doing so is *useful* - so the services offered by the town need to retain their utility throughout the game.
* Finally, the town is the winning condition: once you've grabbed the Amulet of Yala - getting back to town lets you save the world. That implies that the town should have some sort of holy structure to which you have to return the amulet.
* The town is the first thing that new players will encounter - so it has to look alive and somewhat slick, or players will just close the window and try something else. It may also serve as a location for some tutorials.

This sort of discussion is essential to game design; you don't want to implement something just because you can (in most cases; big open world games relax that a bit). The town has a *purpose*, and that purpose guides its *design*.

## So what do we have to include in the town?

So that discussion lets us determine that the town must include:

* One or more merchants. We're not implementing the sale of goods yet, but they need a place to operate.
* Some friendly/neutral NPCs for color.
* A temple.
* A tavern.
* A place that town portals arrive.
* A way out to begin your adventure.

We can also think a little bit about what makes a town:

* There's generally a communication route (land or sea), otherwise the town won't prosper.
* Frequently, there's a market (surrounding villages use towns for commerce).
* There's almost certainly either a river or a deep natural water source.
* Towns typically have authority figures, visible at least as Guards or Watch.
* Towns also generally have a shady side.

## How do we want to generate our town?

We could go for a prefabricated town. This has the upside that the town can be tweaked until it's *just right*, and plays smoothly. It has the downside that getting out of the town becomes a purely mechanical step after the first couple of play-throughs ("runs"); look at Joppa in Caves of Qud - it became little more than a "grab the chest content, talk to these guys, and off you go" speed-bump start to an amazing game.

So - we want a procedurally generated town, but we want to keep it functional - and make it pretty. Not much to ask!

## Making some new tile types

From the above, it sounds like we are going to need some new tiles. The ones that spring to mind for a town are roads, grass, water (both deep and shallow), bridge, wooden floors, and building walls.



**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-47-town1)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-47-town1)
---

Copyright (C) 2019, Herbert Wolverson.

---