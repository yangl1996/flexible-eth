#! /bin/bash -ve

if [ $# -lt 3 ]; then
	echo "Usage: ./run-example.sh <DBPATH> <MINSLOT> <MAXSLOT>"
    exit 1
fi

DBPATH=$1
MINSLOT=$2
MAXSLOT=$3
# BEACONAPIURL=http://flexeth:8Crd2iUnG0qg57M8RPcYKuPbZYZOwrLPLhHpTNvI@localhost:46235
# BEACONAPIURL=http://flexeth:8Crd2iUnG0qg57M8RPcYKuPbZYZOwrLPLhHpTNvI@185.209.178.219:46235
BEACONAPIURL=http://localhost:5052

cargo run --release -- -vv sync --db-path $DBPATH --rpc-url $BEACONAPIURL --min-slot $MINSLOT --max-slot $MAXSLOT --rl-requests 1000
mkdir output-example-$MINSLOT-$MAXSLOT || true
cargo run --release -- -vv conf-rule --db-path $DBPATH --min-slot $MINSLOT --max-slot $MAXSLOT --quorum 0.67 --quorum 0.80 --quorum 0.90 --quorum 0.95 --quorum 0.96 --quorum 0.97 --quorum 0.98 --quorum 0.99 > output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt

cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.67$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q67.txt
cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.8$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q80.txt
cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.9$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q90.txt
cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.95$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q95.txt
cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.96$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q96.txt
cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.97$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q97.txt
cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.98$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q98.txt
cat output-example-$MINSLOT-$MAXSLOT/conf-rule-log.txt | grep "quorum=0.99$" | sed 's/LEDGER t=\([0-9]*\) tip=\([0-9]*\), quorum=.*/(\1, \2)/' > output-example-$MINSLOT-$MAXSLOT/q99.txt

