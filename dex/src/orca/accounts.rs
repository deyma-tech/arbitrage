#![allow(dead_code)]
use std::collections::BTreeMap;

use crate::orca::{swap::compute_swap_v3, Oracle};

use super::constants::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64, NUM_REWARDS};
use ahash::AHashSet;
use anyhow::{format_err, Context};
use orca_whirlpools_client::{Tick, ID};
use orca_whirlpools_core::{
    get_tick_array_start_tick_index, CoreError, MAX_TICK_INDEX, MIN_TICK_INDEX, TICK_ARRAY_NOT_EVENLY_SPACED,
    TICK_ARRAY_SIZE, TICK_INDEX_OUT_OF_BOUNDS,
};
use solana_program::{instruction::AccountMeta, pubkey, pubkey::Pubkey};
use utils::{pool::Pool, quote::QuoteResult};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapV2Keys {
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
    pub memo_program: Pubkey,
    pub token_authority: Pubkey,
    pub whirlpool: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_owner_account_a: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_owner_account_b: Pubkey,
    pub token_vault_b: Pubkey,
    pub tick_array0: Pubkey,
    pub tick_array1: Pubkey,
    pub tick_array2: Pubkey,
    pub oracle: Pubkey,
}
/*
       [
           AccountMeta {
               pubkey: keys.token_program_a,
               is_signer: false,
               is_writable: false,
           },
           AccountMeta {
               pubkey: keys.token_program_b,
               is_signer: false,
               is_writable: false,
           },
           AccountMeta {
               pubkey: keys.memo_program,
               is_signer: false,
               is_writable: false,
           },
           AccountMeta {
               pubkey: keys.token_authority,
               is_signer: true,
               is_writable: false,
           },
           AccountMeta {
               pubkey: keys.whirlpool,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.token_mint_a,
               is_signer: false,
               is_writable: false,
           },
           AccountMeta {
               pubkey: keys.token_mint_b,
               is_signer: false,
               is_writable: false,
           },
           AccountMeta {
               pubkey: keys.token_owner_account_a,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.token_vault_a,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.token_owner_account_b,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.token_vault_b,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.tick_array0,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.tick_array1,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.tick_array2,
               is_signer: false,
               is_writable: true,
           },
           AccountMeta {
               pubkey: keys.oracle,
               is_signer: false,
               is_writable: true,
           },
       ]
*/

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OrcaWhirlpool {
    pub discriminator: [u8; 8],
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    pub fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_growth_global_b: u128,
    //pub reward_last_updated_timestamp: u64,
    //pub reward_infos: [WhirlpoolRewardInfo; 3],
}

//pub struct OrcaWhirlpool(pub Whirlpool);

//pub type QuoteInput<'a> = BTreeMap<i32, (Pubkey, OrcaTickArray)>;

pub struct QuoteInput<'a> {
    pub map: &'a BTreeMap<i32, (Pubkey, OrcaTickArray)>,
    pub oracle: Option<&'a Oracle>,
}

impl OrcaWhirlpool {
    // TODO: need to test
    // pub fn remove_tick_arrays(&self, tick_arrays: &mut BTreeMap<i32, (Pubkey, OrcaTickArray)>) {
    //     let start_index =
    //         get_tick_array_start_tick_index(self.tick_current_index, self.tick_spacing);

