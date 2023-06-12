use ratelimit::Ratelimiter;

pub fn main(
    db_path: String,
    rpc_url: String,
    rl: Ratelimiter,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", db_path);
    println!("{:?}", rpc_url);
    // println!("{:?}", rl);

    Ok(())
}
