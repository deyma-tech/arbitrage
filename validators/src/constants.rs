pub const JITO_VALIDATORS: &str = "https://kobe.mainnet.jito.network/api/v1/validators";
pub const NEW_YORK: &str = "40.730,-73.9352";
pub const SALT_LAKE_CITY: &str = "40.758,-111.876";
pub const LONDON: &str = "51.509,-0.118";
pub const AMSTERDAM: &str = "52.377,4.897";
pub const FRANKFURT: &str = "50.110,8.682";
pub const DALLAS: &str = "32.779,96.808";
pub const LOS_ANGELES: &str = "34.098,-118.327";
pub const TOKYO: &str = "35.652,139.839";
pub const PITTSBURGH: &str = "40.440,-79.995";

pub const BLOXROUTE_PATHS: [(&str, &str); 6] = [
    ("https://uk.solana.dex.blxrbdn.com", LONDON),
    ("https://ny.solana.dex.blxrbdn.com", NEW_YORK),
    ("https://la.solana.dex.blxrbdn.com", LOS_ANGELES),
    ("https://germany.solana.dex.blxrbdn.com", FRANKFURT),
    ("https://amsterdam.solana.dex.blxrbdn.com", AMSTERDAM),
    ("https://tokyo.solana.dex.blxrbdn.com", TOKYO),
];

// "mainnet","amsterdam","frankfurt","ny","tokyo","slc"
pub const JITO_PATHS: [(&str, &str, &str); 6] = [
    ("ny", "https://ny.mainnet.block-engine.jito.wtf", NEW_YORK),
    ("slc", "https://slc.mainnet.block-engine.jito.wtf", SALT_LAKE_CITY),
    (
        "frankfurt",
        "https://frankfurt.mainnet.block-engine.jito.wtf",
        FRANKFURT,
    ),
    (
        "amsterdam",
        "https://amsterdam.mainnet.block-engine.jito.wtf",
        AMSTERDAM,
    ),
    ("tokyo", "https://tokyo.mainnet.block-engine.jito.wtf", TOKYO),
    ("london", "https://london.mainnet.block-engine.jito.wtf", LONDON),
];
/*


https://mainnet.block-engine.jito.wtf
🇳🇱 Amsterdam
https://amsterdam.mainnet.block-engine.jito.wtf
74.118.140.240:1002
http://amsterdam.mainnet.relayer.jito.wtf:8100
ntp.amsterdam.jito.wtf
🇩🇪 Frankfurt
https://frankfurt.mainnet.block-engine.jito.wtf
64.130.50.14:1002
http://frankfurt.mainnet.relayer.jito.wtf:8100
ntp.frankfurt.jito.wtf
🇺🇸 New York
https://ny.mainnet.block-engine.jito.wtf
141.98.216.96:1002
http://ny.mainnet.relayer.jito.wtf:8100
ntp.dallas.jito.wtf

🇯🇵 TokYO
https://tokyo.mainnet.block-engine.jito.wtf
202.8.9.160:1002
http://tokyo.mainnet.relayer.jito.wtf:8100
ntp.tokyo.jito.wtf
🇺🇸 Salt Lake City
https://slc.mainnet.block-engine.jito.wtf
64.130.53.8:1002
http://slc.mainnet.relayer.jito.wtf:8100
ntp.slc.jito.wtf
 */
