use solana_sdk::pubkey::Pubkey;

//use crate::orca::swap::NUM_REWARDS;

/// Stores the state relevant for tracking liquidity mining rewards at the `Whirlpool` level.
/// These values are used in conjunction with `PositionRewardInfo`, `Tick.reward_growths_outside`,
/// and `Whirlpool.reward_last_updated_timestamp` to determine how many rewards are earned by open
/// positions.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct WhirlpoolRewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// reward_infos[0]: Authority account that has permission to initialize the reward and set emissions.
    /// reward_infos[1]: used for a struct that contains fields for extending the functionality of Whirlpool.
    /// reward_infos[2]: reserved for future use.
    ///
    /// Historical notes:
    /// Originally, this was a field named "authority", but it was found that there was no opportunity
    /// to set different authorities for the three rewards. Therefore, the use of this field was changed for Whirlpool's future extensibility.
    pub extension: [u8; 32],
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub growth_global_x64: u128,
}

// impl WhirlpoolRewardInfo {
//     /// Creates a new `WhirlpoolRewardInfo` with the extension set
//     pub fn new(extension: [u8; 32]) -> Self {
//         Self {
//             extension,
//             ..Default::default()
//         }
//     }

//     /// Returns true if this reward is initialized.
//     /// Once initialized, a reward cannot transition back to uninitialized.
//     pub fn initialized(&self) -> bool {
//         self.mint.ne(&Pubkey::default())
//     }

//     /// Maps all reward data to only the reward growth accumulators
//     pub fn to_reward_growths(reward_infos: &[WhirlpoolRewardInfo; NUM_REWARDS]) -> [u128; NUM_REWARDS] {
//         let mut reward_growths = [0u128; NUM_REWARDS];
//         for i in 0..NUM_REWARDS {
//             reward_growths[i] = reward_infos[i].growth_global_x64;
//         }
//         reward_growths
//     }
// }
