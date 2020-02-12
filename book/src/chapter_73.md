# Scanning The Systems

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Specs provides a really nice dispatcher system: it can automatically employ concurrency, making your game really zoom. So why aren't we using it? Web Assembly! WASM doesn't support threading in the same way as every other platform, and a Specs application compiled with a dispatcher to WASM dies hard on the first attempt to dispatch the systems. It isn't really fair on desktop applications to suffer from this. Also, the current `run_systems` isn't at all nice to look at:

```rust
fn run_systems(&mut self) {
    let mut mapindex = MapIndexingSystem{};
    mapindex.run_now(&self.ecs);
    let mut vis = VisibilitySystem{};
    vis.run_now(&self.ecs);
    ... // MANY more
```

So the goal of this chapter is to build an interface that detects WASM, and falls back to a single-threaded dispatcher. If WASM isn't around, we'd like to use the Specs dispatcher. We'd also like a nicer interface to our systems!


---

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-73-systems)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-73-systems)
---

Copyright (C) 2019, Herbert Wolverson.

---