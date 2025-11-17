use super::{
    calculator::CurveCalculator,
    instructions::{
        SwapBaseInputIxArgs, SwapBaseInputIxData, SwapBaseOutputIxArgs, SwapBaseOutputIxData, SwapBaseOutputKeys,
    },
    typedefs::{Observation, PartnerInfo, PartnerType},
    GAMMA_AUTHORITY,
};
use ahash::AHashSet;
use anyhow::Context;
use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use utils::{constants::SPL_TOKEN_2022_ID, now, pool::Pool, quote::QuoteResult};

pub const AMM_CONFIG_ACCOUNT_DISCM: [u8; 8] = [218, 244, 33, 104, 203, 203, 43, 111];
#[derive(Clone, Debug, BorshDeserialize, PartialEq)]
pub struct AmmConfig {
    pub bump: u8,
    pub disable_create_pool: bool,
    pub index: u16,
    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub create_pool_fee: u64,
    pub protocol_owner: Pubkey,
    pub fund_owner: Pubkey,
    pub referral_project: Pubkey,
    pub max_open_time: u64,
    pub secondary_admin: Pubkey,
    pub padding: [u64; 7],
}

#[derive(Clone, Debug, PartialEq)]
pub struct AmmConfigAccount(pub AmmConfig);
impl AmmConfigAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        let mut reader = &buf[8..];
        Ok(Self(AmmConfig::deserialize(&mut reader)?))
    }
}

pub const OBSERVATION_STATE_ACCOUNT_DISCM: [u8; 8] = [122, 174, 197, 53, 129, 9, 165, 132];

#[derive(Clone, Debug, BorshDeserialize, PartialEq)]
pub struct ObservationState {
    pub initialized: bool,
    pub observation_index: u16,
    pub pool_id: Pubkey,
    pub observations: [Observation; 100],
    pub padding: [u64; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObservationStateAccount(pub ObservationState);
impl ObservationStateAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        let mut reader = &buf[8..];
        Ok(Self(ObservationState::deserialize(&mut reader)?))
    }
}

/*
   let result = match CurveCalculator::swap_base_input(
       u128::from(actual_amount_in),
       u128::from(total_input_token_amount),
       u128::from(total_output_token_amount),
       &ctx.accounts.amm_config,
       &pool_state,
       block_timestamp,
       &observation_state,
       is_invoked_by_signed_segmenter,
   ) {
       Ok(value) => value,
       Err(_) => return err!(GammaError::ZeroTradingTokens),
   };
*/

pub const POOL_STATE_ACCOUNT_DISCM: [u8; 8] = [247, 237, 227, 245, 215, 195, 222, 70];
#[derive(Clone, Debug, BorshDeserialize, PartialEq)]
pub struct PoolState {
    pub amm_config: Pubkey,
    pub pool_creator: Pubkey,
    pub token_0vault: Pubkey,
    pub token_1vault: Pubkey,
    pub padding1: [u8; 32],
    pub token_0mint: Pubkey,
    pub token_1mint: Pubkey,
    pub token_0program: Pubkey,
    pub token_1program: Pubkey,
    pub observation_key: Pubkey,
    pub auth_bump: u8,
    pub status: u8,
    pub padding2: u8,
    pub mint_0decimals: u8,
    pub mint_1decimals: u8,
    pub lp_supply: u64,
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub cumulative_trade_fees_token_0: u128,
    pub cumulative_trade_fees_token_1: u128,
    pub cumulative_volume_token_0: u128,
    pub cumulative_volume_token_1: u128,
    pub latest_dynamic_fee_rate: u64,
    pub max_trade_fee_rate: u64,
    pub volatility_factor: u64,
    pub token_0vault_amount: u64,
    pub token_1vault_amount: u64,
    pub max_shared_token0: u64,
    pub max_shared_token1: u64,
    pub partners: [PartnerInfo; 1],
    pub token_0amount_in_kamino: u64,
    pub token_1amount_in_kamino: u64,
    pub withdrawn_kamino_profit_token_0: u64,
    pub withdrawn_kamino_profit_token_1: u64,
    pub padding: [u64; 8],
}

