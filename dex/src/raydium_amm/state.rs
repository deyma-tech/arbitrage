#![allow(deprecated)]
use super::math::Calculator;
use super::swap::RAYDIUM_AUTHORITY_V4;
use super::{CheckedCeilDiv, U128};
use ahash::AHashSet;
use anyhow::{format_err, Context};
use bytemuck::{Pod, Zeroable};
use solana_program::program_error::ProgramError;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use utils::{pool::Pool, quote::QuoteResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum SwapDirection {
    /// Input token pc, output token coin
    PC2Coin = 1u64,
    /// Input token coin, output token pc
    Coin2PC = 2u64,
}

// #[derive(PartialEq)]
// pub enum RaydiumPrice {
//     AmmInfo,
//     Vault,
//     SolUsd,
// }

// pub trait Loadable: Pod {
//     fn load_mut<'a>(account: &'a AccountInfo) -> Result<RefMut<'a, Self>, ProgramError> {
//         // TODO verify if this checks for size
//         Ok(RefMut::map(account.try_borrow_mut_data()?, |data| {
//             from_bytes_mut(data)
//         }))
//     }
//     fn load<'a>(account: &'a AccountInfo) -> Result<Ref<'a, Self>, ProgramError> {
//         Ok(Ref::map(account.try_borrow_data()?, |data| {
//             from_bytes(data)
//         }))
//     }

//     fn load_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
//         Ok(from_bytes(data))
//     }
// }

// macro_rules! impl_loadable {
//     ($type_name:ident) => {
//         unsafe impl Zeroable for $type_name {}
//         unsafe impl Pod for $type_name {}
//         unsafe impl safe_transmute::TriviallyTransmutable for $type_name {}
//         impl Loadable for $type_name {}
//     };
// }

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Fees {
    /// numerator of the min_separate
    pub min_separate_numerator: u64,
    /// denominator of the min_separate
    pub min_separate_denominator: u64,

    /// numerator of the fee
    pub trade_fee_numerator: u64,
    /// denominator of the fee
    /// and 'trade_fee_denominator' must be equal to 'min_separate_denominator'
    pub trade_fee_denominator: u64,

    /// numerator of the pnl
    pub pnl_numerator: u64,
    /// denominator of the pnl
    pub pnl_denominator: u64,

    /// numerator of the swap_fee
    pub swap_fee_numerator: u64,
    /// denominator of the swap_fee
    pub swap_fee_denominator: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct StateData {
    /// delay to take pnl coin
    pub need_take_pnl_coin: u64,
    /// delay to take pnl pc
    pub need_take_pnl_pc: u64,
    /// total pnl pc
    pub total_pnl_pc: u64,
    /// total pnl coin
    pub total_pnl_coin: u64,
    /// ido pool open time
    pub pool_open_time: u64,
    /// padding for future updates
    pub padding: [u64; 2],
    /// switch from orderbookonly to init
    pub orderbook_to_init_time: u64,

    /// swap coin in amount
    pub swap_coin_in_amount: u128,
    /// swap pc out amount
    pub swap_pc_out_amount: u128,
    /// charge pc as swap fee while swap pc to coin
    pub swap_acc_pc_fee: u64,

    /// swap pc in amount
    pub swap_pc_in_amount: u128,
    /// swap coin out amount
    pub swap_coin_out_amount: u128,
    /// charge coin as swap fee while swap coin to pc
    pub swap_acc_coin_fee: u64,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, Pod, Zeroable)]
#[repr(C, packed)]
pub struct AmmInfo {
    /// Initialized status.
    pub status: u64,
    /// Nonce used in program address.
    /// The program address is created deterministically with the nonce,
    /// amm program id, and amm account pubkey.  This program address has
    /// authority over the amm's token coin account, token pc account, and pool
    /// token mint.
    pub nonce: u64,
    /// max order count
    pub order_num: u64,
    /// within this range, 5 => 5% range
    pub depth: u64,
    /// coin decimal
    pub coin_decimals: u64,
    /// pc decimal
    pub pc_decimals: u64,
    /// amm machine state
    pub state: u64,
    /// amm reset_flag
    pub reset_flag: u64,
    /// min size 1->0.000001
    pub min_size: u64,
    /// vol_max_cut_ratio numerator, sys_decimal_value as denominator
    pub vol_max_cut_ratio: u64,
    /// amount wave numerator, sys_decimal_value as denominator
    pub amount_wave: u64,
    /// coinLotSize 1 -> 0.000001
    pub coin_lot_size: u64,
    /// pcLotSize 1 -> 0.000001
    pub pc_lot_size: u64,
    /// min_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub min_price_multiplier: u64,
    /// max_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub max_price_multiplier: u64,
    /// system decimal value, used to normalize the value of coin and pc amount
    pub sys_decimal_value: u64,
    /// All fee information
    pub fees: Fees,
    /// Statistical data
    pub state_data: StateData,
    /// Coin vault
    pub coin_vault: Pubkey,
    /// Pc vault
    pub pc_vault: Pubkey,
    /// Coin vault mint
    pub coin_vault_mint: Pubkey,
    /// Pc vault mint
    pub pc_vault_mint: Pubkey,
    /// lp mint
    pub lp_mint: Pubkey,
    /// open_orders key
    pub open_orders: Pubkey,
    /// market key
    pub market: Pubkey,
    /// market program key
    pub market_program: Pubkey,
    /// target_orders key
    pub target_orders: Pubkey,
    /// padding
    pub padding1: [u64; 8],
    /// amm owner key
    pub amm_owner: Pubkey,
    /// pool lp amount
    pub lp_amount: u64,
    /// client order id
    pub client_order_id: u64,
    /// recent epoch
    pub recent_epoch: u64,
    /// padding
    pub padding2: u64,
}

impl AmmInfo {
    pub fn swap_permission(&self) -> bool {
        if self.status < 8 {
            AmmStatus::from_u64(self.status).swap_permission()
        } else {
            false
        }
    }

    pub fn orderbook_permission(&self) -> bool {
        if self.status < 8 {
            AmmStatus::from_u64(self.status).orderbook_permission()
        } else {
            false
        }
    }
}

pub struct QuoteInput {
    // pub total_pc_without_take_pnl: U128,   // pc = b
    // pub total_coin_without_take_pnl: U128, // coin = a
    pub total_pc: u64,
    pub total_coin: u64, // coin = a
}

#[repr(u64)]
pub enum AmmStatus {
    Uninitialized = 0u64,
    Initialized = 1u64,
    Disabled = 2u64,
    WithdrawOnly = 3u64,
    // pool only can add or remove liquidity, can't swap and plan orders
    LiquidityOnly = 4u64,
    // pool only can add or remove liquidity and plan orders, can't swap
    OrderBookOnly = 5u64,
    // pool only can add or remove liquidity and swap, can't plan orders
    SwapOnly = 6u64,
    // pool status after created and will auto update to SwapOnly during swap after open_time
    WaitingTrade = 7u64,
}

impl AmmStatus {
    #[inline(always)]
    pub fn from_u64(status: u64) -> Self {
        match status {
            0u64 => AmmStatus::Uninitialized,
            1u64 => AmmStatus::Initialized,
            2u64 => AmmStatus::Disabled,
            3u64 => AmmStatus::WithdrawOnly,
            4u64 => AmmStatus::LiquidityOnly,
            5u64 => AmmStatus::OrderBookOnly,
            6u64 => AmmStatus::SwapOnly,
            7u64 => AmmStatus::WaitingTrade,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn into_u64(&self) -> u64 {
        match self {
            AmmStatus::Uninitialized => 0u64,
            AmmStatus::Initialized => 1u64,
            AmmStatus::Disabled => 2u64,
            AmmStatus::WithdrawOnly => 3u64,
            AmmStatus::LiquidityOnly => 4u64,
            AmmStatus::OrderBookOnly => 5u64,
            AmmStatus::SwapOnly => 6u64,
            AmmStatus::WaitingTrade => 7u64,
        }
    }
    pub fn valid_status(status: u64) -> bool {
        matches!(status, 1u64..=7u64)
    }

    #[inline(always)]
    pub fn deposit_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    #[inline(always)]
    pub fn withdraw_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => true,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    #[inline(always)]
    pub fn swap_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => false,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    #[inline(always)]
    pub fn orderbook_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => false,
            AmmStatus::WaitingTrade => false,
        }
    }
}

impl AmmInfo {
    pub fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        _: Option<Pubkey>,
        _: Option<Vec<Pubkey>>,
    ) -> Vec<AccountMeta> {
        vec![
            // spl token
            AccountMeta::new_readonly(
                /*spl_token::id()*/ pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                false,
            ),
            // amm
            AccountMeta::new(pool_pubkey, false),
            AccountMeta::new_readonly(RAYDIUM_AUTHORITY_V4, false),
            // OPEN ORDER ACCOUNT ...
            //AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            AccountMeta::new(self.coin_vault, false),
            AccountMeta::new(self.pc_vault, false),
            // market
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // AccountMeta::new(RAYDIUM_AUTHORITY_V4, false),
            // user
            AccountMeta::new(source_token_account, false),
            AccountMeta::new(destination_token_account, false),
            AccountMeta::new(signer, true),
        ]
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        let total_pc_without_take_pnl = input
            .total_pc
            .checked_sub(self.state_data.need_take_pnl_pc)
            .context("Overflow:RaydiumAMM")?
            .into();

        let total_coin_without_take_pnl = input
            .total_coin
            .checked_sub(self.state_data.need_take_pnl_coin)
            .context("Overflow:RaydiumAMM")?
            .into();
        if exact_in {
            let amount_u128: U128 = amount.into();
            if a_to_b {
                if amount_u128 > total_coin_without_take_pnl {
                    return Err(format_err!("RaydiumAMMPoolOutOfLiquidity"));
                }
            } else if amount_u128 > total_pc_without_take_pnl {
                return Err(format_err!("RaydiumAMMPoolOutOfLiquidity"));
            }

            let swap_fee = amount_u128
                .checked_mul(self.fees.swap_fee_numerator.into())
                .context("MathOverflow")?
                .checked_ceil_div(self.fees.swap_fee_denominator.into())
                .context("MathOverflow")?
                .0;
            let swap_in_after_deduct_fee = amount_u128.checked_sub(swap_fee).context("MathOverflow")?;

            let amount_out = Calculator::swap_token_amount_base_in(
                swap_in_after_deduct_fee,
                total_pc_without_take_pnl,
                total_coin_without_take_pnl,
                if a_to_b {
                    SwapDirection::Coin2PC
                } else {
                    SwapDirection::PC2Coin
                },
            )
            .map_err(|e| format_err!("RaydiumAMMQuote1: {:?}", e))?;

            if a_to_b {
                Ok(QuoteResult {
                    amount_a_in: amount,
                    amount_b_in: 0,
                    amount_a_out: 0,
                    amount_b_out: amount_out.as_u64(),
                    indices: None,
                    compute_units: 27_000,
                })
            } else {
                Ok(QuoteResult {
                    amount_a_in: 0,
                    amount_b_in: amount,
                    amount_a_out: amount_out.as_u64(),
                    amount_b_out: 0,
                    indices: None,
                    compute_units: 27_000,
                })
            }
        } else {
            let amount_out = Calculator::swap_token_amount_base_out(
                amount.into(),
                total_pc_without_take_pnl,
                total_coin_without_take_pnl,
                if a_to_b {
                    SwapDirection::Coin2PC
                } else {
                    SwapDirection::PC2Coin
                },
            )
            .map_err(|e| format_err!("RaydiumAMMQuote2: {:?}", e))?;

            let swap_fee = amount_out
                .checked_mul(self.fees.swap_fee_numerator.into())
                .context("MathOverflow")?
                .checked_ceil_div(self.fees.swap_fee_denominator.into())
                .context("MathOverflow")?
                .0;
            let amount_out = amount_out.checked_add(swap_fee).context("MathOverflow")?;

            if a_to_b {
                Ok(QuoteResult {
                    amount_a_in: amount_out.as_u64(),
                    amount_b_in: 0,
                    amount_a_out: 0,
                    amount_b_out: amount,
                    indices: None,
                    compute_units: 27_000,
                })
            } else {
                Ok(QuoteResult {
                    amount_a_in: 0,
                    amount_b_in: amount_out.as_u64(),
                    amount_a_out: amount,
                    amount_b_out: 0,
                    indices: None,
                    compute_units: 27_000,
                })
            }
        }
    }
}

