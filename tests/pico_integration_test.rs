//! Pico CLI Integration Test
//!
//! This test runs the same FRI proof generation and verification logic as the Risc0
//! integration test (`test_integration_zkvm`), but uses the Pico CLI tool instead
//! of the Risc0 HTTP server.
//!
//! ## Test Flow
//!
//! 1. Generate FRI proof data using FRIVeil (same as risc0 test)
//! 2. Create `GuestInput` with proof, evaluation_point, evaluation_claim, packed_values_log_len
//! 3. Serialize to bincode and write to temp input file
//! 4. Spawn Pico CLI subprocess: `proof-of-proof-prover --input <file> --output <file>`
//! 5. Wait for process to complete and verify exit code is 0
//! 6. Read output file and verify committed value is `true`
//! 7. Clean up temp files
//!
//! ## Prerequisites
//!
//! Before running this test, ensure the Pico binaries are built:
//!
//! ```bash
//! # Build Pico app (guest)
//! cd proof_of_proof_pico/app
//! cargo pico build
//!
//! # Build Pico prover (host)
//! cd proof_of_proof_pico/prover
//! rustup run nightly-2025-08-04 cargo build --release
//! ```
//!
//! ## Running the Test
//!
//! ```bash
//! cargo test test_integration_pico_cli -- --nocapture
//! ```

use proof_core::GuestInput;
use rand::{rngs::StdRng, seq::index::sample, SeedableRng};
use std::fs;
use std::path::PathBuf;
use tokio::process::Command;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, span, Level};
use FRIVeil::{
    friveil::{FriVeilDefault, PackedField, B128},
    poly::Utils,
    traits::{FRIVeilSampling, FriVeilUtils},
};

const LOG_INV_RATE: usize = 1;
const NUM_TEST_QUERIES: usize = 128;
const DATA_SIZE_KB: usize = 9;

