# Work Plan: Risc0 to picoVM Migration

## Context

### Original Request
Migrate the existing Risc0-based `proof_of_proof/` implementation to work with picoVM from Brevis (https://pico-docs.brevis.network/), while keeping both implementations side-by-side.

### Current State (Risc0)
The project has a working Risc0 zkVM setup:
- **Host**: HTTP server (Axum) that accepts JSON requests and generates proofs
- **Guest**: zkVM program using `risc0_zkvm::guest` that deserializes FRIVeil inputs and verifies FRI proofs (verification currently commented out)
- **Build**: Uses `risc0_build::embed_methods()` to embed ELF at compile time
- **Shared**: `proof-core` crate with `GuestInput` and `GuestInputTuple` types

### Target State (picoVM)
New `proof_of_proof_pico/` directory with:
- **Guest**: Uses `pico_sdk` with `#![no_main]` + `entrypoint!` macro
- **Host**: CLI tool using `DefaultProverClient` (no HTTP server)
- **Build**: Uses `cargo pico build` CLI tool (ELF loaded from file)
- **Field**: KoalaBear (default picoVM field)
- **Proofs**: STARK only (no EVM/Groth16)

### Interview Summary

**Key Decisions**:
1. **Side-by-side coexistence**: Keep both Risc0 and picoVM versions
2. **KoalaBear field**: Use default picoVM field
3. **STARK proofs only**: No EVM/Groth16 proving
4. **CLI-only architecture**: Remove HTTP server, use command-line interface
5. **Keep guest structure**: Maintain commented FRI verification (already commented in Risc0 version)

**Scope Boundaries**:
- IN: New `proof_of_proof_pico/` directory with complete picoVM implementation
- OUT: Modifying existing `proof_of_proof/` code, EVM proving, HTTP server
- OUT: Enabling FRI verification (keeping it commented as in Risc0)
- OUT: Refactoring shared `proof-core` types

### Metis Review Findings

**Critical Gap Identified**: FRIVeil/Binius dependencies may not compile in picoVM's constrained guest environment (`#![no_main]`, no_std). A spike test is required as the first task.

**Guardrails Applied**:
1. Independent builds: Both versions must compile without affecting each other
2. No shared mutable state between versions
3. Dependency isolation: picoVM uses separate Cargo.lock
4. Read-only reuse of `proof-core`: No modifications to existing types
5. Simple CLI interface with exit codes (0=success, 1=failure, 2=error)

**Auto-Resolved**:
- Proof output: Save to `proof.bin` file (not stdout)
- Error handling: Use `Box<dyn Error>` (consistent with current approach)
- ELF path: `../app/elf/riscv32im-pico-zkvm-elf` (picoVM standard)

---

## Work Objectives

### Core Objective
Create a side-by-side picoVM implementation (`proof_of_proof_pico/`) that mirrors the Risc0 version's functionality: accepts FRIVeil inputs, runs guest code, and generates STARK proofs using the KoalaBear field.

### Concrete Deliverables
1. `proof_of_proof_pico/app/` - Guest program compiled to RISC-V ELF
2. `proof_of_proof_pico/prover/` - CLI host program that loads ELF and generates proofs
3. `proof_of_proof_pico/Cargo.toml` - Workspace definition
4. `proof_of_proof_pico/README.md` - Usage documentation
5. Working proof generation: `cargo pico build` → `cargo run --release` produces `proof.bin`

### Definition of Done
- [x] Spike test confirms FRIVeil compiles in picoVM guest
- [x] Guest code uses `#![no_main]` + `pico_sdk::entrypoint!`
- [x] Host CLI loads ELF and generates STARK proof
- [x] Both `proof_of_proof/` and `proof_of_proof_pico/` build independently
- [x] Proof output saved to `proof.bin` and can be loaded

### Must Have
- FRIVeil guest code compilation in picoVM environment
- `pico_sdk::io::read_as()` for input deserialization
- `pico_sdk::io::commit()` for public output
- `DefaultProverClient` with KoalaBear field
- CLI interface with clear arguments

### Must NOT Have
- HTTP server in picoVM version
- EVM/Groth16 proving
- Modifications to existing Risc0 code
- Working FRI verification (keep commented)
- Shared mutable state between versions

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (picoVM is new, need to set up toolchain)
- **User wants tests**: Manual QA only (no test framework in picoVM guest)
- **QA approach**: Manual verification with commands and file outputs

### Manual Verification Procedures

Each task includes concrete verification steps with expected outputs.

---

## Task Flow

```
Task 1 (Spike Test)
       ↓
Task 2 (Directory Structure)
       ↓
Task 3 (Guest Code) ─────┐
       ↓                 │
Task 4 (Host Code) ──────┤
       ↓                 │
Task 5 (Build Scripts) ──┤
       ↓                 │
Task 6 (Documentation)   │
       ↓                 │
Task 7 (Final Integration)←
```

## Parallelization

| Group | Tasks | Reason |
|-------|-------|--------|
| A | 3, 4 | Independent once spike test passes |

| Task | Depends On | Reason |
|------|------------|--------|
| 3 (Guest) | 1 (Spike) | Must verify FRIVeil compiles first |
| 4 (Host) | 1 (Spike) | Must verify FRIVeil compiles first |
| 5 (Build) | 3 (Guest) | Need guest ELF path defined |
| 7 (Final) | 3, 4, 5, 6 | Integration test after all components ready |

---

## TODOs

### Task 0: Install picoVM Toolchain (Prerequisite)

**What to do**:
- Install `cargo-pico` CLI tool

**Must NOT do**:
- Skip this step and try to use cargo directly

**Parallelizable**: NO (prerequisite for all)

**Acceptance Criteria**:
- [x] Command: `cargo pico --version` returns version number

**Verification**:
```bash
cargo install cargo-pico  # Or follow https://pico-docs.brevis.network/getting-started/installation.html
cargo pico --version
# Expected: cargo-pico x.y.z
```

**Commit**: NO (toolchain setup)

---

### Task 1: Spike Test - FRIVeil in picoVM Guest

**What to do**:
Create a minimal picoVM guest that imports FRIVeil to verify it compiles in the constrained environment.

**Must NOT do**:
- Write full guest code yet (just test compilation)
- Skip this validation

**Parallelizable**: NO (blocks all other tasks)

**References**:

**Pattern References**:
- `proof_of_proof/methods/guest/src/main.rs:1-15` - Current guest imports for reference
- `proof_of_proof/methods/guest/Cargo.toml:1-15` - Current dependencies

**External References**:
- picoVM docs: `https://pico-docs.brevis.network/writing-apps/programs.html` - Entrypoint macro and constraints

**Acceptance Criteria**:
- [x] Minimal guest with `#![no_main]` + `pico_sdk::entrypoint!` created
- [x] Guest imports `FRIVeil` crate and `proof-core`
- [x] `cargo pico build` compiles successfully

**Verification**:
```bash
cd proof_of_proof_pico_spike/app
cargo pico build
# Expected: Build succeeds, ELF generated at elf/riscv32im-pico-zkvm-elf
```

**If Fails**: Report back to user - FRIVeil/Binius may need patch libraries or be incompatible

**Commit**: YES | NO (spike test, may be discarded)

---

### Task 2: Create Directory Structure

**What to do**:
Create `proof_of_proof_pico/` with standard picoVM workspace layout.

**Must NOT do**:
- Modify existing `proof_of_proof/` directory
- Create nested workspaces

**Parallelizable**: YES (after Task 1)

**Acceptance Criteria**:
- [x] Directory structure created:
  ```
  proof_of_proof_pico/
  ├── Cargo.toml          # Workspace
  ├── app/
  │   ├── Cargo.toml      # Guest
  │   └── src/main.rs
  ├── prover/
  │   ├── Cargo.toml      # Host
  │   └── src/main.rs
  └── README.md
  ```

**Verification**:
```bash
ls -la proof_of_proof_pico/
# Expected: app/ prover/ Cargo.toml README.md
```

**Commit**: YES
- Message: `feat(pico): create side-by-side picoVM directory structure`
- Files: `proof_of_proof_pico/Cargo.toml proof_of_proof_pico/README.md`

---

### Task 3: Implement Guest Code

**What to do**:
Convert Risc0 guest code to picoVM with `#![no_main]` and `pico_sdk`.

**Must NOT do**:
- Uncomment FRI verification loop (keep commented as in Risc0)
- Use `risc0_zkvm` imports

**Parallelizable**: YES (after Task 1, with Task 4)

**References**:

**Pattern References**:
- `proof_of_proof/methods/guest/src/main.rs:1-61` - Current guest code (use as template)
- `proof_of_proof/methods/guest/Cargo.toml:1-15` - Current dependencies (adapt to pico)

**API/Type References**:
- `proof_of_proof/core/src/lib.rs` - `GuestInput`, `GuestInputTuple` types (reuse)

**External References**:
- picoVM docs: `https://pico-docs.brevis.network/writing-apps/programs.html#inputs-and-outputs`
- picoVM entrypoint: `#![no_main]` + `pico_sdk::entrypoint!(main)`

**Acceptance Criteria**:
- [x] Guest uses `#![no_main]` attribute
- [x] Guest uses `pico_sdk::entrypoint!(main)` macro
- [x] Input read via `pico_sdk::io::read_as::<GuestInputTuple>()` (no bincode manual deserialize)
- [x] Output committed via `pico_sdk::io::commit(&true)`
- [x] FRI verification loop remains commented (lines 40-56 from Risc0 version)
- [x] `cargo pico build` succeeds

**Verification**:
```bash
cd proof_of_proof_pico/app
cargo pico build
# Expected: ELF at elf/riscv32im-pico-zkvm-elf

file elf/riscv32im-pico-zkvm-elf
# Expected: ELF 32-bit LSB executable, UCB RISC-V
```

**Commit**: YES
- Message: `feat(pico): implement guest program with pico_sdk`
- Files: `proof_of_proof_pico/app/src/main.rs proof_of_proof_pico/app/Cargo.toml`

---

### Task 4: Implement Host CLI

**What to do**:
Create CLI host program that loads ELF and generates proofs using `DefaultProverClient`.

**Must NOT do**:
- Add HTTP server (axum/tokio)
- Use `risc0_zkvm` imports
- Embed ELF (load from file instead)

**Parallelizable**: YES (after Task 1, with Task 3)

**References**:

**Pattern References**:
- `proof_of_proof/host/src/main.rs:1-53` - Current host code (use logic, not HTTP)

**External References**:
- picoVM docs: `https://pico-docs.brevis.network/writing-apps/proving.html#pico-emulatorstdin` - Prover client usage
- picoVM proving: `KoalaBearProverVKClient` for full proving with verification key

**Acceptance Criteria**:
- [x] CLI accepts input file path as argument
- [x] Loads ELF from `../app/elf/riscv32im-pico-zkvm-elf`
- [x] Uses `DefaultProverClient::new(&elf)` (or `KoalaBearProverVKClient`)
- [x] Writes input via `stdin_builder.write(&guest_input_tuple)`
- [x] Generates proof via `client.prove(stdin_builder)` (not `prove_fast` for production)
- [x] Saves proof to `proof.bin` file
- [x] Prints proof verification status
- [x] Exit codes: 0=success, 1=proof failure, 2=error

**Verification**:
```bash
cd proof_of_proof_pico/prover
cargo build --release
./target/release/prover --input ../../test_input.bin
# Expected: 
# - "Loading ELF from ../app/elf/riscv32im-pico-zkvm-elf"
# - "Generating proof..."
# - "Proof saved to proof.bin"
# - Exit code 0

ls -la proof.bin
# Expected: File exists, size > 0
```

**Commit**: YES
- Message: `feat(pico): implement CLI host with DefaultProverClient`
- Files: `proof_of_proof_pico/prover/src/main.rs proof_of_proof_pico/prover/Cargo.toml`

---

### Task 5: Add Workspace Configuration

**What to do**:
Create root `Cargo.toml` for `proof_of_proof_pico/` workspace.

**Must NOT do**:
- Add to existing workspace at project root
- Include risc0 dependencies

**Parallelizable**: YES (after Task 2, with Tasks 3, 4)

**Acceptance Criteria**:
- [x] Workspace defined with members: `["app", "prover"]`
- [x] No risc0 dependencies
- [x] pico-sdk dependency configured
- [x] proof-core dependency (reuse existing)
- [x] FRIVeil dependency (from parent)

**Verification**:
```bash
cd proof_of_proof_pico
cargo check
# Expected: All dependencies resolve, no errors
```

**Commit**: YES
- Message: `feat(pico): add workspace configuration`
- Files: `proof_of_proof_pico/Cargo.toml`

---

### Task 6: Add Documentation

**What to do**:
Create `README.md` explaining picoVM version usage and differences from Risc0.

**Must NOT do**:
- Duplicate Risc0 documentation
- Claim feature parity where there are differences

**Parallelizable**: YES

**Acceptance Criteria**:
- [x] Installation instructions (cargo-pico)
- [x] Build instructions: `cargo pico build` in app/
- [x] Prove instructions: `cargo run --release` in prover/
- [x] Differences from Risc0 version documented:
  - CLI vs HTTP server
  - ELF file vs embedded
  - KoalaBear field
  - STARK proofs only
- [x] Exit codes documented

**Verification**:
```bash
cat proof_of_proof_pico/README.md | head -50
# Expected: Clear usage instructions, differences section
```

**Commit**: YES
- Message: `docs(pico): add README with usage instructions`
- Files: `proof_of_proof_pico/README.md`

---

### Task 7: Final Integration and Test

**What to do**:
Verify both Risc0 and picoVM versions build and work independently.

**Must NOT do**:
- Skip testing Risc0 version still works
- Skip verifying proof output

**Parallelizable**: NO (final integration)

**Acceptance Criteria**:
- [x] `proof_of_proof/` still builds: `cd proof_of_proof && cargo build`
- [x] `proof_of_proof_pico/app` builds: `cd proof_of_proof_pico/app && cargo pico build`
- [x] `proof_of_proof_pico/prover` builds: `cd proof_of_proof_pico/prover && cargo build --release`
- [x] Can generate test input file
- [x] Prover runs and generates `proof.bin`
- [x] Both versions coexist without conflicts

**Verification**:
```bash
# Test 1: Risc0 version still works
cd proof_of_proof
cargo build
# Expected: Success

# Test 2: picoVM guest builds
cd ../proof_of_proof_pico/app
cargo pico build
# Expected: Success, ELF generated

# Test 3: picoVM prover builds
cd ../prover
cargo build --release
# Expected: Success

# Test 4: Generate test input (create simple binary)
echo "test input data" > test_input.bin

# Test 5: Run prover
./target/release/prover --input test_input.bin
# Expected: Proof generated, proof.bin created

# Test 6: Verify proof file exists
ls -la proof.bin
# Expected: File exists
```

**Commit**: YES (if any fixes needed)
- Message: `fix(pico): integration fixes for side-by-side builds`
- Files: Any fixes required

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 2 | `feat(pico): create directory structure` | `Cargo.toml README.md` | `ls -la` shows structure |
| 3 | `feat(pico): implement guest program` | `app/src/main.rs app/Cargo.toml` | `cargo pico build` succeeds |
| 4 | `feat(pico): implement CLI host` | `prover/src/main.rs prover/Cargo.toml` | `cargo build --release` succeeds |
| 5 | `feat(pico): add workspace config` | `Cargo.toml` | `cargo check` succeeds |
| 6 | `docs(pico): add README` | `README.md` | README readable and accurate |
| 7 | `fix(pico): integration fixes` | Any fixes | Both versions build independently |

---

## Success Criteria

### Verification Commands

```bash
# 1. Verify side-by-side builds work
cd proof_of_proof && cargo build --release
cd ../proof_of_proof_pico/app && cargo pico build
cd ../prover && cargo build --release

# 2. Verify proof generation
cd ../prover
./target/release/prover --input <test-input>
ls proof.bin  # Should exist

# 3. Verify no conflicts
cargo check  # In both directories
```

### Final Checklist

- [x] `proof_of_proof_pico/` created with complete structure
- [x] Guest compiles with `cargo pico build`
- [x] Prover CLI runs and generates proof
- [x] `proof_of_proof/` still builds (no regression)
- [x] README documents usage and differences
- [x] No risc0 dependencies in picoVM code
- [x] FRI verification remains commented (as in Risc0)
- [x] Exit codes work correctly (0, 1, 2)

---

## Notes

### picoVM-Specific Quirks

1. **entrypoint! macro**: Must use `#![no_main]` + `pico_sdk::entrypoint!(main)`
2. **ELF loading**: Host loads ELF from file system (not embedded like Risc0)
3. **Field selection**: Using KoalaBear (default), not BabyBear
4. **Build tool**: `cargo pico build` instead of `cargo build` for guest
5. **Prover client**: Use `KoalaBearProverVKClient` for full proving

### Risk Mitigation

**Critical Risk**: FRIVeil/Binius may not compile in picoVM guest  
**Mitigation**: Task 1 spike test validates this before investing in full migration

**Secondary Risk**: Shared types (`proof-core`) may have compatibility issues  
**Mitigation**: Treat `proof-core` as read-only, duplicate to `lib/` if needed

### Post-Completion

After this plan is executed, you will have:
- Two working zkVM implementations side-by-side
- CLI-based proving for picoVM (no HTTP server)
- STARK proofs using KoalaBear field
- Documentation for using picoVM version

Run `/start-work` to begin execution of this plan.
