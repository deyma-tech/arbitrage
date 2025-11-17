pub const BLOXROUTE_MIN_TIP: u64 = 1_000_000;
pub const JITO_MIN_TIP: u64 = 1_000;
pub const BLOXROUTE_PALADIN_MIN_TIP: u64 = 50_000_000;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExecutionProviderType {
    Jito,
    BloxrouteJito,
    BloxrouteSwQoS,
    JitoQuicknode,
    Rpc,
    Paladin,
    BloxroutePaladin,
    SwQoS,
    HeliusSwQos,
    Nextblock,
    BloxrouteJitoBundle,
}

impl ExecutionProviderType {
    pub fn get_min_tip(&self) -> u64 {
        match &self {
            ExecutionProviderType::BloxrouteJito => BLOXROUTE_MIN_TIP,
            ExecutionProviderType::BloxrouteSwQoS => BLOXROUTE_MIN_TIP,
            ExecutionProviderType::Jito => JITO_MIN_TIP,
            ExecutionProviderType::JitoQuicknode => JITO_MIN_TIP,
            ExecutionProviderType::Rpc => 0,
            ExecutionProviderType::Paladin => 0,
            ExecutionProviderType::BloxroutePaladin => BLOXROUTE_PALADIN_MIN_TIP,
            ExecutionProviderType::SwQoS => 0,
            ExecutionProviderType::HeliusSwQos => 0,
            ExecutionProviderType::Nextblock => BLOXROUTE_MIN_TIP,
            ExecutionProviderType::BloxrouteJitoBundle => BLOXROUTE_MIN_TIP,
        }
    }

    pub fn get_tag(&self) -> u64 {
        match &self {
            ExecutionProviderType::BloxrouteJito => 1,
            ExecutionProviderType::BloxrouteSwQoS => 2,
            ExecutionProviderType::Jito => 3,
            ExecutionProviderType::JitoQuicknode => 4,
            ExecutionProviderType::Rpc => 5,
            ExecutionProviderType::Paladin => 6,
            ExecutionProviderType::BloxroutePaladin => 7,
            ExecutionProviderType::SwQoS => 8,
            ExecutionProviderType::HeliusSwQos => 9,
            ExecutionProviderType::Nextblock => 0,
            ExecutionProviderType::BloxrouteJitoBundle => 1,
        }
    }
}

impl std::fmt::Display for ExecutionProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ExecutionProviderType::Jito => write!(f, "jito"),
            ExecutionProviderType::BloxrouteJito => write!(f, "bloxroute_jito"),
            ExecutionProviderType::BloxrouteSwQoS => write!(f, "bloxroute_swqos"),
            ExecutionProviderType::JitoQuicknode => write!(f, "jito_quicknode"),
            ExecutionProviderType::Rpc => write!(f, "rpc"),
            ExecutionProviderType::Paladin => write!(f, "paladin"),
            ExecutionProviderType::BloxroutePaladin => write!(f, "bloxroute_paladin"),
            ExecutionProviderType::SwQoS => write!(f, "swqos"),
            ExecutionProviderType::HeliusSwQos => write!(f, "helius_swqos"),
            ExecutionProviderType::Nextblock => write!(f, "nextblock"),
            ExecutionProviderType::BloxrouteJitoBundle => write!(f, "bloxroute_jito_bundle"),
        }
    }
}

// #[allow(clippy::large_enum_variant)]
// pub enum Record {
//     Arbitrage(ArbitrageData),
//     PriceData(PriceData),
// }

// pub struct ArbitrageData {
//     pub signature: Signature,
//     pub opportunity: Opportunity,
//     pub provider: ExecutionProviderType,
//     pub message_index: u16,
//     pub threshold: f64,
//     pub quote_amount: u64,
//     pub recent_diff: u64,
// }

// impl ArbitrageData {
//     pub fn new(
//         signature: Signature,
//         opportunity: Opportunity,
//         provider: ExecutionProviderType,
//         message_index: u16,
//         threshold: f64,
//         quote_amount: u64,
//         recent_diff: u64,
//     ) -> Self {
//         Self {
//             signature,
//             opportunity,
//             provider,
//             message_index,
//             threshold,
//             quote_amount,
//             recent_diff,
//         }
//     }

//     pub fn update_buffer(&self, buffer: &mut Buffer) {
//         // precision constant ...
//         let profit = ((self.opportunity.diff as f64) / 1_000_000_f64) * 100.0;
//         let recent_profit = ((self.recent_diff as f64) / self.quote_amount as f64) * 100.0;
//         buffer
//             .table(TABLE_ARBITRAGE_V2)
//             .unwrap()
//             .symbol(
//                 "pool_types",
//                 self.opportunity
//                     .route
//                     .iter()
//                     .map(|(_, pool_type, _)| pool_type.to_string())
//                     .collect::<Vec<String>>()
//                     .join(","),
//             )
//             .unwrap()
//             .column_str("signature", self.signature.to_string())
//             .unwrap()
//             .column_f64("threshold", self.threshold * 100.0) // threshold in %
//             .unwrap()
//             .column_i64("amount", self.quote_amount as i64)
//             .unwrap()
//             .column_f64("recent_profit", recent_profit)
//             .unwrap()
//             .column_f64("profit", profit)
//             .unwrap()
//             .column_str("provider", self.provider.to_string())
//             .unwrap()
//             .column_i64("message_index", self.message_index as i64)
//             .unwrap()
//             // opportunity
//             .column_i64("slot", self.opportunity.slot as i64)
//             .unwrap()
//             .column_str(
//                 "pubkeys",
//                 self.opportunity
//                     .route
//                     .iter()
//                     .map(|(pubkey, _, _)| pubkey.to_string())
//                     .collect::<Vec<String>>()
//                     .join(","),
//             )
//             .unwrap()
//             .column_i64("route", self.opportunity.route.len() as i64)
//             .unwrap()
//             .column_f64(
//                 "price_1",
//                 match self.opportunity.route.first() {
//                     None => 0.0,
//                     Some((_, _, price)) => f64::from_le_bytes(*price),
//                 },
//             )
//             .unwrap()
//             .column_f64(
//                 "price_2",
//                 match self.opportunity.route.get(1) {
//                     None => 0.0,
//                     Some((_, _, price)) => f64::from_le_bytes(*price),
//                 },
//             )
//             .unwrap()
//             .column_f64(
//                 "price_3",
//                 match self.opportunity.route.get(2) {
//                     None => 0.0,
//                     Some((_, _, price)) => f64::from_le_bytes(*price),
//                 },
//             )
//             .unwrap();
//         // .column_f64("price_2", f64::from_le_bytes(self.opportunity.price1))
//         // .unwrap();
//         buffer.at_now().unwrap();
//     }
// }

// pub struct PriceData {}

// #[derive(Debug, Clone)]
// pub struct Statistics {
//     pub init_time: Instant,
//     pub provider: String,
//     pub opportunities: Arc<AtomicU64>,
//     pub tx_sent: Arc<AtomicU64>,
//     pub messages: Arc<AtomicU64>,
//     pub calculators: Arc<AtomicU64>,
// }
