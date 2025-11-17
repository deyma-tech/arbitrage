# Solana Arbitrage Bot

> **⚠️ DISCLAIMER**: This project is provided for **educational and demonstration purposes only**. The bot was operational and profitable during 2025, but cryptocurrency trading involves significant risk. Use at your own risk.

## Overview

High-performance arbitrage bot for Solana blockchain, written in Rust. The bot monitors multiple decentralized exchanges (DEXs) in real-time, identifies profitable arbitrage opportunities, and executes trades automatically.

**Project Status**: Archive (2025) - Bot was operational and generated profit during its active period.

## Features

### Supported DEX Protocols
- **Raydium** (AMM, CLMM, CPMM)
- **Orca** (CLMM, Token Swap v2)
- **Meteora** (DLMM, DAMM v2, Pools, Vault)
- **Pump.fun** (Bonding Curve AMM)
- **Lifinity**
- **Stabble** (Stable Swap, Weighted Swap)
- **Fusion AMM**
- **Saros DLMM**
- **Numeraire (Goose Gamma)**

### Transaction Submission Providers
- **Jito** (Block Engine integration)
- **Jito + QuickNode** (Enhanced routing)
- **Bloxroute** (with Paladin & SwQoS support)
- **Nextblock** (Alternative submission)

### Key Capabilities
- Real-time account monitoring via Geyser plugin (NATS/ZeroMQ)
- Multi-threaded opportunity detection and execution
- Flashloan support for capital-efficient arbitrage
- Dynamic priority fee and tip calculation
- Address Lookup Table (ALT) management
- Telegram notifications and monitoring
- QuestDB integration for performance analytics
- Sophisticated error handling and automatic restarts

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Solana Validator                     │
│                   (Geyser Plugin)                       │
└────────────────┬────────────────────────────────────────┘
                 │ Account Updates
                 ├─> NATS / ZeroMQ
                 │
┌────────────────▼────────────────────────────────────────┐
│                   arb-core                              │
│  - Pool state management                                │
│  - Opportunity detection                                │
│  - Quote calculations                                   │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                   arb-bot                               │
│  - Transaction building                                 │
│  - Provider selection (Jito/Bloxroute/etc.)             │
│  - Execution & monitoring                               │
└─────────────────────────────────────────────────────────┘
```

### Components

- **arb-core**: Core arbitrage logic, pool management, opportunity detection
- **arb-bot**: Main bot executable, transaction execution
- **dex**: DEX protocol implementations and swap calculations
- **geyser-nats**: Geyser plugin for account streaming via NATS
- **geyser-zeromq**: Geyser plugin for account streaming via ZeroMQ
- **jito_searcher_client**: Jito Block Engine client
- **bloxroute-client**: Bloxroute integration
- **nextblock-client**: Nextblock integration
- **config**: Configuration management
- **utils**: Shared utilities (Telegram, serialization, etc.)
- **validators**: Validator selection and scheduling

## Technical Stack

- **Language**: Rust (Edition 2021)
- **Blockchain**: Solana
- **Streaming**: NATS, ZeroMQ
- **Database**: QuestDB (time-series data)
- **Monitoring**: Telegram Bot API
- **Build**: Cargo workspace

## Configuration

The bot uses TOML configuration files. Generate a default configuration template:

```bash
cargo run --bin arb-bot -- --dump-default-config > config.toml
```

Edit `config.toml` with your settings:

**Key configuration parameters:**
- **RPC/WS endpoints** - Helius, QuickNode, or custom endpoints
- **Keypair path** - Your Solana wallet keypair location
- **Execution providers** - Choose from Jito, Bloxroute, Nextblock
- **Arbitrage settings** - Thresholds, quote amounts, filters
- **Priority fees** - Fee percentages and tip strategies
- **Telegram notifications** - Bot token and chat ID
- **Region** - EU/US/GB/JP/NL for optimal routing
- **ALT manager** - Address lookup table settings

**Note**: The `.env.sample` file shows environment variable examples. The `scripts/arb_monitor.sample.toml` is only for the monitoring script, not the main bot configuration.

## Building

```bash
# Build all components
cargo build --release

