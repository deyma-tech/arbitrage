# usage ./update_alt.sh
#       ./update_alt.sh /home/oxy/Work/Krypto/arbitrage/arb-bot/examples/config.toml

CONFIG_PATH=${1:-"/home/ubuntu/.config/arb_config.toml"}

/usr/local/bin/tables -c $CONFIG_PATH
/usr/local/bin/kv_table -c $CONFIG_PATH
