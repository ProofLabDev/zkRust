#![no_main]

risc0_zkvm::guest::entry!(main);

pub fn main() {
    // Example of reading a u32 input
    let _input: u32 = zk_rust_io::read();
}
