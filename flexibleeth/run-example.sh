#! /bin/bash -ve

if [ $# -lt 2 ]; then
	echo "Usage: ./run-example.sh <DBPATH> <MAXSLOT>"
    exit 1
fi

DBPATH=$1
MAXSLOT=$2
# BEACONAPIURL=http://flexeth:8Crd2iUnG0qg57M8RPcYKuPbZYZOwrLPLhHpTNvI@localhost:46235
BEACONAPIURL=http://localhost:5052


cargo run --release -- -vv sync --db-path $DBPATH --rpc-url $BEACONAPIURL --max-slot $MAXSLOT --rl-requests 1000
mkdir output-example-$MAXSLOT || true
cargo run --release -- -vv conf-rule --db-path $DBPATH --max-slot $MAXSLOT --quorum 0.67 --quorum 0.80 --quorum 0.90 --quorum 0.95 --quorum 0.96 --quorum 0.97 --quorum 0.98 --quorum 0.99 > output-example-$MAXSLOT/conf-rule-log.txt

cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.67," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q67.txt
cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.8," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q80.txt
cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.9," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q90.txt
cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.95," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q95.txt
cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.96," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q96.txt
cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.97," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q97.txt
cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.98," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q98.txt
cat output-example-$MAXSLOT/conf-rule-log.txt | grep "quorum: 0.99," | awk -F'[= ]' '{ print "("$3", "$11")" }' > output-example-$MAXSLOT/q99.txt

