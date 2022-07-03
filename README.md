# lightrary
A library for lights, Philips Hue lights.

## Usage 
Add the following to your `cargo.toml`:
```rust
lightrary = { git = "https://github.com/jimvdl/lightrary" }
```
Lightrary will currently not be available on crates.io mainly because I don't feel like I can keep up with Philips' rapidly developing API. Currently Philips Hue is developing v2 of their new CLIP API and is not even close to being feature complete. The large amount of breaking changes to their API makes keeping lightrary stable, and in working condition, quite difficult. Numerous libraries on crates.io that interact with the Philips Hue lights have since been abandoned and are left in a broken state, I don't want to add yet another broken, left-behind and unmaintained Philips Hue library to crates.io and keep the ecosystem in a more healthy state. 

That said, you can still use lightrary by including it using git. Issues on bugs or general improvements are greatly appreciated!
