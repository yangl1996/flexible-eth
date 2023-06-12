use ratelimit::Ratelimiter;
use reqwest;
use rocksdb::{Options, DB};

fn ratelimiter_wait(ratelimiter: &mut Ratelimiter) {
    while let Err(sleep) = ratelimiter.try_wait() {
        std::thread::sleep(sleep);
    }
}

pub async fn main(
    db_path: String,
    rpc_url: String,
    max_slot: usize,
    mut ratelimiter: Ratelimiter,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = DB::open_default(db_path)?;
    let rpc = reqwest::Client::new();

    for slot in 0..max_slot {
        ratelimiter_wait(&mut ratelimiter);
    }

    Ok(())
}
