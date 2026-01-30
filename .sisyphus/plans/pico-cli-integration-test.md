# Work Plan: Pico CLI Integration Test

## Context

### Original Request
Create a test that runs the existing risc0 integration test logic against the Pico CLI tool instead of the HTTP server.

### Interview Summary
**Key Decisions**:
- **Architecture**: Keep Pico as CLI tool (do not convert to HTTP server)
- **Test approach**: Create new test file (don't modify existing risc0 test)
- **Binary discovery**: Assume pre-built binary at `proof_of_proof_pico/target/release/proof-of-proof-prover`
- **Verification level**: Verify proof validity (not just file existence)
- **Test location**: Create new file `tests/pico_integration_test.rs`
- **Execution method**: Use `std::process::Command` to spawn CLI as subprocess

### Technical Understanding
**Risc0 Test Flow** (`test_integration_zkvm`):
1. Generate FRI proof data using FRIVeil
2. Create `GuestInput` with proof, evaluation_point, evaluation_claim, packed_values_log_len
3. Serialize to bincode
4. Send HTTP POST to `http://localhost:3000/get_proof`
5. Expect HTTP 200 OK response

**Pico Test Flow** (to implement):
1. Generate FRI proof data using FRIVeil (same as risc0)
2. Create `GuestInput` with same data
3. Serialize to bincode
4. Write to temp input file
5. Spawn Pico CLI: `proof-of-proof-prover --input <temp_input> --output <temp_output>`
6. Wait for process to complete
7. Read output file
8. Verify committed value is `true` (the guest commits `pico_sdk::io::commit(&true)`)
9. Assert exit code is 0

### Metis Review (Self-Analysis)
**Identified Gaps** (addressed):
1. **Temp file handling**: Need tempfile crate or std::env::temp_dir()
2. **Binary path**: Should handle both debug and release builds
3. **Cleanup**: Must clean up temp files after test
4. **Timeout**: CLI process might hang - need timeout handling
5. **Error messages**: Should provide clear error if binary not found

---

## Work Objectives

### Core Objective
Create `tests/pico_integration_test.rs` that runs the same FRI proof generation and verification logic as `test_integration_zkvm`, but uses the Pico CLI tool instead of the Risc0 HTTP server.

### Concrete Deliverables
- New test file: `tests/pico_integration_test.rs`
- Test function: `test_integration_pico_cli()`
- Proper error handling and cleanup
- Documentation for running the test

### Definition of Done
```bash
cd /Volumes/Personal/Avail/fri-veil
cargo test test_integration_pico_cli -- --nocapture
```
Expected: Test passes, showing proof generation via Pico CLI succeeded.

### Must Have
- [x] Generates same FRI proof data as risc0 test
- [x] Creates `GuestInput` and serializes to bincode
- [x] Spawns Pico CLI subprocess with correct arguments
- [x] Waits for CLI to complete and checks exit code
- [x] Reads output file and verifies proof validity (committed value is `true`)
- [x] Cleans up temp files after test (even on failure)
- [x] Provides clear error messages for common failure modes

### Must NOT Have (Guardrails)
- [x] No modifications to existing risc0 test or code
- [x] No conversion of Pico to HTTP server
- [x] No changes to Pico prover logic
- [x] No assumptions about binary path that would break CI

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (Cargo test framework)
- **User wants tests**: Manual verification via `cargo test`
- **Framework**: Built-in Rust test framework

### Manual Execution Verification

**Test Type**: Integration test using subprocess

**By Deliverable Type**:

| Type | Verification Tool | Procedure |
|------|------------------|-----------|
| **Integration test** | cargo test | Run test, verify it passes |
| **CLI subprocess** | std::process::Command | Spawn, wait, check exit code |
| **File I/O** | std::fs | Write input, read output |

**Evidence Required**:
- [ ] Test file created at correct location
- [ ] `cargo test test_integration_pico_cli` passes
- [ ] Output shows successful proof generation

---

## Task Flow

```
Task 1: Create test file structure
    ↓
Task 2: Import dependencies and setup
    ↓
Task 3: Implement test function (main logic)
    ↓
Task 4: Add temp file handling and cleanup
    ↓
Task 5: Add error handling and assertions
    ↓
Task 6: Run test and verify
```

---

## TODOs

- [x] **1. Create test file structure**

  **What to do**:
  - Create new file `tests/pico_integration_test.rs`
  - Add file header comment explaining purpose
  - Add module-level documentation

  **Must NOT do**:
  - Don't modify existing `tests/integration_test.rs`
  - Don't add tests to other files

  **Parallelizable**: NO (depends on file creation)

  **References**:
  - `tests/integration_test.rs:1-719` - Reference for test structure and patterns
  - `proof_of_proof_pico/prover/src/main.rs:24-65` - CLI interface to match

  **Acceptance Criteria**:
  - [ ] File exists at `tests/pico_integration_test.rs`
  - [ ] File compiles without errors: `cargo check --tests`
  
  **Commit**: YES
  - Message: `test(pico): add integration test file structure`
  - Files: `tests/pico_integration_test.rs`

---

- [x] **2. Import dependencies and setup**

  **What to do**:
  - Add necessary imports:
    - `FRIVeil` types (B128, FriVeilDefault, PackedField, traits)
    - `proof_core::GuestInput`
    - `std::process::Command`
    - `std::fs` for file operations
    - `std::env::temp_dir` or `tempfile` crate
    - `rand` for test data generation
    - `tracing` for logging
  - Copy logging setup from existing test
  - Define constants (LOG_INV_RATE, NUM_TEST_QUERIES, DATA_SIZE_KB)

  **Must NOT do**:
  - Don't add unused dependencies
  - Don't change Cargo.toml unless adding tempfile crate

  **Parallelizable**: NO (depends on Task 1)

  **References**:
  - `tests/integration_test.rs:1-44` - Import patterns and logging setup
  - `Cargo.toml:34` - proof-core dependency (dev-dependencies)

  **Acceptance Criteria**:
  - [ ] All imports compile correctly
  - [ ] Logging initializes in test
  - [ ] Constants match risc0 test values

  **Commit**: YES (groups with Task 1)

---

- [x] **3. Implement test function - FRI proof generation**

  **What to do**:
  - Create async test function `test_integration_pico_cli`
  - Copy FRI proof generation logic from `test_integration_zkvm`:
    1. Generate test data (9KB patterned data)
    2. Convert to MLE using `Utils::<B128>::bytes_to_packed_mle()`
    3. Initialize FriVeilDefault with same parameters
    4. Generate evaluation point
    5. Initialize FRI context (get fri_params and ntt)
    6. Generate commitment and codeword
    7. Generate proof using `friveil.prove()`
    8. Calculate evaluation claim
    9. Get transcript bytes

  **Must NOT do**:
  - Don't modify the proof generation logic (keep it identical to risc0 test)
  - Don't skip any steps

  **Parallelizable**: NO (depends on Task 2)

  **References**:
  - `tests/integration_test.rs:425-604` - Complete test logic to replicate
  - `src/traits.rs` - FriVeilSampling and FriVeilUtils traits
  - `src/friveil.rs` - FriVeilDefault struct

  **Acceptance Criteria**:
  - [ ] Test generates FRI proof successfully
  - [ ] Proof data is collected in a Vec<u8>
  - [ ] All intermediate values are computed correctly

  **Commit**: YES
  - Message: `test(pico): implement FRI proof generation in test`
  - Files: `tests/pico_integration_test.rs`

---

- [x] **4. Implement test function - CLI invocation**

  **What to do**:
  - After generating proof data, create GuestInput:
    ```rust
    let guest_input = GuestInput::from_proofs(
        vec![proof],  // Vec<Vec<u8>>
        evaluation_point,
        evaluation_claim,
        packed_mle_values.packed_mle.log_len(),
    );
    ```
  - Serialize to bincode:
    ```rust
    let input_bytes = bincode::serialize(&guest_input.to_tuple()).unwrap();
    ```
  - Create temp input file in temp directory
  - Write serialized data to temp file
  - Create temp output file path
  - Spawn Pico CLI using `std::process::Command`:
    ```rust
    let prover_path = "proof_of_proof_pico/target/release/proof-of-proof-prover";
    let mut cmd = Command::new(prover_path)
        .arg("--input").arg(&input_path)
        .arg("--output").arg(&output_path)
        .spawn()
        .expect("Failed to spawn prover");
    ```
  - Wait for process with timeout (e.g., 60 seconds)
  - Check exit code is 0

  **Must NOT do**:
  - Don't hardcode paths that won't work in CI
  - Don't forget to handle the case where binary doesn't exist

  **Parallelizable**: NO (depends on Task 3)

  **References**:
  - `proof_of_proof_pico/prover/src/main.rs:24-65` - CLI argument structure
  - `proof_of_proof/core/src/lib.rs:17-44` - GuestInput::from_proofs()

  **Acceptance Criteria**:
  - [ ] GuestInput is created correctly
  - [ ] Bincode serialization works
  - [ ] Temp input file is created and written
  - [ ] CLI process spawns successfully
  - [ ] Process completes with exit code 0

  **Commit**: YES
  - Message: `test(pico): add CLI invocation to test`
  - Files: `tests/pico_integration_test.rs`

---

- [x] **5. Implement test function - proof verification and cleanup**

  **What to do**:
  - Read output file after CLI completes:
    ```rust
    let output_bytes = std::fs::read(&output_path).expect("Failed to read output");
    ```
  - Verify the committed value (Pico guest commits `true`):
    ```rust
    let committed_value: bool = bincode::deserialize(&output_bytes)
        .expect("Failed to deserialize output");
    assert!(committed_value, "Pico proof should commit 'true'");
    ```
  - Implement cleanup that runs even on failure (use `Drop` guard or `defer` pattern)
  - Add logging for each step
  - Add comprehensive error messages

  **Must NOT do**:
  - Don't leave temp files behind on test failure
  - Don't panic without context

  **Parallelizable**: NO (depends on Task 4)

  **References**:
  - `proof_of_proof_pico/app/src/main.rs:51` - Guest commits `&true`
  - `proof_of_proof_pico/prover/src/main.rs:56-61` - Output file writing

  **Acceptance Criteria**:
  - [ ] Output file is read successfully
  - [ ] Deserialized value is `true`
  - [ ] Temp files are cleaned up after test
  - [ ] Test provides clear error messages on failure

  **Commit**: YES
  - Message: `test(pico): add proof verification and cleanup`
  - Files: `tests/pico_integration_test.rs`

---

- [ ] **6. Run and verify the test**

  **What to do**:
  - Ensure Pico prover is built:
    ```bash
    cd proof_of_proof_pico/prover
    rustup run nightly-2025-08-04 cargo build --release
    ```
  - Ensure Pico app is built:
    ```bash
    cd proof_of_proof_pico/app
    cargo pico build
    ```
  - Run the test:
    ```bash
    cd /Volumes/Personal/Avail/fri-veil
    cargo test test_integration_pico_cli -- --nocapture
    ```
  - Verify it passes
  - Fix any compilation errors
  - Fix any runtime errors

  **Must NOT do**:
  - Don't commit broken test
  - Don't ignore test failures

  **Parallelizable**: NO (depends on Task 5)

  **References**:
  - `proof_of_proof_pico/README.md:49-77` - Build instructions

  **Acceptance Criteria**:
  - [ ] Test compiles without errors
  - [ ] Test runs successfully
  - [ ] Test passes (exit code 0)
  - [ ] Output shows proof was generated and verified

  **Evidence**:
  - [ ] Command output captured: `cargo test test_integration_pico_cli -- --nocapture`
  - [ ] Screenshot or terminal output showing success

  **Commit**: YES
  - Message: `test(pico): verify integration test passes`
  - Files: `tests/pico_integration_test.rs`

---

## Auto-Resolved Gaps

### Binary Path Handling
**Gap**: How to handle different build profiles (debug vs release)?
**Resolution**: Try release path first, fall back to debug path with clear error message.

### Temp File Cleanup on Failure
**Gap**: What if test panics before cleanup?
**Resolution**: Use `tempfile` crate which auto-deletes on drop, or implement a cleanup guard struct with `Drop` trait.

### Timeout Handling
**Gap**: What if Pico CLI hangs?
**Resolution**: Use `wait_timeout` from `std::process::Child` or spawn with timeout using `tokio::time::timeout` (test is async anyway).

### Missing Binary
**Gap**: What if prover binary doesn't exist?
**Resolution**: Check file exists before spawning, provide helpful error message with build instructions.

---

## Defaults Applied

1. **Temp file location**: `std::env::temp_dir()` (system temp directory)
2. **Timeout**: 60 seconds for proof generation
3. **Logging level**: INFO (same as risc0 test)
4. **Binary path priority**: Check release first, then debug
5. **Cleanup**: Use `tempfile` crate for automatic cleanup

---

## Decisions Needed

None - all decisions made during interview.

---

## Guardrails Applied

1. **No risc0 code changes**: Test is in separate file, doesn't modify existing code
2. **No Pico architecture changes**: Keeps CLI architecture as requested
3. **No prover logic changes**: Only tests existing prover
4. **Clear error messages**: All failure modes have descriptive messages
5. **Proper cleanup**: Temp files always cleaned up
6. **Timeout protection**: Prevents hanging tests

---

## Commit Strategy

| After Task | Message | Files |
|------------|---------|-------|
| 1-2 | `test(pico): add integration test file structure` | `tests/pico_integration_test.rs` |
| 3 | `test(pico): implement FRI proof generation in test` | `tests/pico_integration_test.rs` |
| 4 | `test(pico): add CLI invocation to test` | `tests/pico_integration_test.rs` |
| 5 | `test(pico): add proof verification and cleanup` | `tests/pico_integration_test.rs` |
| 6 | `test(pico): verify integration test passes` | `tests/pico_integration_test.rs` |

---

## Success Criteria

### Verification Commands
```bash
# Check test file exists
ls -la tests/pico_integration_test.rs

# Check compilation
cargo check --tests

# Run the test (requires pre-built binaries)
cargo test test_integration_pico_cli -- --nocapture

# Expected output:
# test test_integration_pico_cli ... ok
# 🚀 Starting Binius Data Availability Sampling Scheme
# ... (logging output)
# ✅ Pico proof verified successfully
```

### Final Checklist
- [ ] Test file exists at `tests/pico_integration_test.rs`
- [ ] Test compiles without warnings
- [ ] Test passes successfully
- [ ] Temp files are cleaned up
- [ ] No modifications to existing risc0 test
- [ ] Documentation updated (if needed)

---

## Dependencies to Add

May need to add to `Cargo.toml` dev-dependencies:
```toml
[dev-dependencies]
tempfile = "3.0"  # For auto-cleaning temp files
```

Check if already present before adding.
