#!/bin/bash
set -euo pipefail
echo "Building arb-bot..."

cd /home/ubuntu/arbitrage/

export RUSTFLAGS="-Ctarget-cpu=native"
cargo b -r --bin arb-bot

./target/release/arb-bot --version

echo "arb-bot build complete. Helper commands:"

echo "sudo systemctl stop arb"
echo "sudo cp ./target/release/arb-bot /usr/local/bin"
echo "sudo systemctl start arb"