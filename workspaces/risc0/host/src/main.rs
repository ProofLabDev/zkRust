// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

const PROOF_FILE_PATH: &str = "../../proof_data/risc0/risc0.proof";
const IMAGE_ID_FILE_PATH: &str = "../../proof_data/risc0/risc0.imageid";
const PUBLIC_INPUT_FILE_PATH: &str = "../../proof_data/risc0/risc0_pub_input.pub";

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // INPUT //

    let env = ExecutorEnv::builder().build().unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove(env, METHOD_ELF).unwrap().receipt;

    let verification_result = receipt.verify(METHOD_ID).is_ok();

    println!("Verification result: {}", verification_result);

    // OUTPUT //

    let serialized = bincode::serialize(&receipt).unwrap();

    std::fs::write(PROOF_FILE_PATH, serialized).expect("Failed to write proof file");
    std::fs::write(IMAGE_ID_FILE_PATH, convert(&METHOD_ID))
        .expect("Failed to write fibonacci_id file");
    std::fs::write(PUBLIC_INPUT_FILE_PATH, receipt.journal.bytes)
        .expect("Failed to write pub_input file");
}

pub fn convert(data: &[u32; 8]) -> [u8; 32] {
    let mut res = [0; 32];
    for i in 0..8 {
        res[4 * i..4 * (i + 1)].copy_from_slice(&data[i].to_le_bytes());
    }
    res
}
