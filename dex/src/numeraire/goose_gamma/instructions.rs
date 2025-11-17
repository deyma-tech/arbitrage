use crate::goose_gamma::typedefs::{BinLiquidityReduction, RemainingAccountsInfo};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::io::Read;

#[derive(Clone, Debug, PartialEq)]
pub enum GammaProgramIx {
    CalculateRewards,
    ClaimRewards,
    CollectFundFee(CollectFundFeeIxArgs),
    CollectProtocolFee(CollectProtocolFeeIxArgs),
    CreateAmmConfig(CreateAmmConfigIxArgs),
    CreateRewards(CreateRewardsIxArgs),
    CreateSwapReferral(CreateSwapReferralIxArgs),
    Deposit(DepositIxArgs),
    InitUserPoolLiquidity(InitUserPoolLiquidityIxArgs),
    Initialize(InitializeIxArgs),
    MigrateMeteoraDlmmToGamma(MigrateMeteoraDlmmToGammaIxArgs),
    MigrateOrcaWhirlpoolToGamma(MigrateOrcaWhirlpoolToGammaIxArgs),
    MigrateOrcaWhirlpoolToGammaV2(MigrateOrcaWhirlpoolToGammaV2IxArgs),
    MigrateRaydiumClmmToGamma(MigrateRaydiumClmmToGammaIxArgs),
    MigrateRaydiumClmmToGammaV2(MigrateRaydiumClmmToGammaV2IxArgs),
    MigrateRaydiumCpSwapToGamma(MigrateRaydiumCpSwapToGammaIxArgs),
    RebalanceKamino,
    SwapBaseInput(SwapBaseInputIxArgs),
    SwapBaseOutput(SwapBaseOutputIxArgs),
    UpdateAmmConfig(UpdateAmmConfigIxArgs),
    UpdatePool(UpdatePoolIxArgs),
    Withdraw(WithdrawIxArgs),
}
impl GammaProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            CALCULATE_REWARDS_IX_DISCM => Ok(Self::CalculateRewards),
            CLAIM_REWARDS_IX_DISCM => Ok(Self::ClaimRewards),
            COLLECT_FUND_FEE_IX_DISCM => Ok(Self::CollectFundFee(CollectFundFeeIxArgs::deserialize(&mut reader)?)),
            COLLECT_PROTOCOL_FEE_IX_DISCM => Ok(Self::CollectProtocolFee(CollectProtocolFeeIxArgs::deserialize(
                &mut reader,
            )?)),
            CREATE_AMM_CONFIG_IX_DISCM => Ok(Self::CreateAmmConfig(CreateAmmConfigIxArgs::deserialize(&mut reader)?)),
            CREATE_REWARDS_IX_DISCM => Ok(Self::CreateRewards(CreateRewardsIxArgs::deserialize(&mut reader)?)),
            CREATE_SWAP_REFERRAL_IX_DISCM => Ok(Self::CreateSwapReferral(CreateSwapReferralIxArgs::deserialize(
                &mut reader,
            )?)),
            DEPOSIT_IX_DISCM => Ok(Self::Deposit(DepositIxArgs::deserialize(&mut reader)?)),
            INIT_USER_POOL_LIQUIDITY_IX_DISCM => Ok(Self::InitUserPoolLiquidity(
                InitUserPoolLiquidityIxArgs::deserialize(&mut reader)?,
            )),
            INITIALIZE_IX_DISCM => Ok(Self::Initialize(InitializeIxArgs::deserialize(&mut reader)?)),
            MIGRATE_METEORA_DLMM_TO_GAMMA_IX_DISCM => Ok(Self::MigrateMeteoraDlmmToGamma(
                MigrateMeteoraDlmmToGammaIxArgs::deserialize(&mut reader)?,
            )),
            MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_DISCM => Ok(Self::MigrateOrcaWhirlpoolToGamma(
                MigrateOrcaWhirlpoolToGammaIxArgs::deserialize(&mut reader)?,
            )),
            MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_DISCM => Ok(Self::MigrateOrcaWhirlpoolToGammaV2(
                MigrateOrcaWhirlpoolToGammaV2IxArgs::deserialize(&mut reader)?,
            )),
            MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_DISCM => Ok(Self::MigrateRaydiumClmmToGamma(
                MigrateRaydiumClmmToGammaIxArgs::deserialize(&mut reader)?,
            )),
            MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_DISCM => Ok(Self::MigrateRaydiumClmmToGammaV2(
                MigrateRaydiumClmmToGammaV2IxArgs::deserialize(&mut reader)?,
            )),
            MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_DISCM => Ok(Self::MigrateRaydiumCpSwapToGamma(
                MigrateRaydiumCpSwapToGammaIxArgs::deserialize(&mut reader)?,
            )),
            REBALANCE_KAMINO_IX_DISCM => Ok(Self::RebalanceKamino),
            SWAP_BASE_INPUT_IX_DISCM => Ok(Self::SwapBaseInput(SwapBaseInputIxArgs::deserialize(&mut reader)?)),
            SWAP_BASE_OUTPUT_IX_DISCM => Ok(Self::SwapBaseOutput(SwapBaseOutputIxArgs::deserialize(&mut reader)?)),
            UPDATE_AMM_CONFIG_IX_DISCM => Ok(Self::UpdateAmmConfig(UpdateAmmConfigIxArgs::deserialize(&mut reader)?)),
            UPDATE_POOL_IX_DISCM => Ok(Self::UpdatePool(UpdatePoolIxArgs::deserialize(&mut reader)?)),
            WITHDRAW_IX_DISCM => Ok(Self::Withdraw(WithdrawIxArgs::deserialize(&mut reader)?)),
            _ => Err(std::io::Error::other(format!("discm {maybe_discm:?} not found"))),
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::CalculateRewards => writer.write_all(&CALCULATE_REWARDS_IX_DISCM),
            Self::ClaimRewards => writer.write_all(&CLAIM_REWARDS_IX_DISCM),
            Self::CollectFundFee(args) => {
                writer.write_all(&COLLECT_FUND_FEE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CollectProtocolFee(args) => {
                writer.write_all(&COLLECT_PROTOCOL_FEE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CreateAmmConfig(args) => {
                writer.write_all(&CREATE_AMM_CONFIG_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CreateRewards(args) => {
                writer.write_all(&CREATE_REWARDS_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CreateSwapReferral(args) => {
                writer.write_all(&CREATE_SWAP_REFERRAL_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Deposit(args) => {
                writer.write_all(&DEPOSIT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::InitUserPoolLiquidity(args) => {
                writer.write_all(&INIT_USER_POOL_LIQUIDITY_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Initialize(args) => {
                writer.write_all(&INITIALIZE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::MigrateMeteoraDlmmToGamma(args) => {
                writer.write_all(&MIGRATE_METEORA_DLMM_TO_GAMMA_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::MigrateOrcaWhirlpoolToGamma(args) => {
                writer.write_all(&MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::MigrateOrcaWhirlpoolToGammaV2(args) => {
                writer.write_all(&MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::MigrateRaydiumClmmToGamma(args) => {
                writer.write_all(&MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::MigrateRaydiumClmmToGammaV2(args) => {
                writer.write_all(&MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::MigrateRaydiumCpSwapToGamma(args) => {
                writer.write_all(&MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::RebalanceKamino => writer.write_all(&REBALANCE_KAMINO_IX_DISCM),
            Self::SwapBaseInput(args) => {
                writer.write_all(&SWAP_BASE_INPUT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::SwapBaseOutput(args) => {
                writer.write_all(&SWAP_BASE_OUTPUT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::UpdateAmmConfig(args) => {
                writer.write_all(&UPDATE_AMM_CONFIG_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::UpdatePool(args) => {
                writer.write_all(&UPDATE_POOL_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Withdraw(args) => {
                writer.write_all(&WITHDRAW_IX_DISCM)?;
                args.serialize(&mut writer)
            }
        }
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const CALCULATE_REWARDS_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct CalculateRewardsAccounts<'me, 'info> {
    pub user: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub reward_info: &'me AccountInfo<'info>,
    pub user_reward_info: &'me AccountInfo<'info>,
    pub user_pool_liquidity: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CalculateRewardsKeys {
    pub user: Pubkey,
    pub pool_state: Pubkey,
    pub reward_info: Pubkey,
    pub user_reward_info: Pubkey,
    pub user_pool_liquidity: Pubkey,
    pub system_program: Pubkey,
}
impl From<CalculateRewardsAccounts<'_, '_>> for CalculateRewardsKeys {
    fn from(accounts: CalculateRewardsAccounts) -> Self {
        Self {
            user: *accounts.user.key,
            pool_state: *accounts.pool_state.key,
            reward_info: *accounts.reward_info.key,
            user_reward_info: *accounts.user_reward_info.key,
            user_pool_liquidity: *accounts.user_pool_liquidity.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<CalculateRewardsKeys> for [AccountMeta; CALCULATE_REWARDS_IX_ACCOUNTS_LEN] {
    fn from(keys: CalculateRewardsKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.user,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reward_info,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user_reward_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_pool_liquidity,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CALCULATE_REWARDS_IX_ACCOUNTS_LEN]> for CalculateRewardsKeys {
    fn from(pubkeys: [Pubkey; CALCULATE_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            user: pubkeys[0],
            pool_state: pubkeys[1],
            reward_info: pubkeys[2],
            user_reward_info: pubkeys[3],
            user_pool_liquidity: pubkeys[4],
            system_program: pubkeys[5],
        }
    }
}
impl<'info> From<CalculateRewardsAccounts<'_, 'info>> for [AccountInfo<'info>; CALCULATE_REWARDS_IX_ACCOUNTS_LEN] {
    fn from(accounts: CalculateRewardsAccounts<'_, 'info>) -> Self {
        [
            accounts.user.clone(),
            accounts.pool_state.clone(),
            accounts.reward_info.clone(),
            accounts.user_reward_info.clone(),
            accounts.user_pool_liquidity.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CALCULATE_REWARDS_IX_ACCOUNTS_LEN]>
    for CalculateRewardsAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; CALCULATE_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            user: &arr[0],
            pool_state: &arr[1],
            reward_info: &arr[2],
            user_reward_info: &arr[3],
            user_pool_liquidity: &arr[4],
            system_program: &arr[5],
        }
    }
}
pub const CALCULATE_REWARDS_IX_DISCM: [u8; 8] = [199, 115, 201, 124, 71, 81, 143, 252];
#[derive(Clone, Debug, PartialEq)]
pub struct CalculateRewardsIxData;
impl CalculateRewardsIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CALCULATE_REWARDS_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {CALCULATE_REWARDS_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CALCULATE_REWARDS_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn calculate_rewards_ix_with_program_id(
    program_id: Pubkey,
    keys: CalculateRewardsKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CALCULATE_REWARDS_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: CalculateRewardsIxData.try_to_vec()?,
    })
}
pub fn calculate_rewards_ix(keys: CalculateRewardsKeys) -> std::io::Result<Instruction> {
    calculate_rewards_ix_with_program_id(super::ID, keys)
}

pub fn calculate_rewards_verify_account_keys(
    accounts: CalculateRewardsAccounts<'_, '_>,
    keys: CalculateRewardsKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.user.key, keys.user),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.reward_info.key, keys.reward_info),
        (*accounts.user_reward_info.key, keys.user_reward_info),
        (*accounts.user_pool_liquidity.key, keys.user_pool_liquidity),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn calculate_rewards_verify_writable_privileges<'me, 'info>(
    accounts: CalculateRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.user, accounts.user_reward_info] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn calculate_rewards_verify_signer_privileges<'me, 'info>(
    accounts: CalculateRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn calculate_rewards_verify_account_privileges<'me, 'info>(
    accounts: CalculateRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    calculate_rewards_verify_writable_privileges(accounts)?;
    calculate_rewards_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CLAIM_REWARDS_IX_ACCOUNTS_LEN: usize = 11;
#[derive(Copy, Clone, Debug)]
pub struct ClaimRewardsAccounts<'me, 'info> {
    pub user: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub reward_info: &'me AccountInfo<'info>,
    pub reward_vault: &'me AccountInfo<'info>,
    pub user_token_account: &'me AccountInfo<'info>,
    pub user_reward_info: &'me AccountInfo<'info>,
    pub reward_mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClaimRewardsKeys {
    pub user: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub reward_info: Pubkey,
    pub reward_vault: Pubkey,
    pub user_token_account: Pubkey,
    pub user_reward_info: Pubkey,
    pub reward_mint: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub system_program: Pubkey,
}
impl From<ClaimRewardsAccounts<'_, '_>> for ClaimRewardsKeys {
    fn from(accounts: ClaimRewardsAccounts) -> Self {
        Self {
            user: *accounts.user.key,
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            reward_info: *accounts.reward_info.key,
            reward_vault: *accounts.reward_vault.key,
            user_token_account: *accounts.user_token_account.key,
            user_reward_info: *accounts.user_reward_info.key,
            reward_mint: *accounts.reward_mint.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<ClaimRewardsKeys> for [AccountMeta; CLAIM_REWARDS_IX_ACCOUNTS_LEN] {
    fn from(keys: ClaimRewardsKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.user,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reward_info,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reward_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_reward_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reward_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CLAIM_REWARDS_IX_ACCOUNTS_LEN]> for ClaimRewardsKeys {
    fn from(pubkeys: [Pubkey; CLAIM_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            user: pubkeys[0],
            authority: pubkeys[1],
            pool_state: pubkeys[2],
            reward_info: pubkeys[3],
            reward_vault: pubkeys[4],
            user_token_account: pubkeys[5],
            user_reward_info: pubkeys[6],
            reward_mint: pubkeys[7],
            token_program: pubkeys[8],
            token_program_2022: pubkeys[9],
            system_program: pubkeys[10],
        }
    }
}
impl<'info> From<ClaimRewardsAccounts<'_, 'info>> for [AccountInfo<'info>; CLAIM_REWARDS_IX_ACCOUNTS_LEN] {
    fn from(accounts: ClaimRewardsAccounts<'_, 'info>) -> Self {
        [
            accounts.user.clone(),
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.reward_info.clone(),
            accounts.reward_vault.clone(),
            accounts.user_token_account.clone(),
            accounts.user_reward_info.clone(),
            accounts.reward_mint.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CLAIM_REWARDS_IX_ACCOUNTS_LEN]> for ClaimRewardsAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CLAIM_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            user: &arr[0],
            authority: &arr[1],
            pool_state: &arr[2],
            reward_info: &arr[3],
            reward_vault: &arr[4],
            user_token_account: &arr[5],
            user_reward_info: &arr[6],
            reward_mint: &arr[7],
            token_program: &arr[8],
            token_program_2022: &arr[9],
            system_program: &arr[10],
        }
    }
}
pub const CLAIM_REWARDS_IX_DISCM: [u8; 8] = [4, 144, 132, 71, 116, 23, 151, 80];
#[derive(Clone, Debug, PartialEq)]
pub struct ClaimRewardsIxData;
impl ClaimRewardsIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CLAIM_REWARDS_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {CLAIM_REWARDS_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CLAIM_REWARDS_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn claim_rewards_ix_with_program_id(program_id: Pubkey, keys: ClaimRewardsKeys) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CLAIM_REWARDS_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: ClaimRewardsIxData.try_to_vec()?,
    })
}
pub fn claim_rewards_ix(keys: ClaimRewardsKeys) -> std::io::Result<Instruction> {
    claim_rewards_ix_with_program_id(super::ID, keys)
}

pub fn claim_rewards_verify_account_keys(
    accounts: ClaimRewardsAccounts<'_, '_>,
    keys: ClaimRewardsKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.user.key, keys.user),
        (*accounts.authority.key, keys.authority),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.reward_info.key, keys.reward_info),
        (*accounts.reward_vault.key, keys.reward_vault),
        (*accounts.user_token_account.key, keys.user_token_account),
        (*accounts.user_reward_info.key, keys.user_reward_info),
        (*accounts.reward_mint.key, keys.reward_mint),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn claim_rewards_verify_writable_privileges<'me, 'info>(
    accounts: ClaimRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.user,
        accounts.reward_vault,
        accounts.user_token_account,
        accounts.user_reward_info,
        accounts.reward_mint,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn claim_rewards_verify_signer_privileges<'me, 'info>(
    accounts: ClaimRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn claim_rewards_verify_account_privileges<'me, 'info>(
    accounts: ClaimRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    claim_rewards_verify_writable_privileges(accounts)?;
    claim_rewards_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const COLLECT_FUND_FEE_IX_ACCOUNTS_LEN: usize = 12;
#[derive(Copy, Clone, Debug)]
pub struct CollectFundFeeAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub token_0vault: &'me AccountInfo<'info>,
    pub token_1vault: &'me AccountInfo<'info>,
    pub vault_0mint: &'me AccountInfo<'info>,
    pub vault_1mint: &'me AccountInfo<'info>,
    pub recipient_token_0account: &'me AccountInfo<'info>,
    pub recipient_token_1account: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CollectFundFeeKeys {
    pub owner: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub amm_config: Pubkey,
    pub token_0vault: Pubkey,
    pub token_1vault: Pubkey,
    pub vault_0mint: Pubkey,
    pub vault_1mint: Pubkey,
    pub recipient_token_0account: Pubkey,
    pub recipient_token_1account: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
}
impl From<CollectFundFeeAccounts<'_, '_>> for CollectFundFeeKeys {
    fn from(accounts: CollectFundFeeAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            amm_config: *accounts.amm_config.key,
            token_0vault: *accounts.token_0vault.key,
            token_1vault: *accounts.token_1vault.key,
            vault_0mint: *accounts.vault_0mint.key,
            vault_1mint: *accounts.vault_1mint.key,
            recipient_token_0account: *accounts.recipient_token_0account.key,
            recipient_token_1account: *accounts.recipient_token_1account.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
        }
    }
}
impl From<CollectFundFeeKeys> for [AccountMeta; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN] {
    fn from(keys: CollectFundFeeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.recipient_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.recipient_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN]> for CollectFundFeeKeys {
    fn from(pubkeys: [Pubkey; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            authority: pubkeys[1],
            pool_state: pubkeys[2],
            amm_config: pubkeys[3],
            token_0vault: pubkeys[4],
            token_1vault: pubkeys[5],
            vault_0mint: pubkeys[6],
            vault_1mint: pubkeys[7],
            recipient_token_0account: pubkeys[8],
            recipient_token_1account: pubkeys[9],
            token_program: pubkeys[10],
            token_program_2022: pubkeys[11],
        }
    }
}
impl<'info> From<CollectFundFeeAccounts<'_, 'info>> for [AccountInfo<'info>; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN] {
    fn from(accounts: CollectFundFeeAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.amm_config.clone(),
            accounts.token_0vault.clone(),
            accounts.token_1vault.clone(),
            accounts.vault_0mint.clone(),
            accounts.vault_1mint.clone(),
            accounts.recipient_token_0account.clone(),
            accounts.recipient_token_1account.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN]>
    for CollectFundFeeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            authority: &arr[1],
            pool_state: &arr[2],
            amm_config: &arr[3],
            token_0vault: &arr[4],
            token_1vault: &arr[5],
            vault_0mint: &arr[6],
            vault_1mint: &arr[7],
            recipient_token_0account: &arr[8],
            recipient_token_1account: &arr[9],
            token_program: &arr[10],
            token_program_2022: &arr[11],
        }
    }
}
pub const COLLECT_FUND_FEE_IX_DISCM: [u8; 8] = [167, 138, 78, 149, 223, 194, 6, 126];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CollectFundFeeIxArgs {
    pub amount_0requested: u64,
    pub amount_1requested: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollectFundFeeIxData(pub CollectFundFeeIxArgs);
impl From<CollectFundFeeIxArgs> for CollectFundFeeIxData {
    fn from(args: CollectFundFeeIxArgs) -> Self {
        Self(args)
    }
}
impl CollectFundFeeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != COLLECT_FUND_FEE_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {COLLECT_FUND_FEE_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(CollectFundFeeIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&COLLECT_FUND_FEE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn collect_fund_fee_ix_with_program_id(
    program_id: Pubkey,
    keys: CollectFundFeeKeys,
    args: CollectFundFeeIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN] = keys.into();
    let data: CollectFundFeeIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn collect_fund_fee_ix(keys: CollectFundFeeKeys, args: CollectFundFeeIxArgs) -> std::io::Result<Instruction> {
    collect_fund_fee_ix_with_program_id(super::ID, keys, args)
}

pub fn collect_fund_fee_verify_account_keys(
    accounts: CollectFundFeeAccounts<'_, '_>,
    keys: CollectFundFeeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.owner.key, keys.owner),
        (*accounts.authority.key, keys.authority),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.amm_config.key, keys.amm_config),
        (*accounts.token_0vault.key, keys.token_0vault),
        (*accounts.token_1vault.key, keys.token_1vault),
        (*accounts.vault_0mint.key, keys.vault_0mint),
        (*accounts.vault_1mint.key, keys.vault_1mint),
        (*accounts.recipient_token_0account.key, keys.recipient_token_0account),
        (*accounts.recipient_token_1account.key, keys.recipient_token_1account),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn collect_fund_fee_verify_writable_privileges<'me, 'info>(
    accounts: CollectFundFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool_state,
        accounts.token_0vault,
        accounts.token_1vault,
        accounts.recipient_token_0account,
        accounts.recipient_token_1account,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn collect_fund_fee_verify_signer_privileges<'me, 'info>(
    accounts: CollectFundFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn collect_fund_fee_verify_account_privileges<'me, 'info>(
    accounts: CollectFundFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    collect_fund_fee_verify_writable_privileges(accounts)?;
    collect_fund_fee_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN: usize = 12;
#[derive(Copy, Clone, Debug)]
pub struct CollectProtocolFeeAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub token_0vault: &'me AccountInfo<'info>,
    pub token_1vault: &'me AccountInfo<'info>,
    pub vault_0mint: &'me AccountInfo<'info>,
    pub vault_1mint: &'me AccountInfo<'info>,
    pub recipient_token_0account: &'me AccountInfo<'info>,
    pub recipient_token_1account: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CollectProtocolFeeKeys {
    pub owner: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub amm_config: Pubkey,
    pub token_0vault: Pubkey,
    pub token_1vault: Pubkey,
    pub vault_0mint: Pubkey,
    pub vault_1mint: Pubkey,
    pub recipient_token_0account: Pubkey,
    pub recipient_token_1account: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
}
impl From<CollectProtocolFeeAccounts<'_, '_>> for CollectProtocolFeeKeys {
    fn from(accounts: CollectProtocolFeeAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            amm_config: *accounts.amm_config.key,
            token_0vault: *accounts.token_0vault.key,
            token_1vault: *accounts.token_1vault.key,
            vault_0mint: *accounts.vault_0mint.key,
            vault_1mint: *accounts.vault_1mint.key,
            recipient_token_0account: *accounts.recipient_token_0account.key,
            recipient_token_1account: *accounts.recipient_token_1account.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
        }
    }
}
impl From<CollectProtocolFeeKeys> for [AccountMeta; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN] {
    fn from(keys: CollectProtocolFeeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.recipient_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.recipient_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]> for CollectProtocolFeeKeys {
    fn from(pubkeys: [Pubkey; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            authority: pubkeys[1],
            pool_state: pubkeys[2],
            amm_config: pubkeys[3],
            token_0vault: pubkeys[4],
            token_1vault: pubkeys[5],
            vault_0mint: pubkeys[6],
            vault_1mint: pubkeys[7],
            recipient_token_0account: pubkeys[8],
            recipient_token_1account: pubkeys[9],
            token_program: pubkeys[10],
            token_program_2022: pubkeys[11],
        }
    }
}
impl<'info> From<CollectProtocolFeeAccounts<'_, 'info>> for [AccountInfo<'info>; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN] {
    fn from(accounts: CollectProtocolFeeAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.amm_config.clone(),
            accounts.token_0vault.clone(),
            accounts.token_1vault.clone(),
            accounts.vault_0mint.clone(),
            accounts.vault_1mint.clone(),
            accounts.recipient_token_0account.clone(),
            accounts.recipient_token_1account.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]>
    for CollectProtocolFeeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            authority: &arr[1],
            pool_state: &arr[2],
            amm_config: &arr[3],
            token_0vault: &arr[4],
            token_1vault: &arr[5],
            vault_0mint: &arr[6],
            vault_1mint: &arr[7],
            recipient_token_0account: &arr[8],
            recipient_token_1account: &arr[9],
            token_program: &arr[10],
            token_program_2022: &arr[11],
        }
    }
}
pub const COLLECT_PROTOCOL_FEE_IX_DISCM: [u8; 8] = [136, 136, 252, 221, 194, 66, 126, 89];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CollectProtocolFeeIxArgs {
    pub amount_0requested: u64,
    pub amount_1requested: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollectProtocolFeeIxData(pub CollectProtocolFeeIxArgs);
impl From<CollectProtocolFeeIxArgs> for CollectProtocolFeeIxData {
    fn from(args: CollectProtocolFeeIxArgs) -> Self {
        Self(args)
    }
}
impl CollectProtocolFeeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != COLLECT_PROTOCOL_FEE_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {COLLECT_PROTOCOL_FEE_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(CollectProtocolFeeIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&COLLECT_PROTOCOL_FEE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn collect_protocol_fee_ix_with_program_id(
    program_id: Pubkey,
    keys: CollectProtocolFeeKeys,
    args: CollectProtocolFeeIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN] = keys.into();
    let data: CollectProtocolFeeIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn collect_protocol_fee_ix(
    keys: CollectProtocolFeeKeys,
    args: CollectProtocolFeeIxArgs,
) -> std::io::Result<Instruction> {
    collect_protocol_fee_ix_with_program_id(super::ID, keys, args)
}

pub fn collect_protocol_fee_verify_account_keys(
    accounts: CollectProtocolFeeAccounts<'_, '_>,
    keys: CollectProtocolFeeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.owner.key, keys.owner),
        (*accounts.authority.key, keys.authority),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.amm_config.key, keys.amm_config),
        (*accounts.token_0vault.key, keys.token_0vault),
        (*accounts.token_1vault.key, keys.token_1vault),
        (*accounts.vault_0mint.key, keys.vault_0mint),
        (*accounts.vault_1mint.key, keys.vault_1mint),
        (*accounts.recipient_token_0account.key, keys.recipient_token_0account),
        (*accounts.recipient_token_1account.key, keys.recipient_token_1account),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn collect_protocol_fee_verify_writable_privileges<'me, 'info>(
    accounts: CollectProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool_state,
        accounts.token_0vault,
        accounts.token_1vault,
        accounts.recipient_token_0account,
        accounts.recipient_token_1account,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn collect_protocol_fee_verify_signer_privileges<'me, 'info>(
    accounts: CollectProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn collect_protocol_fee_verify_account_privileges<'me, 'info>(
    accounts: CollectProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    collect_protocol_fee_verify_writable_privileges(accounts)?;
    collect_protocol_fee_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct CreateAmmConfigAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreateAmmConfigKeys {
    pub owner: Pubkey,
    pub amm_config: Pubkey,
    pub system_program: Pubkey,
}
impl From<CreateAmmConfigAccounts<'_, '_>> for CreateAmmConfigKeys {
    fn from(accounts: CreateAmmConfigAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            amm_config: *accounts.amm_config.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<CreateAmmConfigKeys> for [AccountMeta; CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN] {
    fn from(keys: CreateAmmConfigKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN]> for CreateAmmConfigKeys {
    fn from(pubkeys: [Pubkey; CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            amm_config: pubkeys[1],
            system_program: pubkeys[2],
        }
    }
}
impl<'info> From<CreateAmmConfigAccounts<'_, 'info>> for [AccountInfo<'info>; CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN] {
    fn from(accounts: CreateAmmConfigAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.amm_config.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN]>
    for CreateAmmConfigAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            amm_config: &arr[1],
            system_program: &arr[2],
        }
    }
}
pub const CREATE_AMM_CONFIG_IX_DISCM: [u8; 8] = [137, 52, 237, 212, 215, 117, 108, 104];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CreateAmmConfigIxArgs {
    pub index: u16,
    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub create_pool_fee: u64,
    pub max_open_time: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreateAmmConfigIxData(pub CreateAmmConfigIxArgs);
impl From<CreateAmmConfigIxArgs> for CreateAmmConfigIxData {
    fn from(args: CreateAmmConfigIxArgs) -> Self {
        Self(args)
    }
}
impl CreateAmmConfigIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CREATE_AMM_CONFIG_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {CREATE_AMM_CONFIG_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(CreateAmmConfigIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CREATE_AMM_CONFIG_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn create_amm_config_ix_with_program_id(
    program_id: Pubkey,
    keys: CreateAmmConfigKeys,
    args: CreateAmmConfigIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CREATE_AMM_CONFIG_IX_ACCOUNTS_LEN] = keys.into();
    let data: CreateAmmConfigIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn create_amm_config_ix(keys: CreateAmmConfigKeys, args: CreateAmmConfigIxArgs) -> std::io::Result<Instruction> {
    create_amm_config_ix_with_program_id(super::ID, keys, args)
}

pub fn create_amm_config_verify_account_keys(
    accounts: CreateAmmConfigAccounts<'_, '_>,
    keys: CreateAmmConfigKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.owner.key, keys.owner),
        (*accounts.amm_config.key, keys.amm_config),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn create_amm_config_verify_writable_privileges<'me, 'info>(
    accounts: CreateAmmConfigAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.owner, accounts.amm_config] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn create_amm_config_verify_signer_privileges<'me, 'info>(
    accounts: CreateAmmConfigAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn create_amm_config_verify_account_privileges<'me, 'info>(
    accounts: CreateAmmConfigAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    create_amm_config_verify_writable_privileges(accounts)?;
    create_amm_config_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CREATE_REWARDS_IX_ACCOUNTS_LEN: usize = 10;
#[derive(Copy, Clone, Debug)]
pub struct CreateRewardsAccounts<'me, 'info> {
    pub reward_provider: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub reward_info: &'me AccountInfo<'info>,
    pub reward_providers_token_account: &'me AccountInfo<'info>,
    pub reward_vault: &'me AccountInfo<'info>,
    pub reward_mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreateRewardsKeys {
    pub reward_provider: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub reward_info: Pubkey,
    pub reward_providers_token_account: Pubkey,
    pub reward_vault: Pubkey,
    pub reward_mint: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub system_program: Pubkey,
}
impl From<CreateRewardsAccounts<'_, '_>> for CreateRewardsKeys {
    fn from(accounts: CreateRewardsAccounts) -> Self {
        Self {
            reward_provider: *accounts.reward_provider.key,
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            reward_info: *accounts.reward_info.key,
            reward_providers_token_account: *accounts.reward_providers_token_account.key,
            reward_vault: *accounts.reward_vault.key,
            reward_mint: *accounts.reward_mint.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<CreateRewardsKeys> for [AccountMeta; CREATE_REWARDS_IX_ACCOUNTS_LEN] {
    fn from(keys: CreateRewardsKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.reward_provider,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reward_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reward_providers_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reward_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reward_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CREATE_REWARDS_IX_ACCOUNTS_LEN]> for CreateRewardsKeys {
    fn from(pubkeys: [Pubkey; CREATE_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            reward_provider: pubkeys[0],
            authority: pubkeys[1],
            pool_state: pubkeys[2],
            reward_info: pubkeys[3],
            reward_providers_token_account: pubkeys[4],
            reward_vault: pubkeys[5],
            reward_mint: pubkeys[6],
            token_program: pubkeys[7],
            token_program_2022: pubkeys[8],
            system_program: pubkeys[9],
        }
    }
}
impl<'info> From<CreateRewardsAccounts<'_, 'info>> for [AccountInfo<'info>; CREATE_REWARDS_IX_ACCOUNTS_LEN] {
    fn from(accounts: CreateRewardsAccounts<'_, 'info>) -> Self {
        [
            accounts.reward_provider.clone(),
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.reward_info.clone(),
            accounts.reward_providers_token_account.clone(),
            accounts.reward_vault.clone(),
            accounts.reward_mint.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_REWARDS_IX_ACCOUNTS_LEN]> for CreateRewardsAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CREATE_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            reward_provider: &arr[0],
            authority: &arr[1],
            pool_state: &arr[2],
            reward_info: &arr[3],
            reward_providers_token_account: &arr[4],
            reward_vault: &arr[5],
            reward_mint: &arr[6],
            token_program: &arr[7],
            token_program_2022: &arr[8],
            system_program: &arr[9],
        }
    }
}
pub const CREATE_REWARDS_IX_DISCM: [u8; 8] = [124, 251, 145, 232, 5, 173, 159, 223];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CreateRewardsIxArgs {
    pub start_time: u64,
    pub end_time: u64,
    pub reward_amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreateRewardsIxData(pub CreateRewardsIxArgs);
impl From<CreateRewardsIxArgs> for CreateRewardsIxData {
    fn from(args: CreateRewardsIxArgs) -> Self {
        Self(args)
    }
}
impl CreateRewardsIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CREATE_REWARDS_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {CREATE_REWARDS_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(CreateRewardsIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CREATE_REWARDS_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn create_rewards_ix_with_program_id(
    program_id: Pubkey,
    keys: CreateRewardsKeys,
    args: CreateRewardsIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CREATE_REWARDS_IX_ACCOUNTS_LEN] = keys.into();
    let data: CreateRewardsIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn create_rewards_ix(keys: CreateRewardsKeys, args: CreateRewardsIxArgs) -> std::io::Result<Instruction> {
    create_rewards_ix_with_program_id(super::ID, keys, args)
}

pub fn create_rewards_verify_account_keys(
    accounts: CreateRewardsAccounts<'_, '_>,
    keys: CreateRewardsKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.reward_provider.key, keys.reward_provider),
        (*accounts.authority.key, keys.authority),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.reward_info.key, keys.reward_info),
        (
            *accounts.reward_providers_token_account.key,
            keys.reward_providers_token_account,
        ),
        (*accounts.reward_vault.key, keys.reward_vault),
        (*accounts.reward_mint.key, keys.reward_mint),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn create_rewards_verify_writable_privileges<'me, 'info>(
    accounts: CreateRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.reward_provider,
        accounts.reward_info,
        accounts.reward_providers_token_account,
        accounts.reward_vault,
        accounts.reward_mint,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn create_rewards_verify_signer_privileges<'me, 'info>(
    accounts: CreateRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.reward_provider] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn create_rewards_verify_account_privileges<'me, 'info>(
    accounts: CreateRewardsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    create_rewards_verify_writable_privileges(accounts)?;
    create_rewards_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN: usize = 7;
#[derive(Copy, Clone, Debug)]
pub struct CreateSwapReferralAccounts<'me, 'info> {
    pub admin: &'me AccountInfo<'info>,
    pub owner: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub project: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub referral_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreateSwapReferralKeys {
    pub admin: Pubkey,
    pub owner: Pubkey,
    pub payer: Pubkey,
    pub amm_config: Pubkey,
    pub project: Pubkey,
    pub system_program: Pubkey,
    pub referral_program: Pubkey,
}
impl From<CreateSwapReferralAccounts<'_, '_>> for CreateSwapReferralKeys {
    fn from(accounts: CreateSwapReferralAccounts) -> Self {
        Self {
            admin: *accounts.admin.key,
            owner: *accounts.owner.key,
            payer: *accounts.payer.key,
            amm_config: *accounts.amm_config.key,
            project: *accounts.project.key,
            system_program: *accounts.system_program.key,
            referral_program: *accounts.referral_program.key,
        }
    }
}
impl From<CreateSwapReferralKeys> for [AccountMeta; CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN] {
    fn from(keys: CreateSwapReferralKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.admin,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.owner,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.project,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.referral_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN]> for CreateSwapReferralKeys {
    fn from(pubkeys: [Pubkey; CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            admin: pubkeys[0],
            owner: pubkeys[1],
            payer: pubkeys[2],
            amm_config: pubkeys[3],
            project: pubkeys[4],
            system_program: pubkeys[5],
            referral_program: pubkeys[6],
        }
    }
}
impl<'info> From<CreateSwapReferralAccounts<'_, 'info>> for [AccountInfo<'info>; CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN] {
    fn from(accounts: CreateSwapReferralAccounts<'_, 'info>) -> Self {
        [
            accounts.admin.clone(),
            accounts.owner.clone(),
            accounts.payer.clone(),
            accounts.amm_config.clone(),
            accounts.project.clone(),
            accounts.system_program.clone(),
            accounts.referral_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN]>
    for CreateSwapReferralAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            admin: &arr[0],
            owner: &arr[1],
            payer: &arr[2],
            amm_config: &arr[3],
            project: &arr[4],
            system_program: &arr[5],
            referral_program: &arr[6],
        }
    }
}
pub const CREATE_SWAP_REFERRAL_IX_DISCM: [u8; 8] = [67, 131, 93, 236, 56, 6, 40, 77];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CreateSwapReferralIxArgs {
    pub name: String,
    pub default_share_bps: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreateSwapReferralIxData(pub CreateSwapReferralIxArgs);
impl From<CreateSwapReferralIxArgs> for CreateSwapReferralIxData {
    fn from(args: CreateSwapReferralIxArgs) -> Self {
        Self(args)
    }
}
impl CreateSwapReferralIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CREATE_SWAP_REFERRAL_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {CREATE_SWAP_REFERRAL_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(CreateSwapReferralIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CREATE_SWAP_REFERRAL_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn create_swap_referral_ix_with_program_id(
    program_id: Pubkey,
    keys: CreateSwapReferralKeys,
    args: CreateSwapReferralIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CREATE_SWAP_REFERRAL_IX_ACCOUNTS_LEN] = keys.into();
    let data: CreateSwapReferralIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn create_swap_referral_ix(
    keys: CreateSwapReferralKeys,
    args: CreateSwapReferralIxArgs,
) -> std::io::Result<Instruction> {
    create_swap_referral_ix_with_program_id(super::ID, keys, args)
}

pub fn create_swap_referral_verify_account_keys(
    accounts: CreateSwapReferralAccounts<'_, '_>,
    keys: CreateSwapReferralKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.admin.key, keys.admin),
        (*accounts.owner.key, keys.owner),
        (*accounts.payer.key, keys.payer),
        (*accounts.amm_config.key, keys.amm_config),
        (*accounts.project.key, keys.project),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.referral_program.key, keys.referral_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn create_swap_referral_verify_writable_privileges<'me, 'info>(
    accounts: CreateSwapReferralAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.payer, accounts.amm_config, accounts.project] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn create_swap_referral_verify_signer_privileges<'me, 'info>(
    accounts: CreateSwapReferralAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.admin, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn create_swap_referral_verify_account_privileges<'me, 'info>(
    accounts: CreateSwapReferralAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    create_swap_referral_verify_writable_privileges(accounts)?;
    create_swap_referral_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const DEPOSIT_IX_ACCOUNTS_LEN: usize = 12;
#[derive(Copy, Clone, Debug)]
pub struct DepositAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub user_pool_liquidity: &'me AccountInfo<'info>,
    pub token_0account: &'me AccountInfo<'info>,
    pub token_1account: &'me AccountInfo<'info>,
    pub token_0vault: &'me AccountInfo<'info>,
    pub token_1vault: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub vault_0mint: &'me AccountInfo<'info>,
    pub vault_1mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DepositKeys {
    pub owner: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub user_pool_liquidity: Pubkey,
    pub token_0account: Pubkey,
    pub token_1account: Pubkey,
    pub token_0vault: Pubkey,
    pub token_1vault: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub vault_0mint: Pubkey,
    pub vault_1mint: Pubkey,
}
impl From<DepositAccounts<'_, '_>> for DepositKeys {
    fn from(accounts: DepositAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            user_pool_liquidity: *accounts.user_pool_liquidity.key,
            token_0account: *accounts.token_0account.key,
            token_1account: *accounts.token_1account.key,
            token_0vault: *accounts.token_0vault.key,
            token_1vault: *accounts.token_1vault.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            vault_0mint: *accounts.vault_0mint.key,
            vault_1mint: *accounts.vault_1mint.key,
        }
    }
}
impl From<DepositKeys> for [AccountMeta; DEPOSIT_IX_ACCOUNTS_LEN] {
    fn from(keys: DepositKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_1mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; DEPOSIT_IX_ACCOUNTS_LEN]> for DepositKeys {
    fn from(pubkeys: [Pubkey; DEPOSIT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            authority: pubkeys[1],
            pool_state: pubkeys[2],
            user_pool_liquidity: pubkeys[3],
            token_0account: pubkeys[4],
            token_1account: pubkeys[5],
            token_0vault: pubkeys[6],
            token_1vault: pubkeys[7],
            token_program: pubkeys[8],
            token_program_2022: pubkeys[9],
            vault_0mint: pubkeys[10],
            vault_1mint: pubkeys[11],
        }
    }
}
impl<'info> From<DepositAccounts<'_, 'info>> for [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN] {
    fn from(accounts: DepositAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.user_pool_liquidity.clone(),
            accounts.token_0account.clone(),
            accounts.token_1account.clone(),
            accounts.token_0vault.clone(),
            accounts.token_1vault.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.vault_0mint.clone(),
            accounts.vault_1mint.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN]> for DepositAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            authority: &arr[1],
            pool_state: &arr[2],
            user_pool_liquidity: &arr[3],
            token_0account: &arr[4],
            token_1account: &arr[5],
            token_0vault: &arr[6],
            token_1vault: &arr[7],
            token_program: &arr[8],
            token_program_2022: &arr[9],
            vault_0mint: &arr[10],
            vault_1mint: &arr[11],
        }
    }
}
pub const DEPOSIT_IX_DISCM: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct DepositIxArgs {
    pub lp_token_amount: u64,
    pub maximum_token_0amount: u64,
    pub maximum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DepositIxData(pub DepositIxArgs);
impl From<DepositIxArgs> for DepositIxData {
    fn from(args: DepositIxArgs) -> Self {
        Self(args)
    }
}
impl DepositIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != DEPOSIT_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {DEPOSIT_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(DepositIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&DEPOSIT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn deposit_ix_with_program_id(
    program_id: Pubkey,
    keys: DepositKeys,
    args: DepositIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; DEPOSIT_IX_ACCOUNTS_LEN] = keys.into();
    let data: DepositIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn deposit_ix(keys: DepositKeys, args: DepositIxArgs) -> std::io::Result<Instruction> {
    deposit_ix_with_program_id(super::ID, keys, args)
}

pub fn deposit_verify_account_keys(
    accounts: DepositAccounts<'_, '_>,
    keys: DepositKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.owner.key, keys.owner),
        (*accounts.authority.key, keys.authority),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.user_pool_liquidity.key, keys.user_pool_liquidity),
        (*accounts.token_0account.key, keys.token_0account),
        (*accounts.token_1account.key, keys.token_1account),
        (*accounts.token_0vault.key, keys.token_0vault),
        (*accounts.token_1vault.key, keys.token_1vault),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
        (*accounts.vault_0mint.key, keys.vault_0mint),
        (*accounts.vault_1mint.key, keys.vault_1mint),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn deposit_verify_writable_privileges<'me, 'info>(
    accounts: DepositAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool_state,
        accounts.user_pool_liquidity,
        accounts.token_0account,
        accounts.token_1account,
        accounts.token_0vault,
        accounts.token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn deposit_verify_signer_privileges<'me, 'info>(
    accounts: DepositAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn deposit_verify_account_privileges<'me, 'info>(
    accounts: DepositAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    deposit_verify_writable_privileges(accounts)?;
    deposit_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN: usize = 4;
#[derive(Copy, Clone, Debug)]
pub struct InitUserPoolLiquidityAccounts<'me, 'info> {
    pub user: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub user_pool_liquidity: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InitUserPoolLiquidityKeys {
    pub user: Pubkey,
    pub pool_state: Pubkey,
    pub user_pool_liquidity: Pubkey,
    pub system_program: Pubkey,
}
impl From<InitUserPoolLiquidityAccounts<'_, '_>> for InitUserPoolLiquidityKeys {
    fn from(accounts: InitUserPoolLiquidityAccounts) -> Self {
        Self {
            user: *accounts.user.key,
            pool_state: *accounts.pool_state.key,
            user_pool_liquidity: *accounts.user_pool_liquidity.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<InitUserPoolLiquidityKeys> for [AccountMeta; INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN] {
    fn from(keys: InitUserPoolLiquidityKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.user,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN]> for InitUserPoolLiquidityKeys {
    fn from(pubkeys: [Pubkey; INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            user: pubkeys[0],
            pool_state: pubkeys[1],
            user_pool_liquidity: pubkeys[2],
            system_program: pubkeys[3],
        }
    }
}
impl<'info> From<InitUserPoolLiquidityAccounts<'_, 'info>>
    for [AccountInfo<'info>; INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN]
{
    fn from(accounts: InitUserPoolLiquidityAccounts<'_, 'info>) -> Self {
        [
            accounts.user.clone(),
            accounts.pool_state.clone(),
            accounts.user_pool_liquidity.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN]>
    for InitUserPoolLiquidityAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            user: &arr[0],
            pool_state: &arr[1],
            user_pool_liquidity: &arr[2],
            system_program: &arr[3],
        }
    }
}
pub const INIT_USER_POOL_LIQUIDITY_IX_DISCM: [u8; 8] = [227, 221, 200, 212, 36, 107, 149, 36];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct InitUserPoolLiquidityIxArgs {
    pub partner: Option<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct InitUserPoolLiquidityIxData(pub InitUserPoolLiquidityIxArgs);
impl From<InitUserPoolLiquidityIxArgs> for InitUserPoolLiquidityIxData {
    fn from(args: InitUserPoolLiquidityIxArgs) -> Self {
        Self(args)
    }
}
impl InitUserPoolLiquidityIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != INIT_USER_POOL_LIQUIDITY_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {INIT_USER_POOL_LIQUIDITY_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(InitUserPoolLiquidityIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&INIT_USER_POOL_LIQUIDITY_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn init_user_pool_liquidity_ix_with_program_id(
    program_id: Pubkey,
    keys: InitUserPoolLiquidityKeys,
    args: InitUserPoolLiquidityIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; INIT_USER_POOL_LIQUIDITY_IX_ACCOUNTS_LEN] = keys.into();
    let data: InitUserPoolLiquidityIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn init_user_pool_liquidity_ix(
    keys: InitUserPoolLiquidityKeys,
    args: InitUserPoolLiquidityIxArgs,
) -> std::io::Result<Instruction> {
    init_user_pool_liquidity_ix_with_program_id(super::ID, keys, args)
}
pub fn init_user_pool_liquidity_verify_account_keys(
    accounts: InitUserPoolLiquidityAccounts<'_, '_>,
    keys: InitUserPoolLiquidityKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.user.key, keys.user),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.user_pool_liquidity.key, keys.user_pool_liquidity),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn init_user_pool_liquidity_verify_writable_privileges<'me, 'info>(
    accounts: InitUserPoolLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.user, accounts.user_pool_liquidity] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn init_user_pool_liquidity_verify_signer_privileges<'me, 'info>(
    accounts: InitUserPoolLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn init_user_pool_liquidity_verify_account_privileges<'me, 'info>(
    accounts: InitUserPoolLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    init_user_pool_liquidity_verify_writable_privileges(accounts)?;
    init_user_pool_liquidity_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const INITIALIZE_IX_ACCOUNTS_LEN: usize = 19;
#[derive(Copy, Clone, Debug)]
pub struct InitializeAccounts<'me, 'info> {
    pub creator: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub user_pool_liquidity: &'me AccountInfo<'info>,
    pub token_0mint: &'me AccountInfo<'info>,
    pub token_1mint: &'me AccountInfo<'info>,
    pub creator_token_0: &'me AccountInfo<'info>,
    pub creator_token_1: &'me AccountInfo<'info>,
    pub token_0vault: &'me AccountInfo<'info>,
    pub token_1vault: &'me AccountInfo<'info>,
    pub create_pool_fee: &'me AccountInfo<'info>,
    pub observation_state: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_0program: &'me AccountInfo<'info>,
    pub token_1program: &'me AccountInfo<'info>,
    pub associated_token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InitializeKeys {
    pub creator: Pubkey,
    pub amm_config: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub user_pool_liquidity: Pubkey,
    pub token_0mint: Pubkey,
    pub token_1mint: Pubkey,
    pub creator_token_0: Pubkey,
    pub creator_token_1: Pubkey,
    pub token_0vault: Pubkey,
    pub token_1vault: Pubkey,
    pub create_pool_fee: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub token_0program: Pubkey,
    pub token_1program: Pubkey,
    pub associated_token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<InitializeAccounts<'_, '_>> for InitializeKeys {
    fn from(accounts: InitializeAccounts) -> Self {
        Self {
            creator: *accounts.creator.key,
            amm_config: *accounts.amm_config.key,
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            user_pool_liquidity: *accounts.user_pool_liquidity.key,
            token_0mint: *accounts.token_0mint.key,
            token_1mint: *accounts.token_1mint.key,
            creator_token_0: *accounts.creator_token_0.key,
            creator_token_1: *accounts.creator_token_1.key,
            token_0vault: *accounts.token_0vault.key,
            token_1vault: *accounts.token_1vault.key,
            create_pool_fee: *accounts.create_pool_fee.key,
            observation_state: *accounts.observation_state.key,
            token_program: *accounts.token_program.key,
            token_0program: *accounts.token_0program.key,
            token_1program: *accounts.token_1program.key,
            associated_token_program: *accounts.associated_token_program.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<InitializeKeys> for [AccountMeta; INITIALIZE_IX_ACCOUNTS_LEN] {
    fn from(keys: InitializeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.creator,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.creator_token_0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.creator_token_1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.create_pool_fee,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.observation_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_0program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_1program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.associated_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; INITIALIZE_IX_ACCOUNTS_LEN]> for InitializeKeys {
    fn from(pubkeys: [Pubkey; INITIALIZE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            creator: pubkeys[0],
            amm_config: pubkeys[1],
            authority: pubkeys[2],
            pool_state: pubkeys[3],
            user_pool_liquidity: pubkeys[4],
            token_0mint: pubkeys[5],
            token_1mint: pubkeys[6],
            creator_token_0: pubkeys[7],
            creator_token_1: pubkeys[8],
            token_0vault: pubkeys[9],
            token_1vault: pubkeys[10],
            create_pool_fee: pubkeys[11],
            observation_state: pubkeys[12],
            token_program: pubkeys[13],
            token_0program: pubkeys[14],
            token_1program: pubkeys[15],
            associated_token_program: pubkeys[16],
            system_program: pubkeys[17],
            rent: pubkeys[18],
        }
    }
}
impl<'info> From<InitializeAccounts<'_, 'info>> for [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN] {
    fn from(accounts: InitializeAccounts<'_, 'info>) -> Self {
        [
            accounts.creator.clone(),
            accounts.amm_config.clone(),
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.user_pool_liquidity.clone(),
            accounts.token_0mint.clone(),
            accounts.token_1mint.clone(),
            accounts.creator_token_0.clone(),
            accounts.creator_token_1.clone(),
            accounts.token_0vault.clone(),
            accounts.token_1vault.clone(),
            accounts.create_pool_fee.clone(),
            accounts.observation_state.clone(),
            accounts.token_program.clone(),
            accounts.token_0program.clone(),
            accounts.token_1program.clone(),
            accounts.associated_token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN]> for InitializeAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            creator: &arr[0],
            amm_config: &arr[1],
            authority: &arr[2],
            pool_state: &arr[3],
            user_pool_liquidity: &arr[4],
            token_0mint: &arr[5],
            token_1mint: &arr[6],
            creator_token_0: &arr[7],
            creator_token_1: &arr[8],
            token_0vault: &arr[9],
            token_1vault: &arr[10],
            create_pool_fee: &arr[11],
            observation_state: &arr[12],
            token_program: &arr[13],
            token_0program: &arr[14],
            token_1program: &arr[15],
            associated_token_program: &arr[16],
            system_program: &arr[17],
            rent: &arr[18],
        }
    }
}
pub const INITIALIZE_IX_DISCM: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct InitializeIxArgs {
    pub init_amount_0: u64,
    pub init_amount_1: u64,
    pub open_time: u64,
    pub max_trade_fee_rate: u64,
    pub volatility_factor: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct InitializeIxData(pub InitializeIxArgs);
impl From<InitializeIxArgs> for InitializeIxData {
    fn from(args: InitializeIxArgs) -> Self {
        Self(args)
    }
}
impl InitializeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != INITIALIZE_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {INITIALIZE_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(InitializeIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&INITIALIZE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn initialize_ix_with_program_id(
    program_id: Pubkey,
    keys: InitializeKeys,
    args: InitializeIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; INITIALIZE_IX_ACCOUNTS_LEN] = keys.into();
    let data: InitializeIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn initialize_ix(keys: InitializeKeys, args: InitializeIxArgs) -> std::io::Result<Instruction> {
    initialize_ix_with_program_id(super::ID, keys, args)
}

pub fn initialize_verify_account_keys(
    accounts: InitializeAccounts<'_, '_>,
    keys: InitializeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.creator.key, keys.creator),
        (*accounts.amm_config.key, keys.amm_config),
        (*accounts.authority.key, keys.authority),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.user_pool_liquidity.key, keys.user_pool_liquidity),
        (*accounts.token_0mint.key, keys.token_0mint),
        (*accounts.token_1mint.key, keys.token_1mint),
        (*accounts.creator_token_0.key, keys.creator_token_0),
        (*accounts.creator_token_1.key, keys.creator_token_1),
        (*accounts.token_0vault.key, keys.token_0vault),
        (*accounts.token_1vault.key, keys.token_1vault),
        (*accounts.create_pool_fee.key, keys.create_pool_fee),
        (*accounts.observation_state.key, keys.observation_state),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_0program.key, keys.token_0program),
        (*accounts.token_1program.key, keys.token_1program),
        (*accounts.associated_token_program.key, keys.associated_token_program),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.rent.key, keys.rent),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn initialize_verify_writable_privileges<'me, 'info>(
    accounts: InitializeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.creator,
        accounts.pool_state,
        accounts.user_pool_liquidity,
        accounts.creator_token_0,
        accounts.creator_token_1,
        accounts.token_0vault,
        accounts.token_1vault,
        accounts.create_pool_fee,
        accounts.observation_state,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn initialize_verify_signer_privileges<'me, 'info>(
    accounts: InitializeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.creator] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn initialize_verify_account_privileges<'me, 'info>(
    accounts: InitializeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    initialize_verify_writable_privileges(accounts)?;
    initialize_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN: usize = 23;
#[derive(Copy, Clone, Debug)]
pub struct MigrateMeteoraDlmmToGammaAccounts<'me, 'info> {
    pub dlmm_position: &'me AccountInfo<'info>,
    pub dlmm_lb_pair: &'me AccountInfo<'info>,
    pub dlmm_bin_array_bitmap_extension: &'me AccountInfo<'info>,
    pub dlmm_reserve_x: &'me AccountInfo<'info>,
    pub dlmm_reserve_y: &'me AccountInfo<'info>,
    pub dlmm_bin_array_lower: &'me AccountInfo<'info>,
    pub dlmm_bin_array_upper: &'me AccountInfo<'info>,
    pub dlmm_program: &'me AccountInfo<'info>,
    pub dlmm_event_authority: &'me AccountInfo<'info>,
    pub token_x_program: &'me AccountInfo<'info>,
    pub token_y_program: &'me AccountInfo<'info>,
    pub gamma_owner: &'me AccountInfo<'info>,
    pub gamma_authority: &'me AccountInfo<'info>,
    pub gamma_pool_state: &'me AccountInfo<'info>,
    pub gamma_user_pool_liquidity: &'me AccountInfo<'info>,
    pub gamma_token_0account: &'me AccountInfo<'info>,
    pub gamma_token_1account: &'me AccountInfo<'info>,
    pub gamma_token_0vault: &'me AccountInfo<'info>,
    pub gamma_token_1vault: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub gamma_vault_0mint: &'me AccountInfo<'info>,
    pub gamma_vault_1mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MigrateMeteoraDlmmToGammaKeys {
    pub dlmm_position: Pubkey,
    pub dlmm_lb_pair: Pubkey,
    pub dlmm_bin_array_bitmap_extension: Pubkey,
    pub dlmm_reserve_x: Pubkey,
    pub dlmm_reserve_y: Pubkey,
    pub dlmm_bin_array_lower: Pubkey,
    pub dlmm_bin_array_upper: Pubkey,
    pub dlmm_program: Pubkey,
    pub dlmm_event_authority: Pubkey,
    pub token_x_program: Pubkey,
    pub token_y_program: Pubkey,
    pub gamma_owner: Pubkey,
    pub gamma_authority: Pubkey,
    pub gamma_pool_state: Pubkey,
    pub gamma_user_pool_liquidity: Pubkey,
    pub gamma_token_0account: Pubkey,
    pub gamma_token_1account: Pubkey,
    pub gamma_token_0vault: Pubkey,
    pub gamma_token_1vault: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub gamma_vault_0mint: Pubkey,
    pub gamma_vault_1mint: Pubkey,
}
impl From<MigrateMeteoraDlmmToGammaAccounts<'_, '_>> for MigrateMeteoraDlmmToGammaKeys {
    fn from(accounts: MigrateMeteoraDlmmToGammaAccounts) -> Self {
        Self {
            dlmm_position: *accounts.dlmm_position.key,
            dlmm_lb_pair: *accounts.dlmm_lb_pair.key,
            dlmm_bin_array_bitmap_extension: *accounts.dlmm_bin_array_bitmap_extension.key,
            dlmm_reserve_x: *accounts.dlmm_reserve_x.key,
            dlmm_reserve_y: *accounts.dlmm_reserve_y.key,
            dlmm_bin_array_lower: *accounts.dlmm_bin_array_lower.key,
            dlmm_bin_array_upper: *accounts.dlmm_bin_array_upper.key,
            dlmm_program: *accounts.dlmm_program.key,
            dlmm_event_authority: *accounts.dlmm_event_authority.key,
            token_x_program: *accounts.token_x_program.key,
            token_y_program: *accounts.token_y_program.key,
            gamma_owner: *accounts.gamma_owner.key,
            gamma_authority: *accounts.gamma_authority.key,
            gamma_pool_state: *accounts.gamma_pool_state.key,
            gamma_user_pool_liquidity: *accounts.gamma_user_pool_liquidity.key,
            gamma_token_0account: *accounts.gamma_token_0account.key,
            gamma_token_1account: *accounts.gamma_token_1account.key,
            gamma_token_0vault: *accounts.gamma_token_0vault.key,
            gamma_token_1vault: *accounts.gamma_token_1vault.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            gamma_vault_0mint: *accounts.gamma_vault_0mint.key,
            gamma_vault_1mint: *accounts.gamma_vault_1mint.key,
        }
    }
}
impl From<MigrateMeteoraDlmmToGammaKeys> for [AccountMeta; MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN] {
    fn from(keys: MigrateMeteoraDlmmToGammaKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.dlmm_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dlmm_lb_pair,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dlmm_bin_array_bitmap_extension,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dlmm_reserve_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dlmm_reserve_y,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dlmm_bin_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dlmm_bin_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dlmm_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.dlmm_event_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_x_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_y_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_1mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN]> for MigrateMeteoraDlmmToGammaKeys {
    fn from(pubkeys: [Pubkey; MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            dlmm_position: pubkeys[0],
            dlmm_lb_pair: pubkeys[1],
            dlmm_bin_array_bitmap_extension: pubkeys[2],
            dlmm_reserve_x: pubkeys[3],
            dlmm_reserve_y: pubkeys[4],
            dlmm_bin_array_lower: pubkeys[5],
            dlmm_bin_array_upper: pubkeys[6],
            dlmm_program: pubkeys[7],
            dlmm_event_authority: pubkeys[8],
            token_x_program: pubkeys[9],
            token_y_program: pubkeys[10],
            gamma_owner: pubkeys[11],
            gamma_authority: pubkeys[12],
            gamma_pool_state: pubkeys[13],
            gamma_user_pool_liquidity: pubkeys[14],
            gamma_token_0account: pubkeys[15],
            gamma_token_1account: pubkeys[16],
            gamma_token_0vault: pubkeys[17],
            gamma_token_1vault: pubkeys[18],
            token_program: pubkeys[19],
            token_program_2022: pubkeys[20],
            gamma_vault_0mint: pubkeys[21],
            gamma_vault_1mint: pubkeys[22],
        }
    }
}
impl<'info> From<MigrateMeteoraDlmmToGammaAccounts<'_, 'info>>
    for [AccountInfo<'info>; MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN]
{
    fn from(accounts: MigrateMeteoraDlmmToGammaAccounts<'_, 'info>) -> Self {
        [
            accounts.dlmm_position.clone(),
            accounts.dlmm_lb_pair.clone(),
            accounts.dlmm_bin_array_bitmap_extension.clone(),
            accounts.dlmm_reserve_x.clone(),
            accounts.dlmm_reserve_y.clone(),
            accounts.dlmm_bin_array_lower.clone(),
            accounts.dlmm_bin_array_upper.clone(),
            accounts.dlmm_program.clone(),
            accounts.dlmm_event_authority.clone(),
            accounts.token_x_program.clone(),
            accounts.token_y_program.clone(),
            accounts.gamma_owner.clone(),
            accounts.gamma_authority.clone(),
            accounts.gamma_pool_state.clone(),
            accounts.gamma_user_pool_liquidity.clone(),
            accounts.gamma_token_0account.clone(),
            accounts.gamma_token_1account.clone(),
            accounts.gamma_token_0vault.clone(),
            accounts.gamma_token_1vault.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.gamma_vault_0mint.clone(),
            accounts.gamma_vault_1mint.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN]>
    for MigrateMeteoraDlmmToGammaAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            dlmm_position: &arr[0],
            dlmm_lb_pair: &arr[1],
            dlmm_bin_array_bitmap_extension: &arr[2],
            dlmm_reserve_x: &arr[3],
            dlmm_reserve_y: &arr[4],
            dlmm_bin_array_lower: &arr[5],
            dlmm_bin_array_upper: &arr[6],
            dlmm_program: &arr[7],
            dlmm_event_authority: &arr[8],
            token_x_program: &arr[9],
            token_y_program: &arr[10],
            gamma_owner: &arr[11],
            gamma_authority: &arr[12],
            gamma_pool_state: &arr[13],
            gamma_user_pool_liquidity: &arr[14],
            gamma_token_0account: &arr[15],
            gamma_token_1account: &arr[16],
            gamma_token_0vault: &arr[17],
            gamma_token_1vault: &arr[18],
            token_program: &arr[19],
            token_program_2022: &arr[20],
            gamma_vault_0mint: &arr[21],
            gamma_vault_1mint: &arr[22],
        }
    }
}
pub const MIGRATE_METEORA_DLMM_TO_GAMMA_IX_DISCM: [u8; 8] = [166, 33, 209, 228, 15, 46, 252, 67];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct MigrateMeteoraDlmmToGammaIxArgs {
    pub bin_liquidity_reduction: Vec<BinLiquidityReduction>,
    pub maximum_token_0amount: u64,
    pub maximum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MigrateMeteoraDlmmToGammaIxData(pub MigrateMeteoraDlmmToGammaIxArgs);
impl From<MigrateMeteoraDlmmToGammaIxArgs> for MigrateMeteoraDlmmToGammaIxData {
    fn from(args: MigrateMeteoraDlmmToGammaIxArgs) -> Self {
        Self(args)
    }
}
impl MigrateMeteoraDlmmToGammaIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != MIGRATE_METEORA_DLMM_TO_GAMMA_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {MIGRATE_METEORA_DLMM_TO_GAMMA_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(MigrateMeteoraDlmmToGammaIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&MIGRATE_METEORA_DLMM_TO_GAMMA_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn migrate_meteora_dlmm_to_gamma_ix_with_program_id(
    program_id: Pubkey,
    keys: MigrateMeteoraDlmmToGammaKeys,
    args: MigrateMeteoraDlmmToGammaIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; MIGRATE_METEORA_DLMM_TO_GAMMA_IX_ACCOUNTS_LEN] = keys.into();
    let data: MigrateMeteoraDlmmToGammaIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn migrate_meteora_dlmm_to_gamma_ix(
    keys: MigrateMeteoraDlmmToGammaKeys,
    args: MigrateMeteoraDlmmToGammaIxArgs,
) -> std::io::Result<Instruction> {
    migrate_meteora_dlmm_to_gamma_ix_with_program_id(super::ID, keys, args)
}

pub fn migrate_meteora_dlmm_to_gamma_verify_account_keys(
    accounts: MigrateMeteoraDlmmToGammaAccounts<'_, '_>,
    keys: MigrateMeteoraDlmmToGammaKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.dlmm_position.key, keys.dlmm_position),
        (*accounts.dlmm_lb_pair.key, keys.dlmm_lb_pair),
        (
            *accounts.dlmm_bin_array_bitmap_extension.key,
            keys.dlmm_bin_array_bitmap_extension,
        ),
        (*accounts.dlmm_reserve_x.key, keys.dlmm_reserve_x),
        (*accounts.dlmm_reserve_y.key, keys.dlmm_reserve_y),
        (*accounts.dlmm_bin_array_lower.key, keys.dlmm_bin_array_lower),
        (*accounts.dlmm_bin_array_upper.key, keys.dlmm_bin_array_upper),
        (*accounts.dlmm_program.key, keys.dlmm_program),
        (*accounts.dlmm_event_authority.key, keys.dlmm_event_authority),
        (*accounts.token_x_program.key, keys.token_x_program),
        (*accounts.token_y_program.key, keys.token_y_program),
        (*accounts.gamma_owner.key, keys.gamma_owner),
        (*accounts.gamma_authority.key, keys.gamma_authority),
        (*accounts.gamma_pool_state.key, keys.gamma_pool_state),
        (*accounts.gamma_user_pool_liquidity.key, keys.gamma_user_pool_liquidity),
        (*accounts.gamma_token_0account.key, keys.gamma_token_0account),
        (*accounts.gamma_token_1account.key, keys.gamma_token_1account),
        (*accounts.gamma_token_0vault.key, keys.gamma_token_0vault),
        (*accounts.gamma_token_1vault.key, keys.gamma_token_1vault),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
        (*accounts.gamma_vault_0mint.key, keys.gamma_vault_0mint),
        (*accounts.gamma_vault_1mint.key, keys.gamma_vault_1mint),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn migrate_meteora_dlmm_to_gamma_verify_writable_privileges<'me, 'info>(
    accounts: MigrateMeteoraDlmmToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.dlmm_position,
        accounts.dlmm_lb_pair,
        accounts.dlmm_bin_array_bitmap_extension,
        accounts.dlmm_reserve_x,
        accounts.dlmm_reserve_y,
        accounts.dlmm_bin_array_lower,
        accounts.dlmm_bin_array_upper,
        accounts.gamma_pool_state,
        accounts.gamma_user_pool_liquidity,
        accounts.gamma_token_0account,
        accounts.gamma_token_1account,
        accounts.gamma_token_0vault,
        accounts.gamma_token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn migrate_meteora_dlmm_to_gamma_verify_signer_privileges<'me, 'info>(
    accounts: MigrateMeteoraDlmmToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.gamma_owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn migrate_meteora_dlmm_to_gamma_verify_account_privileges<'me, 'info>(
    accounts: MigrateMeteoraDlmmToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    migrate_meteora_dlmm_to_gamma_verify_writable_privileges(accounts)?;
    migrate_meteora_dlmm_to_gamma_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN: usize = 23;
#[derive(Copy, Clone, Debug)]
pub struct MigrateOrcaWhirlpoolToGammaAccounts<'me, 'info> {
    pub whirlpool_program: &'me AccountInfo<'info>,
    pub whirlpool: &'me AccountInfo<'info>,
    pub token_program_a: &'me AccountInfo<'info>,
    pub token_program_b: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
    pub whirlpool_position: &'me AccountInfo<'info>,
    pub whirlpool_position_token_account: &'me AccountInfo<'info>,
    pub whirlpool_token_vault_a: &'me AccountInfo<'info>,
    pub whirlpool_token_vault_b: &'me AccountInfo<'info>,
    pub whirlpool_tick_array_lower: &'me AccountInfo<'info>,
    pub whirlpool_tick_array_upper: &'me AccountInfo<'info>,
    pub gamma_owner: &'me AccountInfo<'info>,
    pub gamma_authority: &'me AccountInfo<'info>,
    pub gamma_pool_state: &'me AccountInfo<'info>,
    pub gamma_user_pool_liquidity: &'me AccountInfo<'info>,
    pub gamma_token_0account: &'me AccountInfo<'info>,
    pub gamma_token_1account: &'me AccountInfo<'info>,
    pub gamma_token_0vault: &'me AccountInfo<'info>,
    pub gamma_token_1vault: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub gamma_vault_0mint: &'me AccountInfo<'info>,
    pub gamma_vault_1mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MigrateOrcaWhirlpoolToGammaKeys {
    pub whirlpool_program: Pubkey,
    pub whirlpool: Pubkey,
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
    pub memo_program: Pubkey,
    pub whirlpool_position: Pubkey,
    pub whirlpool_position_token_account: Pubkey,
    pub whirlpool_token_vault_a: Pubkey,
    pub whirlpool_token_vault_b: Pubkey,
    pub whirlpool_tick_array_lower: Pubkey,
    pub whirlpool_tick_array_upper: Pubkey,
    pub gamma_owner: Pubkey,
    pub gamma_authority: Pubkey,
    pub gamma_pool_state: Pubkey,
    pub gamma_user_pool_liquidity: Pubkey,
    pub gamma_token_0account: Pubkey,
    pub gamma_token_1account: Pubkey,
    pub gamma_token_0vault: Pubkey,
    pub gamma_token_1vault: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub gamma_vault_0mint: Pubkey,
    pub gamma_vault_1mint: Pubkey,
}
impl From<MigrateOrcaWhirlpoolToGammaAccounts<'_, '_>> for MigrateOrcaWhirlpoolToGammaKeys {
    fn from(accounts: MigrateOrcaWhirlpoolToGammaAccounts) -> Self {
        Self {
            whirlpool_program: *accounts.whirlpool_program.key,
            whirlpool: *accounts.whirlpool.key,
            token_program_a: *accounts.token_program_a.key,
            token_program_b: *accounts.token_program_b.key,
            memo_program: *accounts.memo_program.key,
            whirlpool_position: *accounts.whirlpool_position.key,
            whirlpool_position_token_account: *accounts.whirlpool_position_token_account.key,
            whirlpool_token_vault_a: *accounts.whirlpool_token_vault_a.key,
            whirlpool_token_vault_b: *accounts.whirlpool_token_vault_b.key,
            whirlpool_tick_array_lower: *accounts.whirlpool_tick_array_lower.key,
            whirlpool_tick_array_upper: *accounts.whirlpool_tick_array_upper.key,
            gamma_owner: *accounts.gamma_owner.key,
            gamma_authority: *accounts.gamma_authority.key,
            gamma_pool_state: *accounts.gamma_pool_state.key,
            gamma_user_pool_liquidity: *accounts.gamma_user_pool_liquidity.key,
            gamma_token_0account: *accounts.gamma_token_0account.key,
            gamma_token_1account: *accounts.gamma_token_1account.key,
            gamma_token_0vault: *accounts.gamma_token_0vault.key,
            gamma_token_1vault: *accounts.gamma_token_1vault.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            gamma_vault_0mint: *accounts.gamma_vault_0mint.key,
            gamma_vault_1mint: *accounts.gamma_vault_1mint.key,
        }
    }
}
impl From<MigrateOrcaWhirlpoolToGammaKeys> for [AccountMeta; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN] {
    fn from(keys: MigrateOrcaWhirlpoolToGammaKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.whirlpool_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.whirlpool,
                is_signer: false,
                is_writable: true,
            },
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
                pubkey: keys.whirlpool_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_position_token_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.whirlpool_token_vault_a,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_token_vault_b,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_1mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN]> for MigrateOrcaWhirlpoolToGammaKeys {
    fn from(pubkeys: [Pubkey; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            whirlpool_program: pubkeys[0],
            whirlpool: pubkeys[1],
            token_program_a: pubkeys[2],
            token_program_b: pubkeys[3],
            memo_program: pubkeys[4],
            whirlpool_position: pubkeys[5],
            whirlpool_position_token_account: pubkeys[6],
            whirlpool_token_vault_a: pubkeys[7],
            whirlpool_token_vault_b: pubkeys[8],
            whirlpool_tick_array_lower: pubkeys[9],
            whirlpool_tick_array_upper: pubkeys[10],
            gamma_owner: pubkeys[11],
            gamma_authority: pubkeys[12],
            gamma_pool_state: pubkeys[13],
            gamma_user_pool_liquidity: pubkeys[14],
            gamma_token_0account: pubkeys[15],
            gamma_token_1account: pubkeys[16],
            gamma_token_0vault: pubkeys[17],
            gamma_token_1vault: pubkeys[18],
            token_program: pubkeys[19],
            token_program_2022: pubkeys[20],
            gamma_vault_0mint: pubkeys[21],
            gamma_vault_1mint: pubkeys[22],
        }
    }
}
impl<'info> From<MigrateOrcaWhirlpoolToGammaAccounts<'_, 'info>>
    for [AccountInfo<'info>; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN]
{
    fn from(accounts: MigrateOrcaWhirlpoolToGammaAccounts<'_, 'info>) -> Self {
        [
            accounts.whirlpool_program.clone(),
            accounts.whirlpool.clone(),
            accounts.token_program_a.clone(),
            accounts.token_program_b.clone(),
            accounts.memo_program.clone(),
            accounts.whirlpool_position.clone(),
            accounts.whirlpool_position_token_account.clone(),
            accounts.whirlpool_token_vault_a.clone(),
            accounts.whirlpool_token_vault_b.clone(),
            accounts.whirlpool_tick_array_lower.clone(),
            accounts.whirlpool_tick_array_upper.clone(),
            accounts.gamma_owner.clone(),
            accounts.gamma_authority.clone(),
            accounts.gamma_pool_state.clone(),
            accounts.gamma_user_pool_liquidity.clone(),
            accounts.gamma_token_0account.clone(),
            accounts.gamma_token_1account.clone(),
            accounts.gamma_token_0vault.clone(),
            accounts.gamma_token_1vault.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.gamma_vault_0mint.clone(),
            accounts.gamma_vault_1mint.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN]>
    for MigrateOrcaWhirlpoolToGammaAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            whirlpool_program: &arr[0],
            whirlpool: &arr[1],
            token_program_a: &arr[2],
            token_program_b: &arr[3],
            memo_program: &arr[4],
            whirlpool_position: &arr[5],
            whirlpool_position_token_account: &arr[6],
            whirlpool_token_vault_a: &arr[7],
            whirlpool_token_vault_b: &arr[8],
            whirlpool_tick_array_lower: &arr[9],
            whirlpool_tick_array_upper: &arr[10],
            gamma_owner: &arr[11],
            gamma_authority: &arr[12],
            gamma_pool_state: &arr[13],
            gamma_user_pool_liquidity: &arr[14],
            gamma_token_0account: &arr[15],
            gamma_token_1account: &arr[16],
            gamma_token_0vault: &arr[17],
            gamma_token_1vault: &arr[18],
            token_program: &arr[19],
            token_program_2022: &arr[20],
            gamma_vault_0mint: &arr[21],
            gamma_vault_1mint: &arr[22],
        }
    }
}
pub const MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_DISCM: [u8; 8] = [197, 196, 129, 5, 64, 184, 129, 242];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct MigrateOrcaWhirlpoolToGammaIxArgs {
    pub liquidity_amount: u128,
    pub token_min_a: u64,
    pub token_min_b: u64,
    pub maximum_token_0amount: u64,
    pub maximum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MigrateOrcaWhirlpoolToGammaIxData(pub MigrateOrcaWhirlpoolToGammaIxArgs);
impl From<MigrateOrcaWhirlpoolToGammaIxArgs> for MigrateOrcaWhirlpoolToGammaIxData {
    fn from(args: MigrateOrcaWhirlpoolToGammaIxArgs) -> Self {
        Self(args)
    }
}
impl MigrateOrcaWhirlpoolToGammaIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_DISCM {
            return Err(std::io::Error::other(
                format!(
                    "discm does not match. Expected: {MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_DISCM:?}. Received: {maybe_discm:?}"
                ),
            ));
        }
        Ok(Self(MigrateOrcaWhirlpoolToGammaIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn migrate_orca_whirlpool_to_gamma_ix_with_program_id(
    program_id: Pubkey,
    keys: MigrateOrcaWhirlpoolToGammaKeys,
    args: MigrateOrcaWhirlpoolToGammaIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_IX_ACCOUNTS_LEN] = keys.into();
    let data: MigrateOrcaWhirlpoolToGammaIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn migrate_orca_whirlpool_to_gamma_ix(
    keys: MigrateOrcaWhirlpoolToGammaKeys,
    args: MigrateOrcaWhirlpoolToGammaIxArgs,
) -> std::io::Result<Instruction> {
    migrate_orca_whirlpool_to_gamma_ix_with_program_id(super::ID, keys, args)
}

pub fn migrate_orca_whirlpool_to_gamma_verify_account_keys(
    accounts: MigrateOrcaWhirlpoolToGammaAccounts<'_, '_>,
    keys: MigrateOrcaWhirlpoolToGammaKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.whirlpool_program.key, keys.whirlpool_program),
        (*accounts.whirlpool.key, keys.whirlpool),
        (*accounts.token_program_a.key, keys.token_program_a),
        (*accounts.token_program_b.key, keys.token_program_b),
        (*accounts.memo_program.key, keys.memo_program),
        (*accounts.whirlpool_position.key, keys.whirlpool_position),
        (
            *accounts.whirlpool_position_token_account.key,
            keys.whirlpool_position_token_account,
        ),
        (*accounts.whirlpool_token_vault_a.key, keys.whirlpool_token_vault_a),
        (*accounts.whirlpool_token_vault_b.key, keys.whirlpool_token_vault_b),
        (
            *accounts.whirlpool_tick_array_lower.key,
            keys.whirlpool_tick_array_lower,
        ),
        (
            *accounts.whirlpool_tick_array_upper.key,
            keys.whirlpool_tick_array_upper,
        ),
        (*accounts.gamma_owner.key, keys.gamma_owner),
        (*accounts.gamma_authority.key, keys.gamma_authority),
        (*accounts.gamma_pool_state.key, keys.gamma_pool_state),
        (*accounts.gamma_user_pool_liquidity.key, keys.gamma_user_pool_liquidity),
        (*accounts.gamma_token_0account.key, keys.gamma_token_0account),
        (*accounts.gamma_token_1account.key, keys.gamma_token_1account),
        (*accounts.gamma_token_0vault.key, keys.gamma_token_0vault),
        (*accounts.gamma_token_1vault.key, keys.gamma_token_1vault),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
        (*accounts.gamma_vault_0mint.key, keys.gamma_vault_0mint),
        (*accounts.gamma_vault_1mint.key, keys.gamma_vault_1mint),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn migrate_orca_whirlpool_to_gamma_verify_writable_privileges<'me, 'info>(
    accounts: MigrateOrcaWhirlpoolToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.whirlpool,
        accounts.whirlpool_position,
        accounts.whirlpool_token_vault_a,
        accounts.whirlpool_token_vault_b,
        accounts.whirlpool_tick_array_lower,
        accounts.whirlpool_tick_array_upper,
        accounts.gamma_pool_state,
        accounts.gamma_user_pool_liquidity,
        accounts.gamma_token_0account,
        accounts.gamma_token_1account,
        accounts.gamma_token_0vault,
        accounts.gamma_token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn migrate_orca_whirlpool_to_gamma_verify_signer_privileges<'me, 'info>(
    accounts: MigrateOrcaWhirlpoolToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.gamma_owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn migrate_orca_whirlpool_to_gamma_verify_account_privileges<'me, 'info>(
    accounts: MigrateOrcaWhirlpoolToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    migrate_orca_whirlpool_to_gamma_verify_writable_privileges(accounts)?;
    migrate_orca_whirlpool_to_gamma_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN: usize = 23;
#[derive(Copy, Clone, Debug)]
pub struct MigrateOrcaWhirlpoolToGammaV2Accounts<'me, 'info> {
    pub whirlpool_program: &'me AccountInfo<'info>,
    pub whirlpool: &'me AccountInfo<'info>,
    pub token_program_a: &'me AccountInfo<'info>,
    pub token_program_b: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
    pub whirlpool_position: &'me AccountInfo<'info>,
    pub whirlpool_position_token_account: &'me AccountInfo<'info>,
    pub whirlpool_token_vault_a: &'me AccountInfo<'info>,
    pub whirlpool_token_vault_b: &'me AccountInfo<'info>,
    pub whirlpool_tick_array_lower: &'me AccountInfo<'info>,
    pub whirlpool_tick_array_upper: &'me AccountInfo<'info>,
    pub gamma_owner: &'me AccountInfo<'info>,
    pub gamma_authority: &'me AccountInfo<'info>,
    pub gamma_pool_state: &'me AccountInfo<'info>,
    pub gamma_user_pool_liquidity: &'me AccountInfo<'info>,
    pub gamma_token_0account: &'me AccountInfo<'info>,
    pub gamma_token_1account: &'me AccountInfo<'info>,
    pub gamma_token_0vault: &'me AccountInfo<'info>,
    pub gamma_token_1vault: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub gamma_vault_0mint: &'me AccountInfo<'info>,
    pub gamma_vault_1mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MigrateOrcaWhirlpoolToGammaV2Keys {
    pub whirlpool_program: Pubkey,
    pub whirlpool: Pubkey,
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
    pub memo_program: Pubkey,
    pub whirlpool_position: Pubkey,
    pub whirlpool_position_token_account: Pubkey,
    pub whirlpool_token_vault_a: Pubkey,
    pub whirlpool_token_vault_b: Pubkey,
    pub whirlpool_tick_array_lower: Pubkey,
    pub whirlpool_tick_array_upper: Pubkey,
    pub gamma_owner: Pubkey,
    pub gamma_authority: Pubkey,
    pub gamma_pool_state: Pubkey,
    pub gamma_user_pool_liquidity: Pubkey,
    pub gamma_token_0account: Pubkey,
    pub gamma_token_1account: Pubkey,
    pub gamma_token_0vault: Pubkey,
    pub gamma_token_1vault: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub gamma_vault_0mint: Pubkey,
    pub gamma_vault_1mint: Pubkey,
}
impl From<MigrateOrcaWhirlpoolToGammaV2Accounts<'_, '_>> for MigrateOrcaWhirlpoolToGammaV2Keys {
    fn from(accounts: MigrateOrcaWhirlpoolToGammaV2Accounts) -> Self {
        Self {
            whirlpool_program: *accounts.whirlpool_program.key,
            whirlpool: *accounts.whirlpool.key,
            token_program_a: *accounts.token_program_a.key,
            token_program_b: *accounts.token_program_b.key,
            memo_program: *accounts.memo_program.key,
            whirlpool_position: *accounts.whirlpool_position.key,
            whirlpool_position_token_account: *accounts.whirlpool_position_token_account.key,
            whirlpool_token_vault_a: *accounts.whirlpool_token_vault_a.key,
            whirlpool_token_vault_b: *accounts.whirlpool_token_vault_b.key,
            whirlpool_tick_array_lower: *accounts.whirlpool_tick_array_lower.key,
            whirlpool_tick_array_upper: *accounts.whirlpool_tick_array_upper.key,
            gamma_owner: *accounts.gamma_owner.key,
            gamma_authority: *accounts.gamma_authority.key,
            gamma_pool_state: *accounts.gamma_pool_state.key,
            gamma_user_pool_liquidity: *accounts.gamma_user_pool_liquidity.key,
            gamma_token_0account: *accounts.gamma_token_0account.key,
            gamma_token_1account: *accounts.gamma_token_1account.key,
            gamma_token_0vault: *accounts.gamma_token_0vault.key,
            gamma_token_1vault: *accounts.gamma_token_1vault.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            gamma_vault_0mint: *accounts.gamma_vault_0mint.key,
            gamma_vault_1mint: *accounts.gamma_vault_1mint.key,
        }
    }
}
impl From<MigrateOrcaWhirlpoolToGammaV2Keys> for [AccountMeta; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN] {
    fn from(keys: MigrateOrcaWhirlpoolToGammaV2Keys) -> Self {
        [
            AccountMeta {
                pubkey: keys.whirlpool_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.whirlpool,
                is_signer: false,
                is_writable: true,
            },
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
                pubkey: keys.whirlpool_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_position_token_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.whirlpool_token_vault_a,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_token_vault_b,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.whirlpool_tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_1mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN]> for MigrateOrcaWhirlpoolToGammaV2Keys {
    fn from(pubkeys: [Pubkey; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            whirlpool_program: pubkeys[0],
            whirlpool: pubkeys[1],
            token_program_a: pubkeys[2],
            token_program_b: pubkeys[3],
            memo_program: pubkeys[4],
            whirlpool_position: pubkeys[5],
            whirlpool_position_token_account: pubkeys[6],
            whirlpool_token_vault_a: pubkeys[7],
            whirlpool_token_vault_b: pubkeys[8],
            whirlpool_tick_array_lower: pubkeys[9],
            whirlpool_tick_array_upper: pubkeys[10],
            gamma_owner: pubkeys[11],
            gamma_authority: pubkeys[12],
            gamma_pool_state: pubkeys[13],
            gamma_user_pool_liquidity: pubkeys[14],
            gamma_token_0account: pubkeys[15],
            gamma_token_1account: pubkeys[16],
            gamma_token_0vault: pubkeys[17],
            gamma_token_1vault: pubkeys[18],
            token_program: pubkeys[19],
            token_program_2022: pubkeys[20],
            gamma_vault_0mint: pubkeys[21],
            gamma_vault_1mint: pubkeys[22],
        }
    }
}
impl<'info> From<MigrateOrcaWhirlpoolToGammaV2Accounts<'_, 'info>>
    for [AccountInfo<'info>; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN]
{
    fn from(accounts: MigrateOrcaWhirlpoolToGammaV2Accounts<'_, 'info>) -> Self {
        [
            accounts.whirlpool_program.clone(),
            accounts.whirlpool.clone(),
            accounts.token_program_a.clone(),
            accounts.token_program_b.clone(),
            accounts.memo_program.clone(),
            accounts.whirlpool_position.clone(),
            accounts.whirlpool_position_token_account.clone(),
            accounts.whirlpool_token_vault_a.clone(),
            accounts.whirlpool_token_vault_b.clone(),
            accounts.whirlpool_tick_array_lower.clone(),
            accounts.whirlpool_tick_array_upper.clone(),
            accounts.gamma_owner.clone(),
            accounts.gamma_authority.clone(),
            accounts.gamma_pool_state.clone(),
            accounts.gamma_user_pool_liquidity.clone(),
            accounts.gamma_token_0account.clone(),
            accounts.gamma_token_1account.clone(),
            accounts.gamma_token_0vault.clone(),
            accounts.gamma_token_1vault.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.gamma_vault_0mint.clone(),
            accounts.gamma_vault_1mint.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN]>
    for MigrateOrcaWhirlpoolToGammaV2Accounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            whirlpool_program: &arr[0],
            whirlpool: &arr[1],
            token_program_a: &arr[2],
            token_program_b: &arr[3],
            memo_program: &arr[4],
            whirlpool_position: &arr[5],
            whirlpool_position_token_account: &arr[6],
            whirlpool_token_vault_a: &arr[7],
            whirlpool_token_vault_b: &arr[8],
            whirlpool_tick_array_lower: &arr[9],
            whirlpool_tick_array_upper: &arr[10],
            gamma_owner: &arr[11],
            gamma_authority: &arr[12],
            gamma_pool_state: &arr[13],
            gamma_user_pool_liquidity: &arr[14],
            gamma_token_0account: &arr[15],
            gamma_token_1account: &arr[16],
            gamma_token_0vault: &arr[17],
            gamma_token_1vault: &arr[18],
            token_program: &arr[19],
            token_program_2022: &arr[20],
            gamma_vault_0mint: &arr[21],
            gamma_vault_1mint: &arr[22],
        }
    }
}
pub const MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_DISCM: [u8; 8] = [48, 74, 173, 201, 189, 38, 220, 244];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct MigrateOrcaWhirlpoolToGammaV2IxArgs {
    pub liquidity_amount: u128,
    pub token_min_a: u64,
    pub token_min_b: u64,
    pub remaining_accounts: RemainingAccountsInfo,
    pub maximum_token_0amount: u64,
    pub maximum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MigrateOrcaWhirlpoolToGammaV2IxData(pub MigrateOrcaWhirlpoolToGammaV2IxArgs);
impl From<MigrateOrcaWhirlpoolToGammaV2IxArgs> for MigrateOrcaWhirlpoolToGammaV2IxData {
    fn from(args: MigrateOrcaWhirlpoolToGammaV2IxArgs) -> Self {
        Self(args)
    }
}
impl MigrateOrcaWhirlpoolToGammaV2IxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_DISCM {
            return Err(std::io::Error::other(
                format!(
                    "discm does not match. Expected: {MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_DISCM:?}. Received: {maybe_discm:?}"
                ),
            ));
        }
        Ok(Self(MigrateOrcaWhirlpoolToGammaV2IxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn migrate_orca_whirlpool_to_gamma_v2_ix_with_program_id(
    program_id: Pubkey,
    keys: MigrateOrcaWhirlpoolToGammaV2Keys,
    args: MigrateOrcaWhirlpoolToGammaV2IxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; MIGRATE_ORCA_WHIRLPOOL_TO_GAMMA_V2_IX_ACCOUNTS_LEN] = keys.into();
    let data: MigrateOrcaWhirlpoolToGammaV2IxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn migrate_orca_whirlpool_to_gamma_v2_ix(
    keys: MigrateOrcaWhirlpoolToGammaV2Keys,
    args: MigrateOrcaWhirlpoolToGammaV2IxArgs,
) -> std::io::Result<Instruction> {
    migrate_orca_whirlpool_to_gamma_v2_ix_with_program_id(super::ID, keys, args)
}
pub fn migrate_orca_whirlpool_to_gamma_v2_verify_account_keys(
    accounts: MigrateOrcaWhirlpoolToGammaV2Accounts<'_, '_>,
    keys: MigrateOrcaWhirlpoolToGammaV2Keys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.whirlpool_program.key, keys.whirlpool_program),
        (*accounts.whirlpool.key, keys.whirlpool),
        (*accounts.token_program_a.key, keys.token_program_a),
        (*accounts.token_program_b.key, keys.token_program_b),
        (*accounts.memo_program.key, keys.memo_program),
        (*accounts.whirlpool_position.key, keys.whirlpool_position),
        (
            *accounts.whirlpool_position_token_account.key,
            keys.whirlpool_position_token_account,
        ),
        (*accounts.whirlpool_token_vault_a.key, keys.whirlpool_token_vault_a),
        (*accounts.whirlpool_token_vault_b.key, keys.whirlpool_token_vault_b),
        (
            *accounts.whirlpool_tick_array_lower.key,
            keys.whirlpool_tick_array_lower,
        ),
        (
            *accounts.whirlpool_tick_array_upper.key,
            keys.whirlpool_tick_array_upper,
        ),
        (*accounts.gamma_owner.key, keys.gamma_owner),
        (*accounts.gamma_authority.key, keys.gamma_authority),
        (*accounts.gamma_pool_state.key, keys.gamma_pool_state),
        (*accounts.gamma_user_pool_liquidity.key, keys.gamma_user_pool_liquidity),
        (*accounts.gamma_token_0account.key, keys.gamma_token_0account),
        (*accounts.gamma_token_1account.key, keys.gamma_token_1account),
        (*accounts.gamma_token_0vault.key, keys.gamma_token_0vault),
        (*accounts.gamma_token_1vault.key, keys.gamma_token_1vault),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
        (*accounts.gamma_vault_0mint.key, keys.gamma_vault_0mint),
        (*accounts.gamma_vault_1mint.key, keys.gamma_vault_1mint),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn migrate_orca_whirlpool_to_gamma_v2_verify_writable_privileges<'me, 'info>(
    accounts: MigrateOrcaWhirlpoolToGammaV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.whirlpool,
        accounts.whirlpool_position,
        accounts.whirlpool_token_vault_a,
        accounts.whirlpool_token_vault_b,
        accounts.whirlpool_tick_array_lower,
        accounts.whirlpool_tick_array_upper,
        accounts.gamma_pool_state,
        accounts.gamma_user_pool_liquidity,
        accounts.gamma_token_0account,
        accounts.gamma_token_1account,
        accounts.gamma_token_0vault,
        accounts.gamma_token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn migrate_orca_whirlpool_to_gamma_v2_verify_signer_privileges<'me, 'info>(
    accounts: MigrateOrcaWhirlpoolToGammaV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.gamma_owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn migrate_orca_whirlpool_to_gamma_v2_verify_account_privileges<'me, 'info>(
    accounts: MigrateOrcaWhirlpoolToGammaV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    migrate_orca_whirlpool_to_gamma_v2_verify_writable_privileges(accounts)?;
    migrate_orca_whirlpool_to_gamma_v2_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN: usize = 22;
#[derive(Copy, Clone, Debug)]
pub struct MigrateRaydiumClmmToGammaAccounts<'me, 'info> {
    pub raydium_clmm_program: &'me AccountInfo<'info>,
    pub raydium_clmm_nft_owner: &'me AccountInfo<'info>,
    pub raydium_clmm_nft_account: &'me AccountInfo<'info>,
    pub raydium_clmm_personal_position: &'me AccountInfo<'info>,
    pub raydium_clmm_pool_state: &'me AccountInfo<'info>,
    pub raydium_clmm_protocol_position: &'me AccountInfo<'info>,
    pub raydium_clmm_token_vault_0: &'me AccountInfo<'info>,
    pub raydium_clmm_token_vault_1: &'me AccountInfo<'info>,
    pub raydium_clmm_tick_array_lower: &'me AccountInfo<'info>,
    pub raydium_clmm_tick_array_upper: &'me AccountInfo<'info>,
    pub gamma_owner: &'me AccountInfo<'info>,
    pub gamma_authority: &'me AccountInfo<'info>,
    pub gamma_pool_state: &'me AccountInfo<'info>,
    pub gamma_user_pool_liquidity: &'me AccountInfo<'info>,
    pub gamma_token_0account: &'me AccountInfo<'info>,
    pub gamma_token_1account: &'me AccountInfo<'info>,
    pub gamma_token_0vault: &'me AccountInfo<'info>,
    pub gamma_token_1vault: &'me AccountInfo<'info>,
    pub gamma_vault_0mint: &'me AccountInfo<'info>,
    pub gamma_vault_1mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MigrateRaydiumClmmToGammaKeys {
    pub raydium_clmm_program: Pubkey,
    pub raydium_clmm_nft_owner: Pubkey,
    pub raydium_clmm_nft_account: Pubkey,
    pub raydium_clmm_personal_position: Pubkey,
    pub raydium_clmm_pool_state: Pubkey,
    pub raydium_clmm_protocol_position: Pubkey,
    pub raydium_clmm_token_vault_0: Pubkey,
    pub raydium_clmm_token_vault_1: Pubkey,
    pub raydium_clmm_tick_array_lower: Pubkey,
    pub raydium_clmm_tick_array_upper: Pubkey,
    pub gamma_owner: Pubkey,
    pub gamma_authority: Pubkey,
    pub gamma_pool_state: Pubkey,
    pub gamma_user_pool_liquidity: Pubkey,
    pub gamma_token_0account: Pubkey,
    pub gamma_token_1account: Pubkey,
    pub gamma_token_0vault: Pubkey,
    pub gamma_token_1vault: Pubkey,
    pub gamma_vault_0mint: Pubkey,
    pub gamma_vault_1mint: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
}
impl From<MigrateRaydiumClmmToGammaAccounts<'_, '_>> for MigrateRaydiumClmmToGammaKeys {
    fn from(accounts: MigrateRaydiumClmmToGammaAccounts) -> Self {
        Self {
            raydium_clmm_program: *accounts.raydium_clmm_program.key,
            raydium_clmm_nft_owner: *accounts.raydium_clmm_nft_owner.key,
            raydium_clmm_nft_account: *accounts.raydium_clmm_nft_account.key,
            raydium_clmm_personal_position: *accounts.raydium_clmm_personal_position.key,
            raydium_clmm_pool_state: *accounts.raydium_clmm_pool_state.key,
            raydium_clmm_protocol_position: *accounts.raydium_clmm_protocol_position.key,
            raydium_clmm_token_vault_0: *accounts.raydium_clmm_token_vault_0.key,
            raydium_clmm_token_vault_1: *accounts.raydium_clmm_token_vault_1.key,
            raydium_clmm_tick_array_lower: *accounts.raydium_clmm_tick_array_lower.key,
            raydium_clmm_tick_array_upper: *accounts.raydium_clmm_tick_array_upper.key,
            gamma_owner: *accounts.gamma_owner.key,
            gamma_authority: *accounts.gamma_authority.key,
            gamma_pool_state: *accounts.gamma_pool_state.key,
            gamma_user_pool_liquidity: *accounts.gamma_user_pool_liquidity.key,
            gamma_token_0account: *accounts.gamma_token_0account.key,
            gamma_token_1account: *accounts.gamma_token_1account.key,
            gamma_token_0vault: *accounts.gamma_token_0vault.key,
            gamma_token_1vault: *accounts.gamma_token_1vault.key,
            gamma_vault_0mint: *accounts.gamma_vault_0mint.key,
            gamma_vault_1mint: *accounts.gamma_vault_1mint.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
        }
    }
}
impl From<MigrateRaydiumClmmToGammaKeys> for [AccountMeta; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN] {
    fn from(keys: MigrateRaydiumClmmToGammaKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.raydium_clmm_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_nft_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_nft_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_personal_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_protocol_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_token_vault_0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_token_vault_1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN]> for MigrateRaydiumClmmToGammaKeys {
    fn from(pubkeys: [Pubkey; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            raydium_clmm_program: pubkeys[0],
            raydium_clmm_nft_owner: pubkeys[1],
            raydium_clmm_nft_account: pubkeys[2],
            raydium_clmm_personal_position: pubkeys[3],
            raydium_clmm_pool_state: pubkeys[4],
            raydium_clmm_protocol_position: pubkeys[5],
            raydium_clmm_token_vault_0: pubkeys[6],
            raydium_clmm_token_vault_1: pubkeys[7],
            raydium_clmm_tick_array_lower: pubkeys[8],
            raydium_clmm_tick_array_upper: pubkeys[9],
            gamma_owner: pubkeys[10],
            gamma_authority: pubkeys[11],
            gamma_pool_state: pubkeys[12],
            gamma_user_pool_liquidity: pubkeys[13],
            gamma_token_0account: pubkeys[14],
            gamma_token_1account: pubkeys[15],
            gamma_token_0vault: pubkeys[16],
            gamma_token_1vault: pubkeys[17],
            gamma_vault_0mint: pubkeys[18],
            gamma_vault_1mint: pubkeys[19],
            token_program: pubkeys[20],
            token_program_2022: pubkeys[21],
        }
    }
}
impl<'info> From<MigrateRaydiumClmmToGammaAccounts<'_, 'info>>
    for [AccountInfo<'info>; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN]
{
    fn from(accounts: MigrateRaydiumClmmToGammaAccounts<'_, 'info>) -> Self {
        [
            accounts.raydium_clmm_program.clone(),
            accounts.raydium_clmm_nft_owner.clone(),
            accounts.raydium_clmm_nft_account.clone(),
            accounts.raydium_clmm_personal_position.clone(),
            accounts.raydium_clmm_pool_state.clone(),
            accounts.raydium_clmm_protocol_position.clone(),
            accounts.raydium_clmm_token_vault_0.clone(),
            accounts.raydium_clmm_token_vault_1.clone(),
            accounts.raydium_clmm_tick_array_lower.clone(),
            accounts.raydium_clmm_tick_array_upper.clone(),
            accounts.gamma_owner.clone(),
            accounts.gamma_authority.clone(),
            accounts.gamma_pool_state.clone(),
            accounts.gamma_user_pool_liquidity.clone(),
            accounts.gamma_token_0account.clone(),
            accounts.gamma_token_1account.clone(),
            accounts.gamma_token_0vault.clone(),
            accounts.gamma_token_1vault.clone(),
            accounts.gamma_vault_0mint.clone(),
            accounts.gamma_vault_1mint.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN]>
    for MigrateRaydiumClmmToGammaAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            raydium_clmm_program: &arr[0],
            raydium_clmm_nft_owner: &arr[1],
            raydium_clmm_nft_account: &arr[2],
            raydium_clmm_personal_position: &arr[3],
            raydium_clmm_pool_state: &arr[4],
            raydium_clmm_protocol_position: &arr[5],
            raydium_clmm_token_vault_0: &arr[6],
            raydium_clmm_token_vault_1: &arr[7],
            raydium_clmm_tick_array_lower: &arr[8],
            raydium_clmm_tick_array_upper: &arr[9],
            gamma_owner: &arr[10],
            gamma_authority: &arr[11],
            gamma_pool_state: &arr[12],
            gamma_user_pool_liquidity: &arr[13],
            gamma_token_0account: &arr[14],
            gamma_token_1account: &arr[15],
            gamma_token_0vault: &arr[16],
            gamma_token_1vault: &arr[17],
            gamma_vault_0mint: &arr[18],
            gamma_vault_1mint: &arr[19],
            token_program: &arr[20],
            token_program_2022: &arr[21],
        }
    }
}
pub const MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_DISCM: [u8; 8] = [53, 162, 11, 109, 57, 57, 186, 248];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct MigrateRaydiumClmmToGammaIxArgs {
    pub liquidity: u128,
    pub amount_0min: u64,
    pub amount_1min: u64,
    pub maximum_token_0amount: u64,
    pub maximum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MigrateRaydiumClmmToGammaIxData(pub MigrateRaydiumClmmToGammaIxArgs);
impl From<MigrateRaydiumClmmToGammaIxArgs> for MigrateRaydiumClmmToGammaIxData {
    fn from(args: MigrateRaydiumClmmToGammaIxArgs) -> Self {
        Self(args)
    }
}
impl MigrateRaydiumClmmToGammaIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(MigrateRaydiumClmmToGammaIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn migrate_raydium_clmm_to_gamma_ix_with_program_id(
    program_id: Pubkey,
    keys: MigrateRaydiumClmmToGammaKeys,
    args: MigrateRaydiumClmmToGammaIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_IX_ACCOUNTS_LEN] = keys.into();
    let data: MigrateRaydiumClmmToGammaIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn migrate_raydium_clmm_to_gamma_ix(
    keys: MigrateRaydiumClmmToGammaKeys,
    args: MigrateRaydiumClmmToGammaIxArgs,
) -> std::io::Result<Instruction> {
    migrate_raydium_clmm_to_gamma_ix_with_program_id(super::ID, keys, args)
}

pub fn migrate_raydium_clmm_to_gamma_verify_account_keys(
    accounts: MigrateRaydiumClmmToGammaAccounts<'_, '_>,
    keys: MigrateRaydiumClmmToGammaKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.raydium_clmm_program.key, keys.raydium_clmm_program),
        (*accounts.raydium_clmm_nft_owner.key, keys.raydium_clmm_nft_owner),
        (*accounts.raydium_clmm_nft_account.key, keys.raydium_clmm_nft_account),
        (
            *accounts.raydium_clmm_personal_position.key,
            keys.raydium_clmm_personal_position,
        ),
        (*accounts.raydium_clmm_pool_state.key, keys.raydium_clmm_pool_state),
        (
            *accounts.raydium_clmm_protocol_position.key,
            keys.raydium_clmm_protocol_position,
        ),
        (
            *accounts.raydium_clmm_token_vault_0.key,
            keys.raydium_clmm_token_vault_0,
        ),
        (
            *accounts.raydium_clmm_token_vault_1.key,
            keys.raydium_clmm_token_vault_1,
        ),
        (
            *accounts.raydium_clmm_tick_array_lower.key,
            keys.raydium_clmm_tick_array_lower,
        ),
        (
            *accounts.raydium_clmm_tick_array_upper.key,
            keys.raydium_clmm_tick_array_upper,
        ),
        (*accounts.gamma_owner.key, keys.gamma_owner),
        (*accounts.gamma_authority.key, keys.gamma_authority),
        (*accounts.gamma_pool_state.key, keys.gamma_pool_state),
        (*accounts.gamma_user_pool_liquidity.key, keys.gamma_user_pool_liquidity),
        (*accounts.gamma_token_0account.key, keys.gamma_token_0account),
        (*accounts.gamma_token_1account.key, keys.gamma_token_1account),
        (*accounts.gamma_token_0vault.key, keys.gamma_token_0vault),
        (*accounts.gamma_token_1vault.key, keys.gamma_token_1vault),
        (*accounts.gamma_vault_0mint.key, keys.gamma_vault_0mint),
        (*accounts.gamma_vault_1mint.key, keys.gamma_vault_1mint),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn migrate_raydium_clmm_to_gamma_verify_writable_privileges<'me, 'info>(
    accounts: MigrateRaydiumClmmToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.raydium_clmm_personal_position,
        accounts.raydium_clmm_pool_state,
        accounts.raydium_clmm_protocol_position,
        accounts.raydium_clmm_token_vault_0,
        accounts.raydium_clmm_token_vault_1,
        accounts.raydium_clmm_tick_array_lower,
        accounts.raydium_clmm_tick_array_upper,
        accounts.gamma_pool_state,
        accounts.gamma_user_pool_liquidity,
        accounts.gamma_token_0account,
        accounts.gamma_token_1account,
        accounts.gamma_token_0vault,
        accounts.gamma_token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn migrate_raydium_clmm_to_gamma_verify_signer_privileges<'me, 'info>(
    accounts: MigrateRaydiumClmmToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.raydium_clmm_nft_owner, accounts.gamma_owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn migrate_raydium_clmm_to_gamma_verify_account_privileges<'me, 'info>(
    accounts: MigrateRaydiumClmmToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    migrate_raydium_clmm_to_gamma_verify_writable_privileges(accounts)?;
    migrate_raydium_clmm_to_gamma_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN: usize = 23;
#[derive(Copy, Clone, Debug)]
pub struct MigrateRaydiumClmmToGammaV2Accounts<'me, 'info> {
    pub raydium_clmm_program: &'me AccountInfo<'info>,
    pub raydium_clmm_nft_owner: &'me AccountInfo<'info>,
    pub raydium_clmm_nft_account: &'me AccountInfo<'info>,
    pub raydium_clmm_personal_position: &'me AccountInfo<'info>,
    pub raydium_clmm_pool_state: &'me AccountInfo<'info>,
    pub raydium_clmm_protocol_position: &'me AccountInfo<'info>,
    pub raydium_clmm_token_vault_0: &'me AccountInfo<'info>,
    pub raydium_clmm_token_vault_1: &'me AccountInfo<'info>,
    pub raydium_clmm_tick_array_lower: &'me AccountInfo<'info>,
    pub raydium_clmm_tick_array_upper: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
    pub gamma_owner: &'me AccountInfo<'info>,
    pub gamma_authority: &'me AccountInfo<'info>,
    pub gamma_pool_state: &'me AccountInfo<'info>,
    pub gamma_user_pool_liquidity: &'me AccountInfo<'info>,
    pub gamma_token_0account: &'me AccountInfo<'info>,
    pub gamma_token_1account: &'me AccountInfo<'info>,
    pub gamma_token_0vault: &'me AccountInfo<'info>,
    pub gamma_token_1vault: &'me AccountInfo<'info>,
    pub gamma_vault_0mint: &'me AccountInfo<'info>,
    pub gamma_vault_1mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MigrateRaydiumClmmToGammaV2Keys {
    pub raydium_clmm_program: Pubkey,
    pub raydium_clmm_nft_owner: Pubkey,
    pub raydium_clmm_nft_account: Pubkey,
    pub raydium_clmm_personal_position: Pubkey,
    pub raydium_clmm_pool_state: Pubkey,
    pub raydium_clmm_protocol_position: Pubkey,
    pub raydium_clmm_token_vault_0: Pubkey,
    pub raydium_clmm_token_vault_1: Pubkey,
    pub raydium_clmm_tick_array_lower: Pubkey,
    pub raydium_clmm_tick_array_upper: Pubkey,
    pub memo_program: Pubkey,
    pub gamma_owner: Pubkey,
    pub gamma_authority: Pubkey,
    pub gamma_pool_state: Pubkey,
    pub gamma_user_pool_liquidity: Pubkey,
    pub gamma_token_0account: Pubkey,
    pub gamma_token_1account: Pubkey,
    pub gamma_token_0vault: Pubkey,
    pub gamma_token_1vault: Pubkey,
    pub gamma_vault_0mint: Pubkey,
    pub gamma_vault_1mint: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
}
impl From<MigrateRaydiumClmmToGammaV2Accounts<'_, '_>> for MigrateRaydiumClmmToGammaV2Keys {
    fn from(accounts: MigrateRaydiumClmmToGammaV2Accounts) -> Self {
        Self {
            raydium_clmm_program: *accounts.raydium_clmm_program.key,
            raydium_clmm_nft_owner: *accounts.raydium_clmm_nft_owner.key,
            raydium_clmm_nft_account: *accounts.raydium_clmm_nft_account.key,
            raydium_clmm_personal_position: *accounts.raydium_clmm_personal_position.key,
            raydium_clmm_pool_state: *accounts.raydium_clmm_pool_state.key,
            raydium_clmm_protocol_position: *accounts.raydium_clmm_protocol_position.key,
            raydium_clmm_token_vault_0: *accounts.raydium_clmm_token_vault_0.key,
            raydium_clmm_token_vault_1: *accounts.raydium_clmm_token_vault_1.key,
            raydium_clmm_tick_array_lower: *accounts.raydium_clmm_tick_array_lower.key,
            raydium_clmm_tick_array_upper: *accounts.raydium_clmm_tick_array_upper.key,
            memo_program: *accounts.memo_program.key,
            gamma_owner: *accounts.gamma_owner.key,
            gamma_authority: *accounts.gamma_authority.key,
            gamma_pool_state: *accounts.gamma_pool_state.key,
            gamma_user_pool_liquidity: *accounts.gamma_user_pool_liquidity.key,
            gamma_token_0account: *accounts.gamma_token_0account.key,
            gamma_token_1account: *accounts.gamma_token_1account.key,
            gamma_token_0vault: *accounts.gamma_token_0vault.key,
            gamma_token_1vault: *accounts.gamma_token_1vault.key,
            gamma_vault_0mint: *accounts.gamma_vault_0mint.key,
            gamma_vault_1mint: *accounts.gamma_vault_1mint.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
        }
    }
}
impl From<MigrateRaydiumClmmToGammaV2Keys> for [AccountMeta; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN] {
    fn from(keys: MigrateRaydiumClmmToGammaV2Keys) -> Self {
        [
            AccountMeta {
                pubkey: keys.raydium_clmm_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_nft_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_nft_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_personal_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_protocol_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_token_vault_0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_token_vault_1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_clmm_tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.memo_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN]> for MigrateRaydiumClmmToGammaV2Keys {
    fn from(pubkeys: [Pubkey; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            raydium_clmm_program: pubkeys[0],
            raydium_clmm_nft_owner: pubkeys[1],
            raydium_clmm_nft_account: pubkeys[2],
            raydium_clmm_personal_position: pubkeys[3],
            raydium_clmm_pool_state: pubkeys[4],
            raydium_clmm_protocol_position: pubkeys[5],
            raydium_clmm_token_vault_0: pubkeys[6],
            raydium_clmm_token_vault_1: pubkeys[7],
            raydium_clmm_tick_array_lower: pubkeys[8],
            raydium_clmm_tick_array_upper: pubkeys[9],
            memo_program: pubkeys[10],
            gamma_owner: pubkeys[11],
            gamma_authority: pubkeys[12],
            gamma_pool_state: pubkeys[13],
            gamma_user_pool_liquidity: pubkeys[14],
            gamma_token_0account: pubkeys[15],
            gamma_token_1account: pubkeys[16],
            gamma_token_0vault: pubkeys[17],
            gamma_token_1vault: pubkeys[18],
            gamma_vault_0mint: pubkeys[19],
            gamma_vault_1mint: pubkeys[20],
            token_program: pubkeys[21],
            token_program_2022: pubkeys[22],
        }
    }
}
impl<'info> From<MigrateRaydiumClmmToGammaV2Accounts<'_, 'info>>
    for [AccountInfo<'info>; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN]
{
    fn from(accounts: MigrateRaydiumClmmToGammaV2Accounts<'_, 'info>) -> Self {
        [
            accounts.raydium_clmm_program.clone(),
            accounts.raydium_clmm_nft_owner.clone(),
            accounts.raydium_clmm_nft_account.clone(),
            accounts.raydium_clmm_personal_position.clone(),
            accounts.raydium_clmm_pool_state.clone(),
            accounts.raydium_clmm_protocol_position.clone(),
            accounts.raydium_clmm_token_vault_0.clone(),
            accounts.raydium_clmm_token_vault_1.clone(),
            accounts.raydium_clmm_tick_array_lower.clone(),
            accounts.raydium_clmm_tick_array_upper.clone(),
            accounts.memo_program.clone(),
            accounts.gamma_owner.clone(),
            accounts.gamma_authority.clone(),
            accounts.gamma_pool_state.clone(),
            accounts.gamma_user_pool_liquidity.clone(),
            accounts.gamma_token_0account.clone(),
            accounts.gamma_token_1account.clone(),
            accounts.gamma_token_0vault.clone(),
            accounts.gamma_token_1vault.clone(),
            accounts.gamma_vault_0mint.clone(),
            accounts.gamma_vault_1mint.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN]>
    for MigrateRaydiumClmmToGammaV2Accounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            raydium_clmm_program: &arr[0],
            raydium_clmm_nft_owner: &arr[1],
            raydium_clmm_nft_account: &arr[2],
            raydium_clmm_personal_position: &arr[3],
            raydium_clmm_pool_state: &arr[4],
            raydium_clmm_protocol_position: &arr[5],
            raydium_clmm_token_vault_0: &arr[6],
            raydium_clmm_token_vault_1: &arr[7],
            raydium_clmm_tick_array_lower: &arr[8],
            raydium_clmm_tick_array_upper: &arr[9],
            memo_program: &arr[10],
            gamma_owner: &arr[11],
            gamma_authority: &arr[12],
            gamma_pool_state: &arr[13],
            gamma_user_pool_liquidity: &arr[14],
            gamma_token_0account: &arr[15],
            gamma_token_1account: &arr[16],
            gamma_token_0vault: &arr[17],
            gamma_token_1vault: &arr[18],
            gamma_vault_0mint: &arr[19],
            gamma_vault_1mint: &arr[20],
            token_program: &arr[21],
            token_program_2022: &arr[22],
        }
    }
}
pub const MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_DISCM: [u8; 8] = [138, 127, 173, 116, 184, 119, 160, 145];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct MigrateRaydiumClmmToGammaV2IxArgs {
    pub liquidity: u128,
    pub amount_0min: u64,
    pub amount_1min: u64,
    pub maximum_token_0amount: u64,
    pub maximum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MigrateRaydiumClmmToGammaV2IxData(pub MigrateRaydiumClmmToGammaV2IxArgs);
impl From<MigrateRaydiumClmmToGammaV2IxArgs> for MigrateRaydiumClmmToGammaV2IxData {
    fn from(args: MigrateRaydiumClmmToGammaV2IxArgs) -> Self {
        Self(args)
    }
}
impl MigrateRaydiumClmmToGammaV2IxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_DISCM {
            return Err(std::io::Error::other(
                format!(
                    "discm does not match. Expected: {MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_DISCM:?}. Received: {maybe_discm:?}"
                ),
            ));
        }
        Ok(Self(MigrateRaydiumClmmToGammaV2IxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn migrate_raydium_clmm_to_gamma_v2_ix_with_program_id(
    program_id: Pubkey,
    keys: MigrateRaydiumClmmToGammaV2Keys,
    args: MigrateRaydiumClmmToGammaV2IxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; MIGRATE_RAYDIUM_CLMM_TO_GAMMA_V2_IX_ACCOUNTS_LEN] = keys.into();
    let data: MigrateRaydiumClmmToGammaV2IxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn migrate_raydium_clmm_to_gamma_v2_ix(
    keys: MigrateRaydiumClmmToGammaV2Keys,
    args: MigrateRaydiumClmmToGammaV2IxArgs,
) -> std::io::Result<Instruction> {
    migrate_raydium_clmm_to_gamma_v2_ix_with_program_id(super::ID, keys, args)
}
pub fn migrate_raydium_clmm_to_gamma_v2_verify_account_keys(
    accounts: MigrateRaydiumClmmToGammaV2Accounts<'_, '_>,
    keys: MigrateRaydiumClmmToGammaV2Keys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.raydium_clmm_program.key, keys.raydium_clmm_program),
        (*accounts.raydium_clmm_nft_owner.key, keys.raydium_clmm_nft_owner),
        (*accounts.raydium_clmm_nft_account.key, keys.raydium_clmm_nft_account),
        (
            *accounts.raydium_clmm_personal_position.key,
            keys.raydium_clmm_personal_position,
        ),
        (*accounts.raydium_clmm_pool_state.key, keys.raydium_clmm_pool_state),
        (
            *accounts.raydium_clmm_protocol_position.key,
            keys.raydium_clmm_protocol_position,
        ),
        (
            *accounts.raydium_clmm_token_vault_0.key,
            keys.raydium_clmm_token_vault_0,
        ),
        (
            *accounts.raydium_clmm_token_vault_1.key,
            keys.raydium_clmm_token_vault_1,
        ),
        (
            *accounts.raydium_clmm_tick_array_lower.key,
            keys.raydium_clmm_tick_array_lower,
        ),
        (
            *accounts.raydium_clmm_tick_array_upper.key,
            keys.raydium_clmm_tick_array_upper,
        ),
        (*accounts.memo_program.key, keys.memo_program),
        (*accounts.gamma_owner.key, keys.gamma_owner),
        (*accounts.gamma_authority.key, keys.gamma_authority),
        (*accounts.gamma_pool_state.key, keys.gamma_pool_state),
        (*accounts.gamma_user_pool_liquidity.key, keys.gamma_user_pool_liquidity),
        (*accounts.gamma_token_0account.key, keys.gamma_token_0account),
        (*accounts.gamma_token_1account.key, keys.gamma_token_1account),
        (*accounts.gamma_token_0vault.key, keys.gamma_token_0vault),
        (*accounts.gamma_token_1vault.key, keys.gamma_token_1vault),
        (*accounts.gamma_vault_0mint.key, keys.gamma_vault_0mint),
        (*accounts.gamma_vault_1mint.key, keys.gamma_vault_1mint),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn migrate_raydium_clmm_to_gamma_v2_verify_writable_privileges<'me, 'info>(
    accounts: MigrateRaydiumClmmToGammaV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.raydium_clmm_personal_position,
        accounts.raydium_clmm_pool_state,
        accounts.raydium_clmm_protocol_position,
        accounts.raydium_clmm_token_vault_0,
        accounts.raydium_clmm_token_vault_1,
        accounts.raydium_clmm_tick_array_lower,
        accounts.raydium_clmm_tick_array_upper,
        accounts.gamma_pool_state,
        accounts.gamma_user_pool_liquidity,
        accounts.gamma_token_0account,
        accounts.gamma_token_1account,
        accounts.gamma_token_0vault,
        accounts.gamma_token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn migrate_raydium_clmm_to_gamma_v2_verify_signer_privileges<'me, 'info>(
    accounts: MigrateRaydiumClmmToGammaV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.raydium_clmm_nft_owner, accounts.gamma_owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn migrate_raydium_clmm_to_gamma_v2_verify_account_privileges<'me, 'info>(
    accounts: MigrateRaydiumClmmToGammaV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    migrate_raydium_clmm_to_gamma_v2_verify_writable_privileges(accounts)?;
    migrate_raydium_clmm_to_gamma_v2_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN: usize = 23;
#[derive(Copy, Clone, Debug)]
pub struct MigrateRaydiumCpSwapToGammaAccounts<'me, 'info> {
    pub raydium_cp_swap_program: &'me AccountInfo<'info>,
    pub owner: &'me AccountInfo<'info>,
    pub raydium_cp_swap_authority: &'me AccountInfo<'info>,
    pub raydium_cp_swap_pool_state: &'me AccountInfo<'info>,
    pub raydium_cp_swap_owner_lp_token: &'me AccountInfo<'info>,
    pub raydium_cp_swap_token_0vault: &'me AccountInfo<'info>,
    pub raydium_cp_swap_token_1vault: &'me AccountInfo<'info>,
    pub raydium_cp_swap_vault_0mint: &'me AccountInfo<'info>,
    pub raydium_cp_swap_vault_1mint: &'me AccountInfo<'info>,
    pub raydium_cp_swap_lp_mint: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
    pub gamma_owner: &'me AccountInfo<'info>,
    pub gamma_authority: &'me AccountInfo<'info>,
    pub gamma_pool_state: &'me AccountInfo<'info>,
    pub gamma_user_pool_liquidity: &'me AccountInfo<'info>,
    pub gamma_token_0account: &'me AccountInfo<'info>,
    pub gamma_token_1account: &'me AccountInfo<'info>,
    pub gamma_token_0vault: &'me AccountInfo<'info>,
    pub gamma_token_1vault: &'me AccountInfo<'info>,
    pub gamma_vault_0mint: &'me AccountInfo<'info>,
    pub gamma_vault_1mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MigrateRaydiumCpSwapToGammaKeys {
    pub raydium_cp_swap_program: Pubkey,
    pub owner: Pubkey,
    pub raydium_cp_swap_authority: Pubkey,
    pub raydium_cp_swap_pool_state: Pubkey,
    pub raydium_cp_swap_owner_lp_token: Pubkey,
    pub raydium_cp_swap_token_0vault: Pubkey,
    pub raydium_cp_swap_token_1vault: Pubkey,
    pub raydium_cp_swap_vault_0mint: Pubkey,
    pub raydium_cp_swap_vault_1mint: Pubkey,
    pub raydium_cp_swap_lp_mint: Pubkey,
    pub memo_program: Pubkey,
    pub gamma_owner: Pubkey,
    pub gamma_authority: Pubkey,
    pub gamma_pool_state: Pubkey,
    pub gamma_user_pool_liquidity: Pubkey,
    pub gamma_token_0account: Pubkey,
    pub gamma_token_1account: Pubkey,
    pub gamma_token_0vault: Pubkey,
    pub gamma_token_1vault: Pubkey,
    pub gamma_vault_0mint: Pubkey,
    pub gamma_vault_1mint: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
}
impl From<MigrateRaydiumCpSwapToGammaAccounts<'_, '_>> for MigrateRaydiumCpSwapToGammaKeys {
    fn from(accounts: MigrateRaydiumCpSwapToGammaAccounts) -> Self {
        Self {
            raydium_cp_swap_program: *accounts.raydium_cp_swap_program.key,
            owner: *accounts.owner.key,
            raydium_cp_swap_authority: *accounts.raydium_cp_swap_authority.key,
            raydium_cp_swap_pool_state: *accounts.raydium_cp_swap_pool_state.key,
            raydium_cp_swap_owner_lp_token: *accounts.raydium_cp_swap_owner_lp_token.key,
            raydium_cp_swap_token_0vault: *accounts.raydium_cp_swap_token_0vault.key,
            raydium_cp_swap_token_1vault: *accounts.raydium_cp_swap_token_1vault.key,
            raydium_cp_swap_vault_0mint: *accounts.raydium_cp_swap_vault_0mint.key,
            raydium_cp_swap_vault_1mint: *accounts.raydium_cp_swap_vault_1mint.key,
            raydium_cp_swap_lp_mint: *accounts.raydium_cp_swap_lp_mint.key,
            memo_program: *accounts.memo_program.key,
            gamma_owner: *accounts.gamma_owner.key,
            gamma_authority: *accounts.gamma_authority.key,
            gamma_pool_state: *accounts.gamma_pool_state.key,
            gamma_user_pool_liquidity: *accounts.gamma_user_pool_liquidity.key,
            gamma_token_0account: *accounts.gamma_token_0account.key,
            gamma_token_1account: *accounts.gamma_token_1account.key,
            gamma_token_0vault: *accounts.gamma_token_0vault.key,
            gamma_token_1vault: *accounts.gamma_token_1vault.key,
            gamma_vault_0mint: *accounts.gamma_vault_0mint.key,
            gamma_vault_1mint: *accounts.gamma_vault_1mint.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
        }
    }
}
impl From<MigrateRaydiumCpSwapToGammaKeys> for [AccountMeta; MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN] {
    fn from(keys: MigrateRaydiumCpSwapToGammaKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.raydium_cp_swap_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_owner_lp_token,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_vault_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.raydium_cp_swap_lp_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.memo_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.gamma_vault_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN]> for MigrateRaydiumCpSwapToGammaKeys {
    fn from(pubkeys: [Pubkey; MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            raydium_cp_swap_program: pubkeys[0],
            owner: pubkeys[1],
            raydium_cp_swap_authority: pubkeys[2],
            raydium_cp_swap_pool_state: pubkeys[3],
            raydium_cp_swap_owner_lp_token: pubkeys[4],
            raydium_cp_swap_token_0vault: pubkeys[5],
            raydium_cp_swap_token_1vault: pubkeys[6],
            raydium_cp_swap_vault_0mint: pubkeys[7],
            raydium_cp_swap_vault_1mint: pubkeys[8],
            raydium_cp_swap_lp_mint: pubkeys[9],
            memo_program: pubkeys[10],
            gamma_owner: pubkeys[11],
            gamma_authority: pubkeys[12],
            gamma_pool_state: pubkeys[13],
            gamma_user_pool_liquidity: pubkeys[14],
            gamma_token_0account: pubkeys[15],
            gamma_token_1account: pubkeys[16],
            gamma_token_0vault: pubkeys[17],
            gamma_token_1vault: pubkeys[18],
            gamma_vault_0mint: pubkeys[19],
            gamma_vault_1mint: pubkeys[20],
            token_program: pubkeys[21],
            token_program_2022: pubkeys[22],
        }
    }
}
impl<'info> From<MigrateRaydiumCpSwapToGammaAccounts<'_, 'info>>
    for [AccountInfo<'info>; MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN]
{
    fn from(accounts: MigrateRaydiumCpSwapToGammaAccounts<'_, 'info>) -> Self {
        [
            accounts.raydium_cp_swap_program.clone(),
            accounts.owner.clone(),
            accounts.raydium_cp_swap_authority.clone(),
            accounts.raydium_cp_swap_pool_state.clone(),
            accounts.raydium_cp_swap_owner_lp_token.clone(),
            accounts.raydium_cp_swap_token_0vault.clone(),
            accounts.raydium_cp_swap_token_1vault.clone(),
            accounts.raydium_cp_swap_vault_0mint.clone(),
            accounts.raydium_cp_swap_vault_1mint.clone(),
            accounts.raydium_cp_swap_lp_mint.clone(),
            accounts.memo_program.clone(),
            accounts.gamma_owner.clone(),
            accounts.gamma_authority.clone(),
            accounts.gamma_pool_state.clone(),
            accounts.gamma_user_pool_liquidity.clone(),
            accounts.gamma_token_0account.clone(),
            accounts.gamma_token_1account.clone(),
            accounts.gamma_token_0vault.clone(),
            accounts.gamma_token_1vault.clone(),
            accounts.gamma_vault_0mint.clone(),
            accounts.gamma_vault_1mint.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN]>
    for MigrateRaydiumCpSwapToGammaAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            raydium_cp_swap_program: &arr[0],
            owner: &arr[1],
            raydium_cp_swap_authority: &arr[2],
            raydium_cp_swap_pool_state: &arr[3],
            raydium_cp_swap_owner_lp_token: &arr[4],
            raydium_cp_swap_token_0vault: &arr[5],
            raydium_cp_swap_token_1vault: &arr[6],
            raydium_cp_swap_vault_0mint: &arr[7],
            raydium_cp_swap_vault_1mint: &arr[8],
            raydium_cp_swap_lp_mint: &arr[9],
            memo_program: &arr[10],
            gamma_owner: &arr[11],
            gamma_authority: &arr[12],
            gamma_pool_state: &arr[13],
            gamma_user_pool_liquidity: &arr[14],
            gamma_token_0account: &arr[15],
            gamma_token_1account: &arr[16],
            gamma_token_0vault: &arr[17],
            gamma_token_1vault: &arr[18],
            gamma_vault_0mint: &arr[19],
            gamma_vault_1mint: &arr[20],
            token_program: &arr[21],
            token_program_2022: &arr[22],
        }
    }
}
pub const MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_DISCM: [u8; 8] = [39, 52, 202, 253, 131, 163, 42, 16];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct MigrateRaydiumCpSwapToGammaIxArgs {
    pub lp_token_amount_withdraw: u64,
    pub minimum_token_0amount: u64,
    pub minimum_token_1amount: u64,
    pub maximum_token_0amount: u64,
    pub maximum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MigrateRaydiumCpSwapToGammaIxData(pub MigrateRaydiumCpSwapToGammaIxArgs);
impl From<MigrateRaydiumCpSwapToGammaIxArgs> for MigrateRaydiumCpSwapToGammaIxData {
    fn from(args: MigrateRaydiumCpSwapToGammaIxArgs) -> Self {
        Self(args)
    }
}
impl MigrateRaydiumCpSwapToGammaIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_DISCM {
            return Err(std::io::Error::other(
                format!(
                    "discm does not match. Expected: {MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_DISCM:?}. Received: {maybe_discm:?}"
                ),
            ));
        }
        Ok(Self(MigrateRaydiumCpSwapToGammaIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn migrate_raydium_cp_swap_to_gamma_ix_with_program_id(
    program_id: Pubkey,
    keys: MigrateRaydiumCpSwapToGammaKeys,
    args: MigrateRaydiumCpSwapToGammaIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; MIGRATE_RAYDIUM_CP_SWAP_TO_GAMMA_IX_ACCOUNTS_LEN] = keys.into();
    let data: MigrateRaydiumCpSwapToGammaIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn migrate_raydium_cp_swap_to_gamma_ix(
    keys: MigrateRaydiumCpSwapToGammaKeys,
    args: MigrateRaydiumCpSwapToGammaIxArgs,
) -> std::io::Result<Instruction> {
    migrate_raydium_cp_swap_to_gamma_ix_with_program_id(super::ID, keys, args)
}

pub fn migrate_raydium_cp_swap_to_gamma_verify_account_keys(
    accounts: MigrateRaydiumCpSwapToGammaAccounts<'_, '_>,
    keys: MigrateRaydiumCpSwapToGammaKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.raydium_cp_swap_program.key, keys.raydium_cp_swap_program),
        (*accounts.owner.key, keys.owner),
        (*accounts.raydium_cp_swap_authority.key, keys.raydium_cp_swap_authority),
        (
            *accounts.raydium_cp_swap_pool_state.key,
            keys.raydium_cp_swap_pool_state,
        ),
        (
            *accounts.raydium_cp_swap_owner_lp_token.key,
            keys.raydium_cp_swap_owner_lp_token,
        ),
        (
            *accounts.raydium_cp_swap_token_0vault.key,
            keys.raydium_cp_swap_token_0vault,
        ),
        (
            *accounts.raydium_cp_swap_token_1vault.key,
            keys.raydium_cp_swap_token_1vault,
        ),
        (
            *accounts.raydium_cp_swap_vault_0mint.key,
            keys.raydium_cp_swap_vault_0mint,
        ),
        (
            *accounts.raydium_cp_swap_vault_1mint.key,
            keys.raydium_cp_swap_vault_1mint,
        ),
        (*accounts.raydium_cp_swap_lp_mint.key, keys.raydium_cp_swap_lp_mint),
        (*accounts.memo_program.key, keys.memo_program),
        (*accounts.gamma_owner.key, keys.gamma_owner),
        (*accounts.gamma_authority.key, keys.gamma_authority),
        (*accounts.gamma_pool_state.key, keys.gamma_pool_state),
        (*accounts.gamma_user_pool_liquidity.key, keys.gamma_user_pool_liquidity),
        (*accounts.gamma_token_0account.key, keys.gamma_token_0account),
        (*accounts.gamma_token_1account.key, keys.gamma_token_1account),
        (*accounts.gamma_token_0vault.key, keys.gamma_token_0vault),
        (*accounts.gamma_token_1vault.key, keys.gamma_token_1vault),
        (*accounts.gamma_vault_0mint.key, keys.gamma_vault_0mint),
        (*accounts.gamma_vault_1mint.key, keys.gamma_vault_1mint),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn migrate_raydium_cp_swap_to_gamma_verify_writable_privileges<'me, 'info>(
    accounts: MigrateRaydiumCpSwapToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.owner,
        accounts.raydium_cp_swap_pool_state,
        accounts.raydium_cp_swap_owner_lp_token,
        accounts.raydium_cp_swap_token_0vault,
        accounts.raydium_cp_swap_token_1vault,
        accounts.raydium_cp_swap_lp_mint,
        accounts.gamma_pool_state,
        accounts.gamma_user_pool_liquidity,
        accounts.gamma_token_0account,
        accounts.gamma_token_1account,
        accounts.gamma_token_0vault,
        accounts.gamma_token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn migrate_raydium_cp_swap_to_gamma_verify_signer_privileges<'me, 'info>(
    accounts: MigrateRaydiumCpSwapToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.owner, accounts.gamma_owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn migrate_raydium_cp_swap_to_gamma_verify_account_privileges<'me, 'info>(
    accounts: MigrateRaydiumCpSwapToGammaAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    migrate_raydium_cp_swap_to_gamma_verify_writable_privileges(accounts)?;
    migrate_raydium_cp_swap_to_gamma_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const REBALANCE_KAMINO_IX_ACCOUNTS_LEN: usize = 18;
#[derive(Copy, Clone, Debug)]
pub struct RebalanceKaminoAccounts<'me, 'info> {
    pub signer: &'me AccountInfo<'info>,
    pub gamma_authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub token_vault: &'me AccountInfo<'info>,
    pub token_mint: &'me AccountInfo<'info>,
    pub kamino_reserve: &'me AccountInfo<'info>,
    pub kamino_lending_market: &'me AccountInfo<'info>,
    pub lending_market_authority: &'me AccountInfo<'info>,
    pub reserve_liquidity_supply: &'me AccountInfo<'info>,
    pub reserve_collateral_mint: &'me AccountInfo<'info>,
    pub gamma_pool_destination_collateral: &'me AccountInfo<'info>,
    pub instruction_sysvar_account: &'me AccountInfo<'info>,
    pub liquidity_token_program: &'me AccountInfo<'info>,
    pub collateral_token_program: &'me AccountInfo<'info>,
    pub kamino_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RebalanceKaminoKeys {
    pub signer: Pubkey,
    pub gamma_authority: Pubkey,
    pub pool_state: Pubkey,
    pub token_vault: Pubkey,
    pub token_mint: Pubkey,
    pub kamino_reserve: Pubkey,
    pub kamino_lending_market: Pubkey,
    pub lending_market_authority: Pubkey,
    pub reserve_liquidity_supply: Pubkey,
    pub reserve_collateral_mint: Pubkey,
    pub gamma_pool_destination_collateral: Pubkey,
    pub instruction_sysvar_account: Pubkey,
    pub liquidity_token_program: Pubkey,
    pub collateral_token_program: Pubkey,
    pub kamino_program: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub system_program: Pubkey,
}
impl From<RebalanceKaminoAccounts<'_, '_>> for RebalanceKaminoKeys {
    fn from(accounts: RebalanceKaminoAccounts) -> Self {
        Self {
            signer: *accounts.signer.key,
            gamma_authority: *accounts.gamma_authority.key,
            pool_state: *accounts.pool_state.key,
            token_vault: *accounts.token_vault.key,
            token_mint: *accounts.token_mint.key,
            kamino_reserve: *accounts.kamino_reserve.key,
            kamino_lending_market: *accounts.kamino_lending_market.key,
            lending_market_authority: *accounts.lending_market_authority.key,
            reserve_liquidity_supply: *accounts.reserve_liquidity_supply.key,
            reserve_collateral_mint: *accounts.reserve_collateral_mint.key,
            gamma_pool_destination_collateral: *accounts.gamma_pool_destination_collateral.key,
            instruction_sysvar_account: *accounts.instruction_sysvar_account.key,
            liquidity_token_program: *accounts.liquidity_token_program.key,
            collateral_token_program: *accounts.collateral_token_program.key,
            kamino_program: *accounts.kamino_program.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<RebalanceKaminoKeys> for [AccountMeta; REBALANCE_KAMINO_IX_ACCOUNTS_LEN] {
    fn from(keys: RebalanceKaminoKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.signer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.kamino_reserve,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.kamino_lending_market,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.lending_market_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reserve_liquidity_supply,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reserve_collateral_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.gamma_pool_destination_collateral,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.instruction_sysvar_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.liquidity_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.collateral_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.kamino_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; REBALANCE_KAMINO_IX_ACCOUNTS_LEN]> for RebalanceKaminoKeys {
    fn from(pubkeys: [Pubkey; REBALANCE_KAMINO_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            signer: pubkeys[0],
            gamma_authority: pubkeys[1],
            pool_state: pubkeys[2],
            token_vault: pubkeys[3],
            token_mint: pubkeys[4],
            kamino_reserve: pubkeys[5],
            kamino_lending_market: pubkeys[6],
            lending_market_authority: pubkeys[7],
            reserve_liquidity_supply: pubkeys[8],
            reserve_collateral_mint: pubkeys[9],
            gamma_pool_destination_collateral: pubkeys[10],
            instruction_sysvar_account: pubkeys[11],
            liquidity_token_program: pubkeys[12],
            collateral_token_program: pubkeys[13],
            kamino_program: pubkeys[14],
            token_program: pubkeys[15],
            token_program_2022: pubkeys[16],
            system_program: pubkeys[17],
        }
    }
}
impl<'info> From<RebalanceKaminoAccounts<'_, 'info>> for [AccountInfo<'info>; REBALANCE_KAMINO_IX_ACCOUNTS_LEN] {
    fn from(accounts: RebalanceKaminoAccounts<'_, 'info>) -> Self {
        [
            accounts.signer.clone(),
            accounts.gamma_authority.clone(),
            accounts.pool_state.clone(),
            accounts.token_vault.clone(),
            accounts.token_mint.clone(),
            accounts.kamino_reserve.clone(),
            accounts.kamino_lending_market.clone(),
            accounts.lending_market_authority.clone(),
            accounts.reserve_liquidity_supply.clone(),
            accounts.reserve_collateral_mint.clone(),
            accounts.gamma_pool_destination_collateral.clone(),
            accounts.instruction_sysvar_account.clone(),
            accounts.liquidity_token_program.clone(),
            accounts.collateral_token_program.clone(),
            accounts.kamino_program.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; REBALANCE_KAMINO_IX_ACCOUNTS_LEN]>
    for RebalanceKaminoAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; REBALANCE_KAMINO_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            signer: &arr[0],
            gamma_authority: &arr[1],
            pool_state: &arr[2],
            token_vault: &arr[3],
            token_mint: &arr[4],
            kamino_reserve: &arr[5],
            kamino_lending_market: &arr[6],
            lending_market_authority: &arr[7],
            reserve_liquidity_supply: &arr[8],
            reserve_collateral_mint: &arr[9],
            gamma_pool_destination_collateral: &arr[10],
            instruction_sysvar_account: &arr[11],
            liquidity_token_program: &arr[12],
            collateral_token_program: &arr[13],
            kamino_program: &arr[14],
            token_program: &arr[15],
            token_program_2022: &arr[16],
            system_program: &arr[17],
        }
    }
}
pub const REBALANCE_KAMINO_IX_DISCM: [u8; 8] = [153, 94, 34, 16, 92, 181, 147, 215];
#[derive(Clone, Debug, PartialEq)]
pub struct RebalanceKaminoIxData;
impl RebalanceKaminoIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != REBALANCE_KAMINO_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {REBALANCE_KAMINO_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&REBALANCE_KAMINO_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn rebalance_kamino_ix_with_program_id(
    program_id: Pubkey,
    keys: RebalanceKaminoKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; REBALANCE_KAMINO_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: RebalanceKaminoIxData.try_to_vec()?,
    })
}
pub fn rebalance_kamino_ix(keys: RebalanceKaminoKeys) -> std::io::Result<Instruction> {
    rebalance_kamino_ix_with_program_id(super::ID, keys)
}

pub fn rebalance_kamino_verify_account_keys(
    accounts: RebalanceKaminoAccounts<'_, '_>,
    keys: RebalanceKaminoKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.signer.key, keys.signer),
        (*accounts.gamma_authority.key, keys.gamma_authority),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.token_vault.key, keys.token_vault),
        (*accounts.token_mint.key, keys.token_mint),
        (*accounts.kamino_reserve.key, keys.kamino_reserve),
        (*accounts.kamino_lending_market.key, keys.kamino_lending_market),
        (*accounts.lending_market_authority.key, keys.lending_market_authority),
        (*accounts.reserve_liquidity_supply.key, keys.reserve_liquidity_supply),
        (*accounts.reserve_collateral_mint.key, keys.reserve_collateral_mint),
        (
            *accounts.gamma_pool_destination_collateral.key,
            keys.gamma_pool_destination_collateral,
        ),
        (
            *accounts.instruction_sysvar_account.key,
            keys.instruction_sysvar_account,
        ),
        (*accounts.liquidity_token_program.key, keys.liquidity_token_program),
        (*accounts.collateral_token_program.key, keys.collateral_token_program),
        (*accounts.kamino_program.key, keys.kamino_program),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.token_program_2022.key, keys.token_program_2022),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn rebalance_kamino_verify_writable_privileges<'me, 'info>(
    accounts: RebalanceKaminoAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.signer,
        accounts.pool_state,
        accounts.token_vault,
        accounts.token_mint,
        accounts.kamino_reserve,
        accounts.kamino_lending_market,
        accounts.reserve_liquidity_supply,
        accounts.reserve_collateral_mint,
        accounts.gamma_pool_destination_collateral,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn rebalance_kamino_verify_signer_privileges<'me, 'info>(
    accounts: RebalanceKaminoAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.signer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn rebalance_kamino_verify_account_privileges<'me, 'info>(
    accounts: RebalanceKaminoAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    rebalance_kamino_verify_writable_privileges(accounts)?;
    rebalance_kamino_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SWAP_BASE_INPUT_IX_ACCOUNTS_LEN: usize = 13;
#[derive(Copy, Clone, Debug)]
pub struct SwapBaseInputAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub input_token_account: &'me AccountInfo<'info>,
    pub output_token_account: &'me AccountInfo<'info>,
    pub input_vault: &'me AccountInfo<'info>,
    pub output_vault: &'me AccountInfo<'info>,
    pub input_token_program: &'me AccountInfo<'info>,
    pub output_token_program: &'me AccountInfo<'info>,
    pub input_token_mint: &'me AccountInfo<'info>,
    pub output_token_mint: &'me AccountInfo<'info>,
    pub observation_state: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapBaseInputKeys {
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub input_token_program: Pubkey,
    pub output_token_program: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub observation_state: Pubkey,
}
impl From<SwapBaseInputAccounts<'_, '_>> for SwapBaseInputKeys {
    fn from(accounts: SwapBaseInputAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            authority: *accounts.authority.key,
            amm_config: *accounts.amm_config.key,
            pool_state: *accounts.pool_state.key,
            input_token_account: *accounts.input_token_account.key,
            output_token_account: *accounts.output_token_account.key,
            input_vault: *accounts.input_vault.key,
            output_vault: *accounts.output_vault.key,
            input_token_program: *accounts.input_token_program.key,
            output_token_program: *accounts.output_token_program.key,
            input_token_mint: *accounts.input_token_mint.key,
            output_token_mint: *accounts.output_token_mint.key,
            observation_state: *accounts.observation_state.key,
        }
    }
}
impl From<SwapBaseInputKeys> for [AccountMeta; SWAP_BASE_INPUT_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapBaseInputKeys) -> Self {
        [
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
        ]
    }
}
impl From<[Pubkey; SWAP_BASE_INPUT_IX_ACCOUNTS_LEN]> for SwapBaseInputKeys {
    fn from(pubkeys: [Pubkey; SWAP_BASE_INPUT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            authority: pubkeys[1],
            amm_config: pubkeys[2],
            pool_state: pubkeys[3],
            input_token_account: pubkeys[4],
            output_token_account: pubkeys[5],
            input_vault: pubkeys[6],
            output_vault: pubkeys[7],
            input_token_program: pubkeys[8],
            output_token_program: pubkeys[9],
            input_token_mint: pubkeys[10],
            output_token_mint: pubkeys[11],
            observation_state: pubkeys[12],
        }
    }
}
impl<'info> From<SwapBaseInputAccounts<'_, 'info>> for [AccountInfo<'info>; SWAP_BASE_INPUT_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapBaseInputAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.authority.clone(),
            accounts.amm_config.clone(),
            accounts.pool_state.clone(),
            accounts.input_token_account.clone(),
            accounts.output_token_account.clone(),
            accounts.input_vault.clone(),
            accounts.output_vault.clone(),
            accounts.input_token_program.clone(),
            accounts.output_token_program.clone(),
            accounts.input_token_mint.clone(),
            accounts.output_token_mint.clone(),
            accounts.observation_state.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_BASE_INPUT_IX_ACCOUNTS_LEN]>
    for SwapBaseInputAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; SWAP_BASE_INPUT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            authority: &arr[1],
            amm_config: &arr[2],
            pool_state: &arr[3],
            input_token_account: &arr[4],
            output_token_account: &arr[5],
            input_vault: &arr[6],
            output_vault: &arr[7],
            input_token_program: &arr[8],
            output_token_program: &arr[9],
            input_token_mint: &arr[10],
            output_token_mint: &arr[11],
            observation_state: &arr[12],
        }
    }
}
pub const SWAP_BASE_INPUT_IX_DISCM: [u8; 8] = [239, 82, 192, 187, 160, 26, 223, 223]; //[143, 190, 90, 218, 196, 30, 51, 222];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapBaseInputIxArgs {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapBaseInputIxData(pub SwapBaseInputIxArgs);
impl From<SwapBaseInputIxArgs> for SwapBaseInputIxData {
    fn from(args: SwapBaseInputIxArgs) -> Self {
        Self(args)
    }
}
impl SwapBaseInputIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SWAP_BASE_INPUT_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {SWAP_BASE_INPUT_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(SwapBaseInputIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SWAP_BASE_INPUT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn swap_base_input_ix_with_program_id(
    program_id: Pubkey,
    keys: SwapBaseInputKeys,
    args: SwapBaseInputIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SWAP_BASE_INPUT_IX_ACCOUNTS_LEN] = keys.into();
    let data: SwapBaseInputIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn swap_base_input_ix(keys: SwapBaseInputKeys, args: SwapBaseInputIxArgs) -> std::io::Result<Instruction> {
    swap_base_input_ix_with_program_id(super::ID, keys, args)
}

pub fn swap_base_input_verify_account_keys(
    accounts: SwapBaseInputAccounts<'_, '_>,
    keys: SwapBaseInputKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.payer.key, keys.payer),
        (*accounts.authority.key, keys.authority),
        (*accounts.amm_config.key, keys.amm_config),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.input_token_account.key, keys.input_token_account),
        (*accounts.output_token_account.key, keys.output_token_account),
        (*accounts.input_vault.key, keys.input_vault),
        (*accounts.output_vault.key, keys.output_vault),
        (*accounts.input_token_program.key, keys.input_token_program),
        (*accounts.output_token_program.key, keys.output_token_program),
        (*accounts.input_token_mint.key, keys.input_token_mint),
        (*accounts.output_token_mint.key, keys.output_token_mint),
        (*accounts.observation_state.key, keys.observation_state),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn swap_base_input_verify_writable_privileges<'me, 'info>(
    accounts: SwapBaseInputAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool_state,
        accounts.input_token_account,
        accounts.output_token_account,
        accounts.input_vault,
        accounts.output_vault,
        accounts.observation_state,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn swap_base_input_verify_signer_privileges<'me, 'info>(
    accounts: SwapBaseInputAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn swap_base_input_verify_account_privileges<'me, 'info>(
    accounts: SwapBaseInputAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    swap_base_input_verify_writable_privileges(accounts)?;
    swap_base_input_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN: usize = 13;
#[derive(Copy, Clone, Debug)]
pub struct SwapBaseOutputAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub input_token_account: &'me AccountInfo<'info>,
    pub output_token_account: &'me AccountInfo<'info>,
    pub input_vault: &'me AccountInfo<'info>,
    pub output_vault: &'me AccountInfo<'info>,
    pub input_token_program: &'me AccountInfo<'info>,
    pub output_token_program: &'me AccountInfo<'info>,
    pub input_token_mint: &'me AccountInfo<'info>,
    pub output_token_mint: &'me AccountInfo<'info>,
    pub observation_state: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapBaseOutputKeys {
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub input_token_program: Pubkey,
    pub output_token_program: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub observation_state: Pubkey,
}
impl From<SwapBaseOutputAccounts<'_, '_>> for SwapBaseOutputKeys {
    fn from(accounts: SwapBaseOutputAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            authority: *accounts.authority.key,
            amm_config: *accounts.amm_config.key,
            pool_state: *accounts.pool_state.key,
            input_token_account: *accounts.input_token_account.key,
            output_token_account: *accounts.output_token_account.key,
            input_vault: *accounts.input_vault.key,
            output_vault: *accounts.output_vault.key,
            input_token_program: *accounts.input_token_program.key,
            output_token_program: *accounts.output_token_program.key,
            input_token_mint: *accounts.input_token_mint.key,
            output_token_mint: *accounts.output_token_mint.key,
            observation_state: *accounts.observation_state.key,
        }
    }
}
impl From<SwapBaseOutputKeys> for [AccountMeta; SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapBaseOutputKeys) -> Self {
        [
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
        ]
    }
}
impl From<[Pubkey; SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN]> for SwapBaseOutputKeys {
    fn from(pubkeys: [Pubkey; SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            authority: pubkeys[1],
            amm_config: pubkeys[2],
            pool_state: pubkeys[3],
            input_token_account: pubkeys[4],
            output_token_account: pubkeys[5],
            input_vault: pubkeys[6],
            output_vault: pubkeys[7],
            input_token_program: pubkeys[8],
            output_token_program: pubkeys[9],
            input_token_mint: pubkeys[10],
            output_token_mint: pubkeys[11],
            observation_state: pubkeys[12],
        }
    }
}
impl<'info> From<SwapBaseOutputAccounts<'_, 'info>> for [AccountInfo<'info>; SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapBaseOutputAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.authority.clone(),
            accounts.amm_config.clone(),
            accounts.pool_state.clone(),
            accounts.input_token_account.clone(),
            accounts.output_token_account.clone(),
            accounts.input_vault.clone(),
            accounts.output_vault.clone(),
            accounts.input_token_program.clone(),
            accounts.output_token_program.clone(),
            accounts.input_token_mint.clone(),
            accounts.output_token_mint.clone(),
            accounts.observation_state.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN]>
    for SwapBaseOutputAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            authority: &arr[1],
            amm_config: &arr[2],
            pool_state: &arr[3],
            input_token_account: &arr[4],
            output_token_account: &arr[5],
            input_vault: &arr[6],
            output_vault: &arr[7],
            input_token_program: &arr[8],
            output_token_program: &arr[9],
            input_token_mint: &arr[10],
            output_token_mint: &arr[11],
            observation_state: &arr[12],
        }
    }
}
pub const SWAP_BASE_OUTPUT_IX_DISCM: [u8; 8] = [55, 217, 98, 86, 163, 74, 180, 173];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapBaseOutputIxArgs {
    pub max_amount_in: u64,
    pub amount_out: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapBaseOutputIxData(pub SwapBaseOutputIxArgs);
impl From<SwapBaseOutputIxArgs> for SwapBaseOutputIxData {
    fn from(args: SwapBaseOutputIxArgs) -> Self {
        Self(args)
    }
}
impl SwapBaseOutputIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SWAP_BASE_OUTPUT_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {SWAP_BASE_OUTPUT_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(SwapBaseOutputIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SWAP_BASE_OUTPUT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn swap_base_output_ix_with_program_id(
    program_id: Pubkey,
    keys: SwapBaseOutputKeys,
    args: SwapBaseOutputIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SWAP_BASE_OUTPUT_IX_ACCOUNTS_LEN] = keys.into();
    let data: SwapBaseOutputIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn swap_base_output_ix(keys: SwapBaseOutputKeys, args: SwapBaseOutputIxArgs) -> std::io::Result<Instruction> {
    swap_base_output_ix_with_program_id(super::ID, keys, args)
}

pub fn swap_base_output_verify_account_keys(
    accounts: SwapBaseOutputAccounts<'_, '_>,
    keys: SwapBaseOutputKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.payer.key, keys.payer),
        (*accounts.authority.key, keys.authority),
        (*accounts.amm_config.key, keys.amm_config),
        (*accounts.pool_state.key, keys.pool_state),
        (*accounts.input_token_account.key, keys.input_token_account),
        (*accounts.output_token_account.key, keys.output_token_account),
        (*accounts.input_vault.key, keys.input_vault),
        (*accounts.output_vault.key, keys.output_vault),
        (*accounts.input_token_program.key, keys.input_token_program),
        (*accounts.output_token_program.key, keys.output_token_program),
        (*accounts.input_token_mint.key, keys.input_token_mint),
        (*accounts.output_token_mint.key, keys.output_token_mint),
        (*accounts.observation_state.key, keys.observation_state),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn swap_base_output_verify_writable_privileges<'me, 'info>(
    accounts: SwapBaseOutputAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool_state,
        accounts.input_token_account,
        accounts.output_token_account,
        accounts.input_vault,
        accounts.output_vault,
        accounts.observation_state,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn swap_base_output_verify_signer_privileges<'me, 'info>(
    accounts: SwapBaseOutputAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn swap_base_output_verify_account_privileges<'me, 'info>(
    accounts: SwapBaseOutputAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    swap_base_output_verify_writable_privileges(accounts)?;
    swap_base_output_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN: usize = 2;
#[derive(Copy, Clone, Debug)]
pub struct UpdateAmmConfigAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdateAmmConfigKeys {
    pub owner: Pubkey,
    pub amm_config: Pubkey,
}
impl From<UpdateAmmConfigAccounts<'_, '_>> for UpdateAmmConfigKeys {
    fn from(accounts: UpdateAmmConfigAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            amm_config: *accounts.amm_config.key,
        }
    }
}
impl From<UpdateAmmConfigKeys> for [AccountMeta; UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdateAmmConfigKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: true,
            },
        ]
    }
}
impl From<[Pubkey; UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN]> for UpdateAmmConfigKeys {
    fn from(pubkeys: [Pubkey; UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            amm_config: pubkeys[1],
        }
    }
}
impl<'info> From<UpdateAmmConfigAccounts<'_, 'info>> for [AccountInfo<'info>; UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdateAmmConfigAccounts<'_, 'info>) -> Self {
        [accounts.owner.clone(), accounts.amm_config.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN]>
    for UpdateAmmConfigAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            amm_config: &arr[1],
        }
    }
}
pub const UPDATE_AMM_CONFIG_IX_DISCM: [u8; 8] = [49, 60, 174, 136, 154, 28, 116, 200];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdateAmmConfigIxArgs {
    pub param: u16,
    pub value: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateAmmConfigIxData(pub UpdateAmmConfigIxArgs);
impl From<UpdateAmmConfigIxArgs> for UpdateAmmConfigIxData {
    fn from(args: UpdateAmmConfigIxArgs) -> Self {
        Self(args)
    }
}
impl UpdateAmmConfigIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UPDATE_AMM_CONFIG_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {UPDATE_AMM_CONFIG_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(UpdateAmmConfigIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UPDATE_AMM_CONFIG_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn update_amm_config_ix_with_program_id(
    program_id: Pubkey,
    keys: UpdateAmmConfigKeys,
    args: UpdateAmmConfigIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UPDATE_AMM_CONFIG_IX_ACCOUNTS_LEN] = keys.into();
    let data: UpdateAmmConfigIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn update_amm_config_ix(keys: UpdateAmmConfigKeys, args: UpdateAmmConfigIxArgs) -> std::io::Result<Instruction> {
    update_amm_config_ix_with_program_id(super::ID, keys, args)
}
pub fn update_amm_config_verify_writable_privileges<'me, 'info>(
    accounts: UpdateAmmConfigAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.amm_config] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn update_amm_config_verify_signer_privileges<'me, 'info>(
    accounts: UpdateAmmConfigAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn update_amm_config_verify_account_privileges<'me, 'info>(
    accounts: UpdateAmmConfigAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    update_amm_config_verify_writable_privileges(accounts)?;
    update_amm_config_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UPDATE_POOL_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct UpdatePoolAccounts<'me, 'info> {
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdatePoolKeys {
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub amm_config: Pubkey,
}
impl From<UpdatePoolAccounts<'_, '_>> for UpdatePoolKeys {
    fn from(accounts: UpdatePoolAccounts) -> Self {
        Self {
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            amm_config: *accounts.amm_config.key,
        }
    }
}
impl From<UpdatePoolKeys> for [AccountMeta; UPDATE_POOL_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdatePoolKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.amm_config,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; UPDATE_POOL_IX_ACCOUNTS_LEN]> for UpdatePoolKeys {
    fn from(pubkeys: [Pubkey; UPDATE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: pubkeys[0],
            pool_state: pubkeys[1],
            amm_config: pubkeys[2],
        }
    }
}
impl<'info> From<UpdatePoolAccounts<'_, 'info>> for [AccountInfo<'info>; UPDATE_POOL_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdatePoolAccounts<'_, 'info>) -> Self {
        [
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.amm_config.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_POOL_IX_ACCOUNTS_LEN]> for UpdatePoolAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: &arr[0],
            pool_state: &arr[1],
            amm_config: &arr[2],
        }
    }
}
pub const UPDATE_POOL_IX_DISCM: [u8; 8] = [239, 214, 170, 78, 36, 35, 30, 34];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdatePoolIxArgs {
    pub param: u32,
    pub value: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdatePoolIxData(pub UpdatePoolIxArgs);
impl From<UpdatePoolIxArgs> for UpdatePoolIxData {
    fn from(args: UpdatePoolIxArgs) -> Self {
        Self(args)
    }
}
impl UpdatePoolIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UPDATE_POOL_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {UPDATE_POOL_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(UpdatePoolIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UPDATE_POOL_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn update_pool_ix_with_program_id(
    program_id: Pubkey,
    keys: UpdatePoolKeys,
    args: UpdatePoolIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UPDATE_POOL_IX_ACCOUNTS_LEN] = keys.into();
    let data: UpdatePoolIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn update_pool_ix(keys: UpdatePoolKeys, args: UpdatePoolIxArgs) -> std::io::Result<Instruction> {
    update_pool_ix_with_program_id(super::ID, keys, args)
}
pub fn update_pool_verify_signer_privileges<'me, 'info>(
    accounts: UpdatePoolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub const WITHDRAW_IX_ACCOUNTS_LEN: usize = 13;
#[derive(Copy, Clone, Debug)]
pub struct WithdrawAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub user_pool_liquidity: &'me AccountInfo<'info>,
    pub token_0account: &'me AccountInfo<'info>,
    pub token_1account: &'me AccountInfo<'info>,
    pub token_0vault: &'me AccountInfo<'info>,
    pub token_1vault: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program_2022: &'me AccountInfo<'info>,
    pub vault_0mint: &'me AccountInfo<'info>,
    pub vault_1mint: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WithdrawKeys {
    pub owner: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub user_pool_liquidity: Pubkey,
    pub token_0account: Pubkey,
    pub token_1account: Pubkey,
    pub token_0vault: Pubkey,
    pub token_1vault: Pubkey,
    pub token_program: Pubkey,
    pub token_program_2022: Pubkey,
    pub vault_0mint: Pubkey,
    pub vault_1mint: Pubkey,
    pub memo_program: Pubkey,
}
impl From<WithdrawAccounts<'_, '_>> for WithdrawKeys {
    fn from(accounts: WithdrawAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
            user_pool_liquidity: *accounts.user_pool_liquidity.key,
            token_0account: *accounts.token_0account.key,
            token_1account: *accounts.token_1account.key,
            token_0vault: *accounts.token_0vault.key,
            token_1vault: *accounts.token_1vault.key,
            token_program: *accounts.token_program.key,
            token_program_2022: *accounts.token_program_2022.key,
            vault_0mint: *accounts.vault_0mint.key,
            vault_1mint: *accounts.vault_1mint.key,
            memo_program: *accounts.memo_program.key,
        }
    }
}
impl From<WithdrawKeys> for [AccountMeta; WITHDRAW_IX_ACCOUNTS_LEN] {
    fn from(keys: WithdrawKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_pool_liquidity,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_0account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_1account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_0vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_1vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program_2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_0mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_1mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.memo_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]> for WithdrawKeys {
    fn from(pubkeys: [Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            authority: pubkeys[1],
            pool_state: pubkeys[2],
            user_pool_liquidity: pubkeys[3],
            token_0account: pubkeys[4],
            token_1account: pubkeys[5],
            token_0vault: pubkeys[6],
            token_1vault: pubkeys[7],
            token_program: pubkeys[8],
            token_program_2022: pubkeys[9],
            vault_0mint: pubkeys[10],
            vault_1mint: pubkeys[11],
            memo_program: pubkeys[12],
        }
    }
}
impl<'info> From<WithdrawAccounts<'_, 'info>> for [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN] {
    fn from(accounts: WithdrawAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.authority.clone(),
            accounts.pool_state.clone(),
            accounts.user_pool_liquidity.clone(),
            accounts.token_0account.clone(),
            accounts.token_1account.clone(),
            accounts.token_0vault.clone(),
            accounts.token_1vault.clone(),
            accounts.token_program.clone(),
            accounts.token_program_2022.clone(),
            accounts.vault_0mint.clone(),
            accounts.vault_1mint.clone(),
            accounts.memo_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]> for WithdrawAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            authority: &arr[1],
            pool_state: &arr[2],
            user_pool_liquidity: &arr[3],
            token_0account: &arr[4],
            token_1account: &arr[5],
            token_0vault: &arr[6],
            token_1vault: &arr[7],
            token_program: &arr[8],
            token_program_2022: &arr[9],
            vault_0mint: &arr[10],
            vault_1mint: &arr[11],
            memo_program: &arr[12],
        }
    }
}
pub const WITHDRAW_IX_DISCM: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct WithdrawIxArgs {
    pub lp_token_amount: u64,
    pub minimum_token_0amount: u64,
    pub minimum_token_1amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct WithdrawIxData(pub WithdrawIxArgs);
impl From<WithdrawIxArgs> for WithdrawIxData {
    fn from(args: WithdrawIxArgs) -> Self {
        Self(args)
    }
}
impl WithdrawIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != WITHDRAW_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {WITHDRAW_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(WithdrawIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&WITHDRAW_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn withdraw_ix_with_program_id(
    program_id: Pubkey,
    keys: WithdrawKeys,
    args: WithdrawIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; WITHDRAW_IX_ACCOUNTS_LEN] = keys.into();
    let data: WithdrawIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn withdraw_ix(keys: WithdrawKeys, args: WithdrawIxArgs) -> std::io::Result<Instruction> {
    withdraw_ix_with_program_id(super::ID, keys, args)
}
pub fn withdraw_verify_writable_privileges<'me, 'info>(
    accounts: WithdrawAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool_state,
        accounts.user_pool_liquidity,
        accounts.token_0account,
        accounts.token_1account,
        accounts.token_0vault,
        accounts.token_1vault,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn withdraw_verify_signer_privileges<'me, 'info>(
    accounts: WithdrawAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.owner] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn withdraw_verify_account_privileges<'me, 'info>(
    accounts: WithdrawAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    withdraw_verify_writable_privileges(accounts)?;
    withdraw_verify_signer_privileges(accounts)?;
    Ok(())
}
