use ahash::AHashSet;
use anyhow::Context;
use solana_program::pubkey::Pubkey;
use solana_sdk::{instruction::AccountMeta, pubkey};
use utils::quote::QuoteResult;

use crate::orca::OrcaWhirlpool;

pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub const FUSION_POOL_ACCOUNT_DISCM: [u8; 8] = [254, 204, 207, 98, 25, 181, 29, 67];

#[derive(Clone, Debug, PartialEq)]
pub struct FusionPool {
    pub bump: [u8; 1],
    pub version: u16,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub clp_reward_rate: u16,
    pub order_protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub fee_growth_global_a: u128,
    pub fee_growth_global_b: u128,
    pub orders_total_amount_a: u64,
    pub orders_total_amount_b: u64,
    pub orders_filled_amount_a: u64,
    pub orders_filled_amount_b: u64,
    pub olp_fee_owed_a: u64,
    pub olp_fee_owed_b: u64,
    pub reserved: [u8; 140],
}

impl FusionPool {
    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let pool = FusionPool {
            bump: data[8..9].try_into().context("FusionAmm:DeserFailed:Bump")?,
            version: u16::from_le_bytes(data[9..11].try_into().context("FusionAmm:DeserFailed:Version")?),
            token_mint_a: Pubkey::new_from_array(data[11..43].try_into().context("FusionAmm:DeserFailed:TokenMintA")?),
            token_mint_b: Pubkey::new_from_array(data[43..75].try_into().context("FusionAmm:DeserFailed:TokenMintB")?),
            token_vault_a: Pubkey::new_from_array(
                data[75..107].try_into().context("FusionAmm:DeserFailed:TokenVaultA")?,
            ),
            token_vault_b: Pubkey::new_from_array(
                data[107..139].try_into().context("FusionAmm:DeserFailed:TokenVaultB")?,
            ),
            tick_spacing: u16::from_le_bytes(data[139..141].try_into().context("FusionAmm:DeserFailed:TickSpacing")?),
            tick_spacing_seed: data[141..143]
                .try_into()
                .context("FusionAmm:DeserFailed:TickSpacingSeed")?,
            fee_rate: u16::from_le_bytes(data[143..145].try_into().context("FusionAmm:DeserFailed:FeeRate")?),
            protocol_fee_rate: u16::from_le_bytes(
                data[145..147]
                    .try_into()
                    .context("FusionAmm:DeserFailed:ProtocolFeeRate")?,
            ),
            clp_reward_rate: u16::from_le_bytes(
                data[147..149]
                    .try_into()
                    .context("FusionAmm:DeserFailed:ClpRewardRate")?,
            ),
            order_protocol_fee_rate: u16::from_le_bytes(
                data[149..151]
                    .try_into()
                    .context("FusionAmm:DeserFailed:OrderProtocolFeeRate")?,
            ),
            liquidity: u128::from_le_bytes(data[151..167].try_into().context("FusionAmm:DeserFailed:Liquidity")?),
            sqrt_price: u128::from_le_bytes(data[167..183].try_into().context("FusionAmm:DeserFailed:SqrtPrice")?),
            tick_current_index: i32::from_le_bytes(
                data[183..187]
                    .try_into()
                    .context("FusionAmm:DeserFailed:TickCurrentIndex")?,
            ),
            protocol_fee_owed_a: u64::from_le_bytes(
                data[187..195]
                    .try_into()
                    .context("FusionAmm:DeserFailed:ProtocolFeeOwedA")?,
            ),
            protocol_fee_owed_b: u64::from_le_bytes(
                data[195..203]
                    .try_into()
                    .context("FusionAmm:DeserFailed:ProtocolFeeOwedB")?,
            ),
            fee_growth_global_a: u128::from_le_bytes(
                data[203..219]
                    .try_into()
                    .context("FusionAmm:DeserFailed:FeeGrowthGlobalA")?,
            ),
            fee_growth_global_b: u128::from_le_bytes(
                data[219..235]
                    .try_into()
                    .context("FusionAmm:DeserFailed:FeeGrowthGlobalB")?,
            ),
            orders_total_amount_a: u64::from_le_bytes(
                data[235..243]
                    .try_into()
                    .context("FusionAmm:DeserFailed:OrdersTotalAmountA")?,
            ),
            orders_total_amount_b: u64::from_le_bytes(
                data[243..251]
                    .try_into()
                    .context("FusionAmm:DeserFailed:OrdersTotalAmountB")?,
            ),
            orders_filled_amount_a: u64::from_le_bytes(
                data[251..259]
                    .try_into()
                    .context("FusionAmm:DeserFailed:OrdersFilledAmountA")?,
            ),
            orders_filled_amount_b: u64::from_le_bytes(
                data[259..267]
                    .try_into()
                    .context("FusionAmm:DeserFailed:OrdersFilledAmountB")?,
            ),
            olp_fee_owed_a: u64::from_le_bytes(data[267..275].try_into().context("FusionAmm:DeserFailed:OlpFeeOwedA")?),
            olp_fee_owed_b: u64::from_le_bytes(data[275..283].try_into().context("FusionAmm:DeserFailed:OlpFeeOwedB")?),
            reserved: [0_u8; 140],
        };
        Ok(pool)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FusionPoolWhirlpool {
    pub whirlpool: OrcaWhirlpool,
}

impl FusionPoolWhirlpool {
    pub fn from_fusion_pool(fusion_pool: &FusionPool) -> Self {
        let whirlpool = OrcaWhirlpool {
            token_mint_a: fusion_pool.token_mint_a,
            token_mint_b: fusion_pool.token_mint_b,
            token_vault_a: fusion_pool.token_vault_a,
            token_vault_b: fusion_pool.token_vault_b,
            tick_spacing: fusion_pool.tick_spacing,
            fee_rate: fusion_pool.fee_rate,
            liquidity: fusion_pool.liquidity,
            sqrt_price: fusion_pool.sqrt_price,
            tick_current_index: fusion_pool.tick_current_index,
            // additional fields
            discriminator: [0_u8; 8],
            whirlpools_config: Pubkey::default(),
            whirlpool_bump: [0_u8; 1],
            tick_spacing_seed: [0_u8; 2],
            protocol_fee_rate: 0_u16,
            protocol_fee_owed_a: 0_u64,
            protocol_fee_owed_b: 0_u64,
            fee_growth_global_a: 0_u128,
            fee_growth_global_b: 0_u128,
        };
        Self { whirlpool }
    }
    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let fusion_pool = FusionPool::deserialize(data)?;
        Ok(Self::from_fusion_pool(&fusion_pool))
    }
}

