use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, instruction::AccountMeta, program_error::ProgramError, pubkey::Pubkey,
};
use std::io::Read;

#[derive(Clone, Debug, PartialEq)]
pub enum LifinityAmmV2ProgramIx {
    Swap(SwapIxArgs),
    DepositAllTokenTypes(DepositAllTokenTypesIxArgs),
    WithdrawAllTokenTypes(WithdrawAllTokenTypesIxArgs),
}
impl LifinityAmmV2ProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            SWAP_IX_DISCM => Ok(Self::Swap(SwapIxArgs::deserialize(&mut reader)?)),
            DEPOSIT_ALL_TOKEN_TYPES_IX_DISCM => Ok(Self::DepositAllTokenTypes(
                DepositAllTokenTypesIxArgs::deserialize(&mut reader)?,
            )),
            WITHDRAW_ALL_TOKEN_TYPES_IX_DISCM => Ok(Self::WithdrawAllTokenTypes(
                WithdrawAllTokenTypesIxArgs::deserialize(&mut reader)?,
            )),
            _ => Err(std::io::Error::other(format!("discm {maybe_discm:?} not found"))),
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::Swap(args) => {
                writer.write_all(&SWAP_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::DepositAllTokenTypes(args) => {
                writer.write_all(&DEPOSIT_ALL_TOKEN_TYPES_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::WithdrawAllTokenTypes(args) => {
                writer.write_all(&WITHDRAW_ALL_TOKEN_TYPES_IX_DISCM)?;
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
pub const SWAP_IX_ACCOUNTS_LEN: usize = 13;
#[derive(Copy, Clone, Debug)]
pub struct SwapAccounts<'me, 'info> {
    pub authority: &'me AccountInfo<'info>,
    pub amm: &'me AccountInfo<'info>,
    pub user_transfer_authority: &'me AccountInfo<'info>,
    pub source_info: &'me AccountInfo<'info>,
    pub destination_info: &'me AccountInfo<'info>,
    pub swap_source: &'me AccountInfo<'info>,
    pub swap_destination: &'me AccountInfo<'info>,
    pub pool_mint: &'me AccountInfo<'info>,
    pub fee_account: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub oracle_main_account: &'me AccountInfo<'info>,
    pub oracle_sub_account: &'me AccountInfo<'info>,
    pub oracle_pc_account: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapKeys {
    pub authority: Pubkey,
    pub amm: Pubkey,
    pub user_transfer_authority: Pubkey,
    pub source_info: Pubkey,
    pub destination_info: Pubkey,
    pub swap_source: Pubkey,
    pub swap_destination: Pubkey,
    pub pool_mint: Pubkey,
    pub fee_account: Pubkey,
    pub token_program: Pubkey,
    pub oracle_main_account: Pubkey,
    pub oracle_sub_account: Pubkey,
    pub oracle_pc_account: Pubkey,
}
impl From<SwapAccounts<'_, '_>> for SwapKeys {
    fn from(accounts: SwapAccounts) -> Self {
        Self {
            authority: *accounts.authority.key,
            amm: *accounts.amm.key,
            user_transfer_authority: *accounts.user_transfer_authority.key,
            source_info: *accounts.source_info.key,
            destination_info: *accounts.destination_info.key,
            swap_source: *accounts.swap_source.key,
            swap_destination: *accounts.swap_destination.key,
            pool_mint: *accounts.pool_mint.key,
            fee_account: *accounts.fee_account.key,
            token_program: *accounts.token_program.key,
            oracle_main_account: *accounts.oracle_main_account.key,
            oracle_sub_account: *accounts.oracle_sub_account.key,
            oracle_pc_account: *accounts.oracle_pc_account.key,
        }
    }
}
impl From<SwapKeys> for [AccountMeta; SWAP_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.amm,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_transfer_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.source_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.destination_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.swap_source,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.swap_destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.fee_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.oracle_main_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.oracle_sub_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.oracle_pc_account,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; SWAP_IX_ACCOUNTS_LEN]> for SwapKeys {
    fn from(pubkeys: [Pubkey; SWAP_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: pubkeys[0],
            amm: pubkeys[1],
            user_transfer_authority: pubkeys[2],
            source_info: pubkeys[3],
            destination_info: pubkeys[4],
            swap_source: pubkeys[5],
            swap_destination: pubkeys[6],
            pool_mint: pubkeys[7],
            fee_account: pubkeys[8],
            token_program: pubkeys[9],
            oracle_main_account: pubkeys[10],
            oracle_sub_account: pubkeys[11],
            oracle_pc_account: pubkeys[12],
        }
    }
}
impl<'info> From<SwapAccounts<'_, 'info>> for [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapAccounts<'_, 'info>) -> Self {
        [
            accounts.authority.clone(),
            accounts.amm.clone(),
            accounts.user_transfer_authority.clone(),
            accounts.source_info.clone(),
            accounts.destination_info.clone(),
            accounts.swap_source.clone(),
            accounts.swap_destination.clone(),
            accounts.pool_mint.clone(),
            accounts.fee_account.clone(),
            accounts.token_program.clone(),
            accounts.oracle_main_account.clone(),
            accounts.oracle_sub_account.clone(),
            accounts.oracle_pc_account.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]> for SwapAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: &arr[0],
            amm: &arr[1],
            user_transfer_authority: &arr[2],
            source_info: &arr[3],
            destination_info: &arr[4],
            swap_source: &arr[5],
            swap_destination: &arr[6],
            pool_mint: &arr[7],
            fee_account: &arr[8],
            token_program: &arr[9],
            oracle_main_account: &arr[10],
            oracle_sub_account: &arr[11],
            oracle_pc_account: &arr[12],
        }
    }
}
pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapIxArgs {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapIxData(pub SwapIxArgs);
impl From<SwapIxArgs> for SwapIxData {
    fn from(args: SwapIxArgs) -> Self {
        Self(args)
    }
}
impl SwapIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SWAP_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {SWAP_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(SwapIxArgs::deserialize(&mut reader)?))
    }
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

pub const DEPOSIT_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN: usize = 10;
#[derive(Copy, Clone, Debug)]
pub struct DepositAllTokenTypesAccounts<'me, 'info> {
    pub amm: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub user_transfer_authority_info: &'me AccountInfo<'info>,
    pub source_a_info: &'me AccountInfo<'info>,
    pub source_b_info: &'me AccountInfo<'info>,
    pub token_a: &'me AccountInfo<'info>,
    pub token_b: &'me AccountInfo<'info>,
    pub pool_mint: &'me AccountInfo<'info>,
    pub destination: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DepositAllTokenTypesKeys {
    pub amm: Pubkey,
    pub authority: Pubkey,
    pub user_transfer_authority_info: Pubkey,
    pub source_a_info: Pubkey,
    pub source_b_info: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub pool_mint: Pubkey,
    pub destination: Pubkey,
    pub token_program: Pubkey,
}
impl From<DepositAllTokenTypesAccounts<'_, '_>> for DepositAllTokenTypesKeys {
    fn from(accounts: DepositAllTokenTypesAccounts) -> Self {
        Self {
            amm: *accounts.amm.key,
            authority: *accounts.authority.key,
            user_transfer_authority_info: *accounts.user_transfer_authority_info.key,
            source_a_info: *accounts.source_a_info.key,
            source_b_info: *accounts.source_b_info.key,
            token_a: *accounts.token_a.key,
            token_b: *accounts.token_b.key,
            pool_mint: *accounts.pool_mint.key,
            destination: *accounts.destination.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<DepositAllTokenTypesKeys> for [AccountMeta; DEPOSIT_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN] {
    fn from(keys: DepositAllTokenTypesKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.amm,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user_transfer_authority_info,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.source_a_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.source_b_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_a,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_b,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; DEPOSIT_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]> for DepositAllTokenTypesKeys {
    fn from(pubkeys: [Pubkey; DEPOSIT_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            amm: pubkeys[0],
            authority: pubkeys[1],
            user_transfer_authority_info: pubkeys[2],
            source_a_info: pubkeys[3],
            source_b_info: pubkeys[4],
            token_a: pubkeys[5],
            token_b: pubkeys[6],
            pool_mint: pubkeys[7],
            destination: pubkeys[8],
            token_program: pubkeys[9],
        }
    }
}
impl<'info> From<DepositAllTokenTypesAccounts<'_, 'info>>
    for [AccountInfo<'info>; DEPOSIT_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]
{
    fn from(accounts: DepositAllTokenTypesAccounts<'_, 'info>) -> Self {
        [
            accounts.amm.clone(),
            accounts.authority.clone(),
            accounts.user_transfer_authority_info.clone(),
            accounts.source_a_info.clone(),
            accounts.source_b_info.clone(),
            accounts.token_a.clone(),
            accounts.token_b.clone(),
            accounts.pool_mint.clone(),
            accounts.destination.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; DEPOSIT_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]>
    for DepositAllTokenTypesAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; DEPOSIT_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            amm: &arr[0],
            authority: &arr[1],
            user_transfer_authority_info: &arr[2],
            source_a_info: &arr[3],
            source_b_info: &arr[4],
            token_a: &arr[5],
            token_b: &arr[6],
            pool_mint: &arr[7],
            destination: &arr[8],
            token_program: &arr[9],
        }
    }
}
pub const DEPOSIT_ALL_TOKEN_TYPES_IX_DISCM: [u8; 8] = [32, 95, 69, 60, 75, 79, 205, 238];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct DepositAllTokenTypesIxArgs {
    pub pool_token_amount: u64,
    pub maximum_token_a_amount: u64,
    pub maximum_token_b_amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DepositAllTokenTypesIxData(pub DepositAllTokenTypesIxArgs);
impl From<DepositAllTokenTypesIxArgs> for DepositAllTokenTypesIxData {
    fn from(args: DepositAllTokenTypesIxArgs) -> Self {
        Self(args)
    }
}
impl DepositAllTokenTypesIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != DEPOSIT_ALL_TOKEN_TYPES_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {DEPOSIT_ALL_TOKEN_TYPES_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(DepositAllTokenTypesIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&DEPOSIT_ALL_TOKEN_TYPES_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub const WITHDRAW_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN: usize = 10;
#[derive(Copy, Clone, Debug)]
pub struct WithdrawAllTokenTypesAccounts<'me, 'info> {
    pub amm: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub user_transfer_authority_info: &'me AccountInfo<'info>,
    pub source_info: &'me AccountInfo<'info>,
    pub token_a: &'me AccountInfo<'info>,
    pub token_b: &'me AccountInfo<'info>,
    pub pool_mint: &'me AccountInfo<'info>,
    pub dest_token_a_info: &'me AccountInfo<'info>,
    pub dest_token_b_info: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WithdrawAllTokenTypesKeys {
    pub amm: Pubkey,
    pub authority: Pubkey,
    pub user_transfer_authority_info: Pubkey,
    pub source_info: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub pool_mint: Pubkey,
    pub dest_token_a_info: Pubkey,
    pub dest_token_b_info: Pubkey,
    pub token_program: Pubkey,
}
impl From<WithdrawAllTokenTypesAccounts<'_, '_>> for WithdrawAllTokenTypesKeys {
    fn from(accounts: WithdrawAllTokenTypesAccounts) -> Self {
        Self {
            amm: *accounts.amm.key,
            authority: *accounts.authority.key,
            user_transfer_authority_info: *accounts.user_transfer_authority_info.key,
            source_info: *accounts.source_info.key,
            token_a: *accounts.token_a.key,
            token_b: *accounts.token_b.key,
            pool_mint: *accounts.pool_mint.key,
            dest_token_a_info: *accounts.dest_token_a_info.key,
            dest_token_b_info: *accounts.dest_token_b_info.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<WithdrawAllTokenTypesKeys> for [AccountMeta; WITHDRAW_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN] {
    fn from(keys: WithdrawAllTokenTypesKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.amm,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user_transfer_authority_info,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.source_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_a,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_b,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dest_token_a_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dest_token_b_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; WITHDRAW_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]> for WithdrawAllTokenTypesKeys {
    fn from(pubkeys: [Pubkey; WITHDRAW_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            amm: pubkeys[0],
            authority: pubkeys[1],
            user_transfer_authority_info: pubkeys[2],
            source_info: pubkeys[3],
            token_a: pubkeys[4],
            token_b: pubkeys[5],
            pool_mint: pubkeys[6],
            dest_token_a_info: pubkeys[7],
            dest_token_b_info: pubkeys[8],
            token_program: pubkeys[9],
        }
    }
}
impl<'info> From<WithdrawAllTokenTypesAccounts<'_, 'info>>
    for [AccountInfo<'info>; WITHDRAW_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]
{
    fn from(accounts: WithdrawAllTokenTypesAccounts<'_, 'info>) -> Self {
        [
            accounts.amm.clone(),
            accounts.authority.clone(),
            accounts.user_transfer_authority_info.clone(),
            accounts.source_info.clone(),
            accounts.token_a.clone(),
            accounts.token_b.clone(),
            accounts.pool_mint.clone(),
            accounts.dest_token_a_info.clone(),
            accounts.dest_token_b_info.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; WITHDRAW_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]>
    for WithdrawAllTokenTypesAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; WITHDRAW_ALL_TOKEN_TYPES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            amm: &arr[0],
            authority: &arr[1],
            user_transfer_authority_info: &arr[2],
            source_info: &arr[3],
            token_a: &arr[4],
            token_b: &arr[5],
            pool_mint: &arr[6],
            dest_token_a_info: &arr[7],
            dest_token_b_info: &arr[8],
            token_program: &arr[9],
        }
    }
}
pub const WITHDRAW_ALL_TOKEN_TYPES_IX_DISCM: [u8; 8] = [189, 254, 156, 174, 210, 9, 164, 216];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct WithdrawAllTokenTypesIxArgs {
    pub pool_token_amount: u64,
    pub minimum_token_a_amount: u64,
    pub minimum_token_b_amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct WithdrawAllTokenTypesIxData(pub WithdrawAllTokenTypesIxArgs);
impl From<WithdrawAllTokenTypesIxArgs> for WithdrawAllTokenTypesIxData {
    fn from(args: WithdrawAllTokenTypesIxArgs) -> Self {
        Self(args)
    }
}
impl WithdrawAllTokenTypesIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != WITHDRAW_ALL_TOKEN_TYPES_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {WITHDRAW_ALL_TOKEN_TYPES_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(WithdrawAllTokenTypesIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&WITHDRAW_ALL_TOKEN_TYPES_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub fn withdraw_all_token_types_verify_account_keys(
    accounts: WithdrawAllTokenTypesAccounts<'_, '_>,
    keys: WithdrawAllTokenTypesKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.amm.key, keys.amm),
        (*accounts.authority.key, keys.authority),
        (
            *accounts.user_transfer_authority_info.key,
            keys.user_transfer_authority_info,
        ),
        (*accounts.source_info.key, keys.source_info),
        (*accounts.token_a.key, keys.token_a),
        (*accounts.token_b.key, keys.token_b),
        (*accounts.pool_mint.key, keys.pool_mint),
        (*accounts.dest_token_a_info.key, keys.dest_token_a_info),
        (*accounts.dest_token_b_info.key, keys.dest_token_b_info),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn withdraw_all_token_types_verify_writable_privileges<'me, 'info>(
    accounts: WithdrawAllTokenTypesAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.amm,
        accounts.source_info,
        accounts.token_a,
        accounts.token_b,
        accounts.pool_mint,
        accounts.dest_token_a_info,
        accounts.dest_token_b_info,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
