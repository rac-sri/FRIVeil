fn main() {
    let data: Vec<Vec<u8>> = vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9, 10]];

    let evaluation_point: Vec<[u8; 16]> = vec![
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let evaluation_claim: [u8; 16] = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let packed_values_log_len: usize = 4;

    let guest_input_tuple = (
        data,
        evaluation_point,
        evaluation_claim,
        packed_values_log_len,
    );

    let serialized =
        bincode::serialize(&guest_input_tuple).expect("Failed to serialize guest input");

    std::fs::write("/tmp/test_input.bin", &serialized).expect("Failed to write test input file");

    println!("Test input file created at /tmp/test_input.bin");
    println!("File size: {} bytes", serialized.len());
}
