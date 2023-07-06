#! /bin/bash -ve

if [ $# -lt 2 ]; then
	echo "Usage: ./run-example.sh <DBPATH> <MAXSLOT>"
    exit 1
fi

DBPATH=$1
MAXSLOT=$2


cargo run --release -- -vv sync --db-path $DBPATH --rpc-url http://jneu:8Crd2iUnG0qg57M8RPcYKuPbZYZOwrLPLhHpTNvI@home.leiy.me:46235 --max-slot $MAXSLOT --rl-requests 1000


mkdir output-example-$MAXSLOT || true

cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.67 > output-example-$MAXSLOT/q67.log
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.80 > output-example-$MAXSLOT/q80.log
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.90 > output-example-$MAXSLOT/q90.log
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.95 > output-example-$MAXSLOT/q95.log
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.96 > output-example-$MAXSLOT/q96.log
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.97 > output-example-$MAXSLOT/q97.log
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.98 > output-example-$MAXSLOT/q98.log
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.99 > output-example-$MAXSLOT/q99.log


cat output-example-$MAXSLOT/q67.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q67.txt
cat output-example-$MAXSLOT/q80.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q80.txt
cat output-example-$MAXSLOT/q90.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q90.txt
cat output-example-$MAXSLOT/q95.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q95.txt
cat output-example-$MAXSLOT/q96.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q96.txt
cat output-example-$MAXSLOT/q97.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q97.txt
cat output-example-$MAXSLOT/q98.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q98.txt
cat output-example-$MAXSLOT/q99.log | awk -F'[= ]' '{ print "("$5", "$9")" }' > output-example-$MAXSLOT/q99.txt
