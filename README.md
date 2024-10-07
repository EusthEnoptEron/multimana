# multimana

![CI Status](https://github.com/EusthEnoptEron/multimana/actions/workflows/rust.yml/badge.svg)

This is a work-in-progress highly experimental multiplayer mod for Visions of Mana.
It is not yet usable and might never see completion, but it may already be interesting as a point of reference.

## Current State

- The current version has only been tested with the trial version.
- Arbitrary Rust & Python code can be injected and executed, and can interact with the runtime.
- P2-P4 are automatically injected and take control of a hero of their own.
- Screen splits and unsplits as required.
- The python script code of the game has been extracted and restored.
- REPL for development.


## Building

`cargo build --release`

### Prerequisites

- [Rust toolchain](https://rustup.rs/)

### Installing

> The `BIN_PATH` is defined as "VisionsofMana\Binaries\Win64".

1. Copy the generated `multimana.dll` to `BIN_PATH`.
2. Copy the contents of the assets folder to `BIN_PATH`.
3. Copy the scripts folder to `BIN_PATH`.


## SDK Generation

The `generator` crate generates the SDK as a build script using a dump made with a slightly modified version of [Dumper-7](https://github.com/Encryqed/Dumper-7) (to properly propagate out params).
The SDK is highly experimental.
