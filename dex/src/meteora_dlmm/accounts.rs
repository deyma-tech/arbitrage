use std::{
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

use super::{
    constants::EVENT_AUTHORITY,
    instructions::{SwapExactOutIxArgs, SwapExactOutIxData, SwapIxArgs, SwapIxData},
    quote::{quote_exact_in_bitmap, quote_exact_out_bitmap, QuoteInput},
    *,
};

use ahash::AHashSet;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use utils::{pool::Pool, quote::QuoteResult};

pub const BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM: [u8; 8] = [80, 111, 124, 113, 55, 237, 18, 5];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct BinArrayBitmapExtension {
    pub lb_pair: Pubkey,
    pub positive_bin_array_bitmap: [[u64; 8]; 12],
    pub negative_bin_array_bitmap: [[u64; 8]; 12],
}
#[derive(Clone, Debug, PartialEq)]
pub struct BinArrayBitmapExtensionAccount(pub BinArrayBitmapExtension);
impl BinArrayBitmapExtensionAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        //use std::io::Read;
        let mut reader = &buf[8..];
        /*
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM, maybe_discm
                ),
            ));
        }
         */
        Ok(Self(BinArrayBitmapExtension::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const BIN_ARRAY_ACCOUNT_DISCM: [u8; 8] = [92, 142, 92, 220, 5, 148, 70, 181];
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct BinArrayBytemuck {
    pub index: i64,
    //pub version: u8,
    pub padding: [u8; 8],
    pub lb_pair: Pubkey,
}
/*
impl BinArray {
    // getter bin = bins0 + bin1 + bins2
    pub fn bins(&self) -> [Bin; 70] {
        let mut bins = [Bin::default(); 70];
        bins[..64].copy_from_slice(&self.bins0);
        bins[64..68].copy_from_slice(&self.bins1);
        bins[68..].copy_from_slice(&self.bins2);
        bins
    }
}
*/

#[derive(Copy, Clone, Debug, PartialEq)]

pub struct BinArray {
    pub index: i64, //  8
    //pub version: u8,
    pub padding: [u8; 8], // 8
    pub lb_pair: Pubkey,  // 32
    pub bins: [Bin; 70],  // 70 * 48 = 3360
                          // total: 8 + 8 + 32 + 3360 = 3408
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinArrayAccount(pub BinArray);
impl BinArrayAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        let bin_array_bytemuck = bytemuck::try_from_bytes::<BinArrayBytemuck>(&buf[8..56]);
        let bin_array_bytemuck =
            bin_array_bytemuck.map_err(|e| anyhow::format_err!("Failed to deserialize BinArrayBytemuck: {:?}", e))?;

        // let bins_deser = buf[56..]
        //     .chunks(144)
        //     .map(|chunk| bytemuck::try_pod_read_unaligned::<Bin>(&chunk[..48]).un_wrap());

        let bins_deser: anyhow::Result<Vec<Bin>> = buf[56..]
            .chunks(144)
            .map(|chunk| {
                bytemuck::try_pod_read_unaligned::<Bin>(&chunk[..48])
                    .map_err(|e| anyhow::format_err!("Failed to deserialize Bin: {:?}", e))
            })
            .collect();
        let bins_deser = bins_deser.map_err(|e| anyhow::format_err!("Failed to deserialize bins: {:?}", e))?;

        let bin_array = BinArray {
            index: bin_array_bytemuck.index,
            //version: bin_array_bytemuck.version,
            padding: bin_array_bytemuck.padding,
            lb_pair: bin_array_bytemuck.lb_pair,
            bins: {
                let mut bins = [Bin {
                    amount_x: 0,
                    amount_y: 0,
                    liquidity_supply: 0,
                    price: 0,
                }; 70];
                if bins_deser.len() != bins.len() {
                    return Err(anyhow::format_err!(
                        "Deserialized bins length mismatch. Expected: {}, Got: {}",
                        bins.len(),
                        bins_deser.len()
                    ));
                }
                //bins.copy_from_slice(bins_deser.collect::<Vec<_>>().as_slice());
                bins.copy_from_slice(&bins_deser);
                bins
            },
        };
        Ok(Self(bin_array))
        // let mut reader = &buf[8..];
        // /*
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != BIN_ARRAY_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             BIN_ARRAY_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }
        //  */
        // Ok(Self(BinArray::deserialize(&mut reader)?))
    }
    // pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
    //     writer.write_all(&BIN_ARRAY_ACCOUNT_DISCM)?;
    //     self.0.serialize(&mut writer)
    // }
    // pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
    //     let mut data = Vec::new();
    //     self.serialize(&mut data)?;
    //     Ok(data)
    // }
}

pub const LB_PAIR_ACCOUNT_DISCM: [u8; 8] = [33, 11, 49, 98, 181, 101, 177, 13];
#[derive(Clone, Debug, PartialEq, Pod, Zeroable, Copy)]
#[repr(C)]
pub struct LbPair {
    pub parameters: StaticParameters,
    pub v_parameters: VariableParameters,
    pub bump_seed: [u8; 1],
    pub bin_step_seed: [u8; 2],
    pub pair_type: u8,
    pub active_id: i32,
    pub bin_step: u16,
    pub status: u8,
    pub require_base_factor_seed: u8,
    pub base_factor_seed: [u8; 2],
    pub activation_type: u8,
    pub padding0: u8,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub protocol_fee: ProtocolFee,
    //pub padding1: [u8; 32], // 32
    //pub reward_infos: [RewardInfo; 2], // 144*2 = 288
    //pub padding1: [u8; 352],
    pub padding1: [u8; 256],
    pub padding2: [u8; 64],
    //pub padding3: [u8; 32],
    pub oracle: Pubkey,
    pub bin_array_bitmap: [u64; 16],
    pub last_updated_at: i64,
    //pub padding2: [u8; 32],
    //pub pre_activation_swap_address: Pubkey,
    //pub base_key: Pubkey,
    pub padding4: [u8; 96],
    pub activation_point: u64,
    pub pre_activation_duration: u64,
    //pub padding3: [u8; 8],
    //pub padding4: u64,
    /* 68 */ // 8 + 8 + 32 + 24 = 72
    pub padding5: [u8; 8],
    pub padding6: u64,
    pub padding7: [u8; 32],

    pub token_mint_x_program_flag: u8,
    pub token_mint_y_program_flag: u8,

    pub padding8: [u8; 22],
    // pub padding8: [u8; 16],
    // pub padding9: [u8; 4],
    // pub padding10: [u8; 2],
}

impl LbPair {
    // TODO: need to test
    // pub fn remove_bin_arrays(&self, bin_array_map: &mut BTreeMap<i32, (Pubkey, BinArray)>) {
    //     let start_bin_array_idx = BinArray::bin_id_to_bin_array_index(self.active_id).unwrap_or(0);
    //     let mut bin_arrays_down: Vec<i32> = bin_array_map
    //         .iter()
    //         .rev()
    //         .filter(|(_id, (_, bin_array))| {
    //             !bin_array.is_zero_liquidity() && bin_array.index as i32 <= start_bin_array_idx
    //         })
    //         .take(5)
    //         .map(|(index, _)| index.clone())
    //         .collect();
    //     let bin_arrays_up: Vec<i32> = bin_array_map
    //         .iter()
    //         .filter(|(_id, (_, bin_array))| {
    //             !bin_array.is_zero_liquidity() && bin_array.index as i32 > start_bin_array_idx
    //         })
    //         .take(5)
    //         .map(|(index, _)| index.clone())
    //         .collect();
    //     bin_arrays_down.extend(bin_arrays_up);
    //     let keys: Vec<i32> = bin_array_map.keys().cloned().collect();
    //     for key in keys.iter() {
    //         bin_array_map.remove(key);
    //     }
    // }

    pub fn is_token_2022(&self) -> bool {
        self.token_mint_x_program_flag != 0 || self.token_mint_y_program_flag != 0
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        let mut accounts = vec![
            AccountMeta {
                pubkey: pool_pubkey,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: if optional_account.is_none() {
                    self.get_program_id()
                } else {
                    optional_account.context("Optional account is None")?
                },
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.reserve_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.reserve_y,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: source_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: destination_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.token_x_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.token_y_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.oracle,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.get_program_id(), //self.host_fee_in,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: signer,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: if self.token_mint_x_program_flag == 0 {
                    spl_token::ID
                } else {
                    spl_token_2022::ID
                },
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: if self.token_mint_y_program_flag == 0 {
                    spl_token::ID
                } else {
                    spl_token_2022::ID
                },
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: EVENT_AUTHORITY,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.get_program_id(),
                is_signer: false,
                is_writable: false,
            },
        ];
        if let Some(optional_accounts) = optional_accounts {
            let meta = optional_accounts
                .into_iter()
                .map(|account| AccountMeta::new(account, false))
                .collect::<Vec<AccountMeta>>();
            accounts.extend(meta);
        }
        Ok(accounts)
    }
}

