#! /bin/bash -ve

if [ $# -lt 3 ]; then
	echo "Usage: ./truncate-log.sh <DATADIR> <MINSLOT> <MAXSLOT>"
    exit 1
fi

DATADIR=$1
MINSLOT=$2
MAXSLOT=$3

truncate () {
	cat $1 | sed 's/[(),]/ /g' | awk '$1 >= '"$MINSLOT"' && $1 <= '"$MAXSLOT"' {print "("$1", "$2")";}'
}

OUTDIR="truncated-example-$MINSLOT-$MAXSLOT"
mkdir -p $OUTDIR

truncate $DATADIR/q67.txt > $OUTDIR/q67.txt
truncate $DATADIR/q80.txt > $OUTDIR/q80.txt
truncate $DATADIR/q90.txt > $OUTDIR/q90.txt
truncate $DATADIR/q95.txt > $OUTDIR/q95.txt
truncate $DATADIR/q96.txt > $OUTDIR/q96.txt
truncate $DATADIR/q97.txt > $OUTDIR/q97.txt
truncate $DATADIR/q98.txt > $OUTDIR/q98.txt
truncate $DATADIR/q99.txt > $OUTDIR/q99.txt
truncate $DATADIR/finalized.txt > $OUTDIR/finalized.txt
MINSLOT=`expr $MINSLOT / 32`
MAXSLOT=`expr $MAXSLOT / 32`
truncate $DATADIR/participation.txt > $OUTDIR/participation.txt