    //     let mut tick_array_down = tick_arrays
    //         .iter()
    //         .rev()
    //         .filter(|(_id, (_, tick_array))| {
    //             tick_array.start_tick_index <= start_index
    //                 && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
    //         })
    //         .take(5)
    //         .map(|(index, _)| index.clone())
    //         .collect::<Vec<_>>();
    //     let tick_array_up = tick_arrays
    //         .iter()
    //         .filter(|(_id, (_, tick_array))| {
    //             tick_array.start_tick_index > start_index
    //                 && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
    //         })
    //         .take(5)
    //         .map(|(index, _)| index.clone())
    //         .collect::<Vec<_>>();
    //     tick_array_down.extend(tick_array_up);
    //     let keys: Vec<i32> = tick_arrays.keys().cloned().collect();
    //     for key in keys.iter() {
    //         if !tick_array_down.contains(key) {
    //             tick_arrays.remove(key);
    //         }
    //     }
    // }

    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let whirlpool = OrcaWhirlpool {
            discriminator: data[0..8].try_into().context("Orca deser failed: discriminator")?,
            whirlpools_config: Pubkey::new_from_array(
                data[8..40].try_into().context("Orca deser failed: whirlpools_config")?,
            ),
            whirlpool_bump: data[40..41].try_into().context("Orca deser failed: whirlpool_bump")?,
            tick_spacing: u16::from_le_bytes(data[41..43].try_into().context("Orca deser failed: tick_spacing")?),
            tick_spacing_seed: data[43..45]
                .try_into()
                .context("Orca deser failed: tick_spacing_seed")?,
            fee_rate: u16::from_le_bytes(data[45..47].try_into().context("Orca deser failed: fee_rate")?),
            protocol_fee_rate: u16::from_le_bytes(
                data[47..49]
                    .try_into()
                    .context("Orca deser failed: protocol_fee_rate")?,
            ),
            liquidity: u128::from_le_bytes(data[49..65].try_into().context("Orca deser failed: liquidity")?),
            sqrt_price: u128::from_le_bytes(data[65..81].try_into().context("Orca deser failed: sqrt_price")?),
            tick_current_index: i32::from_le_bytes(
                data[81..85]
                    .try_into()
                    .context("Orca deser failed: tick_current_index")?,
            ),
            protocol_fee_owed_a: u64::from_le_bytes(
                data[85..93]
                    .try_into()
                    .context("Orca deser failed: protocol_fee_owed_a")?,
            ),
            protocol_fee_owed_b: u64::from_le_bytes(
                data[93..101]
                    .try_into()
                    .context("Orca deser failed: protocol_fee_owed_b")?,
            ),
            token_mint_a: Pubkey::new_from_array(data[101..133].try_into().context("Orca deser failed: token_mint_a")?),
            token_vault_a: Pubkey::new_from_array(
                data[133..165].try_into().context("Orca deser failed: token_vault_a")?,
            ),
            fee_growth_global_a: u128::from_le_bytes(
                data[165..181]
                    .try_into()
                    .context("Orca deser failed: fee_growth_global_a")?,
            ),
            token_mint_b: Pubkey::new_from_array(data[181..213].try_into().context("Orca deser failed: token_mint_b")?),
            token_vault_b: Pubkey::new_from_array(
                data[213..245].try_into().context("Orca deser failed: token_vault_b")?,
            ),
            fee_growth_global_b: u128::from_le_bytes(
                data[245..261]
                    .try_into()
                    .context("Orca deser failed: fee_growth_global_b")?,
            ),
        };
        Ok(whirlpool)
    }

    pub fn get_default_tick(&self) -> OrcaTick {
        OrcaTick {
            initialized: false,
            liquidity_net: 0,
            liquidity_gross: 0,
        }
    }

    pub fn get_default_tick_array(&self, whirlpool: &Pubkey) -> OrcaTickArray {
        let ticks: [OrcaTick; TICK_ARRAY_SIZE] = [self.get_default_tick(); TICK_ARRAY_SIZE];
        OrcaTickArray {
            start_tick_index: 0,
            ticks,
            whirlpool: *whirlpool,
        }
    }
}

pub const SWAP_V2_IX_DISCM: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];
//pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