impl Pool<QuoteInput> for AmmInfo {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.pc_vault, &self.coin_vault]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.pc_vault, self.coin_vault]
    }

    fn get_a_mint<'a>(&self) -> &Pubkey {
        &self.coin_vault_mint
    }

    fn get_b_mint<'a>(&self) -> &Pubkey {
        &self.pc_vault_mint
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        self.quote(a_to_b, exact_in, amount, input)
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
            AmmInstruction::SwapBaseIn(SwapInstructionBaseIn {
                amount_in: amount,
                minimum_amount_out: amount_threshold,
            })
            .pack()
            .context("Raydium AMM: SwapBaseIn")
        } else {
            AmmInstruction::SwapBaseOut(SwapInstructionBaseOut {
                max_amount_in: amount_threshold,
                amount_out: amount,
            })
            .pack()
            .context("Raydium AMM: SwapBaseOut")
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
        Ok(self.get_swap_keys_ix(
            pool_pubkey,
            signer,
            source_token_account,
            destination_token_account,
            optional_account,
            optional_accounts,
        ))
    }
}

//impl_loadable!(AmmInfo);

// #[repr(C, packed)]
// #[derive(Clone, Copy, Default, Debug)]
// pub struct TargetOrder {
//     pub price: u64,
//     pub vol: u64,
// }