#[tokio::test]
async fn test_integration_pico_cli() {
    // Initialize enhanced logging with structured output, filtering out verbose internal logs
    use tracing_subscriber::filter::EnvFilter;

    let filter = EnvFilter::new("info")
        .add_directive("binius_transcript=error".parse().unwrap())
        .add_directive("transcript=error".parse().unwrap());

    // Try to init, but ignore if already initialized (common in tests)
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(filter)
        .with_test_writer()
        .try_init();

    info!("🚀 Starting Binius Data Availability Sampling Scheme (Pico CLI)");
    info!("📋 Configuration:");
    info!("   - Reed-Solomon inverse rate (log2): {}", LOG_INV_RATE);
    info!("   - FRI test queries: {}", NUM_TEST_QUERIES);
    info!("   - Data size: {} KB", DATA_SIZE_KB);

    // Create arbitrary (nonzero, patterned) data instead of all zeroes.
    let _span = span!(Level::INFO, "data_generation").entered();
    info!("📊 Phase 1: Generating test data ({} KB)", DATA_SIZE_KB);
    let random_data_bytes: Vec<u8> = (0..DATA_SIZE_KB * 1024).map(|i| (i % 256) as u8).collect();
    info!(
        "✅ Generated {} bytes of patterned test data",
        random_data_bytes.len()
    );
    drop(_span);

    let _span = span!(Level::INFO, "mle_conversion").entered();
    info!("🔄 Phase 2: Converting bytes to multilinear extension");
    let start = Instant::now();
    let packed_mle_values = Utils::<B128>::new()
        .bytes_to_packed_mle(&random_data_bytes)
        .unwrap();

    let conversion_time = start.elapsed().as_millis();
    info!("✅ MLE conversion completed in {} ms", conversion_time);
    info!(
        "   - Total variables (n_vars): {}",
        packed_mle_values.total_n_vars
    );
    info!(
        "   - Packed values count: {}",
        packed_mle_values.packed_values.len()
    );

    drop(_span);

    let _span = span!(Level::INFO, "fri_initialization").entered();
    info!("🔧 Phase 3: Initializing FRI-based polynomial commitment scheme");
    let start = Instant::now();

    let friveil = FriVeilDefault::new(
        LOG_INV_RATE,
        NUM_TEST_QUERIES,
        packed_mle_values.total_n_vars,
        80, // log_num_shares
    );
    let init_time = start.elapsed().as_millis();
    info!("✅ FRIVeil context initialized in {} ms", init_time);

    let start = Instant::now();
    info!("🎲 Generating random evaluation point");
    let evaluation_point = friveil.calculate_evaluation_point_random().unwrap();
    let eval_time = start.elapsed().as_millis();
    info!("✅ Evaluation point generated in {} ms", eval_time);
    info!(
        "   - Evaluation point dimensions: {}",
        evaluation_point.len()
    );
    drop(_span);

    let _span = span!(Level::INFO, "fri_context_setup").entered();
    info!("⚙️  Setting up FRI protocol parameters");
    let start = Instant::now();
    let (fri_params, ntt) = friveil
        .initialize_fri_context(packed_mle_values.packed_mle.log_len())
        .unwrap();
    let context_time = start.elapsed().as_millis();
    info!("✅ FRI context setup completed in {} ms", context_time);
    info!(
        "   - Reed-Solomon code length (log2): {}",
        fri_params.rs_code().log_len()
    );
    info!(
        "   - Reed-Solomon inverse rate (log2): {}",
        fri_params.rs_code().log_inv_rate()
    );
    info!("   - FRI test queries: {}", fri_params.n_test_queries());
    drop(_span);

    let _span = span!(Level::INFO, "vector_commitment_and_codeword").entered();
    info!("🔒 Phase 4: Generating vector commitment and codeword");
    let start = Instant::now();
    let commit_output = friveil
        .commit(
            packed_mle_values.packed_mle.clone(),
            fri_params.clone(),
            &ntt,
        )
        .unwrap();
    let commit_time = start.elapsed().as_millis();

    info!(
        "✅ Vector commitment and codeword generated in {} ms",
        commit_time
    );
    info!(
        "   - Commitment size: {} bytes",
        commit_output.commitment.len()
    );
    info!(
        "   - Codeword length: {} elements",
        commit_output.codeword.len()
    );

    drop(_span);

    let _span = span!(Level::INFO, "proof_generation").entered();
    info!("📝 Phase 5: Generating evaluation proof");
    let start = Instant::now();
    let mut verifier_transcript = friveil
        .prove(
            packed_mle_values.packed_mle.clone(),
            fri_params.clone(),
            &ntt,
            &commit_output,
            &evaluation_point,
        )
        .unwrap();
    let proof_time = start.elapsed().as_millis();

    info!("✅ Evaluation proof generated in {} ms", proof_time);

    drop(_span);

    let _span = span!(Level::INFO, "evaluation_claim").entered();

    info!("🧮 Computing evaluation claim");
    let start = Instant::now();
    let evaluation_claim = friveil
        .calculate_evaluation_claim(&packed_mle_values.packed_values, &evaluation_point)
        .unwrap();
    let claim_time = start.elapsed().as_millis();
    info!("✅ Evaluation claim computed in {} ms", claim_time);
    debug!("   - Evaluation claim value: {:?}", evaluation_claim);
    drop(_span);

    let proof = friveil.get_transcript_bytes(&verifier_transcript);
    info!("📦 Proof size: {} bytes", proof.len());

    // Task 4: Create GuestInput and spawn Pico CLI
    let _span = span!(Level::INFO, "pico_cli_invocation").entered();
    info!("🖥️  Phase 6: Preparing Pico CLI input");

    // Create GuestInput with proof data
    let guest_input = GuestInput::from_proofs(
        vec![proof],
        evaluation_point,
        evaluation_claim,
        packed_mle_values.packed_mle.log_len(),
    );

    // Serialize to bincode
    let input_bytes = bincode::serialize(&guest_input.to_tuple())
        .expect("Failed to serialize GuestInput");
    info!("✅ Serialized input: {} bytes", input_bytes.len());

    // Create temp files
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("pico_test_input.bin");
    let output_path = temp_dir.join("pico_test_output.bin");

    // Write input to temp file
    fs::write(&input_path, &input_bytes)
        .expect("Failed to write input file");
    info!("✅ Written input to: {:?}", input_path);

    // Find prover binary (try release first, then debug)
    let prover_paths = [
        PathBuf::from("proof_of_proof_pico/target/release/prover"),
        PathBuf::from("proof_of_proof_pico/target/debug/prover"),
    ];

    let prover_path = prover_paths
        .iter()
        .find(|p| p.exists())
        .expect(&format!(
            "Pico prover binary not found. Tried: {:?}\n\
             Please build it first:\n\
             cd proof_of_proof_pico/prover && rustup run nightly-2025-08-04 cargo build --release",
            prover_paths
        ));

    info!("🚀 Phase 7: Spawning Pico CLI prover");
    info!("   - Prover: {:?}", prover_path);
    info!("   - Input: {:?}", input_path);
    info!("   - Output: {:?}", output_path);

    // Spawn Pico CLI from the prover directory so it can find the ELF
    let start = Instant::now();
    // The prover expects to be run from proof_of_proof_pico/prover directory
    // so it can find ../app/elf/riscv32im-pico-zkvm-elf
    let prover_cwd = PathBuf::from("proof_of_proof_pico/prover");
    let mut child = Command::new("../target/release/prover")
        .arg("--mock")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .current_dir(&prover_cwd)
        .spawn()
        .expect("Failed to spawn Pico prover");

    // Wait for process with timeout (5 minutes - proof generation is computationally intensive)
    let exit_status = match timeout(Duration::from_secs(300), child.wait()).await {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => panic!("Failed to wait for prover: {}", e),
        Err(_) => {
            // Timeout - kill the process
            let _ = child.kill().await;
            panic!("Pico prover timed out after 300 seconds (5 minutes)");
        }
    };

    let prove_time = start.elapsed().as_millis();

    if !exit_status.success() {
        panic!(
            "Pico prover failed with exit code: {:?}",
            exit_status.code()
        );
    }

    info!("✅ Pico proof generated in {} ms", prove_time);
    drop(_span);

    // Task 5: Verify proof output and cleanup
    let _span = span!(Level::INFO, "proof_verification").entered();
    info!("🔍 Phase 8: Verifying Pico proof output");

    // Read output file
    let output_bytes = fs::read(&output_path)
        .expect("Failed to read output file");
    info!("✅ Read output: {} bytes", output_bytes.len());

    // Verify the committed value (Pico guest commits `true`)
    let committed_value: bool = bincode::deserialize(&output_bytes)
        .expect("Failed to deserialize output");
    assert!(committed_value, "Pico proof should commit 'true'");
    info!("✅ Committed value verified: true");

    drop(_span);

    // Cleanup temp files
    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_file(&output_path);
    info!("🧹 Cleaned up temp files");

    info!("🎉 Pico CLI integration test completed successfully!");
}
