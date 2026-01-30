fn main() {
    let data = std::fs::read("/tmp/test_input.bin").expect("Failed to read test input file");

    let guest_input_tuple: (Vec<Vec<u8>>, Vec<[u8; 16]>, [u8; 16], usize) =
        bincode::deserialize(&data).expect("Failed to deserialize guest input");

    println!("✓ Successfully deserialized test input");
    println!("  Data vectors: {}", guest_input_tuple.0.len());
    println!("  Evaluation points: {}", guest_input_tuple.1.len());
    println!("  Packed values log len: {}", guest_input_tuple.3);
    println!("  File size: {} bytes", data.len());
}
