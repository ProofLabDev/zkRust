use sha3::{Digest as _, Keccak256};

fn main() {
    let data: Vec<u8> = zk_rust_io::read();
    let hash: [u8; 32] = Keccak256::digest(&data).into();
    zk_rust_io::commit(&hash);
}

fn input() {
    let message = b"Hello, world!".to_vec();
    zk_rust_io::write(&message);
}

fn output() {
    let hash: [u8; 32] = zk_rust_io::out();
    println!("Keccak256 hash: {:?}", hash);
}
