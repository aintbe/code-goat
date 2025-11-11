#!/bin/bash

# =========================================================
# Extract unique syscall names from strace output
# =========================================================
if [ "$#" -eq 0 ]; then
    echo "Error: Enter your program to run and its arguments." >&2
    echo "Usage: $0 ./trace-syscall.sh ./your_program_name [program_args...]" >&2
    exit 1
fi

# Call strace and redirect to stderr
strace -f "$@" 2>&1 |

# Extract the first word(= syscall) from each line
grep -E '^[a-z0-9_]+\(' |
cut -d'(' -f1 |

# No duplication, in alphabetical order
sort -u
