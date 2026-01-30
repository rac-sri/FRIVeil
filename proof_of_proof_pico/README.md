# Proof of Proof - PicoVM Implementation

A picoVM-based proof of proof implementation for FRI-based data availability sampling, side-by-side with the existing Risc0 implementation.

## Overview

This is a picoVM (Brevis) implementation that mirrors the functionality of the Risc0-based `proof_of_proof/` directory. It generates STARK proofs using the KoalaBear field.

## Differences from Risc0 Version

| Aspect | Risc0 | picoVM (this) |
|--------|-------|---------------|
| **Architecture** | HTTP server (Axum) | CLI tool |
| **Build** | `risc0_build::embed_methods()` | `cargo pico build` |
| **ELF handling** | Embedded at compile time | Loaded from file |
| **Field** | Default Risc0 field | KoalaBear |
| **Proof type** | STARK | STARK (no EVM/Groth16) |
| **Guest entry** | Standard `fn main()` | `#![no_main]` + `entrypoint!` |

## Prerequisites

1. **Rust nightly toolchain** (specific version for picoVM):
   ```bash
   rustup install nightly-2025-08-04
   rustup component add rust-src --toolchain nightly-2025-08-04
   ```

2. **cargo-pico CLI**:
   ```bash
   cargo +nightly-2025-08-04 install --git https://github.com/brevis-network/pico pico-cli
   ```

## Structure

```
proof_of_proof_pico/
├── app/                    # Guest program (zkVM bytecode)
│   ├── Cargo.toml
│   ├── src/main.rs        # Guest code with FRIVeil logic
│   └── elf/               # Compiled RISC-V ELF
│       └── riscv32im-pico-zkvm-elf
├── prover/                # Host prover CLI
│   ├── Cargo.toml
│   └── src/main.rs        # CLI tool for proof generation
├── Cargo.toml             # Workspace definition
└── README.md              # This file
```

## Usage

### 1. Build the Guest Program

```bash
cd app
cargo pico build
```

This compiles the guest code to a RISC-V ELF binary at `app/elf/riscv32im-pico-zkvm-elf`.

### 2. Build the Prover CLI

```bash
cd prover
rustup run nightly-2025-08-04 cargo build --release
```

The prover binary will be at `target/release/proof-of-proof-prover`.

### 3. Generate a Proof

```bash
# Create a test input file (or use an existing one)
# The input should be a bincode-serialized GuestInput struct

# Run the prover
./target/release/proof-of-proof-prover --input <input_file.bin> --output proof.bin
```

### Exit Codes

- `0` - Success: Proof generated and saved
- `1` - Proof failure: Verification failed
- `2` - Error: I/O error, parsing error, etc.

## Input Format

The prover expects a bincode-serialized `GuestInput` struct:

```rust
GuestInput {
    data: Vec<Vec<u8>>,           // Proof data
    evaluation_point: Vec<[u8; 16]>,  // Evaluation points
    evaluation_claim: [u8; 16],   // Evaluation claim
    packed_values_log_len: usize, // Log length of packed values
}
```

## Implementation Notes

### Guest Code

The guest code (`app/src/main.rs`) uses:
- `#![no_main]` attribute for picoVM compatibility
- `pico_sdk::entrypoint!(main)` macro for entry point
- `pico_sdk::io::read_as::<GuestInputTuple>()` for input reading
- `pico_sdk::io::commit(&true)` for public output

The FRI verification loop is intentionally commented out (matching the Risc0 version).

### Host CLI

The prover CLI (`prover/src/main.rs`):
- Uses `clap` for argument parsing
- Loads the ELF file from `../app/elf/riscv32im-pico-zkvm-elf`
- Uses `DefaultProverClient` for proof generation
- Saves the proof's public values to a file

## Troubleshooting

### "feature may not be used on the stable release channel"

Use the nightly toolchain:
```bash
rustup run nightly-2025-08-04 cargo build --release
```

### "failed to load manifest for dependency `proof-core`"

Ensure the path in `app/Cargo.toml` is correct:
```toml
proof-core = { path = "../../proof_of_proof/core" }
```

## License

This project is part of the FRIVeil ecosystem.
