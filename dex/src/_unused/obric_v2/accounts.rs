use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use utils::{pool::Pool, quote::QuoteResult};
use super::{SwapIxArgs, SwapIxData, SwapKeys};

//pub const SS_TRADING_PAIR_ACCOUNT_DISCM: [u8; 8] = [154, 154, 241, 58, 97, 92, 33, 61];

pub const SS_TRADING_PAIR_ACCOUNT_DISCM: [u8; 8] = [59, 222, 15, 236, 98, 102, 90, 224];

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SSTradingPair {
    pub is_initialized: bool,
    pub x_price_feed_id: Pubkey,
    pub y_price_feed_id: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub protocol_fee_x: Pubkey,
    pub protocol_fee_y: Pubkey,
    pub bump: u8,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub concentration: u64,
    pub big_k: u128,
    pub target_x: u64,
    pub cumulative_volume: u64,
    pub mult_x: u64,
    pub mult_y: u64,
    pub fee_millionth: u64,
    pub rebate_percentage: u64,
    pub protocol_fee_share_thousandth: u64,
    pub volume_record: [u64; 8],
    pub volume_time_record: [i64; 8],
    pub version: u8,
    pub padding: [u8; 7],
    pub mint_sslp_x: Pubkey,
    pub mint_sslp_y: Pubkey,
    pub padding2: [u64; 15],
}
#[derive(Clone, Debug, PartialEq)]
pub struct SSTradingPairAccount(pub SSTradingPair);
impl SSTradingPairAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SS_TRADING_PAIR_ACCOUNT_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        SS_TRADING_PAIR_ACCOUNT_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(SSTradingPair::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SS_TRADING_PAIR_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

impl SSTradingPair {

    // https://www.npmjs.com/package/obric-solana?activeTab=code
    
    pub fn quote_x_to_y(&self, amount_in: u64, reserve_x: u64, reserve_y: u64) -> anyhow::Result<u64> {
        if amount_in == 0 {
            Err(anyhow::anyhow!("Invalid input x amount"))?;
        }
        let amount_in = amount_in as u128;
        let target_x = self.target_x as u128;
        let current_x = reserve_x as u128;
        let current_y = reserve_y as u128;
        // 0. get target_x on curve-K
        let big_k = self.big_k;
        // target_x_K = sqrt(big_k / p), where p = mult_x / mult_y
        // Note: BN.sqr is not sqrt !
        let target_xk = big_k * (self.mult_y as u128) / (self.mult_x as u128);
        // 1. find current (x,y) on curve-K
        let current_xk = target_xk - (target_x as u128) + (current_x as u128);
        // BN.div(0) = 0
        let current_yk = big_k / current_xk;
        // 2. find new (x, y) on curve-K
        let new_xk = current_xk + amount_in;
        let new_yk = big_k / new_xk;
        let output_before_fee_y = current_yk - new_yk;
        if output_before_fee_y > current_y {
            Err(anyhow::anyhow!("Insufficient active Y"))?;
        }
        let fee_before_rebate_y = output_before_fee_y * (self.fee_millionth as u128) / 1_000_000;
        let rebate_ratio = std::cmp::min(amount_in , target_x - std::cmp::min(target_x, current_x))
            * 100
            / amount_in;
        let rebate_y = fee_before_rebate_y * rebate_ratio / 100 * (self.rebate_percentage as u128) / 100;
        let fee_y = fee_before_rebate_y - rebate_y;
        let output_after_fee_y = output_before_fee_y - fee_y;
        return Ok(output_after_fee_y as u64);
    }

    pub fn quote_y_to_x(&self, amount_in: u64, reserve_x: u64, reserve_y: u64) -> anyhow::Result<u64> {
        if amount_in == 0 {
            Err(anyhow::anyhow!("Invalid input y amount"))?;
        }
        let amount_in = amount_in as u128;
        let target_x = self.target_x as u128;
        let target_y = self.target_x as u128;
        let current_x = reserve_x as u128;
        let current_y = reserve_y as u128;
        // 0. get target_x on curve-K
        let big_k = self.big_k;
        // target_x_K = sqrt(big_k / p), where p = mult_x / mult_y
        // Note: BN.sqr is not sqrt !
        let target_xk = big_k * (self.mult_y as u128) / (self.mult_x as u128);
        // 1. find current (x,y) on curve-K
        let current_xk = target_xk - target_x + current_x;
        // BN.div(0) = 0
        let current_yk = big_k / current_xk;
        // 2. find new (x, y) on curve-K
        let new_yk = current_yk + amount_in;
        let new_xk = big_k / new_yk;
        let output_before_fee_x = current_xk - new_xk;
        if output_before_fee_x > current_x {
            Err(anyhow::anyhow!("Insufficient active X"))?;
        }
        let fee_before_rebate_x = output_before_fee_x * (self.fee_millionth  as u128) / 1_000_000;
        let rebate_ratio = std::cmp::min(amount_in, target_y - std::cmp::min(target_y, current_y))
            * 100
            / amount_in;
        let rebate_x = fee_before_rebate_x * rebate_ratio / 100 * (self.rebate_percentage  as u128) / 100;
        let fee_x = fee_before_rebate_x - rebate_x;
        let output_after_fee_x = output_before_fee_x - fee_x;
        return Ok(output_after_fee_x as u64);
    }

    // pub fn update_target_xy(&mut self, reserve_x: u64, reserve_y: u64)  {
    //     let current_x = reserve_x as u128;
    //     let current_y = reserve_y as u128;
    //     let value_total = current_x * (self.mult_x as u128) + current_y * (self.mult_y as u128);
    //     let target_x = self.target_x as u128;        
    //     let target_x_value = target_x * (self.mult_x as u128);
    //     let target_y_value = value_total - target_x_value;        
    //     let target_y = target_y_value / (self.mult_y as u128);
    //     self.target_x = target_x as u64;        
    //     self.target_y = target_y as u64;     
    // }

    pub fn update_trading_pair_price(&mut self, x_price: u64, y_price: u64, x_decimals: u8, y_decimals: u8) {
        let x_decimals_mult;
        let y_decimals_mult;
        if x_decimals > y_decimals {
            x_decimals_mult = 1;
            y_decimals_mult = 10u64.pow((x_decimals - y_decimals) as u32);
        } else if y_decimals > x_decimals {
            x_decimals_mult = 10u64.pow((y_decimals - x_decimals) as u32);
            y_decimals_mult = 1;
        } else {
            x_decimals_mult = 1;
            y_decimals_mult = 1;
        }
        self.mult_x = x_price * x_decimals_mult;
        self.mult_y = y_price * y_decimals_mult;
    }
        
}

#[derive(Debug)]
pub struct QuoteInput {
    pub coin_a: u64,
    pub coin_b: u64,
}

impl Pool<QuoteInput> for SSTradingPair {
    fn quote(
        &self,
        a_to_b: bool,
        exact_in: bool,
        amount_in: u64,
        input: &QuoteInput,
    ) -> anyhow::Result<QuoteResult> {
        if exact_in {
            if a_to_b {
                let amount_out = self.quote_x_to_y(amount_in, input.coin_a, input.coin_b)?;
                Ok(QuoteResult {
                    amount_a_in: amount_in,
                    amount_b_in: 0,
                    amount_a_out: 0,
                    amount_b_out: amount_out,
                    indices: None,
                    compute_units: 0,
                })
            } else {
                let amount_out = self.quote_y_to_x(amount_in, input.coin_a, input.coin_b)?;
                Ok(QuoteResult {
                    amount_a_in: 0,
                    amount_b_in: amount_in,
                    amount_a_out: amount_out,
                    amount_b_out: 0,
                    indices: None,
                    compute_units: 0,
                })
            }
        } else {
            return Err(anyhow::anyhow!("ExactOutNotSupportedObricV2"))?;
        }
    }

    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![
            &self.reserve_x,
            &self.reserve_y,            
        ]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![
            self.reserve_x,
            self.reserve_y,            
        ]
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.mint_x
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.mint_y
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
    ) -> anyhow::Result<Vec<u8>> {
        if !exact_in {
            return Err(anyhow::anyhow!("ExactOutNotSupportedObricV2"))?;
        }
        SwapIxData(
            SwapIxArgs {
                is_x_to_y: a_to_b,
                input_amt: amount,
                min_output_amt: amount_threshold,
            }
        ).try_to_vec()
            .map_err(|e| anyhow::anyhow!("FailedToSerializeSwapDataObricV2: {}", e))
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
    ) -> anyhow::Result<Vec<solana_sdk::instruction::AccountMeta>> {
        let keys = SwapKeys {
            trading_pair: pool_pubkey,
            mint_x: self.protocol_fee_y,
            mint_y: self.mint_sslp_x,
            reserve_x: self.reserve_x,
            reserve_y: self.reserve_y,
            user_token_account_x: if a_to_b {
                source_token_account
            } else {
                destination_token_account
            },
            user_token_account_y: if a_to_b {
                destination_token_account
            } else {
                source_token_account
            },
            protocol_fee: if a_to_b {
                self.protocol_fee_x
            } else {
                self.protocol_fee_y
            },
            x_price_feed: self.x_price_feed_id,
            y_price_feed: self.y_price_feed_id,
            user: signer,
            token_program: spl_token::ID
        };
        //println!("Self: {:?}", self);
        Ok(vec! [
            AccountMeta {
                pubkey: keys.trading_pair,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint_x,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.mint_y,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reserve_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reserve_y,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_token_account_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_token_account_y,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.x_price_feed,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.y_price_feed,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {                
                pubkey: keys.user,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
        ])
    }
}