use proof_core::GuestInput;
use rand::{rngs::StdRng, SeedableRng};
use std::fs;
use std::time::Instant;
use FRIVeil::{
    friveil::{FriVeilDefault, PackedField, B128},
    poly::Utils,
    traits::{FRIVeilSampling, FriVeilUtils},
};

fn main() {
    const LOG_INV_RATE: usize = 1;
    const NUM_TEST_QUERIES: usize = 128;
    const DATA_SIZE_KB: usize = 9;

    println!("Generating test input...");
    println!("Configuration:");
    println!("   - Reed-Solomon inverse rate (log2): {}", LOG_INV_RATE);
    println!("   - FRI test queries: {}", NUM_TEST_QUERIES);
    println!("   - Data size: {} KB", DATA_SIZE_KB);

    let random_data_bytes: Vec<u8> = (0..DATA_SIZE_KB * 1024).map(|i| (i % 256) as u8).collect();

    let start = Instant::now();
    let packed_mle_values = Utils::<B128>::new()
        .bytes_to_packed_mle(&random_data_bytes)
        .unwrap();

    let friveil = FriVeilDefault::new(
        LOG_INV_RATE,
        NUM_TEST_QUERIES,
        packed_mle_values.total_n_vars,
        80,
    );

    let evaluation_point = friveil.calculate_evaluation_point_random().unwrap();

    let (fri_params, ntt) = friveil
        .initialize_fri_context(packed_mle_values.packed_mle.log_len())
        .unwrap();

    let commit_output = friveil
        .commit(
            packed_mle_values.packed_mle.clone(),
            fri_params.clone(),
            &ntt,
        )
        .unwrap();

    let mut verifier_transcript = friveil
        .prove(
            packed_mle_values.packed_mle.clone(),
            fri_params.clone(),
            &ntt,
            &commit_output,
            &evaluation_point,
        )
        .unwrap();

    let evaluation_claim = friveil
        .calculate_evaluation_claim(&packed_mle_values.packed_values, &evaluation_point)
        .unwrap();

    let proof = friveil.get_transcript_bytes(&verifier_transcript);
    let data = vec![proof.clone()];

    let request_data = GuestInput::from_proofs(
        data,
        evaluation_point,
        evaluation_claim,
        packed_mle_values.packed_mle.log_len(),
    );

    let serialized = bincode::serialize(&request_data).unwrap();
    fs::write("test_input.bin", serialized).unwrap();

    println!(
        "Input generated successfully! ({} ms)",
        start.elapsed().as_millis()
    );
    println!("Saved to test_input.bin");
}