# Build specific binary
cargo build --release --bin arb-bot

# Dump default configuration
cargo run --bin arb-bot -- --dump-default-config

# Show version
cargo run --bin arb-bot -- -v
```

## Deployment Requirements

- Solana validator with Geyser plugin support
- Fast RPC access (dedicated endpoints recommended)
- Low-latency network connection
- Sufficient SOL for transaction fees and tips
- Optional: QuestDB instance for analytics

## Geyser Plugin Setup

**⚠️ Note**: Geyser plugins (`geyser-nats` and `geyser-zeromq`) are **included in this repository for reference only**. In production, these plugins must be compiled and loaded on a Solana validator node.

The bot requires a Geyser plugin to stream account updates in real-time:

1. Compile plugin with Rust 1.81 (required for Geyser plugin compatibility):
   ```bash
   cd geyser-nats  # or geyser-zeromq
   cargo build --release
   ```

2. Copy compiled plugin to validator machine and configure the validator to load it (see `geyser-*/config.json` for configuration examples)

## Monitoring

The project includes a Python-based monitoring script (`scripts/arb_monitor.py`) that:
- Tracks service uptime
- Sends Telegram alerts for issues
- Monitors service health via systemd

## DEX Protocol Deserialization Optimizations

A critical performance optimization in this bot is the **custom deserialization** of DEX protocol account data:

### Zero-Copy Deserialization
- **Borsh and Anchor frameworks removed** - Standard frameworks add significant overhead
- **Direct byte-level deserialization** - Account data is parsed directly from raw bytes using:
  - `bytemuck` crate with `Pod` and `Zeroable` traits for zero-copy transmutation
  - Manual field extraction via `u64::from_le_bytes()`, `u128::from_le_bytes()`, etc.
  - Direct buffer slicing for nested structures

### Performance Impact
This approach provides:
- **Microsecond-level deserialization** vs milliseconds with Borsh/Anchor
- **Zero allocations** for most account structures
- **Minimal CPU overhead** - critical for high-frequency opportunity detection

### Example Implementation
```rust
// Instead of Borsh deserialization:
// let pool = AmmInfo::deserialize(&mut reader)?;

// Direct deserialization from bytes:
let pool = bytemuck::from_bytes::<AmmInfo>(&data[..]);

// Or manual field extraction:
let trade_fee_rate = u64::from_le_bytes(buf[12..20].try_into()?);
```

See `dex/src/*/accounts.rs` files for full implementation details across different DEX protocols.

## Performance Considerations

- Optimized for low-latency execution (sub-second opportunity detection)
- Multi-threaded architecture for parallel processing
- Sparse data structures for efficient memory usage
- Connection pooling and persistent connections
- Strategic use of compute units (CU) optimization
- Custom deserialization (see above) eliminates framework overhead

## Risks and Limitations

**⚠️ Important Warnings:**

1. **Market Risk**: Cryptocurrency markets are highly volatile
2. **Execution Risk**: Network congestion can cause failed transactions
3. **MEV Competition**: Other bots compete for the same opportunities
4. **Smart Contract Risk**: DEX protocols may have bugs or be exploited
5. **Capital Risk**: Bot requires capital for trades and transaction fees
6. **Regulatory Risk**: Crypto regulations vary by jurisdiction

## Development Notes

See inline TODO list in NOTES.md for development roadmap and known issues.

### Key Implementation Details

- Supports exact-in and exact-out swap calculations
- Handles Token2022 (SPL Token 2022) compatibility
- Manages bin/tick arrays for CLMM protocols
- Implements flashloan strategies for capital efficiency
- Dynamic selection of block engines based on region

## Contributing

This is an archived project provided for educational purposes. Fork and modify as needed for your own research and development.

## License

This project is provided as-is without warranty. Use at your own risk.

## Acknowledgments

Built on top of various open-source Solana libraries and protocols:
- Solana SDK
- Anchor Framework
- Jito Labs
- Various DEX protocols

---

**Final Note**: This bot represents significant engineering effort in the MEV/arbitrage space on Solana. While it was profitable in 2025, market conditions change rapidly. Past performance does not guarantee future results. Always conduct your own research and testing.
