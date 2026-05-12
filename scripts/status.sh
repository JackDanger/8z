#!/bin/bash
# Extract current focus and next task from STATUS.md

if [ ! -f STATUS.md ]; then
    echo "ERROR: STATUS.md not found" >&2
    exit 1
fi

echo "Current focus:"
grep "^\*\*Current focus:" STATUS.md | sed 's/^\*\*Current focus:\*\* /  /'

echo ""
echo "Next task:"
# Find the first line with ⬜ (not started) in the status columns
awk '
    NF > 0 && /^\| / && !/^\| Coder \|/ && !/^---/ {
        # Try to find a ⬜ in this row
        if ($0 ~ /⬜/) {
            # Extract the coder name (first cell after opening |)
            sub(/^\| /, "")
            name = $1
            print "  " name
            exit
        }
    }
' STATUS.md
