use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDT: Pubkey = pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
pub const USDG: Pubkey = pubkey!("2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH");
pub const PYUSD: Pubkey = pubkey!("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");

pub const STABLECOINS: [Pubkey; 4] = [USDC, USDT, USDG, PYUSD];

// Program IDs - replace with actual program IDs!!!
pub const SWAP_PROGRAM_ID: Pubkey = pubkey!("Xx11111111111111111111111111111111111111111");
pub const FLASHLOAN_ID: Pubkey = pubkey!("Xx11111111111111111111111111111111111111111");
pub const ADDRESS_LOOKUP_TABLE: Pubkey = pubkey!("Xx11111111111111111111111111111111111111111");

pub const ADDRESS_LOOKUP_TABLE_PROGRAM_ID: Pubkey = pubkey!("AddressLookupTab1e1111111111111111111111111");

pub const SPL_TOKEN_2022_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub const TABLE_ARBITRAGE: &str = "arbitrage";
pub const TABLE_ARBITRAGE_V2: &str = "arbitrage_v2";

pub const ALLOWED_TOKEN_2022: [Pubkey; 73] = [
    pubkey!("pumpCmXqMfrsAkQ5r49WcJnRayYRqmXz6ae8H7H9Dfn"),
    pubkey!("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo"),
    pubkey!("HeLp6NuQkmYB4pYWo2zYs22mESHXPQYzXbB8n4V98jwC"),
    pubkey!("2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH"),
    pubkey!("AUSD1jCcCyPLybk1YnvPWsHQSrZ46dxwoMniN4N2UEB9"),
    pubkey!("HDa3zJc12ahykSsBRvgiWzr6WLEByf36yzKKbVvy4gnF"),
    pubkey!("7dGEYMPsAVxJY3qQJaCHwLPkCCx9SSE52H4k1wF617uE"),
    pubkey!("znv3FZt2HFAvzYf5LxzVyryh3mBXWuTRRng25gEZAjh"),
    pubkey!("KHnxpfoPKYo3mPmiXYE8LGZ1uix5PEARTPYrTRi2MGF"),
    pubkey!("2G4RMDbXu79f5Yff6fBjKgXknuGvrFCr7iYmiTKy3fVc"),
    pubkey!("Ddm4DTxNZxABUYm2A87TFLY6GDG2ktM2eJhGZS3EbzHM"),
    pubkey!("2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH"),
    // xstocks
    // ABT
    pubkey!("XsHtf5RpxsQ7jeJ9ivNewouZKJHbPxhPoEy6yYvULr7"),
    // ABBV
    pubkey!("XswbinNKyPmzTa5CskMbCPvMW6G5CMnZXZEeQSSQoie"),
    // ACN
    pubkey!("Xs5UJzmCRQ8DWZjskExdSQDnbE6iLkRu2jjrRAB1JSU"),
    // GOOGL
    pubkey!("XsCPL9dNWBMvFtTmwcCA5v3xWPSMEBCszbQdiLLq6aN"),
    // AMZN
    pubkey!("Xs3eBt7uRfJX8QUs4suhyU8p2M6DoUDrJyWBa8LLZsg"),
    // AMBR
    pubkey!("XsaQTCgebC2KPbf27KUhdv5JFvHhQ4GDAPURwrEhAzb"),
    // AAPL
    pubkey!("XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp"),
    // AAP
    pubkey!("XsPdAVBi8Zc1xvv53k4JcMrQaEDTgkGqKYeh7AYgPHV"),
    // AZN
    pubkey!("Xs3ZFkPYT2BN7qBMqf1j1bfTeTm1rFzEFSsQ1z3wAKU"),
    // BAC
    pubkey!("XswsQk4duEQmCbGzfqUUWYmi7pV7xpJ9eEmLHXCaEQP"),
    // BRK
    pubkey!("Xs6B6zawENwAbWVi7w92rjazLuAr5Az59qgWKcNb45x"),
    // AVGO
    pubkey!("XsgSaSvNSqLTtFuyWPBhK9196Xb9Bbdyjj4fH3cPJGo"),
    // CVX
    pubkey!("XsNNMt7WTNA2sV3jrb1NNfNgapxRF5i4i6GcnTRRHts"),
    // CRCL
    pubkey!("XsueG8BtpquVJX9LVLLEGuViXUungE6WmK5YZ3p3bd1"),
    // CSCO
    pubkey!("Xsr3pdLQyXvDJBFgpR5nexCEZwXvigb8wbPYp4YoNFf"),
    // KO
    pubkey!("XsaBXg8dU5cPM6ehmVctMkVqoiRG2ZjMo1cyBJ3AykQ"),
    // COIN
    pubkey!("Xs7ZdzSHLU9ftNJsii5fCeJhoRWSC32SQGzGQtePxNu"),
    // CMCSA
    pubkey!("XsvKCaNsxg2GN8jjUmq71qukMJr7Q1c5R2Mk9P8kcS8"),
    // CRWD
    pubkey!("Xs7xXqkcK7K8urEqGg52SECi79dRp2cEKKuYjUePYDw"),
    // DHR
    pubkey!("Xseo8tgCZfkHxWS9xbFYeKFyMSbWEvZGFV1Gh53GtCV"),
    // DFDV
    pubkey!("Xs2yquAgsHByNzx68WJC55WHjHBvG9JsMB7CWjTLyPy"),
    // LLY
    pubkey!("Xsnuv4omNoHozR6EEW5mXkw8Nrny5rB3jVfLqi6gKMH"),
    // XOM
    pubkey!("XsaHND8sHyfMfsWPj6kSdd5VwvCayZvjYgKmmcNL5qh"),
    // GME
    pubkey!("Xsf9mBktVB9BSU5kf4nHxPq5hCBJ2j2ui3ecFGxPRGc"),
    // GLD
    pubkey!("Xsv9hRk1z5ystj9MhnA7Lq4vjSsLwzL2nxrwmwtD3re"),
    // GS
    pubkey!("XsgaUyp4jd1fNBCxgtTKkW64xnnhQcvgaxzsbAq5ZD1"),
    // HD
    pubkey!("XszjVtyhowGjSC5odCqBpW1CtXXwXjYokymrk7fGKD3"),
    // HON
    pubkey!("XsRbLZthfABAPAfumWNEJhPyiKDW6TvDVeAeW7oKqA2"),
    // INTC
    pubkey!("XshPgPdXFRWB8tP1j82rebb2Q9rPgGX37RuqzohmArM"),
    // IBM
    pubkey!("XspwhyYPdWVM8XBHZnpS9hgyag9MKjLRyE3tVfmCbSr"),
    // JNJ
    pubkey!("XsGVi5eo1Dh2zUpic4qACcjuWGjNv8GCt3dm5XcX6Dn"),
    // JPM
    pubkey!("XsMAqkcKsUewDrzVkait4e5u4y8REgtyS7jWgCpLV2C"),
    // LIN
    pubkey!("XsSr8anD1hkvNMu8XQiVcmiaTP7XGvYu7Q58LdmtE8Z"),
    // MRVL
    pubkey!("XsuxRGDzbLjnJ72v74b7p9VY6N66uYgTCyfwwRjVCJA"),
    // MA
    pubkey!("XsApJFV9MAktqnAc6jqzsHVujxkGm9xcSUffaBoYLKC"),
    // MCD
    pubkey!("XsqE9cRRpzxcGKDXj1BJ7Xmg4GRhZoyY1KpmGSxAWT2"),
    // MDT
    pubkey!("XsDgw22qRLTv5Uwuzn6T63cW69exG41T6gwQhEK22u2"),
    // MRK
    pubkey!("XsnQnU7AdbRZYe2akqqpibDdXjkieGFfSkbkjX1Sd1X"),
    // META
    pubkey!("Xsa62P5mvPszXL1krVUnU5ar38bBSVcWAB6fmPCo5Zu"),
    // MSFT
    pubkey!("XspzcW1PRtgf6Wj92HCiZdjzKCyFekVD8P5Ueh3dRMX"),
    // MSTR
    pubkey!("XsP7xzNPvEHS1m6qfanPUGjNmdnmsLKEoNAnHjdxxyZ"),
    // QQQ
    pubkey!("Xs8S1uUs1zvS2p7iwtsG3b6fkhpvmwz4GYU3gWAmWHZ"),
    // NFL
    pubkey!("XsEH7wWfJJu2ZT3UCFeVfALnVA6CP5ur7Ee11KmzVpL"),
    // NVO
    pubkey!("XsfAzPzYrYjd4Dpa9BU3cusBsvWfVB9gBcyGC87S57n"),
    // NVDA
    pubkey!("Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh"),
    // ORCL
    pubkey!("XsjFwUPiLofddX5cWFHW35GCbXcSu1BCUGfxoQAQjeL"),
    // PLTR
    pubkey!("XsoBhf2ufR8fTyNSjqfU71DYGaE6Z3SUGAidpzriAA4"),
    // PEP
    pubkey!("Xsv99frTRUeornyvCfvhnDesQDWuvns1M852Pez91vF"),
    // PFE
    pubkey!("XsAtbqkAP1HJxy7hFDeq7ok6yM43DQ9mQ1Rh861X8rw"),
    // PM
    pubkey!("Xsba6tUnSjDae2VcopDB6FGGDaxRrewFCDa5hKn5vT3"),
    // PG
    pubkey!("XsYdjDjNUygZ7yGKfQaB6TxLh2gC6RRjzLtLAGJrhzV"),
    // HOOD
    pubkey!("XsvNBAYkrDRNhA7wPHQfX3ZUXZyZLdnCQDfHZ56bzpg"),
    // CRM
    pubkey!("XsczbcQ3zfcgAEt9qHQES8pxKAVG5rujPSHQEXi4kaN"),
    // SPY
    pubkey!("XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W"),
    // TSLA
    pubkey!("XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB"),
    // TMO
    pubkey!("Xs8drBWy3Sd5QY3aifG9kt9KFs2K3PGZmx7jWrsrk57"),
    // TQQQ
    pubkey!("XsjQP3iMAaQ3kQScQKthQpx9ALRbjKAjQtHg6TFomoc"),
    // UNH
    pubkey!("XszvaiXGPwvk2nwb3o9C1CX4K6zH8sez11E6uyup6fe"),
    // VTI
    pubkey!("XsssYEQjzxBCFgvYFFNuhJFBeHNdLWYeUSP8F45cDr9"),
    // V
    pubkey!("XsqgsbXwWogGJsNcVZ3TyVouy2MbTkfCFhCGGGcQZ2p"),
    // WMT
    pubkey!("Xs151QeqTCiuKtinzfRATnUESM2xTU6V9Wy8Vy538ci"),
];

pub const STATIC_ATA: [Pubkey; 10] = [
    pubkey!("So11111111111111111111111111111111111111112"),  // wsol
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // usdc
    pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"), // usdt
    pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"), // jitosol
    pubkey!("HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3"), //  pyth
    pubkey!("USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA"),  // usds
    pubkey!("27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4"), // jlp
    pubkey!("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),  // jup
    pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1"),  // bsol
    pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"),
];