pub struct QuoteInput<'a> {
    pub coin_a: u64,
    pub coin_b: u64,
    pub amm_config: &'a AmmConfig,
    pub observation_state: &'a ObservationState,
}

impl Pool<QuoteInput<'_>> for PoolState {
    fn quote(
        &self,
        a_to_b: bool,
        exact_in: bool,
        amount_in: u64,
        input: &QuoteInput,
    ) -> anyhow::Result<utils::quote::QuoteResult> {
        if exact_in {
            let result = CurveCalculator::swap_base_input(
                u128::from(amount_in),
                u128::from(if a_to_b { input.coin_a } else { input.coin_b }),
                u128::from(if a_to_b { input.coin_b } else { input.coin_a }),
                input.amm_config,
                self,
                now::as_secs(),
                input.observation_state,
                false,
            );
            if let Ok(value) = result {
                Ok(QuoteResult {
                    amount_a_in: if a_to_b { value.source_amount_swapped as u64 } else { 0 },
                    amount_b_in: if a_to_b { 0 } else { value.source_amount_swapped as u64 },
                    amount_a_out: if a_to_b {
                        0
                    } else {
                        value.destination_amount_swapped as u64
                    },
                    amount_b_out: if a_to_b {
                        value.destination_amount_swapped as u64
                    } else {
                        0
                    },
                    indices: None,
                    compute_units: 61_000,
                })
            } else {
                Err(anyhow::format_err!("GammaError::ZeroTradingTokens"))
            }
        } else {
            Err(anyhow::anyhow!("Exact out not implemented"))
        }
    }

    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.token_0vault, &self.token_1vault, &self.observation_key]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.token_0vault, self.token_1vault, self.observation_key]
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.token_0mint
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.token_1mint
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        _a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        if exact_in {
            SwapBaseInputIxData(SwapBaseInputIxArgs {
                amount_in: amount,
                minimum_amount_out: amount_threshold,
            })
            .try_to_vec()
            .context("Failed to serialize swap instruction")
        } else {
            SwapBaseOutputIxData(SwapBaseOutputIxArgs {
                max_amount_in: amount_threshold,
                amount_out: amount,
            })
            .try_to_vec()
            .context("Failed to serialize swap instruction")
        }
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        _optional_account: Option<Pubkey>,
        _optional_accounts: Option<Vec<Pubkey>>,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<solana_sdk::instruction::AccountMeta>> {
        let keys = SwapBaseOutputKeys {
            payer: signer,
            authority: GAMMA_AUTHORITY,
            amm_config: self.amm_config,
            pool_state: pool_pubkey,
            input_token_account: source_token_account,
            output_token_account: destination_token_account,
            input_vault: if a_to_b { self.token_0vault } else { self.token_1vault },
            output_vault: if a_to_b { self.token_1vault } else { self.token_0vault },
            input_token_program: if a_to_b {
                self.token_0program
            } else {
                self.token_1program
            },
            output_token_program: if a_to_b {
                self.token_1program
            } else {
                self.token_0program
            },
            input_token_mint: if a_to_b { self.token_0mint } else { self.token_1mint },
            output_token_mint: if a_to_b { self.token_1mint } else { self.token_0mint },
            observation_state: self.observation_key,
        };
        Ok(vec![
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.input_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.output_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.input_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.output_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.input_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.output_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.input_token_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.output_token_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.observation_state,
                is_signer: false,
                is_writable: true,
            },
        ])
    }
}