// pub const MAX_ORDER_LIMIT: usize = 10;

// #[repr(C, packed)]
// #[derive(Clone, Copy, Debug)]
// pub struct TargetOrders {
//     pub owner: [u64; 4],
//     pub buy_orders: [TargetOrder; 50],
//     pub padding1: [u64; 8],
//     pub target_x: u128,
//     pub target_y: u128,
//     pub plan_x_buy: u128,
//     pub plan_y_buy: u128,
//     pub plan_x_sell: u128,
//     pub plan_y_sell: u128,
//     pub placed_x: u128,
//     pub placed_y: u128,
//     pub calc_pnl_x: u128,
//     pub calc_pnl_y: u128,
//     pub sell_orders: [TargetOrder; 50],
//     pub padding2: [u64; 6],
//     pub replace_buy_client_id: [u64; MAX_ORDER_LIMIT],
//     pub replace_sell_client_id: [u64; MAX_ORDER_LIMIT],
//     pub last_order_numerator: u64,
//     pub last_order_denominator: u64,

//     pub plan_orders_cur: u64,
//     pub place_orders_cur: u64,

//     pub valid_buy_order_num: u64,
//     pub valid_sell_order_num: u64,

//     pub padding3: [u64; 10],

//     pub free_slot_bits: u128,
// }

