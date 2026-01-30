#![no_main]

use pico_sdk::entrypoint;
use proof_core::GuestInputTuple;
use FRIVeil::{
    friveil::FriVeilDefault,
    traits::B128,
    traits::{FRIVeilSampling, FriVeilUtils},
};

const LOG_INV_RATE: usize = 1;
const NUM_TEST_QUERIES: usize = 128;

entrypoint!(main);

pub fn main() {
    let (data, evaluation_point, evaluation_claim, packed_log_val): GuestInputTuple =
        pico_sdk::io::read_as();

    // Add 7 (LOG_SCALAR_BIT_WIDTH for B128) to packed_log_val to get total_n_vars
    // This matches how gen_input.rs initializes FriVeil
    let friveil = FriVeilDefault::new(LOG_INV_RATE, NUM_TEST_QUERIES, packed_log_val + 7, 80);

    let (fri_params, _) = friveil
        .initialize_fri_context(packed_log_val)
        .expect("Failed to initialize FRI context");

    let evaluation_claim_val = B128::from(u128::from_le_bytes(evaluation_claim));
    let evaluation_point_vec = evaluation_point
        .iter()
        .map(|p| B128::from(u128::from_le_bytes(*p)))
        .collect::<Vec<_>>();

    for proof in data.iter() {
        // let mut verifier_transcript = friveil.reconstruct_transcript_from_bytes(proof.to_vec());
        //
        // let result = friveil.verify_evaluation(
        //     &mut verifier_transcript,
        //     evaluation_claim_val,
        //     &evaluation_point_vec,
        //     &fri_params,
        // );
        //
        // assert!(
        //     result.is_ok(),
        //     "FRI verification failed for proof {:?}",
        //     result
        // );
    }

    pico_sdk::io::commit(&true);
}