impl PoolState {
    pub fn is_token_2022(&self) -> bool {
        self.token_0program == SPL_TOKEN_2022_ID && self.token_1program == SPL_TOKEN_2022_ID
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolStateAccount(pub PoolState);
impl PoolStateAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        //use std::io::Read;
        let mut reader = &buf[8..];
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != POOL_STATE_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             POOL_STATE_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }
        Ok(Self(PoolState::deserialize(&mut reader)?))
    }
    // pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
    //     writer.write_all(&POOL_STATE_ACCOUNT_DISCM)?;
    //     self.0.serialize(&mut writer)
    // }
    // pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
    //     let mut data = Vec::new();
    //     self.serialize(&mut data)?;
    //     Ok(data)
    // }
}
pub const REWARD_INFO_ACCOUNT_DISCM: [u8; 8] = [39, 7, 129, 22, 241, 96, 83, 133];
#[derive(Clone, Debug, BorshDeserialize, PartialEq)]
pub struct RewardInfo {
    pub pool: Pubkey,
    pub start_at: u64,
    pub end_rewards_at: u64,
    pub mint: Pubkey,
    pub total_to_disburse: u64,
    pub rewarded_by: Pubkey,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RewardInfoAccount(pub RewardInfo);
impl RewardInfoAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        //use std::io::Read;
        let mut reader = &buf[8..];
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != REWARD_INFO_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             REWARD_INFO_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }
        Ok(Self(
            RewardInfo::deserialize(&mut reader).map_err(|_| anyhow::anyhow!("Failed to deserialize RewardInfo"))?,
        ))
    }
    // pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
    //     writer.write_all(&REWARD_INFO_ACCOUNT_DISCM)?;
    //     self.0.serialize(&mut writer)
    // }
    // pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
    //     let mut data = Vec::new();
    //     self.serialize(&mut data)?;
    //     Ok(data)
    // }
}
pub const USER_POOL_LIQUIDITY_ACCOUNT_DISCM: [u8; 8] = [0, 141, 89, 29, 236, 6, 14, 15];
#[derive(Clone, Debug, BorshDeserialize, PartialEq)]
pub struct UserPoolLiquidity {
    pub user: Pubkey,
    pub pool_state: Pubkey,
    pub token_0deposited: u128,
    pub token_1deposited: u128,
    pub token_0withdrawn: u128,
    pub token_1withdrawn: u128,
    pub lp_tokens_owned: u128,
    pub partner: PartnerType,
    pub padding: [u8; 23],
}
#[derive(Clone, Debug, PartialEq)]
pub struct UserPoolLiquidityAccount(pub UserPoolLiquidity);
impl UserPoolLiquidityAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        //use std::io::Read;
        let mut reader = &buf[8..];
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != USER_POOL_LIQUIDITY_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             USER_POOL_LIQUIDITY_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }
        Ok(Self(UserPoolLiquidity::deserialize(&mut reader)?))
    }
    // pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
    //     writer.write_all(&USER_POOL_LIQUIDITY_ACCOUNT_DISCM)?;
    //     self.0.serialize(&mut writer)
    // }
    // pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
    //     let mut data = Vec::new();
    //     self.serialize(&mut data)?;
    //     Ok(data)
    // }
}
pub const USER_REWARD_INFO_ACCOUNT_DISCM: [u8; 8] = [110, 57, 251, 139, 250, 236, 213, 178];
#[derive(Clone, Debug, BorshDeserialize, PartialEq)]
pub struct UserRewardInfo {
    pub total_claimed: u64,
    pub total_rewards: u64,
    pub rewards_last_calculated_at: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UserRewardInfoAccount(pub UserRewardInfo);
impl UserRewardInfoAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        //use std::io::Read;
        let mut reader = &buf[8..];
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != USER_REWARD_INFO_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             USER_REWARD_INFO_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }
        Ok(Self(UserRewardInfo::deserialize(&mut reader)?))
    }
    // pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
    //     writer.write_all(&USER_REWARD_INFO_ACCOUNT_DISCM)?;
    //     self.0.serialize(&mut writer)
    // }
    // pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
    //     let mut data = Vec::new();
    //     self.serialize(&mut data)?;
    //     Ok(data)
    // }
}