impl<'a> utils::pool::Pool<crate::orca::QuoteInput<'a>> for FusionPoolWhirlpool {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.whirlpool.token_vault_a, &self.whirlpool.token_vault_b]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.whirlpool.token_vault_a, self.whirlpool.token_vault_b]
    }

    fn get_a_mint(&self) -> &Pubkey {
        &self.whirlpool.token_mint_a
    }

    fn get_b_mint(&self) -> &Pubkey {
        &self.whirlpool.token_mint_b
    }

    fn quote(
        &self,
        a_to_b: bool,
        exact_in: bool,
        amount: u64,
        input: &crate::orca::QuoteInput<'a>,
    ) -> anyhow::Result<QuoteResult> {
        let tick_array_current_index = orca_whirlpools_core::get_tick_array_start_tick_index(
            self.whirlpool.tick_current_index,
            self.whirlpool.tick_spacing,
        );

        let (_, tick_array_0) = input
            .map
            .get(&tick_array_current_index)
            .context("Fusion:TickArray0NotFound")?;
        // no u16
        let offset = self.whirlpool.tick_spacing as usize * orca_whirlpools_core::TICK_ARRAY_SIZE;

        let default_values = (
            Pubkey::default(),
            self.whirlpool.get_default_tick_array(&tick_array_0.whirlpool),
        );
        // TODO add 2 accounts if empty ...
        let (_, tick_arrray_1) = if a_to_b {
            input
                .map
                .get(&(tick_array_current_index - offset as i32))
                .unwrap_or(&default_values)
        } else {
            input
                .map
                .get(&(tick_array_current_index + offset as i32))
                .unwrap_or(&default_values)
        };

        let (_, tick_arrray_2) = if a_to_b {
            input
                .map
                .get(&(tick_array_current_index - (offset * 2) as i32))
                .unwrap_or(&default_values)
        } else {
            input
                .map
                .get(&(tick_array_current_index + (offset * 2) as i32))
                .unwrap_or(&default_values)
        };

        let tick_array = [tick_array_0, tick_arrray_1, tick_arrray_2];
        crate::orca::compute_swap_v2(amount, 0, &self.whirlpool, &tick_array, a_to_b, exact_in)
            .map_err(|e| anyhow::format_err!("{:?}", e))
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(&SWAP_IX_DISCM);
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&amount_threshold.to_le_bytes());
        data.extend_from_slice(
            &if a_to_b {
                crate::orca::MIN_SQRT_PRICE_X64 + 1
            } else {
                crate::orca::MAX_SQRT_PRICE_X64 - 1
            }
            .to_le_bytes(),
        );
        data.push(exact_in as u8);
        data.push(a_to_b as u8);
        data.push(0);
        Ok(data.to_vec())
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        _optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        let mut tick_arrays = vec![];
        if optional_accounts.is_none() {
            return Ok(vec![]);
        }
        if let Some(optional_accounts) = optional_accounts {
            tick_arrays = optional_accounts;
            while tick_arrays.len() < 3 {
                if let Some(tick_array) = tick_arrays.last() {
                    tick_arrays.insert(tick_arrays.len(), *tick_array);
                }
            }
        }

        if tick_arrays.len() < 3 {
            return Err(anyhow::format_err!("NotEnoughTickArrays"));
        }

        let accounts = vec![
            AccountMeta {
                pubkey: if !allowed_token2022.contains(&self.whirlpool.token_mint_a) {
                    spl_token::ID
                } else {
                    spl_token_2022::ID
                },
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: if !allowed_token2022.contains(&self.whirlpool.token_mint_b) {
                    spl_token::ID
                } else {
                    spl_token_2022::ID
                },
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: signer,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: pool_pubkey,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.whirlpool.token_mint_a,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.whirlpool.token_mint_b,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: if a_to_b {
                    source_token_account
                } else {
                    destination_token_account
                },
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: if a_to_b {
                    destination_token_account
                } else {
                    source_token_account
                },
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.whirlpool.token_vault_a,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.whirlpool.token_vault_b,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: tick_arrays[0],
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: tick_arrays[1],
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: tick_arrays[2],
                is_signer: false,
                is_writable: true,
            },
        ];

        Ok(accounts)
    }
}

