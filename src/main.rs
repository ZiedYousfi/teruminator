fn main() {
    println!("Hello, teruminator!");
    let mut counter = 0;
    loop {
        counter += 1;
        print!("Test {counter} \r");
    }
}