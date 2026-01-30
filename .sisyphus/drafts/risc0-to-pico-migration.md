# Draft: Risc0 to picoVM Migration Plan

## Current Risc0 Setup (proof_of_proof/)

### Project Structure
```
proof_of_proof/
├── Cargo.toml          # Workspace with members: core, host, methods
├── core/               # Shared types (GuestInput, GuestInputTuple)
├── host/               # HTTP server + prover
│   └── src/main.rs     # Axum server, uses risc0_zkvm
└── methods/
    ├── build.rs        # risc0_build::embed_methods()
    ├── guest/
    │   └── src/main.rs # Guest program using risc0-zkvm
    └── src/lib.rs      # Auto-generated ELF constants
```

### Current Dependencies (Risc0)
- Guest: `risc0-zkvm = "^3.0.4"`, `risc0-zkvm-platform = "2.1.1"`
- Host: `risc0-zkvm = "^3.0.4"`
- Build: `risc0-build::embed_methods()`

### Current Guest Code Pattern (Risc0)
```rust
// No #![no_main] needed in risc0
use risc0_zkvm::guest::{self, env};

fn main() {
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();
    let (data, evaluation_point, evaluation_claim, packed_log_val): GuestInputTuple =
        bincode::deserialize(&input_bytes).unwrap();
    // ... compute ...
    env::commit(&true);  // Commit public output
}
```

### Current Host Code Pattern (Risc0)
```rust
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use methods::{PROOF_OF_PROOF_GUEST_ELF, PROOF_OF_PROOF_GUEST_ID};

let env = ExecutorEnv::builder().write_slice(&serialized).build()?;
let prover = default_prover();
let prove_info = prover.prove(env, PROOF_OF_PROOF_GUEST_ELF)?;
let receipt = prove_info.receipt;
receipt.verify(PROOF_OF_PROOF_GUEST_ID)?;
```

---

## Target picoVM Setup

### New Project Structure (picoVM standard)
```
proof_of_proof_pico/ (or migrate existing)
├── app/                # Guest program (was methods/guest)
│   ├── src/main.rs     # #![no_main] + entrypoint!
│   └── Cargo.toml      # pico-sdk dependency
├── prover/             # Host program (was host)
│   ├── src/main.rs     # DefaultProverClient
│   └── Cargo.toml      # pico-sdk dependency
├── lib/                # Shared types (from core)
│   ├── src/lib.rs
│   └── Cargo.toml
├── elf/                # Compiled RISC-V ELF
│   └── riscv32im-pico-zkvm-elf
└── Cargo.toml          # Workspace definition
```

### New Dependencies (picoVM)
- Guest: `pico-sdk = { git = "https://github.com/brevis-network/pico" }`
- Host: `pico-sdk = { git = "https://github.com/brevis-network/pico" }`
- Build: `cargo pico build` (CLI tool, no build.rs needed)

### New Guest Code Pattern (picoVM)
```rust
#![no_main]  // Required in picoVM
use pico_sdk::entrypoint;

entrypoint!(main);

pub fn main() {
    // Read serialized input
    let inputs = pico_sdk::io::read_as::<GuestInputTuple>();
    // ... compute ...
    pico_sdk::io::commit(&true);  // Commit public output
}
```

### New Host Code Pattern (picoVM)
```rust
use pico_sdk::client::DefaultProverClient;

let elf = load_elf("../elf/riscv32im-pico-zkvm-elf");
let client = DefaultProverClient::new(&elf);
let mut stdin_builder = client.new_stdin_builder();

// Write inputs
stdin_builder.write(&guest_input_tuple);

// Generate proof
let proof = client.prove_fast(stdin_builder)?;  // or .prove() for full recursion

// Verify public values
let public_buffer = proof.pv_stream.unwrap();
```

---

## Key Differences Summary

