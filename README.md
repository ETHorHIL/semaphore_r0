# Semaphore in Risc-0

This repository contains an implementation of [Semaphore](https://github.com/semaphore-protocol/semaphore/blob/main/packages/circuits/semaphore.circom) using [risczero](https://www.risczero.com/), it can serve as a practical example for those interested in cryptographic circuits.

## Learning path

If you want to learn how to use Risc-0 you should start with [this](https://dev.risczero.com/api/zkvm/quickstart). You can ignore Bonsai for learning about circuits. The risc0 github contains a lot of very useful [examples](https://github.com/risc0/risc0/tree/ea88d8f39416509b4a3fd0e71c123d4eeb8c2b06/examples), frankly its better to spend time looking through those instead of here. Another great ressource is [Thor K.'s examples](https://github.com/thor314/circuit-examples)

## Quick Start

First, make sure [rustup] is installed. The
[`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to
automatically install the correct version.

To build all methods and execute the method within the zkVM, run the following
command:

```bash
cargo run
```

## Directory Structure

The "Guest" is essentially the circuit and the "Host" is the prover where witness is generated.

```text

├── Cargo.toml
├── merkletree
│   └── src
│       └── lib.rs
├── host
│   ├── Cargo.toml
│   └── src
│       └── main.rs                        <-- [Host, generate circuit inputs]
└── methods
    ├── Cargo.toml
    ├── build.rs
    ├── guest
    │   ├── Cargo.toml
    │   └── src
    │       └── bin
    │           └── method_name.rs         <-- [Guest, contains Semaphore logic]
    └── src
        └── lib.rs
```