impl<'a> Pool<QuoteInput<'a>> for LbPair {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.reserve_x, &self.reserve_y]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.reserve_x, self.reserve_y]
    }

    fn get_a_mint(&self) -> &Pubkey {
        &self.token_x_mint
    }

    fn get_b_mint(&self) -> &Pubkey {
        &self.token_y_mint
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount: u64, input: &QuoteInput<'a>) -> anyhow::Result<QuoteResult> {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get current timestamp")?
            .as_secs();

        if exact_in {
            quote_exact_in_bitmap(
                self,
                amount,
                a_to_b,
                input.bin_arrays,
                input.bitmap_extension,
                current_timestamp,
                input.current_slot,
            )
        } else {
            quote_exact_out_bitmap(
                self,
                amount,
                a_to_b,
                input.bin_arrays,
                input.bitmap_extension,
                current_timestamp,
                input.current_slot,
            )
        }
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        _: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        if exact_in {
            SwapIxData(SwapIxArgs {
                amount_in: amount,
                min_amount_out: amount_threshold,
            })
            .try_to_vec()
            .context("Failed to serialize SwapIxData")
        } else {
            SwapExactOutIxData(SwapExactOutIxArgs {
                out_amount: amount,
                max_in_amount: amount_threshold,
            })
            .try_to_vec()
            .context("Failed to serialize SwapExactOutIxData")
        }
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        _a_to_b: bool,
        optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        self.get_swap_keys_ix(
            pool_pubkey,
            signer,
            source_token_account,
            destination_token_account,
            optional_account,
            optional_accounts,
        )
    }

    // fn coefficient_to_cu(&self, coefficient: u64) -> u64 {
    //     50_200 + (coefficient * 5200)
    // }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LbPairAccount(pub LbPair);
