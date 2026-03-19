# rusty_spine3.8

This is a fork of [rusty_spine](https://github.com/jabuwu/rusty_spine) that supports [Spine 3.8](http://esotericsoftware.com/). Thanks to [pocams](https://github.com/pocams) for the [original PR](https://github.com/jabuwu/rusty_spine/pull/3).

## License

Because this project uses the official Spine Runtime, you must follow the Spine Runtimes License Agreement. If using the `libc` crate (via the `use_libc` feature), then see the [libc crate](https://crates.io/crates/libc) for licensing. If using the built-in (wasm compatible) libc implementation, you must follow the BSD 3-clause license of The Regents of the University of California. See the `LICENSE` file for complete licenses. The Rust code is licensed under dual MIT / Apache-2.0 but with no attribution necessary. All contributions must agree to this licensing.

## Features

### use_libc

Default: no

A small subset of libc is provided in this repo to avoid a dependency on [libc](https://crates.io/crates/libc) and to allow the code to run in the `wasm32-unknown-unknown` target. However, it's possible to rely on the OS implementation of libc instead by using the `use_libc` feature.

### draw_functions

Default: yes

Provides [helper functions](https://github.com/jabuwu/rusty_spine/tree/main/src/draw) for generating mesh data, as well as the `SkeletonController` helper struct.

### egui_debugger

Default: no

Provides an [egui](https://github.com/emilk/egui) debugger window for viewing skeleton and animation state. See it in action by running the `bevy` example with this feature enabled:

`cargo run --release --example bevy --features egui_debugger`

### mint

Default: yes

Provides additional math functions using [mint](https://docs.rs/mint).
