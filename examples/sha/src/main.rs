//For acceleration we require the user defines the respective crate import since they are specific and needed to compile
use sha2::{Digest, Sha256};
use zk_rust_io;

fn main() {
    let data: String = zk_rust_io::read();
    let digest = Sha256::digest(&data.as_bytes());
    let digest_array: [u8; 32] = digest.into();
    zk_rust_io::commit(&data);
    zk_rust_io::commit(&digest_array);
}

fn input() {
    let data: String = "RISCV IS COOL!!!".to_string();
    zk_rust_io::write(&data);
}

fn output() {
    let (data, digest): (String, [u8; 32]) = zk_rust_io::out();
    println!("Input data: {}", data);
    println!("SHA256 digest: {:?}", digest);
}
