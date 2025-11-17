### Notes

geyser plugin needs to be compiled with 1.81 rustup!!! 

### TODO

  - [ ] Deser: Saros DLMM
  - [ ] Deser: Numeraire
  - [ ] Saros DLMM - to launch
  - [ ] Support for fees in Token2022
  - [ ] Dynamic filter

#### Done

  - [x] Orca Whirpool calc fix (tick arrays) 
  - [x] Saros AMM
  - [x] Deser: Meteora DAMM
  - [x] Deser: Stabble Stable Swap 
  - [x] ALT manager (service) + refresh bots without restart
  - [x] Full support for Token2022
  - [x] ALT for tick/bin arrays
  - [x] Fusion AMM
  - [x] Check Orca dynamic arrays

### TODO (historical)

  - [x] Check calculations
  - [x] Helius provider
  - [ ] Statistics (cumulative)
  - [ ] Invariant swap **on hold**
  - [x] Jito proxy
  - [ ] Finalize refresh of the ALT
  - [ ] Four combinations (depends on ALT)
  - [x] Estimations of CU
  - [x] Automated bot restarts by cron
  - [x] Automated bot restarts - sophisticated version
  - [ ] meteora pools
  - [ ] Perena perena.org (stable coins)
  - [ ] Monitoring of Bloxroute GW
  - [ ] Monitoring of RPC
  - [x] Bump solana to version 2.2.x (primary: Geyser & RPC)
  - [x] SPLToken2022 - as list
  - [ ] Send tx according to leader's location
  - [ ] orca selecting and generating empty arrays ... if not - change of index needs to be continous
  - [x] usdc/usdt arbitrage (nepriamo)
  - [ ] [orca token swap v2](https://github.com/jup-ag/jupiter-amm-implementation/blob/main/jupiter-core/src/amms/spl_token_swap_amm.rs)
  - [x] spawn objemov
  - [x] logy quest db
  - [x] sparse vectors for bin/tick arrays
  - [x] helius sqwos!!!! so simulaciou todo vyskusat
  - [x] quicknode jito!!! vyskusat 10rps
  - [x] geyser-nats (zeromq)
  - [x] pump swap amm
  - [x] raydium amm
  - [x] raydium clmm
  - [x] meteora dlmm
  - [x] meteora damm v2 (cp amm)
  - [x] orca clmm
  - [ ] Obric v2 **on hold**
  - [ ] Sanctum  
  - [ ] phoenix
  - [ ] solfi
  - [ ] Kamino swap
  - [x] lifinity
  - [x] jup.ag (nerealizovateľné, binárky vyhúli RCP)
  - [x] stabble stable swap https://github.com/stabbleorg/amm-sdk
  - [x] Počet bin/tick array ako param s default = 3
  - [x] implement in Pool trait method quote_exact_in_ix and quote_exact_out_ix for generating instruction
  - [ ] Orca: 5 accounts (2 additionals)
  - [x] Paladin - for big arbitrages ...
  - [x] flashloan
  - [x] Overiť či FxHashMap je lepšíe ako AHashMap
  - [ ] Zaviesť viac typov chyby (jeden error maže všetky cenové dáta, ďalší maže len dáta od hranice vyššie)
  - [ ] Selekcia oportunitých na základe aktivity mint tokenu (aktivity všetkých pool-ov pre nejaký token)
  - [ ] sync data via rwlock
  - [x] kombinacie ab/bc/ca kde a->b exact out,  b->c exact in, c->a exact in

### arb-bot command

**Dump version**

```
cargo r --bin arb-bot -- -v
```


**Dump config**

```
cargo r --bin arb-bot -- --dump-default-config 
```

### Jito servers

https://docs.jito.wtf/lowlatencytxnsend/

### Bloxroute 

https://docs.bloxroute.com/solana/trader-api/introduction/regions

###

```
cargo install cargo-udeps
# cargo +nightly udeps --workspace --all-targets --exclude examples
# cargo +nightly udeps --workspace --lib --bins --tests --benches
```

Telegram:

1. reset offset: `curl https://api.telegram.org/bot${TELEGRAM_TOKEN}/getUpdates?offset=-1`
2. send test message
3. get updates: `curl https://api.telegram.org/bot${TELEGRAM_TOKEN}/getUpdates`
