cd /home/ubuntu/arbitrage
cargo b -r --example tables
cargo b -r --example kv_table

sudo cp target/release/examples/tables /usr/local/bin/
sudo cp target/release/examples/kv_table /usr/local/bin/