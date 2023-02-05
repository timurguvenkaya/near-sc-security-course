fn main() {
    let mut vec = Vec::<String>::new();

    vec.push("A".repeat(usize::MAX).to_string());
}