impl<'a> Pool<QuoteInput<'a>> for OrcaWhirlpool {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.token_vault_a, &self.token_vault_b]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.token_vault_a, self.token_vault_b]
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.token_mint_a
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.token_mint_b
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount: u64, input: &QuoteInput<'a>) -> anyhow::Result<QuoteResult> {
        let tick_array_current_index = get_tick_array_start_tick_index(self.tick_current_index, self.tick_spacing);

        let (_, tick_array_0) = input
            .map
            .get(&tick_array_current_index)
            .context("Orca:TickArray0NotFound")?;

        let offset = self.tick_spacing as usize * TICK_ARRAY_SIZE;

        let default_values = (Pubkey::default(), self.get_default_tick_array(&tick_array_0.whirlpool));
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
        // let indices =  tick_array.iter().map(|tick_array| tick_array.start_tick_index).collect();

        // let tick_array = if a_to_b {
        //     // down
        //     input
        //         .iter()
        //         .rev()
        //         .filter(|(_id, (_, tick_array))| {
        //             tick_array.start_tick_index <= tick_array_current_index
        //                 && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
        //         })
        //         .take(3)
        //         .map(|(_, (_, tick_array))| tick_array)
        //         .collect::<Vec<_>>()
        // } else {
        //     input
        //         .iter()
        //         .filter(|(_id, (_, tick_array))| {
        //             tick_array.start_tick_index >= tick_array_current_index
        //                 && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
        //         })
        //         .take(3)
        //         .map(|(_, (_, tick_array))| tick_array)
        //         .collect::<Vec<_>>()
        // };

        // quote result overwrite indices ...

        //compute_swap_v2(amount, 0, self, &tick_array, a_to_b, exact_in).map_err(|e| anyhow::format_err!("{:?}", e))

        compute_swap_v3(amount, self, &tick_array, a_to_b, exact_in, None).map_err(|e| anyhow::format_err!("{:?}", e))
    }

    fn get_program_id(&self) -> Pubkey {
        orca_whirlpools_client::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        let discriminator = SWAP_V2_IX_DISCM;
        let args = SwapIxArgs {
            // 34 bytes + 8 bytes (discr) = 42 bytes
            amount,
            other_amount_threshold: amount_threshold,
            sqrt_price_limit: if a_to_b {
                MIN_SQRT_PRICE_X64 + 1
            } else {
                MAX_SQRT_PRICE_X64 - 1
            },
            amount_specified_is_input: exact_in,
            a_to_b,
        };
        let mut data = vec![];
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&args.amount.to_le_bytes());
        data.extend_from_slice(&args.other_amount_threshold.to_le_bytes());
        data.extend_from_slice(&args.sqrt_price_limit.to_le_bytes());
        data.push(args.amount_specified_is_input as u8);
        data.push(args.a_to_b as u8);
        data.push(0);
        Ok(data.to_vec())
    }

    // TODO support token 2022
    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        optional_account: Option<Pubkey>,
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
            return Err(format_err!("NotEnoughTickArrays"));
        }

        let oracle = if let Some(optional_account) = optional_account {
            optional_account
        } else if let Ok((pda, _bump)) = get_oracle_address(&pool_pubkey) {
            pda
        } else {
            Pubkey::default()
        };
        let keys = SwapV2Keys {
            token_program_a: if !allowed_token2022.contains(&self.token_mint_a) {
                spl_token::ID
            } else {
                spl_token_2022::ID
            },
            token_program_b: if !allowed_token2022.contains(&self.token_mint_b) {
                spl_token::ID
            } else {
                spl_token_2022::ID
            },
            memo_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
            token_authority: signer,
            whirlpool: pool_pubkey,
            token_mint_a: self.token_mint_a,
            token_mint_b: self.token_mint_b,
            token_owner_account_a: if a_to_b {
                source_token_account
            } else {
                destination_token_account
            },
            token_vault_a: self.token_vault_a,
            token_owner_account_b: if a_to_b {
                destination_token_account
            } else {
                source_token_account
            },
            token_vault_b: self.token_vault_b,
            tick_array0: tick_arrays[0],
            tick_array1: tick_arrays[1],
            tick_array2: tick_arrays[2],
            oracle,
        };
        let accounts = vec![
            AccountMeta {
                pubkey: keys.token_program_a,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_b,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.memo_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.whirlpool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_mint_a,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_mint_b,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_owner_account_a,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault_a,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_owner_account_b,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault_b,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array2,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.oracle,
                is_signer: false,
                is_writable: true,
            },
        ];

        // accounts.insert(
        //     accounts.len(),
        //     AccountMeta {
        //         pubkey: oracle,
        //         is_signer: false,
        //         is_writable: false,
        //     },
        // );

        Ok(accounts)
    }
}

//pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
pub struct SwapIxArgs {
    pub amount: u64,                     //  8
    pub other_amount_threshold: u64,     // 8
    pub sqrt_price_limit: u128,          // 16
    pub amount_specified_is_input: bool, // 1
    pub a_to_b: bool,                    // 1
                                         // total 8 + 8 + 16 + 1 + 1 = 34
}

pub fn get_oracle_address(whirlpool: &Pubkey) -> Result<(Pubkey, u8), anyhow::Error> {
    let seeds = &[b"oracle", whirlpool.as_ref()];
    Pubkey::try_find_program_address(seeds, &ID).ok_or(anyhow::format_err!("InvalidSeeds"))
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct OrcaTickArray {
    pub start_tick_index: i32,
    pub ticks: [OrcaTick; 88],
    pub whirlpool: Pubkey,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrcaSparseTickArray {
    pub start_tick_index: i32,
    pub ticks: Vec<OrcaSparseTick>, // instead of array ...
    pub whirlpool: Pubkey,
}

impl OrcaTickArray {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<OrcaTickArray> {
        let chunks = buf[12..buf.len() - 32].chunks(113);
        // let ticks = chunks
        //     .map(|chunk| OrcaTick {
        //         initialized: chunk[0] == 1,
        //         liquidity_net: i128::from_le_bytes(chunk[1..17].try_into().un_wrap()),
        //         liquidity_gross: u128::from_le_bytes(chunk[17..33].try_into().un_wrap()),
        //     })
        //     .collect::<Vec<OrcaTick>>();

        let ticks = chunks
            .map(|chunk| {
                let liquidity_net = i128::from_le_bytes(
                    chunk[1..17]
                        .try_into()
                        .map_err(|e| anyhow::format_err!("Failed to parse liquidity_net: {:?}", e))?,
                );
                let liquidity_gross = u128::from_le_bytes(
                    chunk[17..33]
                        .try_into()
                        .map_err(|e| anyhow::format_err!("Failed to parse liquidity_gross: {:?}", e))?,
                );
                Ok(OrcaTick {
                    initialized: chunk[0] == 1,
                    liquidity_net,
                    liquidity_gross,
                })
            })
            .collect::<Result<Vec<OrcaTick>, anyhow::Error>>()?;

        let mut pubkey_array = [0_u8; 32];
        pubkey_array.copy_from_slice(&buf[buf.len() - 32..]);
        let ticks = ticks.try_into();
        let ticks = ticks.map_err(|e| anyhow::format_err!("Failed to convert ticks to array: {:?}", e))?;
        let mut start_tick_index = [0; 4];
        start_tick_index.copy_from_slice(&buf[8..12]);
        // let mut discriminator = [0; 8];
        // discriminator.copy_from_slice(&buf[..8]);
        Ok(OrcaTickArray {
            //discriminator,
            start_tick_index: i32::from_le_bytes(start_tick_index),
            ticks,
            whirlpool: Pubkey::new_from_array(pubkey_array),
        })
    }

    pub fn deserialize_from_dynamic(buf: &[u8]) -> anyhow::Result<OrcaTickArray> {
        let mut default_ticks = [OrcaTick::default(); TICK_ARRAY_SIZE];
        if buf.len() > 60 {
            let mut cursor = 60;
            #[allow(clippy::needless_range_loop)]
            for idx in 0..TICK_ARRAY_SIZE {
                let first_byte = buf.get(cursor);
                if let Some(first_byte) = first_byte {
                    cursor += 1;
                    if first_byte == &1 && buf.len() > cursor + 112 {
                        let liquidity_net = i128::from_le_bytes(
                            buf[cursor..cursor + 16]
                                .try_into()
                                .map_err(|e| anyhow::format_err!("Failed to parse liquidity_net: {:?}", e))?,
                        );
                        let liquidity_gross = u128::from_le_bytes(
                            buf[cursor + 16..cursor + 32]
                                .try_into()
                                .map_err(|e| anyhow::format_err!("Failed to parse liquidity_gross: {:?}", e))?,
                        );
                        default_ticks[idx] = OrcaTick {
                            initialized: true,
                            liquidity_net,
                            liquidity_gross,
                        };
                        cursor += 112;
                    }
                }
            }
        }

        let mut pubkey_array = [0_u8; 32];
        pubkey_array.copy_from_slice(&buf[12..44]);
        let mut start_tick_index = [0; 4];
        start_tick_index.copy_from_slice(&buf[8..12]);
        // let mut discriminator = [0; 8];
        // discriminator.copy_from_slice(&buf[..8]);
        Ok(OrcaTickArray {
            //discriminator,
            start_tick_index: i32::from_le_bytes(start_tick_index),
            ticks: default_ticks,
            whirlpool: Pubkey::new_from_array(pubkey_array),
        })
    }
}

impl OrcaTickArray {
    //     /// Updates the Tick object at the given tick-index & tick-spacing
    //     ///
    //     /// # Parameters
    //     /// - `tick_index` - the tick index the desired Tick object is stored in
    //     /// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    //     /// - `update` - A reference to a TickUpdate object to update the Tick object at the given index
    //     ///
    //     /// # Errors
    //     /// - `TickNotFound`: - The provided tick-index is not an initializable tick index in this Whirlpool w/ this tick-spacing.
    //     fn update_tick(&mut self, tick_index: i32, tick_spacing: u16, update: &TickUpdate) -> anyhow::Result<()> {
    //         if !self.check_in_array_bounds(tick_index, tick_spacing)
    //             || !Tick::check_is_usable_tick(tick_index, tick_spacing)
    //         {
    //             return Err(ErrorCode::TickNotFound.into());
    //         }
    //         let offset = self.tick_offset(tick_index, tick_spacing)?;
    //         if offset < 0 {
    //             return Err(ErrorCode::TickNotFound.into());
    //         }
    //         self.ticks.get_mut(offset as usize).unwrap().update(update);
    //         Ok(())
    //     }

    #[inline(always)]
    pub fn is_min_tick_array(&self) -> bool {
        self.start_tick_index() <= MIN_TICK_INDEX
    }

    #[inline(always)]
    pub fn is_max_tick_array(&self, tick_spacing: u16) -> bool {
        self.start_tick_index() + (TICK_ARRAY_SIZE as i32) * (tick_spacing as i32) > MAX_TICK_INDEX
    }
}

#[derive(Default, Debug, PartialEq, Copy, Clone)]
#[repr(C, packed)]
pub struct OrcaSparseTick {
    // Total 137 bytes
    pub index: u8,
    pub initialized: bool,   // 1
    pub liquidity_net: i128, // 16 17
    pub liquidity_gross: u128, // 16 33

                             // Q64.64
                             // pub fee_growth_outside_a: u128, // 16 49
                             // // Q64.64
                             // pub fee_growth_outside_b: u128, // 16 65
                             //
                             // // Array of Q64.64
                             // pub reward_growths_outside: [u128; NUM_REWARDS], // 48 = 16 * 3  // 113
}

#[derive(Default, Debug, PartialEq, Copy, Clone)]
#[repr(C, packed)]
pub struct OrcaTick {
    // Total 137 bytes
    pub initialized: bool,   // 1
    pub liquidity_net: i128, // 16 17
    pub liquidity_gross: u128, // 16 33

                             // Q64.64
                             // pub fee_growth_outside_a: u128, // 16 49
                             // // Q64.64
                             // pub fee_growth_outside_b: u128, // 16 65
                             //
                             // // Array of Q64.64
                             // pub reward_growths_outside: [u128; NUM_REWARDS], // 48 = 16 * 3  // 113
}

impl OrcaTick {
    pub fn to_tick(self) -> Tick {
        Tick {
            initialized: self.initialized,            // 1
            liquidity_net: self.liquidity_net,        // 16 17
            liquidity_gross: self.liquidity_gross,    // 16 33
            fee_growth_outside_a: 0,                  // 16 49
            fee_growth_outside_b: 0,                  // 16 65
            reward_growths_outside: [0; NUM_REWARDS], // 48 113
        }
    }

    // Check that the tick index is a valid start tick for a tick array in this whirlpool
    // A valid start-tick-index is a multiple of tick_spacing & number of ticks in a tick-array.
    //
    // # Parameters
    // - `tick_index` - A i32 integer representing the tick index
    // - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    //
    // # Returns
    // - `true`: The tick index is a valid start-tick-index for this whirlpool
    // - `false`: The tick index is not a valid start-tick-index for this whirlpool
    //            or the tick index not within the range supported by this contract

    // pub fn check_is_valid_start_tick(tick_index: i32, tick_spacing: u16) -> bool {
    //     let ticks_in_array = TICK_ARRAY_SIZE as i32 * tick_spacing as i32;

    //     if Tick::check_is_out_of_bounds(tick_index) {
    //         // Left-edge tick-array can have a start-tick-index smaller than the min tick index
    //         if tick_index > MIN_TICK_INDEX {
    //             return false;
    //         }

    //         let min_array_start_index = MIN_TICK_INDEX - (MIN_TICK_INDEX % ticks_in_array + ticks_in_array);
    //         return tick_index == min_array_start_index;
    //     }
    //     tick_index % ticks_in_array == 0
    // }

    /// Check that the tick index is within the supported range of this contract
    ///
    /// # Parameters
    /// - `tick_index` - A i32 integer representing the tick index
    ///
    /// # Returns
    /// - `true`: The tick index is not within the range supported by this contract
    /// - `false`: The tick index is within the range supported by this contract
    pub fn check_is_out_of_bounds(tick_index: i32) -> bool {
        !(MIN_TICK_INDEX..=MAX_TICK_INDEX).contains(&tick_index)
    }

    /// Check that the tick index is within bounds and is a usable tick index for the given tick spacing.
    ///
    /// # Parameters
    /// - `tick_index` - A i32 integer representing the tick index
    /// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    ///
    /// # Returns
    /// - `true`: The tick index is within max/min index bounds for this protocol and is a usable tick-index given the tick-spacing
    /// - `false`: The tick index is out of bounds or is not a usable tick for this tick-spacing
    pub fn check_is_usable_tick(tick_index: i32, tick_spacing: u16) -> bool {
        if OrcaTick::check_is_out_of_bounds(tick_index) {
            return false;
        }

        tick_index % tick_spacing as i32 == 0
    }
}

//

/// Search for the next initialized tick in this array.
///
/// # Parameters
/// - `tick_index` - A i32 integer representing the tick index to start searching for
/// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
/// - `a_to_b` - If the trade is from a_to_b, the search will move to the left and the starting search tick is inclusive.
///   If the trade is from b_to_a, the search will move to the right and the starting search tick is not inclusive.
///
/// # Returns
/// - `Some(i32)`: The next initialized tick index of this array
/// - `None`: An initialized tick index was not found in this array
/// - `InvalidTickArraySequence` - error if `tick_index` is not a valid search tick for the array
/// - `InvalidTickSpacing` - error if the provided tick spacing is 0
pub fn get_next_init_tick_index(
    tick_array: &OrcaTickArray,
    tick_index: i32,
    tick_spacing: u16,
    a_to_b: bool,
) -> anyhow::Result<Option<i32>> {
    if !in_search_range(tick_array, tick_index, tick_spacing, !a_to_b) {
        return Err(anyhow::format_err!("InvalidTickArraySequence"));
    }

    let mut curr_offset = match tick_offset(tick_array, tick_index, tick_spacing) {
        Ok(value) => value as i32,
        Err(e) => return Err(anyhow::format_err!("{:?}", e)),
    };

    // For a_to_b searches, the search moves to the left. The next possible init-tick can be the 1st tick in the current offset
    // For b_to_a searches, the search moves to the right. The next possible init-tick cannot be within the current offset
    if !a_to_b {
        curr_offset += 1;
    }

    while (0..(TICK_ARRAY_SIZE as i32)).contains(&curr_offset) {
        let curr_tick = &tick_array.ticks[curr_offset as usize];
        if curr_tick.initialized {
            return Ok(Some((curr_offset * tick_spacing as i32) + tick_array.start_tick_index));
        }

        curr_offset = if a_to_b { curr_offset - 1 } else { curr_offset + 1 };
    }

    Ok(None)
}

fn in_search_range(tick_array: &OrcaTickArray, tick_index: i32, tick_spacing: u16, shifted: bool) -> bool {
    let mut lower = tick_array.start_tick_index;
    let mut upper = tick_array.start_tick_index + (TICK_ARRAY_SIZE as i32) * (tick_spacing as i32);
    if shifted {
        lower -= tick_spacing as i32;
        upper -= tick_spacing as i32;
    }
    tick_index >= lower && tick_index < upper
}

fn check_in_array_bounds(tick_array: &OrcaTickArray, tick_index: i32, tick_spacing: u16) -> bool {
    in_search_range(tick_array, tick_index, tick_spacing, false)
}

pub fn is_min_tick_array(tick_array: &OrcaTickArray) -> bool {
    tick_array.start_tick_index <= MIN_TICK_INDEX
}

pub fn is_max_tick_array(tick_array: &OrcaTickArray, tick_spacing: u16) -> bool {
    tick_array.start_tick_index + (TICK_ARRAY_SIZE as i32) * (tick_spacing as i32) > MAX_TICK_INDEX
}

fn tick_offset(tick_array: &OrcaTickArray, tick_index: i32, tick_spacing: u16) -> Result<isize, CoreError> {
    if tick_spacing == 0 {
        return Err(TICK_ARRAY_NOT_EVENLY_SPACED);
    }

    Ok(get_offset(tick_index, tick_array.start_tick_index, tick_spacing))
}

fn get_offset(tick_index: i32, start_tick_index: i32, tick_spacing: u16) -> isize {
    // TODO: replace with i32.div_floor once not experimental
    let lhs = tick_index - start_tick_index;
    let rhs = tick_spacing as i32;
    let d = lhs / rhs;
    let r = lhs % rhs;
    let o = if (r > 0 && rhs < 0) || (r < 0 && rhs > 0) {
        d - 1
    } else {
        d
    };
    o as isize
}

fn check_is_out_of_bounds(tick_index: i32) -> bool {
    !(MIN_TICK_INDEX..=MAX_TICK_INDEX).contains(&tick_index)
}

fn check_is_usable_tick(tick_index: i32, tick_spacing: u16) -> bool {
    if check_is_out_of_bounds(tick_index) {
        return false;
    }

    tick_index % tick_spacing as i32 == 0
}

fn get_tick_in_tick_array(
    tick_array: &OrcaTickArray,
    tick_index: i32,
    tick_spacing: u16,
) -> Result<&OrcaTick, CoreError> {
    if !check_in_array_bounds(tick_array, tick_index, tick_spacing) || !check_is_usable_tick(tick_index, tick_spacing) {
        return Err(TICK_INDEX_OUT_OF_BOUNDS);
    }
    let offset = tick_offset(tick_array, tick_index, tick_spacing)?;
    if offset < 0 {
        return Err(TICK_INDEX_OUT_OF_BOUNDS);
    }
    Ok(&tick_array.ticks[offset as usize])
}

pub fn get_tick<'b>(
    tick_arrays: &'b [&OrcaTickArray],
    array_index: i32,
    tick_index: i32,
    tick_spacing: u16,
) -> Result<&'b OrcaTick, CoreError> {
    let array = tick_arrays
        .iter()
        .find(|tick_array| tick_array.start_tick_index == array_index);
    match array {
        Some(array) => get_tick_in_tick_array(array, tick_index, tick_spacing),
        _ => Err(TICK_INDEX_OUT_OF_BOUNDS),
    }
}
