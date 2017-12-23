# ReactRust

ReactRust is a Rust library, written for a course project, which offers reactive
concepts to its users. It is greatly inspired from the
[ReactiveML](http://rml.lri.fr/index.html) language,
using concepts such as *reactive processes* and *signals*.


### Content
Even though it is working, it does not provide much to the end-user,
and was not completely completed. It also *not* focuses on performances!

However, it still has basic reactive capabilities, such as:
* **Continuations**, as basic computation units;
* **A Runtime**, managing logical time *instants* to run the above;
* **Processes**, abstracting continuations, and providing various methods to chain them
  (for pausing, mapping functions, joining and repeating processes, etc).
* **Signals**, allowing communication over processes and instants.


### Building
ReactRust requires no external crate.
It can be built using `cargo build`, (slightly) tested using `cargo test`, and
some documentation can be generated with `cargo doc`.