// /// LogType enum
// #[derive(Debug)]
// pub enum LogType {
//     Init(InitLog),
//     Deposit(DepositLog),
//     Withdraw(WithdrawLog),
//     SwapBaseIn(SwapBaseInLog),
//     SwapBaseOut(SwapBaseOutLog),
//     Unknown,
// }

// impl LogType {
//     pub fn from_data(log_type: u8, data: Vec<u8>) -> Option<Self> {
//         match log_type {
//             0 => Some(LogType::Init(bincode::deserialize(&data).un_wrap())),
//             1 => Some(LogType::Deposit(bincode::deserialize(&data).un_wrap())),
//             2 => Some(LogType::Withdraw(bincode::deserialize(&data).un_wrap())),
//             3 => Some(LogType::SwapBaseIn(bincode::deserialize(&data).un_wrap())),
//             4 => Some(LogType::SwapBaseOut(bincode::deserialize(&data).un_wrap())),
//             _ => None,
//         }
//     }
// }

// #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
// pub struct InitLog {
//     pub log_type: u8,
//     pub time: u64,
//     pub pc_decimals: u8,
//     pub coin_decimals: u8,
//     pub pc_lot_size: u64,
//     pub coin_lot_size: u64,
//     pub pc_amount: u64,
//     pub coin_amount: u64,
//     pub market: Pubkey,
// }

// #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
// pub struct DepositLog {
//     pub log_type: u8,
//     // input
//     pub max_coin: u64,
//     pub max_pc: u64,
//     pub base: u64,
//     // pool info
//     pub pool_coin: u64,
//     pub pool_pc: u64,
//     pub pool_lp: u64,
//     pub calc_pnl_x: u128,
//     pub calc_pnl_y: u128,
//     // calc result
//     pub deduct_coin: u64,
//     pub deduct_pc: u64,
//     pub mint_lp: u64,
// }

// #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
// pub struct WithdrawLog {
//     pub log_type: u8,
//     // input
//     pub withdraw_lp: u64,
//     // user info
//     pub user_lp: u64,
//     // pool info
//     pub pool_coin: u64,
//     pub pool_pc: u64,
//     pub pool_lp: u64,
//     pub calc_pnl_x: u128,
//     pub calc_pnl_y: u128,
//     // calc result
//     pub out_coin: u64,
//     pub out_pc: u64,
// }

// #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
// pub struct SwapBaseInLog {
//     pub log_type: u8,
//     // input
//     pub amount_in: u64,
//     pub minimum_out: u64,
//     pub direction: u64,
//     // user info
//     pub user_source: u64,
//     // pool info
//     pub pool_coin: u64,
//     pub pool_pc: u64,
//     // calc result
//     pub out_amount: u64,
// }

