#![allow(dead_code)]
use super::typedefs::InitializeRewardParam;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

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

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CreateAmmConfigIxArgs {
    pub index: u16,
    pub tick_spacing: u16,
    pub trade_fee_rate: u32,
    pub protocol_fee_rate: u32,
    pub fund_fee_rate: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreateAmmConfigIxData(pub CreateAmmConfigIxArgs);
impl From<CreateAmmConfigIxArgs> for CreateAmmConfigIxData {
    fn from(args: CreateAmmConfigIxArgs) -> Self {
        Self(args)
    }
}
impl CreateAmmConfigIxData {}

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

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdateAmmConfigIxArgs {
    pub param: u8,
    pub value: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateAmmConfigIxData(pub UpdateAmmConfigIxArgs);
impl From<UpdateAmmConfigIxArgs> for UpdateAmmConfigIxData {
    fn from(args: UpdateAmmConfigIxArgs) -> Self {
        Self(args)
    }
}
impl UpdateAmmConfigIxData {}

pub const CREATE_POOL_IX_ACCOUNTS_LEN: usize = 13;
#[derive(Copy, Clone, Debug)]
pub struct CreatePoolAccounts<'me, 'info> {
    pub pool_creator: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub token_mint0: &'me AccountInfo<'info>,
    pub token_mint1: &'me AccountInfo<'info>,
    pub token_vault0: &'me AccountInfo<'info>,
    pub token_vault1: &'me AccountInfo<'info>,
    pub observation_state: &'me AccountInfo<'info>,
    pub tick_array_bitmap: &'me AccountInfo<'info>,
    pub token_program0: &'me AccountInfo<'info>,
    pub token_program1: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreatePoolKeys {
    pub pool_creator: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub observation_state: Pubkey,
    pub tick_array_bitmap: Pubkey,
    pub token_program0: Pubkey,
    pub token_program1: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<CreatePoolAccounts<'_, '_>> for CreatePoolKeys {
    fn from(accounts: CreatePoolAccounts) -> Self {
        Self {
            pool_creator: *accounts.pool_creator.key,
            amm_config: *accounts.amm_config.key,
            pool_state: *accounts.pool_state.key,
            token_mint0: *accounts.token_mint0.key,
            token_mint1: *accounts.token_mint1.key,
            token_vault0: *accounts.token_vault0.key,
            token_vault1: *accounts.token_vault1.key,
            observation_state: *accounts.observation_state.key,
            tick_array_bitmap: *accounts.tick_array_bitmap.key,
            token_program0: *accounts.token_program0.key,
            token_program1: *accounts.token_program1.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<CreatePoolKeys> for [AccountMeta; CREATE_POOL_IX_ACCOUNTS_LEN] {
    fn from(keys: CreatePoolKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool_creator,
                is_signer: true,
                is_writable: true,
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
                pubkey: keys.token_mint0,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_mint1,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_vault0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.observation_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_bitmap,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program0,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program1,
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
impl From<[Pubkey; CREATE_POOL_IX_ACCOUNTS_LEN]> for CreatePoolKeys {
    fn from(pubkeys: [Pubkey; CREATE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool_creator: pubkeys[0],
            amm_config: pubkeys[1],
            pool_state: pubkeys[2],
            token_mint0: pubkeys[3],
            token_mint1: pubkeys[4],
            token_vault0: pubkeys[5],
            token_vault1: pubkeys[6],
            observation_state: pubkeys[7],
            tick_array_bitmap: pubkeys[8],
            token_program0: pubkeys[9],
            token_program1: pubkeys[10],
            system_program: pubkeys[11],
            rent: pubkeys[12],
        }
    }
}
impl<'info> From<CreatePoolAccounts<'_, 'info>> for [AccountInfo<'info>; CREATE_POOL_IX_ACCOUNTS_LEN] {
    fn from(accounts: CreatePoolAccounts<'_, 'info>) -> Self {
        [
            accounts.pool_creator.clone(),
            accounts.amm_config.clone(),
            accounts.pool_state.clone(),
            accounts.token_mint0.clone(),
            accounts.token_mint1.clone(),
            accounts.token_vault0.clone(),
            accounts.token_vault1.clone(),
            accounts.observation_state.clone(),
            accounts.tick_array_bitmap.clone(),
            accounts.token_program0.clone(),
            accounts.token_program1.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_POOL_IX_ACCOUNTS_LEN]> for CreatePoolAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CREATE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool_creator: &arr[0],
            amm_config: &arr[1],
            pool_state: &arr[2],
            token_mint0: &arr[3],
            token_mint1: &arr[4],
            token_vault0: &arr[5],
            token_vault1: &arr[6],
            observation_state: &arr[7],
            tick_array_bitmap: &arr[8],
            token_program0: &arr[9],
            token_program1: &arr[10],
            system_program: &arr[11],
            rent: &arr[12],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CreatePoolIxArgs {
    pub sqrt_price_x64: u128,
    pub open_time: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreatePoolIxData(pub CreatePoolIxArgs);
impl From<CreatePoolIxArgs> for CreatePoolIxData {
    fn from(args: CreatePoolIxArgs) -> Self {
        Self(args)
    }
}
impl CreatePoolIxData {}
pub const UPDATE_POOL_STATUS_IX_ACCOUNTS_LEN: usize = 2;
#[derive(Copy, Clone, Debug)]
pub struct UpdatePoolStatusAccounts<'me, 'info> {
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdatePoolStatusKeys {
    pub authority: Pubkey,
    pub pool_state: Pubkey,
}
impl From<UpdatePoolStatusAccounts<'_, '_>> for UpdatePoolStatusKeys {
    fn from(accounts: UpdatePoolStatusAccounts) -> Self {
        Self {
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
        }
    }
}
impl From<UpdatePoolStatusKeys> for [AccountMeta; UPDATE_POOL_STATUS_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdatePoolStatusKeys) -> Self {
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
        ]
    }
}
impl From<[Pubkey; UPDATE_POOL_STATUS_IX_ACCOUNTS_LEN]> for UpdatePoolStatusKeys {
    fn from(pubkeys: [Pubkey; UPDATE_POOL_STATUS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: pubkeys[0],
            pool_state: pubkeys[1],
        }
    }
}
impl<'info> From<UpdatePoolStatusAccounts<'_, 'info>> for [AccountInfo<'info>; UPDATE_POOL_STATUS_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdatePoolStatusAccounts<'_, 'info>) -> Self {
        [accounts.authority.clone(), accounts.pool_state.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_POOL_STATUS_IX_ACCOUNTS_LEN]>
    for UpdatePoolStatusAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_POOL_STATUS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: &arr[0],
            pool_state: &arr[1],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdatePoolStatusIxArgs {
    pub status: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdatePoolStatusIxData(pub UpdatePoolStatusIxArgs);
impl From<UpdatePoolStatusIxArgs> for UpdatePoolStatusIxData {
    fn from(args: UpdatePoolStatusIxArgs) -> Self {
        Self(args)
    }
}
impl UpdatePoolStatusIxData {}

pub const CREATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct CreateOperationAccountAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub operation_state: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreateOperationAccountKeys {
    pub owner: Pubkey,
    pub operation_state: Pubkey,
    pub system_program: Pubkey,
}
impl From<CreateOperationAccountAccounts<'_, '_>> for CreateOperationAccountKeys {
    fn from(accounts: CreateOperationAccountAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            operation_state: *accounts.operation_state.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<CreateOperationAccountKeys> for [AccountMeta; CREATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(keys: CreateOperationAccountKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.operation_state,
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
impl From<[Pubkey; CREATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]> for CreateOperationAccountKeys {
    fn from(pubkeys: [Pubkey; CREATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            operation_state: pubkeys[1],
            system_program: pubkeys[2],
        }
    }
}
impl<'info> From<CreateOperationAccountAccounts<'_, 'info>>
    for [AccountInfo<'info>; CREATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]
{
    fn from(accounts: CreateOperationAccountAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.operation_state.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]>
    for CreateOperationAccountAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; CREATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            operation_state: &arr[1],
            system_program: &arr[2],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateOperationAccountIxData;
impl CreateOperationAccountIxData {}

pub const UPDATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct UpdateOperationAccountAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub operation_state: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdateOperationAccountKeys {
    pub owner: Pubkey,
    pub operation_state: Pubkey,
    pub system_program: Pubkey,
}
impl From<UpdateOperationAccountAccounts<'_, '_>> for UpdateOperationAccountKeys {
    fn from(accounts: UpdateOperationAccountAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            operation_state: *accounts.operation_state.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<UpdateOperationAccountKeys> for [AccountMeta; UPDATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdateOperationAccountKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.operation_state,
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
impl From<[Pubkey; UPDATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]> for UpdateOperationAccountKeys {
    fn from(pubkeys: [Pubkey; UPDATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: pubkeys[0],
            operation_state: pubkeys[1],
            system_program: pubkeys[2],
        }
    }
}
impl<'info> From<UpdateOperationAccountAccounts<'_, 'info>>
    for [AccountInfo<'info>; UPDATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]
{
    fn from(accounts: UpdateOperationAccountAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.operation_state.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]>
    for UpdateOperationAccountAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_OPERATION_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            operation_state: &arr[1],
            system_program: &arr[2],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdateOperationAccountIxArgs {
    pub param: u8,
    pub keys: Vec<Pubkey>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateOperationAccountIxData(pub UpdateOperationAccountIxArgs);
impl From<UpdateOperationAccountIxArgs> for UpdateOperationAccountIxData {
    fn from(args: UpdateOperationAccountIxArgs) -> Self {
        Self(args)
    }
}
impl UpdateOperationAccountIxData {}

pub const TRANSFER_REWARD_OWNER_IX_ACCOUNTS_LEN: usize = 2;
#[derive(Copy, Clone, Debug)]
pub struct TransferRewardOwnerAccounts<'me, 'info> {
    pub authority: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TransferRewardOwnerKeys {
    pub authority: Pubkey,
    pub pool_state: Pubkey,
}
impl From<TransferRewardOwnerAccounts<'_, '_>> for TransferRewardOwnerKeys {
    fn from(accounts: TransferRewardOwnerAccounts) -> Self {
        Self {
            authority: *accounts.authority.key,
            pool_state: *accounts.pool_state.key,
        }
    }
}
impl From<TransferRewardOwnerKeys> for [AccountMeta; TRANSFER_REWARD_OWNER_IX_ACCOUNTS_LEN] {
    fn from(keys: TransferRewardOwnerKeys) -> Self {
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
        ]
    }
}
impl From<[Pubkey; TRANSFER_REWARD_OWNER_IX_ACCOUNTS_LEN]> for TransferRewardOwnerKeys {
    fn from(pubkeys: [Pubkey; TRANSFER_REWARD_OWNER_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: pubkeys[0],
            pool_state: pubkeys[1],
        }
    }
}
impl<'info> From<TransferRewardOwnerAccounts<'_, 'info>>
    for [AccountInfo<'info>; TRANSFER_REWARD_OWNER_IX_ACCOUNTS_LEN]
{
    fn from(accounts: TransferRewardOwnerAccounts<'_, 'info>) -> Self {
        [accounts.authority.clone(), accounts.pool_state.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; TRANSFER_REWARD_OWNER_IX_ACCOUNTS_LEN]>
    for TransferRewardOwnerAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; TRANSFER_REWARD_OWNER_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: &arr[0],
            pool_state: &arr[1],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct TransferRewardOwnerIxArgs {
    pub new_owner: Pubkey,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TransferRewardOwnerIxData(pub TransferRewardOwnerIxArgs);
impl From<TransferRewardOwnerIxArgs> for TransferRewardOwnerIxData {
    fn from(args: TransferRewardOwnerIxArgs) -> Self {
        Self(args)
    }
}
impl TransferRewardOwnerIxData {}

pub const INITIALIZE_REWARD_IX_ACCOUNTS_LEN: usize = 10;
#[derive(Copy, Clone, Debug)]
pub struct InitializeRewardAccounts<'me, 'info> {
    pub reward_funder: &'me AccountInfo<'info>,
    pub funder_token_account: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub operation_state: &'me AccountInfo<'info>,
    pub reward_token_mint: &'me AccountInfo<'info>,
    pub reward_token_vault: &'me AccountInfo<'info>,
    pub reward_token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InitializeRewardKeys {
    pub reward_funder: Pubkey,
    pub funder_token_account: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub operation_state: Pubkey,
    pub reward_token_mint: Pubkey,
    pub reward_token_vault: Pubkey,
    pub reward_token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<InitializeRewardAccounts<'_, '_>> for InitializeRewardKeys {
    fn from(accounts: InitializeRewardAccounts) -> Self {
        Self {
            reward_funder: *accounts.reward_funder.key,
            funder_token_account: *accounts.funder_token_account.key,
            amm_config: *accounts.amm_config.key,
            pool_state: *accounts.pool_state.key,
            operation_state: *accounts.operation_state.key,
            reward_token_mint: *accounts.reward_token_mint.key,
            reward_token_vault: *accounts.reward_token_vault.key,
            reward_token_program: *accounts.reward_token_program.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<InitializeRewardKeys> for [AccountMeta; INITIALIZE_REWARD_IX_ACCOUNTS_LEN] {
    fn from(keys: InitializeRewardKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.reward_funder,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.funder_token_account,
                is_signer: false,
                is_writable: true,
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
                pubkey: keys.operation_state,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reward_token_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reward_token_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reward_token_program,
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
impl From<[Pubkey; INITIALIZE_REWARD_IX_ACCOUNTS_LEN]> for InitializeRewardKeys {
    fn from(pubkeys: [Pubkey; INITIALIZE_REWARD_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            reward_funder: pubkeys[0],
            funder_token_account: pubkeys[1],
            amm_config: pubkeys[2],
            pool_state: pubkeys[3],
            operation_state: pubkeys[4],
            reward_token_mint: pubkeys[5],
            reward_token_vault: pubkeys[6],
            reward_token_program: pubkeys[7],
            system_program: pubkeys[8],
            rent: pubkeys[9],
        }
    }
}
impl<'info> From<InitializeRewardAccounts<'_, 'info>> for [AccountInfo<'info>; INITIALIZE_REWARD_IX_ACCOUNTS_LEN] {
    fn from(accounts: InitializeRewardAccounts<'_, 'info>) -> Self {
        [
            accounts.reward_funder.clone(),
            accounts.funder_token_account.clone(),
            accounts.amm_config.clone(),
            accounts.pool_state.clone(),
            accounts.operation_state.clone(),
            accounts.reward_token_mint.clone(),
            accounts.reward_token_vault.clone(),
            accounts.reward_token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INITIALIZE_REWARD_IX_ACCOUNTS_LEN]>
    for InitializeRewardAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; INITIALIZE_REWARD_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            reward_funder: &arr[0],
            funder_token_account: &arr[1],
            amm_config: &arr[2],
            pool_state: &arr[3],
            operation_state: &arr[4],
            reward_token_mint: &arr[5],
            reward_token_vault: &arr[6],
            reward_token_program: &arr[7],
            system_program: &arr[8],
            rent: &arr[9],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct InitializeRewardIxArgs {
    pub param: InitializeRewardParam,
}
#[derive(Clone, Debug, PartialEq)]
pub struct InitializeRewardIxData(pub InitializeRewardIxArgs);
impl From<InitializeRewardIxArgs> for InitializeRewardIxData {
    fn from(args: InitializeRewardIxArgs) -> Self {
        Self(args)
    }
}
impl InitializeRewardIxData {}

pub const COLLECT_REMAINING_REWARDS_IX_ACCOUNTS_LEN: usize = 8;
#[derive(Copy, Clone, Debug)]
pub struct CollectRemainingRewardsAccounts<'me, 'info> {
    pub reward_funder: &'me AccountInfo<'info>,
    pub funder_token_account: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub reward_token_vault: &'me AccountInfo<'info>,
    pub reward_vault_mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CollectRemainingRewardsKeys {
    pub reward_funder: Pubkey,
    pub funder_token_account: Pubkey,
    pub pool_state: Pubkey,
    pub reward_token_vault: Pubkey,
    pub reward_vault_mint: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub memo_program: Pubkey,
}
impl From<CollectRemainingRewardsAccounts<'_, '_>> for CollectRemainingRewardsKeys {
    fn from(accounts: CollectRemainingRewardsAccounts) -> Self {
        Self {
            reward_funder: *accounts.reward_funder.key,
            funder_token_account: *accounts.funder_token_account.key,
            pool_state: *accounts.pool_state.key,
            reward_token_vault: *accounts.reward_token_vault.key,
            reward_vault_mint: *accounts.reward_vault_mint.key,
            token_program: *accounts.token_program.key,
            token_program2022: *accounts.token_program2022.key,
            memo_program: *accounts.memo_program.key,
        }
    }
}
impl From<CollectRemainingRewardsKeys> for [AccountMeta; COLLECT_REMAINING_REWARDS_IX_ACCOUNTS_LEN] {
    fn from(keys: CollectRemainingRewardsKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.reward_funder,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.funder_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.reward_token_vault,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.reward_vault_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
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
impl From<[Pubkey; COLLECT_REMAINING_REWARDS_IX_ACCOUNTS_LEN]> for CollectRemainingRewardsKeys {
    fn from(pubkeys: [Pubkey; COLLECT_REMAINING_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            reward_funder: pubkeys[0],
            funder_token_account: pubkeys[1],
            pool_state: pubkeys[2],
            reward_token_vault: pubkeys[3],
            reward_vault_mint: pubkeys[4],
            token_program: pubkeys[5],
            token_program2022: pubkeys[6],
            memo_program: pubkeys[7],
        }
    }
}
impl<'info> From<CollectRemainingRewardsAccounts<'_, 'info>>
    for [AccountInfo<'info>; COLLECT_REMAINING_REWARDS_IX_ACCOUNTS_LEN]
{
    fn from(accounts: CollectRemainingRewardsAccounts<'_, 'info>) -> Self {
        [
            accounts.reward_funder.clone(),
            accounts.funder_token_account.clone(),
            accounts.pool_state.clone(),
            accounts.reward_token_vault.clone(),
            accounts.reward_vault_mint.clone(),
            accounts.token_program.clone(),
            accounts.token_program2022.clone(),
            accounts.memo_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; COLLECT_REMAINING_REWARDS_IX_ACCOUNTS_LEN]>
    for CollectRemainingRewardsAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; COLLECT_REMAINING_REWARDS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            reward_funder: &arr[0],
            funder_token_account: &arr[1],
            pool_state: &arr[2],
            reward_token_vault: &arr[3],
            reward_vault_mint: &arr[4],
            token_program: &arr[5],
            token_program2022: &arr[6],
            memo_program: &arr[7],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CollectRemainingRewardsIxArgs {
    pub reward_index: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollectRemainingRewardsIxData(pub CollectRemainingRewardsIxArgs);
impl From<CollectRemainingRewardsIxArgs> for CollectRemainingRewardsIxData {
    fn from(args: CollectRemainingRewardsIxArgs) -> Self {
        Self(args)
    }
}
impl CollectRemainingRewardsIxData {}

pub const UPDATE_REWARD_INFOS_IX_ACCOUNTS_LEN: usize = 1;
#[derive(Copy, Clone, Debug)]
pub struct UpdateRewardInfosAccounts<'me, 'info> {
    pub pool_state: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdateRewardInfosKeys {
    pub pool_state: Pubkey,
}
impl From<UpdateRewardInfosAccounts<'_, '_>> for UpdateRewardInfosKeys {
    fn from(accounts: UpdateRewardInfosAccounts) -> Self {
        Self {
            pool_state: *accounts.pool_state.key,
        }
    }
}
impl From<UpdateRewardInfosKeys> for [AccountMeta; UPDATE_REWARD_INFOS_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdateRewardInfosKeys) -> Self {
        [AccountMeta {
            pubkey: keys.pool_state,
            is_signer: false,
            is_writable: true,
        }]
    }
}
impl From<[Pubkey; UPDATE_REWARD_INFOS_IX_ACCOUNTS_LEN]> for UpdateRewardInfosKeys {
    fn from(pubkeys: [Pubkey; UPDATE_REWARD_INFOS_IX_ACCOUNTS_LEN]) -> Self {
        Self { pool_state: pubkeys[0] }
    }
}
impl<'info> From<UpdateRewardInfosAccounts<'_, 'info>> for [AccountInfo<'info>; UPDATE_REWARD_INFOS_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdateRewardInfosAccounts<'_, 'info>) -> Self {
        [accounts.pool_state.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_REWARD_INFOS_IX_ACCOUNTS_LEN]>
    for UpdateRewardInfosAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_REWARD_INFOS_IX_ACCOUNTS_LEN]) -> Self {
        Self { pool_state: &arr[0] }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateRewardInfosIxData;
impl UpdateRewardInfosIxData {}

pub const SET_REWARD_PARAMS_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct SetRewardParamsAccounts<'me, 'info> {
    pub authority: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub operation_state: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SetRewardParamsKeys {
    pub authority: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub operation_state: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
}
impl From<SetRewardParamsAccounts<'_, '_>> for SetRewardParamsKeys {
    fn from(accounts: SetRewardParamsAccounts) -> Self {
        Self {
            authority: *accounts.authority.key,
            amm_config: *accounts.amm_config.key,
            pool_state: *accounts.pool_state.key,
            operation_state: *accounts.operation_state.key,
            token_program: *accounts.token_program.key,
            token_program2022: *accounts.token_program2022.key,
        }
    }
}
impl From<SetRewardParamsKeys> for [AccountMeta; SET_REWARD_PARAMS_IX_ACCOUNTS_LEN] {
    fn from(keys: SetRewardParamsKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.authority,
                is_signer: true,
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
                pubkey: keys.operation_state,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; SET_REWARD_PARAMS_IX_ACCOUNTS_LEN]> for SetRewardParamsKeys {
    fn from(pubkeys: [Pubkey; SET_REWARD_PARAMS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: pubkeys[0],
            amm_config: pubkeys[1],
            pool_state: pubkeys[2],
            operation_state: pubkeys[3],
            token_program: pubkeys[4],
            token_program2022: pubkeys[5],
        }
    }
}
impl<'info> From<SetRewardParamsAccounts<'_, 'info>> for [AccountInfo<'info>; SET_REWARD_PARAMS_IX_ACCOUNTS_LEN] {
    fn from(accounts: SetRewardParamsAccounts<'_, 'info>) -> Self {
        [
            accounts.authority.clone(),
            accounts.amm_config.clone(),
            accounts.pool_state.clone(),
            accounts.operation_state.clone(),
            accounts.token_program.clone(),
            accounts.token_program2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SET_REWARD_PARAMS_IX_ACCOUNTS_LEN]>
    for SetRewardParamsAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; SET_REWARD_PARAMS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: &arr[0],
            amm_config: &arr[1],
            pool_state: &arr[2],
            operation_state: &arr[3],
            token_program: &arr[4],
            token_program2022: &arr[5],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SetRewardParamsIxArgs {
    pub reward_index: u8,
    pub emissions_per_second_x64: u128,
    pub open_time: u64,
    pub end_time: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SetRewardParamsIxData(pub SetRewardParamsIxArgs);
impl From<SetRewardParamsIxArgs> for SetRewardParamsIxData {
    fn from(args: SetRewardParamsIxArgs) -> Self {
        Self(args)
    }
}
impl SetRewardParamsIxData {}

pub const COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN: usize = 11;
#[derive(Copy, Clone, Debug)]
pub struct CollectProtocolFeeAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub token_vault0: &'me AccountInfo<'info>,
    pub token_vault1: &'me AccountInfo<'info>,
    pub vault0_mint: &'me AccountInfo<'info>,
    pub vault1_mint: &'me AccountInfo<'info>,
    pub recipient_token_account0: &'me AccountInfo<'info>,
    pub recipient_token_account1: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CollectProtocolFeeKeys {
    pub owner: Pubkey,
    pub pool_state: Pubkey,
    pub amm_config: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
    pub recipient_token_account0: Pubkey,
    pub recipient_token_account1: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
}
impl From<CollectProtocolFeeAccounts<'_, '_>> for CollectProtocolFeeKeys {
    fn from(accounts: CollectProtocolFeeAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            pool_state: *accounts.pool_state.key,
            amm_config: *accounts.amm_config.key,
            token_vault0: *accounts.token_vault0.key,
            token_vault1: *accounts.token_vault1.key,
            vault0_mint: *accounts.vault0_mint.key,
            vault1_mint: *accounts.vault1_mint.key,
            recipient_token_account0: *accounts.recipient_token_account0.key,
            recipient_token_account1: *accounts.recipient_token_account1.key,
            token_program: *accounts.token_program.key,
            token_program2022: *accounts.token_program2022.key,
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
                pubkey: keys.token_vault0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault0_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault1_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.recipient_token_account0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.recipient_token_account1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
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
            pool_state: pubkeys[1],
            amm_config: pubkeys[2],
            token_vault0: pubkeys[3],
            token_vault1: pubkeys[4],
            vault0_mint: pubkeys[5],
            vault1_mint: pubkeys[6],
            recipient_token_account0: pubkeys[7],
            recipient_token_account1: pubkeys[8],
            token_program: pubkeys[9],
            token_program2022: pubkeys[10],
        }
    }
}
impl<'info> From<CollectProtocolFeeAccounts<'_, 'info>> for [AccountInfo<'info>; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN] {
    fn from(accounts: CollectProtocolFeeAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.pool_state.clone(),
            accounts.amm_config.clone(),
            accounts.token_vault0.clone(),
            accounts.token_vault1.clone(),
            accounts.vault0_mint.clone(),
            accounts.vault1_mint.clone(),
            accounts.recipient_token_account0.clone(),
            accounts.recipient_token_account1.clone(),
            accounts.token_program.clone(),
            accounts.token_program2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]>
    for CollectProtocolFeeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; COLLECT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            pool_state: &arr[1],
            amm_config: &arr[2],
            token_vault0: &arr[3],
            token_vault1: &arr[4],
            vault0_mint: &arr[5],
            vault1_mint: &arr[6],
            recipient_token_account0: &arr[7],
            recipient_token_account1: &arr[8],
            token_program: &arr[9],
            token_program2022: &arr[10],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CollectProtocolFeeIxArgs {
    pub amount0_requested: u64,
    pub amount1_requested: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollectProtocolFeeIxData(pub CollectProtocolFeeIxArgs);
impl From<CollectProtocolFeeIxArgs> for CollectProtocolFeeIxData {
    fn from(args: CollectProtocolFeeIxArgs) -> Self {
        Self(args)
    }
}
impl CollectProtocolFeeIxData {}

pub const COLLECT_FUND_FEE_IX_ACCOUNTS_LEN: usize = 11;
#[derive(Copy, Clone, Debug)]
pub struct CollectFundFeeAccounts<'me, 'info> {
    pub owner: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub token_vault0: &'me AccountInfo<'info>,
    pub token_vault1: &'me AccountInfo<'info>,
    pub vault0_mint: &'me AccountInfo<'info>,
    pub vault1_mint: &'me AccountInfo<'info>,
    pub recipient_token_account0: &'me AccountInfo<'info>,
    pub recipient_token_account1: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CollectFundFeeKeys {
    pub owner: Pubkey,
    pub pool_state: Pubkey,
    pub amm_config: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
    pub recipient_token_account0: Pubkey,
    pub recipient_token_account1: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
}
impl From<CollectFundFeeAccounts<'_, '_>> for CollectFundFeeKeys {
    fn from(accounts: CollectFundFeeAccounts) -> Self {
        Self {
            owner: *accounts.owner.key,
            pool_state: *accounts.pool_state.key,
            amm_config: *accounts.amm_config.key,
            token_vault0: *accounts.token_vault0.key,
            token_vault1: *accounts.token_vault1.key,
            vault0_mint: *accounts.vault0_mint.key,
            vault1_mint: *accounts.vault1_mint.key,
            recipient_token_account0: *accounts.recipient_token_account0.key,
            recipient_token_account1: *accounts.recipient_token_account1.key,
            token_program: *accounts.token_program.key,
            token_program2022: *accounts.token_program2022.key,
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
                pubkey: keys.token_vault0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault0_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault1_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.recipient_token_account0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.recipient_token_account1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
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
            pool_state: pubkeys[1],
            amm_config: pubkeys[2],
            token_vault0: pubkeys[3],
            token_vault1: pubkeys[4],
            vault0_mint: pubkeys[5],
            vault1_mint: pubkeys[6],
            recipient_token_account0: pubkeys[7],
            recipient_token_account1: pubkeys[8],
            token_program: pubkeys[9],
            token_program2022: pubkeys[10],
        }
    }
}
impl<'info> From<CollectFundFeeAccounts<'_, 'info>> for [AccountInfo<'info>; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN] {
    fn from(accounts: CollectFundFeeAccounts<'_, 'info>) -> Self {
        [
            accounts.owner.clone(),
            accounts.pool_state.clone(),
            accounts.amm_config.clone(),
            accounts.token_vault0.clone(),
            accounts.token_vault1.clone(),
            accounts.vault0_mint.clone(),
            accounts.vault1_mint.clone(),
            accounts.recipient_token_account0.clone(),
            accounts.recipient_token_account1.clone(),
            accounts.token_program.clone(),
            accounts.token_program2022.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN]>
    for CollectFundFeeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; COLLECT_FUND_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            owner: &arr[0],
            pool_state: &arr[1],
            amm_config: &arr[2],
            token_vault0: &arr[3],
            token_vault1: &arr[4],
            vault0_mint: &arr[5],
            vault1_mint: &arr[6],
            recipient_token_account0: &arr[7],
            recipient_token_account1: &arr[8],
            token_program: &arr[9],
            token_program2022: &arr[10],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CollectFundFeeIxArgs {
    pub amount0_requested: u64,
    pub amount1_requested: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollectFundFeeIxData(pub CollectFundFeeIxArgs);
impl From<CollectFundFeeIxArgs> for CollectFundFeeIxData {
    fn from(args: CollectFundFeeIxArgs) -> Self {
        Self(args)
    }
}
impl CollectFundFeeIxData {}

pub const OPEN_POSITION_IX_ACCOUNTS_LEN: usize = 19;
#[derive(Copy, Clone, Debug)]
pub struct OpenPositionAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub position_nft_owner: &'me AccountInfo<'info>,
    pub position_nft_mint: &'me AccountInfo<'info>,
    pub position_nft_account: &'me AccountInfo<'info>,
    pub metadata_account: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub protocol_position: &'me AccountInfo<'info>,
    pub tick_array_lower: &'me AccountInfo<'info>,
    pub tick_array_upper: &'me AccountInfo<'info>,
    pub personal_position: &'me AccountInfo<'info>,
    pub token_account0: &'me AccountInfo<'info>,
    pub token_account1: &'me AccountInfo<'info>,
    pub token_vault0: &'me AccountInfo<'info>,
    pub token_vault1: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub associated_token_program: &'me AccountInfo<'info>,
    pub metadata_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OpenPositionKeys {
    pub payer: Pubkey,
    pub position_nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub metadata_account: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub personal_position: Pubkey,
    pub token_account0: Pubkey,
    pub token_account1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub rent: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub metadata_program: Pubkey,
}
impl From<OpenPositionAccounts<'_, '_>> for OpenPositionKeys {
    fn from(accounts: OpenPositionAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            position_nft_owner: *accounts.position_nft_owner.key,
            position_nft_mint: *accounts.position_nft_mint.key,
            position_nft_account: *accounts.position_nft_account.key,
            metadata_account: *accounts.metadata_account.key,
            pool_state: *accounts.pool_state.key,
            protocol_position: *accounts.protocol_position.key,
            tick_array_lower: *accounts.tick_array_lower.key,
            tick_array_upper: *accounts.tick_array_upper.key,
            personal_position: *accounts.personal_position.key,
            token_account0: *accounts.token_account0.key,
            token_account1: *accounts.token_account1.key,
            token_vault0: *accounts.token_vault0.key,
            token_vault1: *accounts.token_vault1.key,
            rent: *accounts.rent.key,
            system_program: *accounts.system_program.key,
            token_program: *accounts.token_program.key,
            associated_token_program: *accounts.associated_token_program.key,
            metadata_program: *accounts.metadata_program.key,
        }
    }
}
impl From<OpenPositionKeys> for [AccountMeta; OPEN_POSITION_IX_ACCOUNTS_LEN] {
    fn from(keys: OpenPositionKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_owner,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.position_nft_mint,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.metadata_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.personal_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.associated_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.metadata_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; OPEN_POSITION_IX_ACCOUNTS_LEN]> for OpenPositionKeys {
    fn from(pubkeys: [Pubkey; OPEN_POSITION_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            position_nft_owner: pubkeys[1],
            position_nft_mint: pubkeys[2],
            position_nft_account: pubkeys[3],
            metadata_account: pubkeys[4],
            pool_state: pubkeys[5],
            protocol_position: pubkeys[6],
            tick_array_lower: pubkeys[7],
            tick_array_upper: pubkeys[8],
            personal_position: pubkeys[9],
            token_account0: pubkeys[10],
            token_account1: pubkeys[11],
            token_vault0: pubkeys[12],
            token_vault1: pubkeys[13],
            rent: pubkeys[14],
            system_program: pubkeys[15],
            token_program: pubkeys[16],
            associated_token_program: pubkeys[17],
            metadata_program: pubkeys[18],
        }
    }
}
impl<'info> From<OpenPositionAccounts<'_, 'info>> for [AccountInfo<'info>; OPEN_POSITION_IX_ACCOUNTS_LEN] {
    fn from(accounts: OpenPositionAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.position_nft_owner.clone(),
            accounts.position_nft_mint.clone(),
            accounts.position_nft_account.clone(),
            accounts.metadata_account.clone(),
            accounts.pool_state.clone(),
            accounts.protocol_position.clone(),
            accounts.tick_array_lower.clone(),
            accounts.tick_array_upper.clone(),
            accounts.personal_position.clone(),
            accounts.token_account0.clone(),
            accounts.token_account1.clone(),
            accounts.token_vault0.clone(),
            accounts.token_vault1.clone(),
            accounts.rent.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
            accounts.associated_token_program.clone(),
            accounts.metadata_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; OPEN_POSITION_IX_ACCOUNTS_LEN]> for OpenPositionAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; OPEN_POSITION_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            position_nft_owner: &arr[1],
            position_nft_mint: &arr[2],
            position_nft_account: &arr[3],
            metadata_account: &arr[4],
            pool_state: &arr[5],
            protocol_position: &arr[6],
            tick_array_lower: &arr[7],
            tick_array_upper: &arr[8],
            personal_position: &arr[9],
            token_account0: &arr[10],
            token_account1: &arr[11],
            token_vault0: &arr[12],
            token_vault1: &arr[13],
            rent: &arr[14],
            system_program: &arr[15],
            token_program: &arr[16],
            associated_token_program: &arr[17],
            metadata_program: &arr[18],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct OpenPositionIxArgs {
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OpenPositionIxData(pub OpenPositionIxArgs);
impl From<OpenPositionIxArgs> for OpenPositionIxData {
    fn from(args: OpenPositionIxArgs) -> Self {
        Self(args)
    }
}
impl OpenPositionIxData {}

pub const OPEN_POSITION_V2_IX_ACCOUNTS_LEN: usize = 22;
#[derive(Copy, Clone, Debug)]
pub struct OpenPositionV2Accounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub position_nft_owner: &'me AccountInfo<'info>,
    pub position_nft_mint: &'me AccountInfo<'info>,
    pub position_nft_account: &'me AccountInfo<'info>,
    pub metadata_account: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub protocol_position: &'me AccountInfo<'info>,
    pub tick_array_lower: &'me AccountInfo<'info>,
    pub tick_array_upper: &'me AccountInfo<'info>,
    pub personal_position: &'me AccountInfo<'info>,
    pub token_account0: &'me AccountInfo<'info>,
    pub token_account1: &'me AccountInfo<'info>,
    pub token_vault0: &'me AccountInfo<'info>,
    pub token_vault1: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub associated_token_program: &'me AccountInfo<'info>,
    pub metadata_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
    pub vault0_mint: &'me AccountInfo<'info>,
    pub vault1_mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OpenPositionV2Keys {
    pub payer: Pubkey,
    pub position_nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub metadata_account: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub personal_position: Pubkey,
    pub token_account0: Pubkey,
    pub token_account1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub rent: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub metadata_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
}
impl From<OpenPositionV2Accounts<'_, '_>> for OpenPositionV2Keys {
    fn from(accounts: OpenPositionV2Accounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            position_nft_owner: *accounts.position_nft_owner.key,
            position_nft_mint: *accounts.position_nft_mint.key,
            position_nft_account: *accounts.position_nft_account.key,
            metadata_account: *accounts.metadata_account.key,
            pool_state: *accounts.pool_state.key,
            protocol_position: *accounts.protocol_position.key,
            tick_array_lower: *accounts.tick_array_lower.key,
            tick_array_upper: *accounts.tick_array_upper.key,
            personal_position: *accounts.personal_position.key,
            token_account0: *accounts.token_account0.key,
            token_account1: *accounts.token_account1.key,
            token_vault0: *accounts.token_vault0.key,
            token_vault1: *accounts.token_vault1.key,
            rent: *accounts.rent.key,
            system_program: *accounts.system_program.key,
            token_program: *accounts.token_program.key,
            associated_token_program: *accounts.associated_token_program.key,
            metadata_program: *accounts.metadata_program.key,
            token_program2022: *accounts.token_program2022.key,
            vault0_mint: *accounts.vault0_mint.key,
            vault1_mint: *accounts.vault1_mint.key,
        }
    }
}
impl From<OpenPositionV2Keys> for [AccountMeta; OPEN_POSITION_V2_IX_ACCOUNTS_LEN] {
    fn from(keys: OpenPositionV2Keys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_owner,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.position_nft_mint,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.metadata_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.personal_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.associated_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.metadata_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault0_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault1_mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; OPEN_POSITION_V2_IX_ACCOUNTS_LEN]> for OpenPositionV2Keys {
    fn from(pubkeys: [Pubkey; OPEN_POSITION_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            position_nft_owner: pubkeys[1],
            position_nft_mint: pubkeys[2],
            position_nft_account: pubkeys[3],
            metadata_account: pubkeys[4],
            pool_state: pubkeys[5],
            protocol_position: pubkeys[6],
            tick_array_lower: pubkeys[7],
            tick_array_upper: pubkeys[8],
            personal_position: pubkeys[9],
            token_account0: pubkeys[10],
            token_account1: pubkeys[11],
            token_vault0: pubkeys[12],
            token_vault1: pubkeys[13],
            rent: pubkeys[14],
            system_program: pubkeys[15],
            token_program: pubkeys[16],
            associated_token_program: pubkeys[17],
            metadata_program: pubkeys[18],
            token_program2022: pubkeys[19],
            vault0_mint: pubkeys[20],
            vault1_mint: pubkeys[21],
        }
    }
}
impl<'info> From<OpenPositionV2Accounts<'_, 'info>> for [AccountInfo<'info>; OPEN_POSITION_V2_IX_ACCOUNTS_LEN] {
    fn from(accounts: OpenPositionV2Accounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.position_nft_owner.clone(),
            accounts.position_nft_mint.clone(),
            accounts.position_nft_account.clone(),
            accounts.metadata_account.clone(),
            accounts.pool_state.clone(),
            accounts.protocol_position.clone(),
            accounts.tick_array_lower.clone(),
            accounts.tick_array_upper.clone(),
            accounts.personal_position.clone(),
            accounts.token_account0.clone(),
            accounts.token_account1.clone(),
            accounts.token_vault0.clone(),
            accounts.token_vault1.clone(),
            accounts.rent.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
            accounts.associated_token_program.clone(),
            accounts.metadata_program.clone(),
            accounts.token_program2022.clone(),
            accounts.vault0_mint.clone(),
            accounts.vault1_mint.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; OPEN_POSITION_V2_IX_ACCOUNTS_LEN]>
    for OpenPositionV2Accounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; OPEN_POSITION_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            position_nft_owner: &arr[1],
            position_nft_mint: &arr[2],
            position_nft_account: &arr[3],
            metadata_account: &arr[4],
            pool_state: &arr[5],
            protocol_position: &arr[6],
            tick_array_lower: &arr[7],
            tick_array_upper: &arr[8],
            personal_position: &arr[9],
            token_account0: &arr[10],
            token_account1: &arr[11],
            token_vault0: &arr[12],
            token_vault1: &arr[13],
            rent: &arr[14],
            system_program: &arr[15],
            token_program: &arr[16],
            associated_token_program: &arr[17],
            metadata_program: &arr[18],
            token_program2022: &arr[19],
            vault0_mint: &arr[20],
            vault1_mint: &arr[21],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct OpenPositionV2IxArgs {
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub with_metadata: bool,
    pub base_flag: Option<bool>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OpenPositionV2IxData(pub OpenPositionV2IxArgs);
impl From<OpenPositionV2IxArgs> for OpenPositionV2IxData {
    fn from(args: OpenPositionV2IxArgs) -> Self {
        Self(args)
    }
}
impl OpenPositionV2IxData {}

pub const OPEN_POSITION_WITH_TOKEN22_NFT_IX_ACCOUNTS_LEN: usize = 20;
#[derive(Copy, Clone, Debug)]
pub struct OpenPositionWithToken22NftAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub position_nft_owner: &'me AccountInfo<'info>,
    pub position_nft_mint: &'me AccountInfo<'info>,
    pub position_nft_account: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub protocol_position: &'me AccountInfo<'info>,
    pub tick_array_lower: &'me AccountInfo<'info>,
    pub tick_array_upper: &'me AccountInfo<'info>,
    pub personal_position: &'me AccountInfo<'info>,
    pub token_account0: &'me AccountInfo<'info>,
    pub token_account1: &'me AccountInfo<'info>,
    pub token_vault0: &'me AccountInfo<'info>,
    pub token_vault1: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub associated_token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
    pub vault0_mint: &'me AccountInfo<'info>,
    pub vault1_mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OpenPositionWithToken22NftKeys {
    pub payer: Pubkey,
    pub position_nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub personal_position: Pubkey,
    pub token_account0: Pubkey,
    pub token_account1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub rent: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
}
impl From<OpenPositionWithToken22NftAccounts<'_, '_>> for OpenPositionWithToken22NftKeys {
    fn from(accounts: OpenPositionWithToken22NftAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            position_nft_owner: *accounts.position_nft_owner.key,
            position_nft_mint: *accounts.position_nft_mint.key,
            position_nft_account: *accounts.position_nft_account.key,
            pool_state: *accounts.pool_state.key,
            protocol_position: *accounts.protocol_position.key,
            tick_array_lower: *accounts.tick_array_lower.key,
            tick_array_upper: *accounts.tick_array_upper.key,
            personal_position: *accounts.personal_position.key,
            token_account0: *accounts.token_account0.key,
            token_account1: *accounts.token_account1.key,
            token_vault0: *accounts.token_vault0.key,
            token_vault1: *accounts.token_vault1.key,
            rent: *accounts.rent.key,
            system_program: *accounts.system_program.key,
            token_program: *accounts.token_program.key,
            associated_token_program: *accounts.associated_token_program.key,
            token_program2022: *accounts.token_program2022.key,
            vault0_mint: *accounts.vault0_mint.key,
            vault1_mint: *accounts.vault1_mint.key,
        }
    }
}
impl From<OpenPositionWithToken22NftKeys> for [AccountMeta; OPEN_POSITION_WITH_TOKEN22_NFT_IX_ACCOUNTS_LEN] {
    fn from(keys: OpenPositionWithToken22NftKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_owner,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.position_nft_mint,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.personal_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.associated_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault0_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault1_mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; OPEN_POSITION_WITH_TOKEN22_NFT_IX_ACCOUNTS_LEN]> for OpenPositionWithToken22NftKeys {
    fn from(pubkeys: [Pubkey; OPEN_POSITION_WITH_TOKEN22_NFT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            position_nft_owner: pubkeys[1],
            position_nft_mint: pubkeys[2],
            position_nft_account: pubkeys[3],
            pool_state: pubkeys[4],
            protocol_position: pubkeys[5],
            tick_array_lower: pubkeys[6],
            tick_array_upper: pubkeys[7],
            personal_position: pubkeys[8],
            token_account0: pubkeys[9],
            token_account1: pubkeys[10],
            token_vault0: pubkeys[11],
            token_vault1: pubkeys[12],
            rent: pubkeys[13],
            system_program: pubkeys[14],
            token_program: pubkeys[15],
            associated_token_program: pubkeys[16],
            token_program2022: pubkeys[17],
            vault0_mint: pubkeys[18],
            vault1_mint: pubkeys[19],
        }
    }
}
impl<'info> From<OpenPositionWithToken22NftAccounts<'_, 'info>>
    for [AccountInfo<'info>; OPEN_POSITION_WITH_TOKEN22_NFT_IX_ACCOUNTS_LEN]
{
    fn from(accounts: OpenPositionWithToken22NftAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.position_nft_owner.clone(),
            accounts.position_nft_mint.clone(),
            accounts.position_nft_account.clone(),
            accounts.pool_state.clone(),
            accounts.protocol_position.clone(),
            accounts.tick_array_lower.clone(),
            accounts.tick_array_upper.clone(),
            accounts.personal_position.clone(),
            accounts.token_account0.clone(),
            accounts.token_account1.clone(),
            accounts.token_vault0.clone(),
            accounts.token_vault1.clone(),
            accounts.rent.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
            accounts.associated_token_program.clone(),
            accounts.token_program2022.clone(),
            accounts.vault0_mint.clone(),
            accounts.vault1_mint.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; OPEN_POSITION_WITH_TOKEN22_NFT_IX_ACCOUNTS_LEN]>
    for OpenPositionWithToken22NftAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; OPEN_POSITION_WITH_TOKEN22_NFT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            position_nft_owner: &arr[1],
            position_nft_mint: &arr[2],
            position_nft_account: &arr[3],
            pool_state: &arr[4],
            protocol_position: &arr[5],
            tick_array_lower: &arr[6],
            tick_array_upper: &arr[7],
            personal_position: &arr[8],
            token_account0: &arr[9],
            token_account1: &arr[10],
            token_vault0: &arr[11],
            token_vault1: &arr[12],
            rent: &arr[13],
            system_program: &arr[14],
            token_program: &arr[15],
            associated_token_program: &arr[16],
            token_program2022: &arr[17],
            vault0_mint: &arr[18],
            vault1_mint: &arr[19],
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct OpenPositionWithToken22NftIxArgs {
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub with_metadata: bool,
    pub base_flag: Option<bool>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OpenPositionWithToken22NftIxData(pub OpenPositionWithToken22NftIxArgs);
impl From<OpenPositionWithToken22NftIxArgs> for OpenPositionWithToken22NftIxData {
    fn from(args: OpenPositionWithToken22NftIxArgs) -> Self {
        Self(args)
    }
}
impl OpenPositionWithToken22NftIxData {}

pub const CLOSE_POSITION_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct ClosePositionAccounts<'me, 'info> {
    pub nft_owner: &'me AccountInfo<'info>,
    pub position_nft_mint: &'me AccountInfo<'info>,
    pub position_nft_account: &'me AccountInfo<'info>,
    pub personal_position: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClosePositionKeys {
    pub nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub personal_position: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
}
impl From<ClosePositionAccounts<'_, '_>> for ClosePositionKeys {
    fn from(accounts: ClosePositionAccounts) -> Self {
        Self {
            nft_owner: *accounts.nft_owner.key,
            position_nft_mint: *accounts.position_nft_mint.key,
            position_nft_account: *accounts.position_nft_account.key,
            personal_position: *accounts.personal_position.key,
            system_program: *accounts.system_program.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<ClosePositionKeys> for [AccountMeta; CLOSE_POSITION_IX_ACCOUNTS_LEN] {
    fn from(keys: ClosePositionKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.nft_owner,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.position_nft_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.personal_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CLOSE_POSITION_IX_ACCOUNTS_LEN]> for ClosePositionKeys {
    fn from(pubkeys: [Pubkey; CLOSE_POSITION_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            nft_owner: pubkeys[0],
            position_nft_mint: pubkeys[1],
            position_nft_account: pubkeys[2],
            personal_position: pubkeys[3],
            system_program: pubkeys[4],
            token_program: pubkeys[5],
        }
    }
}
impl<'info> From<ClosePositionAccounts<'_, 'info>> for [AccountInfo<'info>; CLOSE_POSITION_IX_ACCOUNTS_LEN] {
    fn from(accounts: ClosePositionAccounts<'_, 'info>) -> Self {
        [
            accounts.nft_owner.clone(),
            accounts.position_nft_mint.clone(),
            accounts.position_nft_account.clone(),
            accounts.personal_position.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CLOSE_POSITION_IX_ACCOUNTS_LEN]> for ClosePositionAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CLOSE_POSITION_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            nft_owner: &arr[0],
            position_nft_mint: &arr[1],
            position_nft_account: &arr[2],
            personal_position: &arr[3],
            system_program: &arr[4],
            token_program: &arr[5],
        }
    }
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct ClosePositionIxData;
// impl ClosePositionIxData {}

// pub const INCREASE_LIQUIDITY_IX_ACCOUNTS_LEN: usize = 12;
// #[derive(Copy, Clone, Debug)]
// pub struct IncreaseLiquidityAccounts<'me, 'info> {
//     pub nft_owner: &'me AccountInfo<'info>,
//     pub nft_account: &'me AccountInfo<'info>,
//     pub pool_state: &'me AccountInfo<'info>,
//     pub protocol_position: &'me AccountInfo<'info>,
//     pub personal_position: &'me AccountInfo<'info>,
//     pub tick_array_lower: &'me AccountInfo<'info>,
//     pub tick_array_upper: &'me AccountInfo<'info>,
//     pub token_account0: &'me AccountInfo<'info>,
//     pub token_account1: &'me AccountInfo<'info>,
//     pub token_vault0: &'me AccountInfo<'info>,
//     pub token_vault1: &'me AccountInfo<'info>,
//     pub token_program: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct IncreaseLiquidityKeys {
//     pub nft_owner: Pubkey,
//     pub nft_account: Pubkey,
//     pub pool_state: Pubkey,
//     pub protocol_position: Pubkey,
//     pub personal_position: Pubkey,
//     pub tick_array_lower: Pubkey,
//     pub tick_array_upper: Pubkey,
//     pub token_account0: Pubkey,
//     pub token_account1: Pubkey,
//     pub token_vault0: Pubkey,
//     pub token_vault1: Pubkey,
//     pub token_program: Pubkey,
// }
// impl From<IncreaseLiquidityAccounts<'_, '_>> for IncreaseLiquidityKeys {
//     fn from(accounts: IncreaseLiquidityAccounts) -> Self {
//         Self {
//             nft_owner: *accounts.nft_owner.key,
//             nft_account: *accounts.nft_account.key,
//             pool_state: *accounts.pool_state.key,
//             protocol_position: *accounts.protocol_position.key,
//             personal_position: *accounts.personal_position.key,
//             tick_array_lower: *accounts.tick_array_lower.key,
//             tick_array_upper: *accounts.tick_array_upper.key,
//             token_account0: *accounts.token_account0.key,
//             token_account1: *accounts.token_account1.key,
//             token_vault0: *accounts.token_vault0.key,
//             token_vault1: *accounts.token_vault1.key,
//             token_program: *accounts.token_program.key,
//         }
//     }
// }

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct IncreaseLiquidityIxArgs {
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IncreaseLiquidityIxData(pub IncreaseLiquidityIxArgs);
impl From<IncreaseLiquidityIxArgs> for IncreaseLiquidityIxData {
    fn from(args: IncreaseLiquidityIxArgs) -> Self {
        Self(args)
    }
}
impl IncreaseLiquidityIxData {}
pub const INCREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN: usize = 15;
#[derive(Copy, Clone, Debug)]
pub struct IncreaseLiquidityV2Accounts<'me, 'info> {
    pub nft_owner: &'me AccountInfo<'info>,
    pub nft_account: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub protocol_position: &'me AccountInfo<'info>,
    pub personal_position: &'me AccountInfo<'info>,
    pub tick_array_lower: &'me AccountInfo<'info>,
    pub tick_array_upper: &'me AccountInfo<'info>,
    pub token_account0: &'me AccountInfo<'info>,
    pub token_account1: &'me AccountInfo<'info>,
    pub token_vault0: &'me AccountInfo<'info>,
    pub token_vault1: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
    pub vault0_mint: &'me AccountInfo<'info>,
    pub vault1_mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IncreaseLiquidityV2Keys {
    pub nft_owner: Pubkey,
    pub nft_account: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub personal_position: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub token_account0: Pubkey,
    pub token_account1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
}
impl From<IncreaseLiquidityV2Accounts<'_, '_>> for IncreaseLiquidityV2Keys {
    fn from(accounts: IncreaseLiquidityV2Accounts) -> Self {
        Self {
            nft_owner: *accounts.nft_owner.key,
            nft_account: *accounts.nft_account.key,
            pool_state: *accounts.pool_state.key,
            protocol_position: *accounts.protocol_position.key,
            personal_position: *accounts.personal_position.key,
            tick_array_lower: *accounts.tick_array_lower.key,
            tick_array_upper: *accounts.tick_array_upper.key,
            token_account0: *accounts.token_account0.key,
            token_account1: *accounts.token_account1.key,
            token_vault0: *accounts.token_vault0.key,
            token_vault1: *accounts.token_vault1.key,
            token_program: *accounts.token_program.key,
            token_program2022: *accounts.token_program2022.key,
            vault0_mint: *accounts.vault0_mint.key,
            vault1_mint: *accounts.vault1_mint.key,
        }
    }
}
impl From<IncreaseLiquidityV2Keys> for [AccountMeta; INCREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN] {
    fn from(keys: IncreaseLiquidityV2Keys) -> Self {
        [
            AccountMeta {
                pubkey: keys.nft_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.nft_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.personal_position,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_lower,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.tick_array_upper,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_account1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_vault1,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault0_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault1_mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; INCREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]> for IncreaseLiquidityV2Keys {
    fn from(pubkeys: [Pubkey; INCREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            nft_owner: pubkeys[0],
            nft_account: pubkeys[1],
            pool_state: pubkeys[2],
            protocol_position: pubkeys[3],
            personal_position: pubkeys[4],
            tick_array_lower: pubkeys[5],
            tick_array_upper: pubkeys[6],
            token_account0: pubkeys[7],
            token_account1: pubkeys[8],
            token_vault0: pubkeys[9],
            token_vault1: pubkeys[10],
            token_program: pubkeys[11],
            token_program2022: pubkeys[12],
            vault0_mint: pubkeys[13],
            vault1_mint: pubkeys[14],
        }
    }
}
// impl<'info> From<IncreaseLiquidityV2Accounts<'_, 'info>>
//     for [AccountInfo<'info>; INCREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]
// {
//     fn from(accounts: IncreaseLiquidityV2Accounts<'_, 'info>) -> Self {
//         [
//             accounts.nft_owner.clone(),
//             accounts.nft_account.clone(),
//             accounts.pool_state.clone(),
//             accounts.protocol_position.clone(),
//             accounts.personal_position.clone(),
//             accounts.tick_array_lower.clone(),
//             accounts.tick_array_upper.clone(),
//             accounts.token_account0.clone(),
//             accounts.token_account1.clone(),
//             accounts.token_vault0.clone(),
//             accounts.token_vault1.clone(),
//             accounts.token_program.clone(),
//             accounts.token_program2022.clone(),
//             accounts.vault0_mint.clone(),
//             accounts.vault1_mint.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; INCREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]>
//     for IncreaseLiquidityV2Accounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; INCREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             nft_owner: &arr[0],
//             nft_account: &arr[1],
//             pool_state: &arr[2],
//             protocol_position: &arr[3],
//             personal_position: &arr[4],
//             tick_array_lower: &arr[5],
//             tick_array_upper: &arr[6],
//             token_account0: &arr[7],
//             token_account1: &arr[8],
//             token_vault0: &arr[9],
//             token_vault1: &arr[10],
//             token_program: &arr[11],
//             token_program2022: &arr[12],
//             vault0_mint: &arr[13],
//             vault1_mint: &arr[14],
//         }
//     }
// }
//
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct IncreaseLiquidityV2IxArgs {
//     pub liquidity: u128,
//     pub amount0_max: u64,
//     pub amount1_max: u64,
//     pub base_flag: Option<bool>,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct IncreaseLiquidityV2IxData(pub IncreaseLiquidityV2IxArgs);
// impl From<IncreaseLiquidityV2IxArgs> for IncreaseLiquidityV2IxData {
//     fn from(args: IncreaseLiquidityV2IxArgs) -> Self {
//         Self(args)
//     }
// }
// impl IncreaseLiquidityV2IxData {}
// pub const DECREASE_LIQUIDITY_IX_ACCOUNTS_LEN: usize = 12;
// #[derive(Copy, Clone, Debug)]
// pub struct DecreaseLiquidityAccounts<'me, 'info> {
//     pub nft_owner: &'me AccountInfo<'info>,
//     pub nft_account: &'me AccountInfo<'info>,
//     pub personal_position: &'me AccountInfo<'info>,
//     pub pool_state: &'me AccountInfo<'info>,
//     pub protocol_position: &'me AccountInfo<'info>,
//     pub token_vault0: &'me AccountInfo<'info>,
//     pub token_vault1: &'me AccountInfo<'info>,
//     pub tick_array_lower: &'me AccountInfo<'info>,
//     pub tick_array_upper: &'me AccountInfo<'info>,
//     pub recipient_token_account0: &'me AccountInfo<'info>,
//     pub recipient_token_account1: &'me AccountInfo<'info>,
//     pub token_program: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct DecreaseLiquidityKeys {
//     pub nft_owner: Pubkey,
//     pub nft_account: Pubkey,
//     pub personal_position: Pubkey,
//     pub pool_state: Pubkey,
//     pub protocol_position: Pubkey,
//     pub token_vault0: Pubkey,
//     pub token_vault1: Pubkey,
//     pub tick_array_lower: Pubkey,
//     pub tick_array_upper: Pubkey,
//     pub recipient_token_account0: Pubkey,
//     pub recipient_token_account1: Pubkey,
//     pub token_program: Pubkey,
// }
// impl From<DecreaseLiquidityAccounts<'_, '_>> for DecreaseLiquidityKeys {
//     fn from(accounts: DecreaseLiquidityAccounts) -> Self {
//         Self {
//             nft_owner: *accounts.nft_owner.key,
//             nft_account: *accounts.nft_account.key,
//             personal_position: *accounts.personal_position.key,
//             pool_state: *accounts.pool_state.key,
//             protocol_position: *accounts.protocol_position.key,
//             token_vault0: *accounts.token_vault0.key,
//             token_vault1: *accounts.token_vault1.key,
//             tick_array_lower: *accounts.tick_array_lower.key,
//             tick_array_upper: *accounts.tick_array_upper.key,
//             recipient_token_account0: *accounts.recipient_token_account0.key,
//             recipient_token_account1: *accounts.recipient_token_account1.key,
//             token_program: *accounts.token_program.key,
//         }
//     }
// }
// impl From<DecreaseLiquidityKeys> for [AccountMeta; DECREASE_LIQUIDITY_IX_ACCOUNTS_LEN] {
//     fn from(keys: DecreaseLiquidityKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.nft_owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.nft_account,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.personal_position,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool_state,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.protocol_position,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.token_vault0,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.token_vault1,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.tick_array_lower,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.tick_array_upper,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.recipient_token_account0,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.recipient_token_account1,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; DECREASE_LIQUIDITY_IX_ACCOUNTS_LEN]> for DecreaseLiquidityKeys {
//     fn from(pubkeys: [Pubkey; DECREASE_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             nft_owner: pubkeys[0],
//             nft_account: pubkeys[1],
//             personal_position: pubkeys[2],
//             pool_state: pubkeys[3],
//             protocol_position: pubkeys[4],
//             token_vault0: pubkeys[5],
//             token_vault1: pubkeys[6],
//             tick_array_lower: pubkeys[7],
//             tick_array_upper: pubkeys[8],
//             recipient_token_account0: pubkeys[9],
//             recipient_token_account1: pubkeys[10],
//             token_program: pubkeys[11],
//         }
//     }
// }
// impl<'info> From<DecreaseLiquidityAccounts<'_, 'info>> for [AccountInfo<'info>; DECREASE_LIQUIDITY_IX_ACCOUNTS_LEN] {
//     fn from(accounts: DecreaseLiquidityAccounts<'_, 'info>) -> Self {
//         [
//             accounts.nft_owner.clone(),
//             accounts.nft_account.clone(),
//             accounts.personal_position.clone(),
//             accounts.pool_state.clone(),
//             accounts.protocol_position.clone(),
//             accounts.token_vault0.clone(),
//             accounts.token_vault1.clone(),
//             accounts.tick_array_lower.clone(),
//             accounts.tick_array_upper.clone(),
//             accounts.recipient_token_account0.clone(),
//             accounts.recipient_token_account1.clone(),
//             accounts.token_program.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; DECREASE_LIQUIDITY_IX_ACCOUNTS_LEN]>
//     for DecreaseLiquidityAccounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; DECREASE_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             nft_owner: &arr[0],
//             nft_account: &arr[1],
//             personal_position: &arr[2],
//             pool_state: &arr[3],
//             protocol_position: &arr[4],
//             token_vault0: &arr[5],
//             token_vault1: &arr[6],
//             tick_array_lower: &arr[7],
//             tick_array_upper: &arr[8],
//             recipient_token_account0: &arr[9],
//             recipient_token_account1: &arr[10],
//             token_program: &arr[11],
//         }
//     }
// }
//
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct DecreaseLiquidityIxArgs {
//     pub liquidity: u128,
//     pub amount0_min: u64,
//     pub amount1_min: u64,
// }
// // #[derive(Clone, Debug, PartialEq)]
// // pub struct DecreaseLiquidityIxData(pub DecreaseLiquidityIxArgs);
// // impl From<DecreaseLiquidityIxArgs> for DecreaseLiquidityIxData {
// //     fn from(args: DecreaseLiquidityIxArgs) -> Self {
// //         Self(args)
// //     }
// // }
// // impl DecreaseLiquidityIxData {}
//
// pub const DECREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN: usize = 16;
// #[derive(Copy, Clone, Debug)]
// pub struct DecreaseLiquidityV2Accounts<'me, 'info> {
//     pub nft_owner: &'me AccountInfo<'info>,
//     pub nft_account: &'me AccountInfo<'info>,
//     pub personal_position: &'me AccountInfo<'info>,
//     pub pool_state: &'me AccountInfo<'info>,
//     pub protocol_position: &'me AccountInfo<'info>,
//     pub token_vault0: &'me AccountInfo<'info>,
//     pub token_vault1: &'me AccountInfo<'info>,
//     pub tick_array_lower: &'me AccountInfo<'info>,
//     pub tick_array_upper: &'me AccountInfo<'info>,
//     pub recipient_token_account0: &'me AccountInfo<'info>,
//     pub recipient_token_account1: &'me AccountInfo<'info>,
//     pub token_program: &'me AccountInfo<'info>,
//     pub token_program2022: &'me AccountInfo<'info>,
//     pub memo_program: &'me AccountInfo<'info>,
//     pub vault0_mint: &'me AccountInfo<'info>,
//     pub vault1_mint: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct DecreaseLiquidityV2Keys {
//     pub nft_owner: Pubkey,
//     pub nft_account: Pubkey,
//     pub personal_position: Pubkey,
//     pub pool_state: Pubkey,
//     pub protocol_position: Pubkey,
//     pub token_vault0: Pubkey,
//     pub token_vault1: Pubkey,
//     pub tick_array_lower: Pubkey,
//     pub tick_array_upper: Pubkey,
//     pub recipient_token_account0: Pubkey,
//     pub recipient_token_account1: Pubkey,
//     pub token_program: Pubkey,
//     pub token_program2022: Pubkey,
//     pub memo_program: Pubkey,
//     pub vault0_mint: Pubkey,
//     pub vault1_mint: Pubkey,
// }
// impl From<DecreaseLiquidityV2Accounts<'_, '_>> for DecreaseLiquidityV2Keys {
//     fn from(accounts: DecreaseLiquidityV2Accounts) -> Self {
//         Self {
//             nft_owner: *accounts.nft_owner.key,
//             nft_account: *accounts.nft_account.key,
//             personal_position: *accounts.personal_position.key,
//             pool_state: *accounts.pool_state.key,
//             protocol_position: *accounts.protocol_position.key,
//             token_vault0: *accounts.token_vault0.key,
//             token_vault1: *accounts.token_vault1.key,
//             tick_array_lower: *accounts.tick_array_lower.key,
//             tick_array_upper: *accounts.tick_array_upper.key,
//             recipient_token_account0: *accounts.recipient_token_account0.key,
//             recipient_token_account1: *accounts.recipient_token_account1.key,
//             token_program: *accounts.token_program.key,
//             token_program2022: *accounts.token_program2022.key,
//             memo_program: *accounts.memo_program.key,
//             vault0_mint: *accounts.vault0_mint.key,
//             vault1_mint: *accounts.vault1_mint.key,
//         }
//     }
// }
// impl From<DecreaseLiquidityV2Keys> for [AccountMeta; DECREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN] {
//     fn from(keys: DecreaseLiquidityV2Keys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.nft_owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.nft_account,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.personal_position,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool_state,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.protocol_position,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.token_vault0,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.token_vault1,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.tick_array_lower,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.tick_array_upper,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.recipient_token_account0,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.recipient_token_account1,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program2022,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.memo_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault0_mint,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault1_mint,
//                 is_signer: false,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; DECREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]> for DecreaseLiquidityV2Keys {
//     fn from(pubkeys: [Pubkey; DECREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             nft_owner: pubkeys[0],
//             nft_account: pubkeys[1],
//             personal_position: pubkeys[2],
//             pool_state: pubkeys[3],
//             protocol_position: pubkeys[4],
//             token_vault0: pubkeys[5],
//             token_vault1: pubkeys[6],
//             tick_array_lower: pubkeys[7],
//             tick_array_upper: pubkeys[8],
//             recipient_token_account0: pubkeys[9],
//             recipient_token_account1: pubkeys[10],
//             token_program: pubkeys[11],
//             token_program2022: pubkeys[12],
//             memo_program: pubkeys[13],
//             vault0_mint: pubkeys[14],
//             vault1_mint: pubkeys[15],
//         }
//     }
// }
// impl<'info> From<DecreaseLiquidityV2Accounts<'_, 'info>>
//     for [AccountInfo<'info>; DECREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]
// {
//     fn from(accounts: DecreaseLiquidityV2Accounts<'_, 'info>) -> Self {
//         [
//             accounts.nft_owner.clone(),
//             accounts.nft_account.clone(),
//             accounts.personal_position.clone(),
//             accounts.pool_state.clone(),
//             accounts.protocol_position.clone(),
//             accounts.token_vault0.clone(),
//             accounts.token_vault1.clone(),
//             accounts.tick_array_lower.clone(),
//             accounts.tick_array_upper.clone(),
//             accounts.recipient_token_account0.clone(),
//             accounts.recipient_token_account1.clone(),
//             accounts.token_program.clone(),
//             accounts.token_program2022.clone(),
//             accounts.memo_program.clone(),
//             accounts.vault0_mint.clone(),
//             accounts.vault1_mint.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; DECREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]>
//     for DecreaseLiquidityV2Accounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; DECREASE_LIQUIDITY_V2_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             nft_owner: &arr[0],
//             nft_account: &arr[1],
//             personal_position: &arr[2],
//             pool_state: &arr[3],
//             protocol_position: &arr[4],
//             token_vault0: &arr[5],
//             token_vault1: &arr[6],
//             tick_array_lower: &arr[7],
//             tick_array_upper: &arr[8],
//             recipient_token_account0: &arr[9],
//             recipient_token_account1: &arr[10],
//             token_program: &arr[11],
//             token_program2022: &arr[12],
//             memo_program: &arr[13],
//             vault0_mint: &arr[14],
//             vault1_mint: &arr[15],
//         }
//     }
// }

// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct DecreaseLiquidityV2IxArgs {
//     pub liquidity: u128,
//     pub amount0_min: u64,
//     pub amount1_min: u64,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct DecreaseLiquidityV2IxData(pub DecreaseLiquidityV2IxArgs);
// impl From<DecreaseLiquidityV2IxArgs> for DecreaseLiquidityV2IxData {
//     fn from(args: DecreaseLiquidityV2IxArgs) -> Self {
//         Self(args)
//     }
// }
// impl DecreaseLiquidityV2IxData {}

pub const SWAP_IX_ACCOUNTS_LEN: usize = 10;
#[derive(Copy, Clone, Debug)]
pub struct SwapAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub input_token_account: &'me AccountInfo<'info>,
    pub output_token_account: &'me AccountInfo<'info>,
    pub input_vault: &'me AccountInfo<'info>,
    pub output_vault: &'me AccountInfo<'info>,
    pub observation_state: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub tick_array: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapKeys {
    pub payer: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub tick_array: Pubkey,
}
impl From<SwapAccounts<'_, '_>> for SwapKeys {
    fn from(accounts: SwapAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            amm_config: *accounts.amm_config.key,
            pool_state: *accounts.pool_state.key,
            input_token_account: *accounts.input_token_account.key,
            output_token_account: *accounts.output_token_account.key,
            input_vault: *accounts.input_vault.key,
            output_vault: *accounts.output_vault.key,
            observation_state: *accounts.observation_state.key,
            token_program: *accounts.token_program.key,
            tick_array: *accounts.tick_array.key,
        }
    }
}
impl From<SwapKeys> for [AccountMeta; SWAP_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
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
                pubkey: keys.tick_array,
                is_signer: false,
                is_writable: true,
            },
        ]
    }
}
impl From<[Pubkey; SWAP_IX_ACCOUNTS_LEN]> for SwapKeys {
    fn from(pubkeys: [Pubkey; SWAP_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            amm_config: pubkeys[1],
            pool_state: pubkeys[2],
            input_token_account: pubkeys[3],
            output_token_account: pubkeys[4],
            input_vault: pubkeys[5],
            output_vault: pubkeys[6],
            observation_state: pubkeys[7],
            token_program: pubkeys[8],
            tick_array: pubkeys[9],
        }
    }
}
impl<'info> From<SwapAccounts<'_, 'info>> for [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.amm_config.clone(),
            accounts.pool_state.clone(),
            accounts.input_token_account.clone(),
            accounts.output_token_account.clone(),
            accounts.input_vault.clone(),
            accounts.output_vault.clone(),
            accounts.observation_state.clone(),
            accounts.token_program.clone(),
            accounts.tick_array.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]> for SwapAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            amm_config: &arr[1],
            pool_state: &arr[2],
            input_token_account: &arr[3],
            output_token_account: &arr[4],
            input_vault: &arr[5],
            output_vault: &arr[6],
            observation_state: &arr[7],
            token_program: &arr[8],
            tick_array: &arr[9],
        }
    }
}

//pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapIxArgs {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapIxData(pub SwapIxArgs);
impl From<SwapIxArgs> for SwapIxData {
    fn from(args: SwapIxArgs) -> Self {
        Self(args)
    }
}

impl SwapIxData {
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SWAP_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub const SWAP_V2_IX_ACCOUNTS_LEN: usize = 13;
#[derive(Copy, Clone, Debug)]
pub struct SwapV2Accounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub amm_config: &'me AccountInfo<'info>,
    pub pool_state: &'me AccountInfo<'info>,
    pub input_token_account: &'me AccountInfo<'info>,
    pub output_token_account: &'me AccountInfo<'info>,
    pub input_vault: &'me AccountInfo<'info>,
    pub output_vault: &'me AccountInfo<'info>,
    pub observation_state: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
    pub input_vault_mint: &'me AccountInfo<'info>,
    pub output_vault_mint: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapV2Keys {
    pub payer: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub memo_program: Pubkey,
    pub input_vault_mint: Pubkey,
    pub output_vault_mint: Pubkey,
}
impl From<SwapV2Accounts<'_, '_>> for SwapV2Keys {
    fn from(accounts: SwapV2Accounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            amm_config: *accounts.amm_config.key,
            pool_state: *accounts.pool_state.key,
            input_token_account: *accounts.input_token_account.key,
            output_token_account: *accounts.output_token_account.key,
            input_vault: *accounts.input_vault.key,
            output_vault: *accounts.output_vault.key,
            observation_state: *accounts.observation_state.key,
            token_program: *accounts.token_program.key,
            token_program2022: *accounts.token_program2022.key,
            memo_program: *accounts.memo_program.key,
            input_vault_mint: *accounts.input_vault_mint.key,
            output_vault_mint: *accounts.output_vault_mint.key,
        }
    }
}
impl From<SwapV2Keys> for [AccountMeta; SWAP_V2_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapV2Keys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
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
                pubkey: keys.token_program2022,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.memo_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.input_vault_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.output_vault_mint,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; SWAP_V2_IX_ACCOUNTS_LEN]> for SwapV2Keys {
    fn from(pubkeys: [Pubkey; SWAP_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            amm_config: pubkeys[1],
            pool_state: pubkeys[2],
            input_token_account: pubkeys[3],
            output_token_account: pubkeys[4],
            input_vault: pubkeys[5],
            output_vault: pubkeys[6],
            observation_state: pubkeys[7],
            token_program: pubkeys[8],
            token_program2022: pubkeys[9],
            memo_program: pubkeys[10],
            input_vault_mint: pubkeys[11],
            output_vault_mint: pubkeys[12],
        }
    }
}
impl<'info> From<SwapV2Accounts<'_, 'info>> for [AccountInfo<'info>; SWAP_V2_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapV2Accounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.amm_config.clone(),
            accounts.pool_state.clone(),
            accounts.input_token_account.clone(),
            accounts.output_token_account.clone(),
            accounts.input_vault.clone(),
            accounts.output_vault.clone(),
            accounts.observation_state.clone(),
            accounts.token_program.clone(),
            accounts.token_program2022.clone(),
            accounts.memo_program.clone(),
            accounts.input_vault_mint.clone(),
            accounts.output_vault_mint.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_V2_IX_ACCOUNTS_LEN]> for SwapV2Accounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SWAP_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            amm_config: &arr[1],
            pool_state: &arr[2],
            input_token_account: &arr[3],
            output_token_account: &arr[4],
            input_vault: &arr[5],
            output_vault: &arr[6],
            observation_state: &arr[7],
            token_program: &arr[8],
            token_program2022: &arr[9],
            memo_program: &arr[10],
            input_vault_mint: &arr[11],
            output_vault_mint: &arr[12],
        }
    }
}

pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapV2IxArgs {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapV2IxData(pub SwapV2IxArgs);
impl From<SwapV2IxArgs> for SwapV2IxData {
    fn from(args: SwapV2IxArgs) -> Self {
        Self(args)
    }
}
pub const SWAP_V2_IX_DISCM: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];

impl SwapV2IxData {
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SWAP_V2_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub const SWAP_ROUTER_BASE_IN_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct SwapRouterBaseInAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub input_token_account: &'me AccountInfo<'info>,
    pub input_token_mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub token_program2022: &'me AccountInfo<'info>,
    pub memo_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapRouterBaseInKeys {
    pub payer: Pubkey,
    pub input_token_account: Pubkey,
    pub input_token_mint: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub memo_program: Pubkey,
}
impl From<SwapRouterBaseInAccounts<'_, '_>> for SwapRouterBaseInKeys {
    fn from(accounts: SwapRouterBaseInAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            input_token_account: *accounts.input_token_account.key,
            input_token_mint: *accounts.input_token_mint.key,
            token_program: *accounts.token_program.key,
            token_program2022: *accounts.token_program2022.key,
            memo_program: *accounts.memo_program.key,
        }
    }
}
impl From<SwapRouterBaseInKeys> for [AccountMeta; SWAP_ROUTER_BASE_IN_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapRouterBaseInKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.input_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.input_token_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program2022,
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
impl From<[Pubkey; SWAP_ROUTER_BASE_IN_IX_ACCOUNTS_LEN]> for SwapRouterBaseInKeys {
    fn from(pubkeys: [Pubkey; SWAP_ROUTER_BASE_IN_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            input_token_account: pubkeys[1],
            input_token_mint: pubkeys[2],
            token_program: pubkeys[3],
            token_program2022: pubkeys[4],
            memo_program: pubkeys[5],
        }
    }
}
impl<'info> From<SwapRouterBaseInAccounts<'_, 'info>> for [AccountInfo<'info>; SWAP_ROUTER_BASE_IN_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapRouterBaseInAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.input_token_account.clone(),
            accounts.input_token_mint.clone(),
            accounts.token_program.clone(),
            accounts.token_program2022.clone(),
            accounts.memo_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_ROUTER_BASE_IN_IX_ACCOUNTS_LEN]>
    for SwapRouterBaseInAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; SWAP_ROUTER_BASE_IN_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            input_token_account: &arr[1],
            input_token_mint: &arr[2],
            token_program: &arr[3],
            token_program2022: &arr[4],
            memo_program: &arr[5],
        }
    }
}
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct SwapRouterBaseInIxArgs {
//     pub amount_in: u64,
//     pub amount_out_minimum: u64,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct SwapRouterBaseInIxData(pub SwapRouterBaseInIxArgs);
// impl From<SwapRouterBaseInIxArgs> for SwapRouterBaseInIxData {
//     fn from(args: SwapRouterBaseInIxArgs) -> Self {
//         Self(args)
//     }
// }
