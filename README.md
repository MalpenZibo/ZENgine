<img align="left" width="64px" src="assets/branding/logo.svg" />

# ZENgine &emsp; ![CI](https://github.com/MalpenZibo/ZENgine/workflows/CI/badge.svg)

## What is ZENgine?

ZENgine is a very simple 2D data-driven game engine for didactic purposes written in Rust using an ECS architecture.

## Inspiration
ZENgine is heavily inspired by:
* [Bevy](https://github.com/bevyengine/bevy) A refreshingly simple data-driven game engine built in Rust 
* [Amethyst](https://github.com/amethyst/amethyst) Data-oriented and data-driven game engine written in Rust 
* [kudo](https://github.com/kettle11/kudo) An Entity Component System for Rust.

## Didactic purpose
I started this project mainly to improve my knowledge in Rust, which is a programming language that I love, and in game engine architecture which is a subject that has always fascinated me.

I created a small series of videos (only in Italian 😏) that cover the first iteration of the engine before the massive refactor of all engine.
In the future, I plan to create a series of posts about the engine and how it works.

## Get Started
There's a very crude implementation of `pong` in the example folder that "should" run on Windows, Mac, Linux, Android, and every modern browser.

### Desktop Env (Windows, Mac, Linux)
Simply run `cargo run` in the  `pong` folder

### Android

To build the android version is mandatory to setup correctly the environment.

You have to install the `Android SDK` and the `Android NDK` and correctly setup the env variables.

Then install `cargo-apk` with:
```bash
cargo install cargo-apk
```

and lastly, install the required toolchain:
```bash
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```

Now to install the application on your device simply run: 
```bash
cargo apk run --lib
``` 
in the `pong` folder.

### WASM ENV

To launch the web version is mandatory to setup correctly the environment.
Install the wasm32-unknown-unknown target with:
```bash
rustup target add wasm32-unknown-unknown
```

Then install `wasm-bindgen-cli` and [Trunk](https://trunkrs.dev/) with:
```bash
cargo install trunk wasm-bindgen-cli
```

Now you can serve the application using `trunk serve` 

(it could be necessary to click on the browser page to make the input system work).



