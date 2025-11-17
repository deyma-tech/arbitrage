use crate::raydium_cpmm::calculator::{swap_base_input, swap_base_output};
use crate::raydium_cpmm::RaydiumCpmmProgramIx::{SwapBaseInput, SwapBaseOutput};
use crate::raydium_cpmm::{SwapBaseInputIxArgs, SwapBaseOutputIxArgs, SwapBaseOutputKeys, RAYDIUM_VAULT_AUTHORITY};
use ahash::AHashSet;
use anyhow::{format_err, Context};
use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::ToPrimitive;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use utils::constants::SPL_TOKEN_2022_ID;
use utils::pool::Pool;
use utils::quote::QuoteResult;
use utils::types::MintPair;

pub const AMM_CONFIG_ACCOUNT_DISCM: [u8; 8] = [218, 244, 33, 104, 203, 203, 43, 111];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
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
    pub padding: [u64; 16],
}

#[derive(Clone, Debug, PartialEq)]
pub struct AmmConfigAccount(pub AmmConfig);
impl AmmConfigAccount {
    /* // Borsh deserialization
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        // use std::io::Read;
        // let mut reader = buf;
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != AMM_CONFIG_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             AMM_CONFIG_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }

        let mut reader = &buf[8..];
        Ok(Self(AmmConfig::deserialize(&mut reader)?))
    }
    */
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<AmmConfig> {
        let amm_config = AmmConfig {
            bump: buf[8],
            disable_create_pool: buf[9] != 0,
            index: u16::from_le_bytes(
                buf[10..12]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: Index. {:?}", e))?,
            ),
            trade_fee_rate: u64::from_le_bytes(
                buf[12..20]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: TradeFeeRate. {:?}", e))?,
            ),
            protocol_fee_rate: u64::from_le_bytes(
                buf[20..28]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: ProtocolFeeRate. {:?}", e))?,
            ),
            fund_fee_rate: u64::from_le_bytes(
                buf[28..36]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: FundFeeRate. {:?}", e))?,
            ),
            create_pool_fee: u64::from_le_bytes(
                buf[36..44]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: CreatePoolFee. {:?}", e))?,
            ),
            protocol_owner: *bytemuck::try_from_bytes::<Pubkey>(&buf[44..76])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: ProtocolOwner. {:?}", e))?,
            fund_owner: *bytemuck::try_from_bytes::<Pubkey>(&buf[76..108])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: FundOwner. {:?}", e))?,
            padding: [0_u64; 16],
        };
        Ok(amm_config)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&AMM_CONFIG_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

// pub const OBSERVATION_STATE_ACCOUNT_DISCM: [u8; 8] = [122, 174, 197, 53, 129, 9, 165, 132];
//
// #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
// pub struct ObservationState {
//     pub initialized: bool,
//     pub observation_index: u16,
//     pub pool_id: Pubkey,
//     pub observations: [Observation; 100],
//     pub padding: [u64; 4],
// }

// #[derive(Clone, Debug, PartialEq)]
// pub struct ObservationStateAccount(pub ObservationState);
// impl ObservationStateAccount {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         use std::io::Read;
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != OBSERVATION_STATE_ACCOUNT_DISCM {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!(
//                     "discm does not match. Expected: {:?}. Received: {:?}",
//                     OBSERVATION_STATE_ACCOUNT_DISCM, maybe_discm
//                 ),
//             ));
//         }
//         Ok(Self(ObservationState::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&OBSERVATION_STATE_ACCOUNT_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }

pub const POOL_STATE_ACCOUNT_DISCM: [u8; 8] = [247, 237, 227, 245, 215, 195, 222, 70];

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolState {
    pub amm_config: Pubkey,
    pub pool_creator: Pubkey,
    pub token0_vault: Pubkey,
    pub token1_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub token0_mint: Pubkey,
    pub token1_mint: Pubkey,
    pub token0_program: Pubkey,
    pub token1_program: Pubkey,
    pub observation_key: Pubkey,
    pub auth_bump: u8,
    pub status: u8,
    pub lp_mint_decimals: u8,
    pub mint0_decimals: u8,
    pub mint1_decimals: u8,
    pub lp_supply: u64,
    pub protocol_fees_token0: u64,
    pub protocol_fees_token1: u64,
    pub fund_fees_token0: u64,
    pub fund_fees_token1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub padding: [u64; 31],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolStateAccount(pub PoolState);
impl PoolStateAccount {
    /* // Borsh deserialization
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        // use std::io::Read;
        // let mut reader = buf;
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
        let mut reader = &buf[8..];
        Ok(Self(PoolState::deserialize(&mut reader)?))
    }
    */
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<PoolState> {
        let pool_state = PoolState {
            amm_config: *bytemuck::try_from_bytes::<Pubkey>(&buf[8..40])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: AmmConfig. {:?}", e))?,
            pool_creator: *bytemuck::try_from_bytes::<Pubkey>(&buf[40..72])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: PoolCreator. {:?}", e))?,
            token0_vault: *bytemuck::try_from_bytes::<Pubkey>(&buf[72..104])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: Token0Vault. {:?}", e))?,
            token1_vault: *bytemuck::try_from_bytes::<Pubkey>(&buf[104..136])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: Token1Vault. {:?}", e))?,
            lp_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[136..168])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: LpMint. {:?}", e))?,
            token0_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[168..200])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: Token0Mint. {:?}", e))?,
            token1_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[200..232])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: Token1Mint. {:?}", e))?,
            token0_program: *bytemuck::try_from_bytes::<Pubkey>(&buf[232..264])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: Token0Program. {:?}", e))?,
            token1_program: *bytemuck::try_from_bytes::<Pubkey>(&buf[264..296])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: Token1Program. {:?}", e))?,
            observation_key: *bytemuck::try_from_bytes::<Pubkey>(&buf[296..328])
                .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: ObservationKey. {:?}", e))?,
            auth_bump: buf[328],
            status: buf[329],
            lp_mint_decimals: buf[330],
            mint0_decimals: buf[331],
            mint1_decimals: buf[332],
            lp_supply: u64::from_le_bytes(
                buf[333..341]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: LpSupply. {:?}", e))?,
            ),
            protocol_fees_token0: u64::from_le_bytes(
                buf[341..349]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: ProtocolFeesToken0. {:?}", e))?,
            ),
            protocol_fees_token1: u64::from_le_bytes(
                buf[349..357]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: ProtocolFeesToken1. {:?}", e))?,
            ),
            fund_fees_token0: u64::from_le_bytes(
                buf[357..365]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: FundFeesToken0. {:?}", e))?,
            ),
            fund_fees_token1: u64::from_le_bytes(
                buf[365..373]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: FundFeesToken1. {:?}", e))?,
            ),
            open_time: u64::from_le_bytes(
                buf[373..381]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: OpenTime. {:?}", e))?,
            ),
            recent_epoch: u64::from_le_bytes(
                buf[381..389]
                    .try_into()
                    .map_err(|e| anyhow::format_err!("RaydiumCPMMDeserFailed: RecentEpoch. {:?}", e))?,
            ),
            padding: [0_u64; 31],
        };
        Ok(pool_state)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&POOL_STATE_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub struct QuoteInput<'a> {
    pub coin_a: u64,
    pub coin_b: u64,
    pub config: &'a AmmConfig,
}
impl Pool<QuoteInput<'_>> for PoolState {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.token0_vault, &self.token1_vault, &self.observation_key]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.token0_vault, self.token1_vault, self.observation_key]
    }

    // TODO check remaining ...
    fn quote(&self, a_to_b: bool, exact_in: bool, amount_in: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        let (coin_a, coin_b) = self.vault_amount_without_fee(input.coin_a, input.coin_b)?;
        // let coin_a = input.coin_a;
        // let coin_b = input.coin_b;
        if exact_in {
            if a_to_b {
                if amount_in > input.coin_a {
                    return Err(format_err!("RaydiumCPMMPoolOutOfLiquidity"));
                }
            } else if amount_in > input.coin_b {
                return Err(format_err!("RaydiumCPMMPoolOutOfLiquidity"));
            }

            let amount_out = swap_base_input(
                amount_in.into(),
                if a_to_b { coin_a.into() } else { coin_b.into() },
                if a_to_b { coin_b.into() } else { coin_a.into() },
                input.config.trade_fee_rate,
                input.config.protocol_fee_rate,
                input.config.fund_fee_rate,
            )
            .context("CalculationError RaydiumCPMM exact_in")?;

            if a_to_b {
                Ok(QuoteResult {
                    amount_a_in: amount_in,
                    amount_b_in: 0,
                    amount_a_out: 0,
                    amount_b_out: amount_out.destination_amount_swapped.to_u64().context("Overflow")?,
                    indices: None,
                    compute_units: 41_250,
                })
            } else {
                Ok(QuoteResult {
                    amount_a_in: 0,
                    amount_b_in: amount_in,
                    amount_a_out: amount_out.destination_amount_swapped.to_u64().context("Overflow")?,
                    amount_b_out: 0,
                    indices: None,
                    compute_units: 41_250,
                })
            }
        } else {
            let amount_out = swap_base_output(
                amount_in.into(),
                if a_to_b { coin_a.into() } else { coin_b.into() },
                if a_to_b { coin_b.into() } else { coin_a.into() },
                input.config.trade_fee_rate,
                input.config.protocol_fee_rate,
                input.config.fund_fee_rate,
            )
            .context("CalculationError RaydiumCPMM exact_out")?;
            if a_to_b {
                Ok(QuoteResult {
                    amount_a_in: amount_out.source_amount_swapped.to_u64().context("Overflow")?,
                    amount_b_in: 0,
                    amount_a_out: 0,
                    amount_b_out: amount_in,
                    indices: None,
                    compute_units: 41_250,
                })
            } else {
                Ok(QuoteResult {
                    amount_a_in: 0,
                    amount_b_in: amount_out.source_amount_swapped.to_u64().context("Overflow")?,
                    amount_a_out: amount_in,
                    amount_b_out: 0,
                    indices: None,
                    compute_units: 41_250,
                })
            }
        }
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.token0_mint
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.token1_mint
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
            SwapBaseInput(SwapBaseInputIxArgs {
                amount_in: amount,
                minimum_amount_out: amount_threshold,
            })
            .try_to_vec()
            .context("Raydium CPMM: SwapBaseInput")
        } else {
            SwapBaseOutput(SwapBaseOutputIxArgs {
                max_amount_in: amount_threshold,
                amount_out: amount,
            })
            .try_to_vec()
            .context("Raydium CPMM: SwapBaseOutput")
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
    ) -> anyhow::Result<Vec<AccountMeta>> {
        let keys = SwapBaseOutputKeys {
            payer: signer,
            authority: RAYDIUM_VAULT_AUTHORITY,
            amm_config: self.amm_config,
            pool_state: pool_pubkey,
            input_token_account: source_token_account,
            output_token_account: destination_token_account,
            input_vault: if a_to_b { self.token0_vault } else { self.token1_vault },
            output_vault: if a_to_b { self.token1_vault } else { self.token0_vault },
            input_token_program: if a_to_b {
                self.token0_program
            } else {
                self.token1_program
            },
            output_token_program: if a_to_b {
                self.token1_program
            } else {
                self.token0_program
            },
            input_token_mint: if a_to_b { self.token0_mint } else { self.token1_mint },
            output_token_mint: if a_to_b { self.token1_mint } else { self.token0_mint },
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

    fn get_count_mint_pairs(&self) -> u64 {
        1
    }

    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        vec![self.get_sorted_mints_as_array()]
    }
}

impl PoolState {
    pub fn is_token_2022(&self) -> bool {
        self.token0_program == SPL_TOKEN_2022_ID || self.token1_program == SPL_TOKEN_2022_ID
    }

    pub fn vault_amount_without_fee(&self, vault_0: u64, vault_1: u64) -> anyhow::Result<(u64, u64)> {
        if let (Some(vault_0), Some(vault_1)) = (
            vault_0.checked_sub(self.protocol_fees_token0 + self.fund_fees_token0),
            vault_1.checked_sub(self.protocol_fees_token1 + self.fund_fees_token1),
        ) {
            Ok((vault_0, vault_1))
        } else {
            Err(format_err!("RaydiumCPMM: VaultAmountOverflow"))
        }
        // (
        //     vault_0
        //         .checked_sub(self.protocol_fees_token0 + self.fund_fees_token0)
        //         .un_wrap(),
        //     vault_1
        //         .checked_sub(self.protocol_fees_token1 + self.fund_fees_token1)
        //         .un_wrap(),
        // )
    }
}
