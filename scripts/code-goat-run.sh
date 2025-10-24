#!/bin/bash

# cargo build && sudo ./target/debug/cli

function get_time_us() {
    date +%s%6N
}

cd "/workspaces/code-goat/code-goat"
cargo build --release -p judger

cd "$(dirname "$0")"
gcc code_goat_run.c -o code_goat_run.o -L/workspaces/code-goat/code-goat/target/release -ljudger

echo "Current time in µs (using gdate): $CURRENT_TIME_US"

START_TIME=$(get_time_us)

cd /workspaces/code-goat/code-goat
cargo build && sudo ./target/debug/cli
# sudo env "LD_LIBRARY_PATH=$LD_LIBRARY_PATH:." ./code_goat_run.o

END_TIME=$(get_time_us)

echo "$((END_TIME - START_TIME))ms elapsed"


