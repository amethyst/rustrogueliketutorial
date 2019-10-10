# Adding Symmetry and Brush Size as Library Functions

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

In the previous chapter on Diffusion-Limited Aggregation, we introduced two new concepts for map building: *symmetry* and *brush size*. These readily apply to other algorithms, so we're going to take a moment to move them into library functions (in `map_builders/common.rs`), make them generic, and demonstrate how they can alter the Drunkard's Walk.

## Building the library versions




**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-31-symmetry)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-31-symmetry/)
---

Copyright (C) 2019, Herbert Wolverson.

---