| Aspect | Risc0 | picoVM |
|--------|-------|--------|
| Guest entrypoint | Standard `fn main()` | `#![no_main]` + `entrypoint!(main)` |
| Input reading | `env::stdin().read_to_end()` + bincode | `pico_sdk::io::read_as::<T>()` |
| Output commitment | `env::commit(&value)` | `pico_sdk::io::commit(&value)` |
| Build system | `risc0_build::embed_methods()` in build.rs | `cargo pico build` CLI |
| ELF handling | Auto-embedded as constants | Load from file path |
| Prover client | `default_prover()` | `DefaultProverClient::new(&elf)` |
| Proof receipt | `Receipt` struct | Proof with `pv_stream` |
| Field options | Default field | KoalaBear (default) or BabyBear |
| EVM proving | Bonsai service | Built-in Groth16 via `cargo pico prove --evm` |

---

## Migration Tasks Identified

1. **Project Structure Reorganization**
   - Rename/restructure `methods/guest/` → `app/`
   - Rename/restructure `host/` → `prover/`
   - Keep `core/` or move to `lib/`
   - Remove `methods/build.rs` (no longer needed)

2. **Guest Code Migration**
   - Add `#![no_main]` attribute
   - Add `pico_sdk::entrypoint!(main)` macro
   - Replace `risc0_zkvm::guest::env` imports with `pico_sdk::io`
   - Replace `env::stdin().read_to_end()` + bincode with `pico_sdk::io::read_as()`
   - Replace `env::commit()` with `pico_sdk::io::commit()`

3. **Host Code Migration**
   - Replace `risc0_zkvm` imports with `pico_sdk::client`
   - Load ELF from file instead of embedded constants
   - Replace `ExecutorEnv` with `stdin_builder`
   - Replace `default_prover()` with `DefaultProverClient::new()`
   - Replace `Receipt` handling with pico proof verification

4. **Build System Migration**
   - Remove `risc0-build` dependency
   - Remove `methods/build.rs`
   - Add `cargo pico` CLI tool installation requirement
   - Create build scripts or document build commands

5. **Cargo.toml Updates**
   - Replace `risc0-zkvm` with `pico-sdk`
   - Update workspace structure
   - Remove risc0-specific features

6. **Integration Test Updates**
   - Update `tests/integration_test.rs` zkvm test

---

## User Decisions (Confirmed)

✅ **Side-by-side**: Keep both Risc0 and picoVM versions  
✅ **Field**: KoalaBear (default)  
✅ **Proof type**: STARK only (no EVM/Groth16)  
✅ **Architecture**: CLI-only (remove HTTP server)  
✅ **Guest logic**: Keep current structure with commented FRI verification  

## Critical Validation Needed (Metis Finding)

**⚠️ SPIKE TEST REQUIRED**: Will FRIVeil/Binius compile in picoVM's constrained guest environment?

The picoVM guest uses `#![no_main]` and has no_std constraints. Binius field arithmetic may not work. **First TODO must be a compilation spike test.**

## Guardrails from Metis Review

### Must Have Guardrails
1. **No Shared State**: picoVM and Risc0 versions must build independently
2. **FRI Verification Scope**: Keep commented (it's already commented in Risc0 version)
3. **CLI Interface**: Single-shot CLI, exit codes (0=success, 1=failure, 2=error)
4. **Dependency Isolation**: picoVM must not add deps to existing workspace
5. **No Risc0 Refactoring**: Don't touch existing proof_of_proof/ code

### Must NOT Have
- HTTP server in picoVM version
- EVM/Groth16 verification
- risc0-zkvm dependencies in picoVM
- Changes to proof_of_proof/core/ (read-only reuse)
- Working FRI verification (unless explicitly requested)

### Defaults Applied (from decisions)
- KoalaBear field (not BabyBear)
- Side-by-side directory structure
- STARK proofs only
- CLI interface (no server)
- Proof output to file (not stdout)

## Edge Cases Addressed

1. **FRIVeil compilation**: Covered by spike test (Task 1)
2. **ELF loading**: Host loads from `../app/elf/riscv32im-pico-zkvm-elf`
3. **Input size**: Uses `pico_sdk::io::read_as()` (handles buffering)
4. **Proof output**: Save to `proof.bin` file in prover directory
5. **Error handling**: Simple `Box<dyn Error>` approach (same as current)

## Guardrails from Metis Review

- Preserve FRIVeil core logic (just change zkVM wrapper)
- Maintain bincode serialization compatibility for GuestInput
- Keep HTTP API interface if server is kept
- Document build process changes clearly
- Test proof generation end-to-end after migration
