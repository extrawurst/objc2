# About the generated framework crates

The `objc2` project provides (mostly) autogenerated interfaces to Apple's
Objective-C frameworks like AppKit, Foundation, Metal, WebKit, and so on.

The bindings currently contain some documentation from the headers, but you
should view [Apple's canonical developer documentation][apple-doc-index] for
detailed information about each API (there are [some plans][#309] for
importing that documentation, but it's difficult).

These crates uses `objc2` to declare the external interface to the
Objective-C classes and protocols. It is highly recommended that you read
the documentation here for details on how the Objective-C interop works.

They also use `block2::Block` in the public API, check out the documentation
for the [`block2`] crate for how to call such methods using a closure.

[apple-doc-index]: https://developer.apple.com/documentation/technologies
[#309]: https://github.com/madsmtm/objc2/issues/309
[`block2`]: https://docs.rs/block2
