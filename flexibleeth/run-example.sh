#! /bin/sh

MAXSLOT=25600

cargo run --release -- -vv sync --rpc-url http://jneu:8Crd2iUnG0qg57M8RPcYKuPbZYZOwrLPLhHpTNvI@home.leiy.me:46235 --max-slot $MAXSLOT --rl-requests 1000

mkdir output-example-$MAXSLOT

cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.67 > output-example-$MAXSLOT/q67.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.80 > output-example-$MAXSLOT/q80.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.90 > output-example-$MAXSLOT/q90.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.95 > output-example-$MAXSLOT/q95.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.96 > output-example-$MAXSLOT/q96.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.97 > output-example-$MAXSLOT/q97.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.98 > output-example-$MAXSLOT/q98.txt
cargo run --release -- -vv conf-rule --max-slot $MAXSLOT --quorum 0.99 > output-example-$MAXSLOT/q99.txt

