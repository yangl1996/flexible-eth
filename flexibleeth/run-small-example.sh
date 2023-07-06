#! /bin/sh

MAXSLOT=2560

cargo run --release -- -vv sync --rpc-url http://jneu:8Crd2iUnG0qg57M8RPcYKuPbZYZOwrLPLhHpTNvI@home.leiy.me:46235 --max-slot $MAXSLOT --rl-requests 1000

mkdir output-small-example

cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.67 > output-small-example/q67.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.80 > output-small-example/q80.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.90 > output-small-example/q90.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.95 > output-small-example/q95.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.96 > output-small-example/q96.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.97 > output-small-example/q97.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.98 > output-small-example/q98.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.99 > output-small-example/q99.txt

