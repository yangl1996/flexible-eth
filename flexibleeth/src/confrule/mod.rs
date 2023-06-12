pub async fn main(
    db_path: String,
    quorum: f64,
    max_slot: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Confirmation rule called!");
    println!("DB path: {:?}", db_path);
    println!("Quorum: {:?}", quorum);
    println!("Max slot: {:?}", max_slot);

    Ok(())
}
