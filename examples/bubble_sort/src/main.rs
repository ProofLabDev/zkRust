use zk_rust_io;

fn main() {
    // Read the input array
    let mut input: Vec<i32> = zk_rust_io::read();

    // Commit the original array
    zk_rust_io::commit(&input);

    // Bubble sort implementation
    let n = input.len();
    for i in 0..n {
        for j in 0..n - i - 1 {
            if input[j] > input[j + 1] {
                input.swap(j, j + 1);
            }
        }
    }

    // Commit the sorted array
    zk_rust_io::commit(&input);
}

fn input() {
    // Example input array
    let numbers = vec![64, 34, 25, 12, 22, 11, 90];
    zk_rust_io::write(&numbers);
}

fn output() {
    let (original, sorted): (Vec<i32>, Vec<i32>) = zk_rust_io::out();

    println!("Original array: {:?}", original);
    println!("Sorted array:   {:?}", sorted);
}