impl LbPairAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        let pair = bytemuck::try_from_bytes::<LbPair>(&buf[8..]);
        let pair = pair.map_err(|e| anyhow::format_err!("Failed to deserialize LbPair: {:?}. Error: {:?}", pair, e))?;
        Ok(Self(*pair))
    }
    // pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
    //     writer.write_all(&LB_PAIR_ACCOUNT_DISCM)?;
    //     self.0.serialize(&mut writer)
    // }
    // pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
    //     let mut data = Vec::new();
    //     self.serialize(&mut data)?;
    //     Ok(data)
    // }
}
pub const ORACLE_ACCOUNT_DISCM: [u8; 8] = [139, 194, 131, 179, 140, 179, 229, 244];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Oracle {
    pub idx: u64,
    pub active_size: u64,
    pub length: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OracleAccount(pub Oracle);
impl OracleAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != ORACLE_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {ORACLE_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(Oracle::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&ORACLE_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const POSITION_ACCOUNT_DISCM: [u8; 8] = [170, 188, 143, 228, 122, 64, 247, 208];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Position {
    pub lb_pair: Pubkey,
    pub owner: Pubkey,
    pub liquidity_shares: [u64; 70],
    pub reward_infos: [UserRewardInfo; 70],
    pub fee_infos: [FeeInfo; 70],
    pub lower_bin_id: i32,
    pub upper_bin_id: i32,
    pub last_updated_at: i64,
    pub total_claimed_fee_x_amount: u64,
    pub total_claimed_fee_y_amount: u64,
    pub total_claimed_rewards: [u64; 2],
    pub reserved: [u8; 160],
}
#[derive(Clone, Debug, PartialEq)]
pub struct PositionAccount(pub Position);
impl PositionAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != POSITION_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {POSITION_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(Position::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&POSITION_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const POSITION_V2_ACCOUNT_DISCM: [u8; 8] = [117, 176, 212, 199, 245, 180, 133, 182];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PositionV2 {
    pub lb_pair: Pubkey,
    pub owner: Pubkey,
    pub liquidity_shares: [u128; 70],
    pub reward_infos: [UserRewardInfo; 70],
    pub fee_infos: [FeeInfo; 70],
    pub lower_bin_id: i32,
    pub upper_bin_id: i32,
    pub last_updated_at: i64,
    pub total_claimed_fee_x_amount: u64,
    pub total_claimed_fee_y_amount: u64,
    pub total_claimed_rewards: [u64; 2],
    pub operator: Pubkey,
    pub lock_release_point: u64,
    pub padding0: u8,
    pub fee_owner: Pubkey,
    pub reserved: [u8; 87],
}
#[derive(Clone, Debug, PartialEq)]
pub struct PositionV2Account(pub PositionV2);
impl PositionV2Account {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != POSITION_V2_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {POSITION_V2_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(PositionV2::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&POSITION_V2_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const PRESET_PARAMETER_ACCOUNT_DISCM: [u8; 8] = [242, 62, 244, 34, 181, 112, 58, 170];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PresetParameter {
    pub bin_step: u16,
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub protocol_share: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PresetParameterAccount(pub PresetParameter);
impl PresetParameterAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != PRESET_PARAMETER_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {PRESET_PARAMETER_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(PresetParameter::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&PRESET_PARAMETER_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
