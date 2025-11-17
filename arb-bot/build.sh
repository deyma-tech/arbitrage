SRC_DIR=~/arbitrage
BIN_DIR=/usr/local/bin

cd $SRC_DIR/arb-bot

cargo rustc --release --bin arb-bot -- \
  -C target-cpu=native \
  -C codegen-units=1 \
  -C opt-level=3 \
  -C embed-bitcode=yes \
  -C lto=thin \
  -C panic=abort \
  -C overflow-checks=off

cd $SRC_DIR

sha256sum target/release/arb-bot

sudo systemctl stop arb
sudo cp $BIN_DIR/arb-bot $BIN_DIR/arb-bot.bak
sudo cp target/release/arb-bot $BIN_DIR/arb-bot
sudo systemctl restart arb

sudo systemctl status arb
echo "arb-bot build complete"
$BIN_DIR/arb-bot --version
