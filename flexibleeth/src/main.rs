use clap::{Parser, Subcommand};
use ratelimit::Ratelimiter;
use std::time::Duration;

mod confrule;
mod data;
mod sync;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Synchronize consensus metadata (blocks, votes, ...) from beacon chain RPC endpoint to caching database
    Sync {
        /// Caching database path
        #[arg(long, default_value = "cache.rocksdb")]
        db_path: String,

        /// Beacon chain RPC endpoint URL
        #[arg(long, default_value = "https://lodestar-mainnet.chainsafe.io")]
        rpc_url: String,

        /// Minimum slot to synchronize
        #[arg(long, default_value = "0")]
        min_slot: usize,

        /// Maximum slot to synchronize
        #[arg(long)]
        max_slot: usize,

        /// Rate limit for beacon chain RPC endpoint: requests (numerator)
        #[arg(long, default_value_t = 10)]
        rl_requests: usize,

        /// Rate limit for beacon chain RPC endpoint: seconds (denominator)
        #[arg(long, default_value_t = 1.0)]
        rl_seconds: f64,
    },

    /// Run flexible confirmation-rule based on consensus metadata found in caching database
    ConfRule {
        /// Caching database path
        #[arg(long, default_value = "cache.rocksdb")]
        db_path: String,

        /// Confirmation quorum
        #[arg(long, num_args = 1..)]
        quorum: Vec<f64>,

        /// Minimum slot to process
        #[arg(long, default_value = "0")]
        min_slot: usize,

        /// Maximum slot to process
        #[arg(long)]
        max_slot: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    env_logger::Builder::from_default_env()
        .filter_level(match cli.verbose {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Warn,
            2 => log::LevelFilter::Info,
            3 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .format_module_path(true)
        .format_timestamp_millis()
        .init();

    match cli.command {
        Commands::Sync {
            db_path,
            rpc_url,
            min_slot,
            max_slot,
            rl_requests,
            rl_seconds,
        } => {
            crate::sync::main(
                db_path,
                rpc_url,
                min_slot,
                max_slot,
                Ratelimiter::builder(rl_requests as u64, Duration::from_secs_f64(rl_seconds))
                    .max_tokens(rl_requests as u64 * 3)
                    .build()
                    .unwrap(),
            )
            .await
        }
        Commands::ConfRule {
            db_path,
            quorum,
            min_slot,
            max_slot,
        } => crate::confrule::main(db_path, quorum, min_slot, max_slot).await,
    }
}
