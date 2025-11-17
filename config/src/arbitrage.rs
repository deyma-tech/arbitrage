use serde::{Deserialize, Serialize};

use crate::DEFAULT_OPTIMIZATION_THREADS;

const TX_GROUP_TIME_MICROS: u64 = 700;
const MAX_COUNTER: u64 = 7;

const DEFAULT_VOLUMES: [u64; 30] = [
    500000000000,
    350000000000,
    244999999999,
    171499999999,
    120049999999,
    84034999999,
    58824499999,
    41177149999,
    28824004999,
    20176803499,
    14123762449,
    9886633714,
    6920643600,
    4844450520,
    3391115364,
    2373780754,
    1661646528,
    1163152569,
    814206798,
    569944759,
    398961331,
    279272932,
    195491052,
    136843736,
    95790615,
    67053430,
    46937401,
    32856181,
    22999326,
    16099528,
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[allow(unused)]
pub struct ArbitrageSettings {
    /// Allow two combinations of mints
    pub c2: bool,

    /// Allow three combinations of mints
    pub c3: bool,

    /// Volumes
    pub volumes: Vec<u64>,

    /// Size of the price map
    pub price_map_size: usize,
    //pub price_map_trigger_diff: f64,
    pub optimization_threads: usize,

    // Opportunities
    pub opportunity_filter_factor: u64,
    /// group time in milliseconds
    pub opportunity_group_interval: u64,
    /// max number of calculators in a group
    pub opportunity_group_saturation: u64,

    // flashloan
    pub min_amount_for_flashloan: u64,
    /// price map switch
    pub price_map_tick_switch: u32,

    pub optimize_method: String,

    pub min_wsol: u64,
    //pub min_stable: u64,
}

impl ArbitrageSettings {
    #[inline(always)]
    pub fn is_c2_only(&self) -> bool {
        self.c2 && !self.c3
    }

    #[inline(always)]
    pub fn is_c3_only(&self) -> bool {
        self.c3 && !self.c2
    }
}

impl Default for ArbitrageSettings {
    fn default() -> Self {
        ArbitrageSettings {
            c2: true,
            c3: true,
            volumes: DEFAULT_VOLUMES.to_vec(),
            price_map_size: 6,
            optimization_threads: DEFAULT_OPTIMIZATION_THREADS,
            opportunity_filter_factor: 5,
            opportunity_group_interval: TX_GROUP_TIME_MICROS,
            opportunity_group_saturation: MAX_COUNTER,
            min_amount_for_flashloan: 2_000_000_000,
            price_map_tick_switch: 2500,
            optimize_method: "optimize_convex".to_string(),
            min_wsol: 1_000_000_000,
        }
    }
}