pub const FUSION_POOLS_CONFIG_ACCOUNT_DISCM: [u8; 8] = [191, 199, 19, 11, 75, 86, 239, 169];

#[derive(Clone, Debug, PartialEq)]
pub struct FusionPoolsConfig {
    pub version: u16,
    pub fee_authority: Pubkey,
    pub collect_protocol_fees_authority: Pubkey,
    pub token_badge_authority: Pubkey,
    pub default_protocol_fee_rate: u16,
    pub default_order_protocol_fee_rate: u16,
    pub default_clp_reward_rate: u16,
    pub reserved: [u8; 170],
}

impl FusionPoolsConfig {
    pub fn deserialize_opt(data: &[u8]) -> anyhow::Result<Self> {
        let config = FusionPoolsConfig {
            version: u16::from_le_bytes(data[8..10].try_into().context("FusionAmm:DeserFailed:Version")?),
            fee_authority: Pubkey::new_from_array(
                data[10..42].try_into().context("FusionAmm:DeserFailed:FeeAuthority")?,
            ),
            collect_protocol_fees_authority: Pubkey::new_from_array(
                data[42..74]
                    .try_into()
                    .context("FusionAmm:DeserFailed:CollectProtocolFeesAuthority")?,
            ),
            token_badge_authority: Pubkey::new_from_array(
                data[74..106]
                    .try_into()
                    .context("FusionAmm:DeserFailed:TokenBadgeAuthority")?,
            ),
            default_protocol_fee_rate: u16::from_le_bytes(
                data[106..108]
                    .try_into()
                    .context("FusionAmm:DeserFailed:DefaultProtocolFeeRate")?,
            ),
            default_order_protocol_fee_rate: u16::from_le_bytes(
                data[108..110]
                    .try_into()
                    .context("FusionAmm:DeserFailed:DefaultOrderProtocolFeeRate")?,
            ),
            default_clp_reward_rate: u16::from_le_bytes(
                data[110..112]
                    .try_into()
                    .context("FusionAmm:DeserFailed:DefaultClpRewardRate")?,
            ),
            reserved: [0_u8; 170],
        };
        Ok(config)
    }
}

pub const TICK_ARRAY_ACCOUNT_DISCM: [u8; 8] = [69, 97, 189, 190, 110, 7, 66, 187];

// #[derive(Clone, Debug, PartialEq)]
// pub struct TickArray {
//     pub start_tick_index: i32,
//     pub ticks: [Tick; 88],
//     pub fusion_pool: Pubkey,
// }