// #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
// pub struct SwapBaseOutLog {
//     pub log_type: u8,
//     // input
//     pub max_in: u64,
//     pub amount_out: u64,
//     pub direction: u64,
//     // user info
//     pub user_source: u64,
//     // pool info
//     pub pool_coin: u64,
//     pub pool_pc: u64,
//     // calc result
//     pub deduct_in: u64,
// }

// pub fn decode_ray_log(log: &str) -> Option<LogType> {
//     let bytes = base64::decode(log).un_wrap();
//     LogType::from_data(bytes[0], bytes)
// }

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwapInstructionBaseIn {
    // SOURCE amount to transfer, output to DESTINATION is based on the exchange rate
    pub amount_in: u64,
    /// Minimum amount of DESTINATION token to output, prevents excessive slippage
    pub minimum_amount_out: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwapInstructionBaseOut {
    // SOURCE amount to transfer, output to DESTINATION is based on the exchange rate
    pub max_amount_in: u64,
    /// Minimum amount of DESTINATION token to output, prevents excessive slippage
    pub amount_out: u64,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum AmmInstruction {
    /// Swap coin or pc from pool, base amount_in with a slippage of minimum_amount_out
    ///
    ///   0. `[]` Spl Token program id
    ///   1. `[writable]` AMM Account
    ///   2. `[]` $authority derived from `create_program_address(&[AUTHORITY_AMM, &[nonce]])`.
    ///   3. `[writable]` AMM open orders Account
    ///   4. `[writable]` (optional)AMM target orders Account, no longer used in the contract, recommended no need to add this Account.
    ///   5. `[writable]` AMM coin vault Account to swap FROM or To.
    ///   6. `[writable]` AMM pc vault Account to swap FROM or To.
    ///   7. `[]` Market program id
    ///   8. `[writable]` Market Account. Market program is the owner.
    ///   9. `[writable]` Market bids Account
    ///   10. `[writable]` Market asks Account
    ///   11. `[writable]` Market event queue Account
    ///   12. `[writable]` Market coin vault Account
    ///   13. `[writable]` Market pc vault Account
    ///   14. '[]` Market vault signer Account
    ///   15. `[writable]` User source token Account.
    ///   16. `[writable]` User destination token Account.
    ///   17. `[signer]` User wallet Account
    SwapBaseIn(SwapInstructionBaseIn),

    /// Swap coin or pc from pool, base amount_out with a slippage of max_amount_in
    ///
    ///   0. `[]` Spl Token program id
    ///   1. `[writable]` AMM Account
    ///   2. `[]` $authority derived from `create_program_address(&[AUTHORITY_AMM, &[nonce]])`.
    ///   3. `[writable]` AMM open orders Account
    ///   4. `[writable]` (optional)AMM target orders Account, no longer used in the contract, recommended no need to add this Account.
    ///   5. `[writable]` AMM coin vault Account to swap FROM or To.
    ///   6. `[writable]` AMM pc vault Account to swap FROM or To.
    ///   7. `[]` Market program id
    ///   8. `[writable]` Market Account. Market program is the owner.
    ///   9. `[writable]` Market bids Account
    ///   10. `[writable]` Market asks Account
    ///   11. `[writable]` Market event queue Account
    ///   12. `[writable]` Market coin vault Account
    ///   13. `[writable]` Market pc vault Account
    ///   14. '[]` Market vault signer Account
    ///   15. `[writable]` User source token Account.
    ///   16. `[writable]` User destination token Account.
    ///   17. `[signer]` User wallet Account
    SwapBaseOut(SwapInstructionBaseOut),
}

impl AmmInstruction {
    pub fn pack(&self) -> Result<Vec<u8>, ProgramError> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::SwapBaseIn(SwapInstructionBaseIn {
                amount_in,
                minimum_amount_out,
            }) => {
                buf.push(16);
                buf.extend_from_slice(&amount_in.to_le_bytes());
                buf.extend_from_slice(&minimum_amount_out.to_le_bytes());
            }
            Self::SwapBaseOut(SwapInstructionBaseOut {
                max_amount_in,
                amount_out,
            }) => {
                buf.push(17);
                buf.extend_from_slice(&max_amount_in.to_le_bytes());
                buf.extend_from_slice(&amount_out.to_le_bytes())
            }
        }
        Ok(buf)
    }
}
