#!/bin/sh

HDLC=${HDLC:-hdlc}

SRCDIR="src"
BUILDDIR="build"

if [ -n "$1" ]; then
    SRCFILE="$1.hdl"
    OUTFILE="$1.json"
else
    SRCFILE="main.hdl"
    OUTFILE="main.json"
    HDLCFLAGS="$HDLCFLAGS -B=--no-io-components,--testbench,$SRCDIR/testbench.json"
fi

OUTPATH="$BUILDDIR/$OUTFILE"

mkdir -p $(dirname "$OUTPATH")

$HDLC "$SRCDIR/$SRCFILE" -o "$OUTPATH" $HDLCFLAGS
