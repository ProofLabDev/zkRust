use sp1_sdk::{ProverClient, SP1Stdin};
mod metrics;
use metrics::{MetricsCollector, SP1Metrics};
use tracing::{error, info};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
///
/// This file is generated by running `cargo prove build` inside the `program` directory.
pub const METHOD_ELF: &[u8] = include_bytes!(
    "../../program/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/method"
);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let current_dir = std::path::PathBuf::from(args[1].clone());
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    let mut metrics = SP1Metrics::default();
    let mut core_timer = MetricsCollector::new();
    let mut compress_timer = MetricsCollector::new();

    // Setup the inputs and set as mutable to allow for template code to access it if needed
    let mut stdin = SP1Stdin::new();

    // INPUT //

    let client = ProverClient::from_env();
    let (pk, vk) = client.setup(METHOD_ELF);

    // First run executor to get cycle count
    let (_, report) = client.execute(METHOD_ELF, &stdin.clone()).run().unwrap();
    // Get total cycles from cycle tracker
    metrics.cycles = report.cycle_tracker.iter().map(|(_, cycles)| *cycles).sum();
    // Number of segments is the number of cycle tracking entries
    metrics.num_segments = report.cycle_tracker.len();

    // Generate uncompressed proof
    core_timer.start_timing();
    // Set as mutable to allow for template code to access it if needed
    let mut proof = client.prove(&pk, &stdin.clone()).run().unwrap();
    metrics.core_prove_duration = core_timer.elapsed().unwrap();

    // Get uncompressed proof size
    let core_bytes = bincode::serialize(&proof).unwrap();
    metrics.core_proof_size = core_bytes.len();

    // Verify uncompressed proof
    core_timer.start_timing();
    client
        .verify(&proof, &vk)
        .expect("Failed to verify uncompressed proof");
    metrics.core_verify_duration = core_timer.elapsed().unwrap();

    // Generate compressed proof
    compress_timer.start_timing();
    let compressed = client
        .prove(&pk, &stdin)
        .compressed() // Enable compression
        .run()
        .unwrap();
    metrics.compress_prove_duration = compress_timer.elapsed().unwrap();

    // Get compressed proof size
    let compressed_bytes = bincode::serialize(&compressed).unwrap();
    metrics.recursive_proof_size = compressed_bytes.len();

    // Verify compressed proof
    compress_timer.start_timing();
    client
        .verify(&compressed, &vk)
        .expect("Failed to verify compressed proof");
    metrics.compress_verify_duration = compress_timer.elapsed().unwrap();

    // OUTPUT //

    // Save proof artifacts
    std::fs::create_dir_all(current_dir.join("proof_data/sp1"))
        .expect("Failed to create proof_data/sp1");
    std::fs::write(
        current_dir.join("proof_data/sp1/sp1.proof"),
        compressed_bytes,
    )
    .expect("Failed to save SP1 Proof file");
    std::fs::write(current_dir.join("proof_data/sp1/sp1.elf"), METHOD_ELF)
        .expect("Failed to create SP1 elf file");
    std::fs::write(
        current_dir.join("proof_data/sp1/sp1.pub"),
        &compressed.public_values,
    )
    .expect("Failed to save SP1 public input");

    // Save metrics
    info!("Attempting to save metrics...");
    match metrics::write_metrics(&metrics, current_dir.join("proof_data/sp1").as_path()) {
        Ok(_) => info!("Successfully saved metrics"),
        Err(e) => error!("Failed to save metrics: {}", e),
    };
}
