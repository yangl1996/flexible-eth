pub fn main(
    db_path: String,
    quorum: f64,
    max_slot: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", db_path);
    println!("{:?}", quorum);
    println!("{:?}", max_slot);

    Ok(())
}
