use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    pubkey::Pubkey, program_error::ProgramError,
};
use std::io::Read;
#[derive(Clone, Debug, PartialEq)]
pub enum ObricSolanaProgramIx {
    CreatePair(CreatePairIxArgs),
    CreatePairV2(CreatePairV2IxArgs),
    UpdateConcentration(UpdateConcentrationIxArgs),
    UpdateVersion(UpdateVersionIxArgs),
    UpdateFeeParams(UpdateFeeParamsIxArgs),
    UpdateOracles,
    WithdrawFees,
    Deposit(DepositIxArgs),
    Withdraw(WithdrawIxArgs),
    SwapXToY(SwapXToYIxArgs),
    SwapYToX(SwapYToXIxArgs),
    Swap(SwapIxArgs),
}
impl ObricSolanaProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            CREATE_PAIR_IX_DISCM => {
                Ok(Self::CreatePair(CreatePairIxArgs::deserialize(&mut reader)?))
            }
            CREATE_PAIR_V2_IX_DISCM => {
                Ok(Self::CreatePairV2(CreatePairV2IxArgs::deserialize(&mut reader)?))
            }
            UPDATE_CONCENTRATION_IX_DISCM => {
                Ok(
                    Self::UpdateConcentration(
                        UpdateConcentrationIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            UPDATE_VERSION_IX_DISCM => {
                Ok(Self::UpdateVersion(UpdateVersionIxArgs::deserialize(&mut reader)?))
            }
            UPDATE_FEE_PARAMS_IX_DISCM => {
                Ok(
                    Self::UpdateFeeParams(
                        UpdateFeeParamsIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            UPDATE_ORACLES_IX_DISCM => Ok(Self::UpdateOracles),
            WITHDRAW_FEES_IX_DISCM => Ok(Self::WithdrawFees),
            DEPOSIT_IX_DISCM => {
                Ok(Self::Deposit(DepositIxArgs::deserialize(&mut reader)?))
            }
            WITHDRAW_IX_DISCM => {
                Ok(Self::Withdraw(WithdrawIxArgs::deserialize(&mut reader)?))
            }
            SWAP_X_TO_Y_IX_DISCM => {
                Ok(Self::SwapXToY(SwapXToYIxArgs::deserialize(&mut reader)?))
            }
            SWAP_Y_TO_X_IX_DISCM => {
                Ok(Self::SwapYToX(SwapYToXIxArgs::deserialize(&mut reader)?))
            }
            SWAP_IX_DISCM => Ok(Self::Swap(SwapIxArgs::deserialize(&mut reader)?)),
            _ => {
                Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("discm {:?} not found", maybe_discm),
                    ),
                )
            }
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::CreatePair(args) => {
                writer.write_all(&CREATE_PAIR_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CreatePairV2(args) => {
                writer.write_all(&CREATE_PAIR_V2_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::UpdateConcentration(args) => {
                writer.write_all(&UPDATE_CONCENTRATION_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::UpdateVersion(args) => {
                writer.write_all(&UPDATE_VERSION_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::UpdateFeeParams(args) => {
                writer.write_all(&UPDATE_FEE_PARAMS_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::UpdateOracles => writer.write_all(&UPDATE_ORACLES_IX_DISCM),
            Self::WithdrawFees => writer.write_all(&WITHDRAW_FEES_IX_DISCM),
            Self::Deposit(args) => {
                writer.write_all(&DEPOSIT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Withdraw(args) => {
                writer.write_all(&WITHDRAW_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::SwapXToY(args) => {
                writer.write_all(&SWAP_X_TO_Y_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::SwapYToX(args) => {
                writer.write_all(&SWAP_Y_TO_X_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Swap(args) => {
                writer.write_all(&SWAP_IX_DISCM)?;
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
fn invoke_instruction<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
    ix: &Instruction,
    accounts: A,
) -> ProgramResult {
    let account_info: [AccountInfo<'info>; N] = accounts.into();
    invoke(ix, &account_info)
}
fn invoke_instruction_signed<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
    ix: &Instruction,
    accounts: A,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let account_info: [AccountInfo<'info>; N] = accounts.into();
    invoke_signed(ix, &account_info, seeds)
}
pub const CREATE_PAIR_IX_ACCOUNTS_LEN: usize = 14;
#[derive(Copy, Clone, Debug)]
pub struct CreatePairAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub mint_sslp_x: &'me AccountInfo<'info>,
    pub mint_sslp_y: &'me AccountInfo<'info>,
    pub reserve_x: &'me AccountInfo<'info>,
    pub reserve_y: &'me AccountInfo<'info>,
    pub protocol_fee_x: &'me AccountInfo<'info>,
    pub protocol_fee_y: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreatePairKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub mint_sslp_x: Pubkey,
    pub mint_sslp_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub protocol_fee_x: Pubkey,
    pub protocol_fee_y: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
}
impl From<CreatePairAccounts<'_, '_>> for CreatePairKeys {
    fn from(accounts: CreatePairAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            mint_sslp_x: *accounts.mint_sslp_x.key,
            mint_sslp_y: *accounts.mint_sslp_y.key,
            reserve_x: *accounts.reserve_x.key,
            reserve_y: *accounts.reserve_y.key,
            protocol_fee_x: *accounts.protocol_fee_x.key,
            protocol_fee_y: *accounts.protocol_fee_y.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
            system_program: *accounts.system_program.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<CreatePairKeys> for [AccountMeta; CREATE_PAIR_IX_ACCOUNTS_LEN] {
    fn from(keys: CreatePairKeys) -> Self {
        [
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
                pubkey: keys.mint_sslp_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint_sslp_y,
                is_signer: false,
                is_writable: true,
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
                pubkey: keys.protocol_fee_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_y,
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
impl From<[Pubkey; CREATE_PAIR_IX_ACCOUNTS_LEN]> for CreatePairKeys {
    fn from(pubkeys: [Pubkey; CREATE_PAIR_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            mint_sslp_x: pubkeys[3],
            mint_sslp_y: pubkeys[4],
            reserve_x: pubkeys[5],
            reserve_y: pubkeys[6],
            protocol_fee_x: pubkeys[7],
            protocol_fee_y: pubkeys[8],
            x_price_feed: pubkeys[9],
            y_price_feed: pubkeys[10],
            user: pubkeys[11],
            system_program: pubkeys[12],
            token_program: pubkeys[13],
        }
    }
}
impl<'info> From<CreatePairAccounts<'_, 'info>>
for [AccountInfo<'info>; CREATE_PAIR_IX_ACCOUNTS_LEN] {
    fn from(accounts: CreatePairAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.mint_sslp_x.clone(),
            accounts.mint_sslp_y.clone(),
            accounts.reserve_x.clone(),
            accounts.reserve_y.clone(),
            accounts.protocol_fee_x.clone(),
            accounts.protocol_fee_y.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_PAIR_IX_ACCOUNTS_LEN]>
for CreatePairAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CREATE_PAIR_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            mint_sslp_x: &arr[3],
            mint_sslp_y: &arr[4],
            reserve_x: &arr[5],
            reserve_y: &arr[6],
            protocol_fee_x: &arr[7],
            protocol_fee_y: &arr[8],
            x_price_feed: &arr[9],
            y_price_feed: &arr[10],
            user: &arr[11],
            system_program: &arr[12],
            token_program: &arr[13],
        }
    }
}
pub const CREATE_PAIR_IX_DISCM: [u8; 8] = [156, 190, 126, 151, 163, 62, 192, 220];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CreatePairIxArgs {
    pub concentration: u64,
    pub fee_millionth: u64,
    pub rebate_percentage: u64,
    pub protocol_fee_share_thousandth: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreatePairIxData(pub CreatePairIxArgs);
impl From<CreatePairIxArgs> for CreatePairIxData {
    fn from(args: CreatePairIxArgs) -> Self {
        Self(args)
    }
}
impl CreatePairIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CREATE_PAIR_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        CREATE_PAIR_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(CreatePairIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CREATE_PAIR_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn create_pair_ix_with_program_id(
    program_id: Pubkey,
    keys: CreatePairKeys,
    args: CreatePairIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CREATE_PAIR_IX_ACCOUNTS_LEN] = keys.into();
    let data: CreatePairIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn create_pair_ix(
    keys: CreatePairKeys,
    args: CreatePairIxArgs,
) -> std::io::Result<Instruction> {
    create_pair_ix_with_program_id(super::ID, keys, args)
}
pub fn create_pair_invoke_with_program_id(
    program_id: Pubkey,
    accounts: CreatePairAccounts<'_, '_>,
    args: CreatePairIxArgs,
) -> ProgramResult {
    let keys: CreatePairKeys = accounts.into();
    let ix = create_pair_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn create_pair_invoke(
    accounts: CreatePairAccounts<'_, '_>,
    args: CreatePairIxArgs,
) -> ProgramResult {
    create_pair_invoke_with_program_id(super::ID, accounts, args)
}
pub fn create_pair_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: CreatePairAccounts<'_, '_>,
    args: CreatePairIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: CreatePairKeys = accounts.into();
    let ix = create_pair_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn create_pair_invoke_signed(
    accounts: CreatePairAccounts<'_, '_>,
    args: CreatePairIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    create_pair_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn create_pair_verify_account_keys(
    accounts: CreatePairAccounts<'_, '_>,
    keys: CreatePairKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.mint_sslp_x.key, keys.mint_sslp_x),
        (*accounts.mint_sslp_y.key, keys.mint_sslp_y),
        (*accounts.reserve_x.key, keys.reserve_x),
        (*accounts.reserve_y.key, keys.reserve_y),
        (*accounts.protocol_fee_x.key, keys.protocol_fee_x),
        (*accounts.protocol_fee_y.key, keys.protocol_fee_y),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn create_pair_verify_writable_privileges<'me, 'info>(
    accounts: CreatePairAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.trading_pair,
        accounts.mint_sslp_x,
        accounts.mint_sslp_y,
        accounts.reserve_x,
        accounts.reserve_y,
        accounts.protocol_fee_x,
        accounts.protocol_fee_y,
        accounts.user,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn create_pair_verify_signer_privileges<'me, 'info>(
    accounts: CreatePairAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn create_pair_verify_account_privileges<'me, 'info>(
    accounts: CreatePairAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    create_pair_verify_writable_privileges(accounts)?;
    create_pair_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CREATE_PAIR_V2_IX_ACCOUNTS_LEN: usize = 14;
#[derive(Copy, Clone, Debug)]
pub struct CreatePairV2Accounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub mint_sslp_x: &'me AccountInfo<'info>,
    pub mint_sslp_y: &'me AccountInfo<'info>,
    pub reserve_x: &'me AccountInfo<'info>,
    pub reserve_y: &'me AccountInfo<'info>,
    pub protocol_fee_x: &'me AccountInfo<'info>,
    pub protocol_fee_y: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreatePairV2Keys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub mint_sslp_x: Pubkey,
    pub mint_sslp_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub protocol_fee_x: Pubkey,
    pub protocol_fee_y: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
}
impl From<CreatePairV2Accounts<'_, '_>> for CreatePairV2Keys {
    fn from(accounts: CreatePairV2Accounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            mint_sslp_x: *accounts.mint_sslp_x.key,
            mint_sslp_y: *accounts.mint_sslp_y.key,
            reserve_x: *accounts.reserve_x.key,
            reserve_y: *accounts.reserve_y.key,
            protocol_fee_x: *accounts.protocol_fee_x.key,
            protocol_fee_y: *accounts.protocol_fee_y.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
            system_program: *accounts.system_program.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<CreatePairV2Keys> for [AccountMeta; CREATE_PAIR_V2_IX_ACCOUNTS_LEN] {
    fn from(keys: CreatePairV2Keys) -> Self {
        [
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
                pubkey: keys.mint_sslp_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint_sslp_y,
                is_signer: false,
                is_writable: true,
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
                pubkey: keys.protocol_fee_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_y,
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
impl From<[Pubkey; CREATE_PAIR_V2_IX_ACCOUNTS_LEN]> for CreatePairV2Keys {
    fn from(pubkeys: [Pubkey; CREATE_PAIR_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            mint_sslp_x: pubkeys[3],
            mint_sslp_y: pubkeys[4],
            reserve_x: pubkeys[5],
            reserve_y: pubkeys[6],
            protocol_fee_x: pubkeys[7],
            protocol_fee_y: pubkeys[8],
            x_price_feed: pubkeys[9],
            y_price_feed: pubkeys[10],
            user: pubkeys[11],
            system_program: pubkeys[12],
            token_program: pubkeys[13],
        }
    }
}
impl<'info> From<CreatePairV2Accounts<'_, 'info>>
for [AccountInfo<'info>; CREATE_PAIR_V2_IX_ACCOUNTS_LEN] {
    fn from(accounts: CreatePairV2Accounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.mint_sslp_x.clone(),
            accounts.mint_sslp_y.clone(),
            accounts.reserve_x.clone(),
            accounts.reserve_y.clone(),
            accounts.protocol_fee_x.clone(),
            accounts.protocol_fee_y.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_PAIR_V2_IX_ACCOUNTS_LEN]>
for CreatePairV2Accounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CREATE_PAIR_V2_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            mint_sslp_x: &arr[3],
            mint_sslp_y: &arr[4],
            reserve_x: &arr[5],
            reserve_y: &arr[6],
            protocol_fee_x: &arr[7],
            protocol_fee_y: &arr[8],
            x_price_feed: &arr[9],
            y_price_feed: &arr[10],
            user: &arr[11],
            system_program: &arr[12],
            token_program: &arr[13],
        }
    }
}
pub const CREATE_PAIR_V2_IX_DISCM: [u8; 8] = [55, 255, 159, 236, 123, 161, 174, 155];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct CreatePairV2IxArgs {
    pub concentration: u64,
    pub fee_millionth: u64,
    pub rebate_percentage: u64,
    pub protocol_fee_share_thousandth: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreatePairV2IxData(pub CreatePairV2IxArgs);
impl From<CreatePairV2IxArgs> for CreatePairV2IxData {
    fn from(args: CreatePairV2IxArgs) -> Self {
        Self(args)
    }
}
impl CreatePairV2IxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CREATE_PAIR_V2_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        CREATE_PAIR_V2_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(CreatePairV2IxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CREATE_PAIR_V2_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn create_pair_v2_ix_with_program_id(
    program_id: Pubkey,
    keys: CreatePairV2Keys,
    args: CreatePairV2IxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CREATE_PAIR_V2_IX_ACCOUNTS_LEN] = keys.into();
    let data: CreatePairV2IxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn create_pair_v2_ix(
    keys: CreatePairV2Keys,
    args: CreatePairV2IxArgs,
) -> std::io::Result<Instruction> {
    create_pair_v2_ix_with_program_id(super::ID, keys, args)
}
pub fn create_pair_v2_invoke_with_program_id(
    program_id: Pubkey,
    accounts: CreatePairV2Accounts<'_, '_>,
    args: CreatePairV2IxArgs,
) -> ProgramResult {
    let keys: CreatePairV2Keys = accounts.into();
    let ix = create_pair_v2_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn create_pair_v2_invoke(
    accounts: CreatePairV2Accounts<'_, '_>,
    args: CreatePairV2IxArgs,
) -> ProgramResult {
    create_pair_v2_invoke_with_program_id(super::ID, accounts, args)
}
pub fn create_pair_v2_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: CreatePairV2Accounts<'_, '_>,
    args: CreatePairV2IxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: CreatePairV2Keys = accounts.into();
    let ix = create_pair_v2_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn create_pair_v2_invoke_signed(
    accounts: CreatePairV2Accounts<'_, '_>,
    args: CreatePairV2IxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    create_pair_v2_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn create_pair_v2_verify_account_keys(
    accounts: CreatePairV2Accounts<'_, '_>,
    keys: CreatePairV2Keys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.mint_sslp_x.key, keys.mint_sslp_x),
        (*accounts.mint_sslp_y.key, keys.mint_sslp_y),
        (*accounts.reserve_x.key, keys.reserve_x),
        (*accounts.reserve_y.key, keys.reserve_y),
        (*accounts.protocol_fee_x.key, keys.protocol_fee_x),
        (*accounts.protocol_fee_y.key, keys.protocol_fee_y),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn create_pair_v2_verify_writable_privileges<'me, 'info>(
    accounts: CreatePairV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.trading_pair,
        accounts.mint_sslp_x,
        accounts.mint_sslp_y,
        accounts.reserve_x,
        accounts.reserve_y,
        accounts.protocol_fee_x,
        accounts.protocol_fee_y,
        accounts.user,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn create_pair_v2_verify_signer_privileges<'me, 'info>(
    accounts: CreatePairV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn create_pair_v2_verify_account_privileges<'me, 'info>(
    accounts: CreatePairV2Accounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    create_pair_v2_verify_writable_privileges(accounts)?;
    create_pair_v2_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct UpdateConcentrationAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdateConcentrationKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
}
impl From<UpdateConcentrationAccounts<'_, '_>> for UpdateConcentrationKeys {
    fn from(accounts: UpdateConcentrationAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
        }
    }
}
impl From<UpdateConcentrationKeys>
for [AccountMeta; UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdateConcentrationKeys) -> Self {
        [
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
        ]
    }
}
impl From<[Pubkey; UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN]> for UpdateConcentrationKeys {
    fn from(pubkeys: [Pubkey; UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            x_price_feed: pubkeys[3],
            y_price_feed: pubkeys[4],
            user: pubkeys[5],
        }
    }
}
impl<'info> From<UpdateConcentrationAccounts<'_, 'info>>
for [AccountInfo<'info>; UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdateConcentrationAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN]>
for UpdateConcentrationAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            x_price_feed: &arr[3],
            y_price_feed: &arr[4],
            user: &arr[5],
        }
    }
}
pub const UPDATE_CONCENTRATION_IX_DISCM: [u8; 8] = [252, 177, 98, 133, 34, 227, 202, 11];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdateConcentrationIxArgs {
    pub concentration: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateConcentrationIxData(pub UpdateConcentrationIxArgs);
impl From<UpdateConcentrationIxArgs> for UpdateConcentrationIxData {
    fn from(args: UpdateConcentrationIxArgs) -> Self {
        Self(args)
    }
}
impl UpdateConcentrationIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UPDATE_CONCENTRATION_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        UPDATE_CONCENTRATION_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(UpdateConcentrationIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UPDATE_CONCENTRATION_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn update_concentration_ix_with_program_id(
    program_id: Pubkey,
    keys: UpdateConcentrationKeys,
    args: UpdateConcentrationIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UPDATE_CONCENTRATION_IX_ACCOUNTS_LEN] = keys.into();
    let data: UpdateConcentrationIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn update_concentration_ix(
    keys: UpdateConcentrationKeys,
    args: UpdateConcentrationIxArgs,
) -> std::io::Result<Instruction> {
    update_concentration_ix_with_program_id(super::ID, keys, args)
}
pub fn update_concentration_invoke_with_program_id(
    program_id: Pubkey,
    accounts: UpdateConcentrationAccounts<'_, '_>,
    args: UpdateConcentrationIxArgs,
) -> ProgramResult {
    let keys: UpdateConcentrationKeys = accounts.into();
    let ix = update_concentration_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn update_concentration_invoke(
    accounts: UpdateConcentrationAccounts<'_, '_>,
    args: UpdateConcentrationIxArgs,
) -> ProgramResult {
    update_concentration_invoke_with_program_id(super::ID, accounts, args)
}
pub fn update_concentration_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: UpdateConcentrationAccounts<'_, '_>,
    args: UpdateConcentrationIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: UpdateConcentrationKeys = accounts.into();
    let ix = update_concentration_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn update_concentration_invoke_signed(
    accounts: UpdateConcentrationAccounts<'_, '_>,
    args: UpdateConcentrationIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    update_concentration_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn update_concentration_verify_account_keys(
    accounts: UpdateConcentrationAccounts<'_, '_>,
    keys: UpdateConcentrationKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn update_concentration_verify_writable_privileges<'me, 'info>(
    accounts: UpdateConcentrationAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.trading_pair] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn update_concentration_verify_signer_privileges<'me, 'info>(
    accounts: UpdateConcentrationAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn update_concentration_verify_account_privileges<'me, 'info>(
    accounts: UpdateConcentrationAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    update_concentration_verify_writable_privileges(accounts)?;
    update_concentration_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UPDATE_VERSION_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct UpdateVersionAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub mint_sslp_x: &'me AccountInfo<'info>,
    pub mint_sslp_y: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdateVersionKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub mint_sslp_x: Pubkey,
    pub mint_sslp_y: Pubkey,
    pub user: Pubkey,
}
impl From<UpdateVersionAccounts<'_, '_>> for UpdateVersionKeys {
    fn from(accounts: UpdateVersionAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            mint_sslp_x: *accounts.mint_sslp_x.key,
            mint_sslp_y: *accounts.mint_sslp_y.key,
            user: *accounts.user.key,
        }
    }
}
impl From<UpdateVersionKeys> for [AccountMeta; UPDATE_VERSION_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdateVersionKeys) -> Self {
        [
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
                pubkey: keys.mint_sslp_x,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.mint_sslp_y,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user,
                is_signer: true,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; UPDATE_VERSION_IX_ACCOUNTS_LEN]> for UpdateVersionKeys {
    fn from(pubkeys: [Pubkey; UPDATE_VERSION_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            mint_sslp_x: pubkeys[3],
            mint_sslp_y: pubkeys[4],
            user: pubkeys[5],
        }
    }
}
impl<'info> From<UpdateVersionAccounts<'_, 'info>>
for [AccountInfo<'info>; UPDATE_VERSION_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdateVersionAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.mint_sslp_x.clone(),
            accounts.mint_sslp_y.clone(),
            accounts.user.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_VERSION_IX_ACCOUNTS_LEN]>
for UpdateVersionAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_VERSION_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            mint_sslp_x: &arr[3],
            mint_sslp_y: &arr[4],
            user: &arr[5],
        }
    }
}
pub const UPDATE_VERSION_IX_DISCM: [u8; 8] = [248, 152, 89, 69, 41, 63, 148, 30];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdateVersionIxArgs {
    pub version: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateVersionIxData(pub UpdateVersionIxArgs);
impl From<UpdateVersionIxArgs> for UpdateVersionIxData {
    fn from(args: UpdateVersionIxArgs) -> Self {
        Self(args)
    }
}
impl UpdateVersionIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UPDATE_VERSION_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        UPDATE_VERSION_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(UpdateVersionIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UPDATE_VERSION_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn update_version_ix_with_program_id(
    program_id: Pubkey,
    keys: UpdateVersionKeys,
    args: UpdateVersionIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UPDATE_VERSION_IX_ACCOUNTS_LEN] = keys.into();
    let data: UpdateVersionIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn update_version_ix(
    keys: UpdateVersionKeys,
    args: UpdateVersionIxArgs,
) -> std::io::Result<Instruction> {
    update_version_ix_with_program_id(super::ID, keys, args)
}
pub fn update_version_invoke_with_program_id(
    program_id: Pubkey,
    accounts: UpdateVersionAccounts<'_, '_>,
    args: UpdateVersionIxArgs,
) -> ProgramResult {
    let keys: UpdateVersionKeys = accounts.into();
    let ix = update_version_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn update_version_invoke(
    accounts: UpdateVersionAccounts<'_, '_>,
    args: UpdateVersionIxArgs,
) -> ProgramResult {
    update_version_invoke_with_program_id(super::ID, accounts, args)
}
pub fn update_version_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: UpdateVersionAccounts<'_, '_>,
    args: UpdateVersionIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: UpdateVersionKeys = accounts.into();
    let ix = update_version_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn update_version_invoke_signed(
    accounts: UpdateVersionAccounts<'_, '_>,
    args: UpdateVersionIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    update_version_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn update_version_verify_account_keys(
    accounts: UpdateVersionAccounts<'_, '_>,
    keys: UpdateVersionKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.mint_sslp_x.key, keys.mint_sslp_x),
        (*accounts.mint_sslp_y.key, keys.mint_sslp_y),
        (*accounts.user.key, keys.user),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn update_version_verify_writable_privileges<'me, 'info>(
    accounts: UpdateVersionAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.trading_pair] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn update_version_verify_signer_privileges<'me, 'info>(
    accounts: UpdateVersionAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn update_version_verify_account_privileges<'me, 'info>(
    accounts: UpdateVersionAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    update_version_verify_writable_privileges(accounts)?;
    update_version_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN: usize = 4;
#[derive(Copy, Clone, Debug)]
pub struct UpdateFeeParamsAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdateFeeParamsKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub user: Pubkey,
}
impl From<UpdateFeeParamsAccounts<'_, '_>> for UpdateFeeParamsKeys {
    fn from(accounts: UpdateFeeParamsAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            user: *accounts.user.key,
        }
    }
}
impl From<UpdateFeeParamsKeys> for [AccountMeta; UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdateFeeParamsKeys) -> Self {
        [
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
                pubkey: keys.user,
                is_signer: true,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN]> for UpdateFeeParamsKeys {
    fn from(pubkeys: [Pubkey; UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            user: pubkeys[3],
        }
    }
}
impl<'info> From<UpdateFeeParamsAccounts<'_, 'info>>
for [AccountInfo<'info>; UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdateFeeParamsAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.user.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN]>
for UpdateFeeParamsAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            user: &arr[3],
        }
    }
}
pub const UPDATE_FEE_PARAMS_IX_DISCM: [u8; 8] = [223, 116, 30, 197, 161, 4, 201, 146];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct UpdateFeeParamsIxArgs {
    pub fee_millionth: u64,
    pub protocol_fee_share_thousandth: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateFeeParamsIxData(pub UpdateFeeParamsIxArgs);
impl From<UpdateFeeParamsIxArgs> for UpdateFeeParamsIxData {
    fn from(args: UpdateFeeParamsIxArgs) -> Self {
        Self(args)
    }
}
impl UpdateFeeParamsIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UPDATE_FEE_PARAMS_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        UPDATE_FEE_PARAMS_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(UpdateFeeParamsIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UPDATE_FEE_PARAMS_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn update_fee_params_ix_with_program_id(
    program_id: Pubkey,
    keys: UpdateFeeParamsKeys,
    args: UpdateFeeParamsIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UPDATE_FEE_PARAMS_IX_ACCOUNTS_LEN] = keys.into();
    let data: UpdateFeeParamsIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn update_fee_params_ix(
    keys: UpdateFeeParamsKeys,
    args: UpdateFeeParamsIxArgs,
) -> std::io::Result<Instruction> {
    update_fee_params_ix_with_program_id(super::ID, keys, args)
}
pub fn update_fee_params_invoke_with_program_id(
    program_id: Pubkey,
    accounts: UpdateFeeParamsAccounts<'_, '_>,
    args: UpdateFeeParamsIxArgs,
) -> ProgramResult {
    let keys: UpdateFeeParamsKeys = accounts.into();
    let ix = update_fee_params_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn update_fee_params_invoke(
    accounts: UpdateFeeParamsAccounts<'_, '_>,
    args: UpdateFeeParamsIxArgs,
) -> ProgramResult {
    update_fee_params_invoke_with_program_id(super::ID, accounts, args)
}
pub fn update_fee_params_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: UpdateFeeParamsAccounts<'_, '_>,
    args: UpdateFeeParamsIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: UpdateFeeParamsKeys = accounts.into();
    let ix = update_fee_params_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn update_fee_params_invoke_signed(
    accounts: UpdateFeeParamsAccounts<'_, '_>,
    args: UpdateFeeParamsIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    update_fee_params_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn update_fee_params_verify_account_keys(
    accounts: UpdateFeeParamsAccounts<'_, '_>,
    keys: UpdateFeeParamsKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.user.key, keys.user),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn update_fee_params_verify_writable_privileges<'me, 'info>(
    accounts: UpdateFeeParamsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.trading_pair] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn update_fee_params_verify_signer_privileges<'me, 'info>(
    accounts: UpdateFeeParamsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn update_fee_params_verify_account_privileges<'me, 'info>(
    accounts: UpdateFeeParamsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    update_fee_params_verify_writable_privileges(accounts)?;
    update_fee_params_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UPDATE_ORACLES_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct UpdateOraclesAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpdateOraclesKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
}
impl From<UpdateOraclesAccounts<'_, '_>> for UpdateOraclesKeys {
    fn from(accounts: UpdateOraclesAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
        }
    }
}
impl From<UpdateOraclesKeys> for [AccountMeta; UPDATE_ORACLES_IX_ACCOUNTS_LEN] {
    fn from(keys: UpdateOraclesKeys) -> Self {
        [
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
        ]
    }
}
impl From<[Pubkey; UPDATE_ORACLES_IX_ACCOUNTS_LEN]> for UpdateOraclesKeys {
    fn from(pubkeys: [Pubkey; UPDATE_ORACLES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            x_price_feed: pubkeys[3],
            y_price_feed: pubkeys[4],
            user: pubkeys[5],
        }
    }
}
impl<'info> From<UpdateOraclesAccounts<'_, 'info>>
for [AccountInfo<'info>; UPDATE_ORACLES_IX_ACCOUNTS_LEN] {
    fn from(accounts: UpdateOraclesAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPDATE_ORACLES_IX_ACCOUNTS_LEN]>
for UpdateOraclesAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; UPDATE_ORACLES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            x_price_feed: &arr[3],
            y_price_feed: &arr[4],
            user: &arr[5],
        }
    }
}
pub const UPDATE_ORACLES_IX_DISCM: [u8; 8] = [209, 115, 103, 72, 108, 69, 218, 189];
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateOraclesIxData;
impl UpdateOraclesIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UPDATE_ORACLES_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        UPDATE_ORACLES_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UPDATE_ORACLES_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn update_oracles_ix_with_program_id(
    program_id: Pubkey,
    keys: UpdateOraclesKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UPDATE_ORACLES_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: UpdateOraclesIxData.try_to_vec()?,
    })
}
pub fn update_oracles_ix(keys: UpdateOraclesKeys) -> std::io::Result<Instruction> {
    update_oracles_ix_with_program_id(super::ID, keys)
}
pub fn update_oracles_invoke_with_program_id(
    program_id: Pubkey,
    accounts: UpdateOraclesAccounts<'_, '_>,
) -> ProgramResult {
    let keys: UpdateOraclesKeys = accounts.into();
    let ix = update_oracles_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn update_oracles_invoke(accounts: UpdateOraclesAccounts<'_, '_>) -> ProgramResult {
    update_oracles_invoke_with_program_id(super::ID, accounts)
}
pub fn update_oracles_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: UpdateOraclesAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: UpdateOraclesKeys = accounts.into();
    let ix = update_oracles_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn update_oracles_invoke_signed(
    accounts: UpdateOraclesAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    update_oracles_invoke_signed_with_program_id(super::ID, accounts, seeds)
}
pub fn update_oracles_verify_account_keys(
    accounts: UpdateOraclesAccounts<'_, '_>,
    keys: UpdateOraclesKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn update_oracles_verify_writable_privileges<'me, 'info>(
    accounts: UpdateOraclesAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.trading_pair] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn update_oracles_verify_signer_privileges<'me, 'info>(
    accounts: UpdateOraclesAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn update_oracles_verify_account_privileges<'me, 'info>(
    accounts: UpdateOraclesAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    update_oracles_verify_writable_privileges(accounts)?;
    update_oracles_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const WITHDRAW_FEES_IX_ACCOUNTS_LEN: usize = 9;
#[derive(Copy, Clone, Debug)]
pub struct WithdrawFeesAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub protocol_fee_x: &'me AccountInfo<'info>,
    pub protocol_fee_y: &'me AccountInfo<'info>,
    pub user_token_account_x: &'me AccountInfo<'info>,
    pub user_token_account_y: &'me AccountInfo<'info>,
    pub signer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WithdrawFeesKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub protocol_fee_x: Pubkey,
    pub protocol_fee_y: Pubkey,
    pub user_token_account_x: Pubkey,
    pub user_token_account_y: Pubkey,
    pub signer: Pubkey,
    pub token_program: Pubkey,
}
impl From<WithdrawFeesAccounts<'_, '_>> for WithdrawFeesKeys {
    fn from(accounts: WithdrawFeesAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            protocol_fee_x: *accounts.protocol_fee_x.key,
            protocol_fee_y: *accounts.protocol_fee_y.key,
            user_token_account_x: *accounts.user_token_account_x.key,
            user_token_account_y: *accounts.user_token_account_y.key,
            signer: *accounts.signer.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<WithdrawFeesKeys> for [AccountMeta; WITHDRAW_FEES_IX_ACCOUNTS_LEN] {
    fn from(keys: WithdrawFeesKeys) -> Self {
        [
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
                pubkey: keys.protocol_fee_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_y,
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
                pubkey: keys.signer,
                is_signer: true,
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
impl From<[Pubkey; WITHDRAW_FEES_IX_ACCOUNTS_LEN]> for WithdrawFeesKeys {
    fn from(pubkeys: [Pubkey; WITHDRAW_FEES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            protocol_fee_x: pubkeys[3],
            protocol_fee_y: pubkeys[4],
            user_token_account_x: pubkeys[5],
            user_token_account_y: pubkeys[6],
            signer: pubkeys[7],
            token_program: pubkeys[8],
        }
    }
}
impl<'info> From<WithdrawFeesAccounts<'_, 'info>>
for [AccountInfo<'info>; WITHDRAW_FEES_IX_ACCOUNTS_LEN] {
    fn from(accounts: WithdrawFeesAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.protocol_fee_x.clone(),
            accounts.protocol_fee_y.clone(),
            accounts.user_token_account_x.clone(),
            accounts.user_token_account_y.clone(),
            accounts.signer.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; WITHDRAW_FEES_IX_ACCOUNTS_LEN]>
for WithdrawFeesAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; WITHDRAW_FEES_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            protocol_fee_x: &arr[3],
            protocol_fee_y: &arr[4],
            user_token_account_x: &arr[5],
            user_token_account_y: &arr[6],
            signer: &arr[7],
            token_program: &arr[8],
        }
    }
}
pub const WITHDRAW_FEES_IX_DISCM: [u8; 8] = [198, 212, 171, 109, 144, 215, 174, 89];
#[derive(Clone, Debug, PartialEq)]
pub struct WithdrawFeesIxData;
impl WithdrawFeesIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != WITHDRAW_FEES_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        WITHDRAW_FEES_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&WITHDRAW_FEES_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn withdraw_fees_ix_with_program_id(
    program_id: Pubkey,
    keys: WithdrawFeesKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; WITHDRAW_FEES_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: WithdrawFeesIxData.try_to_vec()?,
    })
}
pub fn withdraw_fees_ix(keys: WithdrawFeesKeys) -> std::io::Result<Instruction> {
    withdraw_fees_ix_with_program_id(super::ID, keys)
}
pub fn withdraw_fees_invoke_with_program_id(
    program_id: Pubkey,
    accounts: WithdrawFeesAccounts<'_, '_>,
) -> ProgramResult {
    let keys: WithdrawFeesKeys = accounts.into();
    let ix = withdraw_fees_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn withdraw_fees_invoke(accounts: WithdrawFeesAccounts<'_, '_>) -> ProgramResult {
    withdraw_fees_invoke_with_program_id(super::ID, accounts)
}
pub fn withdraw_fees_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: WithdrawFeesAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: WithdrawFeesKeys = accounts.into();
    let ix = withdraw_fees_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn withdraw_fees_invoke_signed(
    accounts: WithdrawFeesAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    withdraw_fees_invoke_signed_with_program_id(super::ID, accounts, seeds)
}
pub fn withdraw_fees_verify_account_keys(
    accounts: WithdrawFeesAccounts<'_, '_>,
    keys: WithdrawFeesKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.protocol_fee_x.key, keys.protocol_fee_x),
        (*accounts.protocol_fee_y.key, keys.protocol_fee_y),
        (*accounts.user_token_account_x.key, keys.user_token_account_x),
        (*accounts.user_token_account_y.key, keys.user_token_account_y),
        (*accounts.signer.key, keys.signer),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn withdraw_fees_verify_writable_privileges<'me, 'info>(
    accounts: WithdrawFeesAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.trading_pair,
        accounts.protocol_fee_x,
        accounts.protocol_fee_y,
        accounts.user_token_account_x,
        accounts.user_token_account_y,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn withdraw_fees_verify_signer_privileges<'me, 'info>(
    accounts: WithdrawFeesAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.signer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn withdraw_fees_verify_account_privileges<'me, 'info>(
    accounts: WithdrawFeesAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    withdraw_fees_verify_writable_privileges(accounts)?;
    withdraw_fees_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const DEPOSIT_IX_ACCOUNTS_LEN: usize = 15;
#[derive(Copy, Clone, Debug)]
pub struct DepositAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub reserve_x: &'me AccountInfo<'info>,
    pub reserve_y: &'me AccountInfo<'info>,
    pub user_token_account_x: &'me AccountInfo<'info>,
    pub user_token_account_y: &'me AccountInfo<'info>,
    pub mint_sslp_x: &'me AccountInfo<'info>,
    pub mint_sslp_y: &'me AccountInfo<'info>,
    pub user_sslp_x: &'me AccountInfo<'info>,
    pub user_sslp_y: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DepositKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub user_token_account_x: Pubkey,
    pub user_token_account_y: Pubkey,
    pub mint_sslp_x: Pubkey,
    pub mint_sslp_y: Pubkey,
    pub user_sslp_x: Pubkey,
    pub user_sslp_y: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
    pub token_program: Pubkey,
}
impl From<DepositAccounts<'_, '_>> for DepositKeys {
    fn from(accounts: DepositAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            reserve_x: *accounts.reserve_x.key,
            reserve_y: *accounts.reserve_y.key,
            user_token_account_x: *accounts.user_token_account_x.key,
            user_token_account_y: *accounts.user_token_account_y.key,
            mint_sslp_x: *accounts.mint_sslp_x.key,
            mint_sslp_y: *accounts.mint_sslp_y.key,
            user_sslp_x: *accounts.user_sslp_x.key,
            user_sslp_y: *accounts.user_sslp_y.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<DepositKeys> for [AccountMeta; DEPOSIT_IX_ACCOUNTS_LEN] {
    fn from(keys: DepositKeys) -> Self {
        [
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
                pubkey: keys.mint_sslp_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint_sslp_y,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_sslp_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_sslp_y,
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
impl From<[Pubkey; DEPOSIT_IX_ACCOUNTS_LEN]> for DepositKeys {
    fn from(pubkeys: [Pubkey; DEPOSIT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            reserve_x: pubkeys[3],
            reserve_y: pubkeys[4],
            user_token_account_x: pubkeys[5],
            user_token_account_y: pubkeys[6],
            mint_sslp_x: pubkeys[7],
            mint_sslp_y: pubkeys[8],
            user_sslp_x: pubkeys[9],
            user_sslp_y: pubkeys[10],
            x_price_feed: pubkeys[11],
            y_price_feed: pubkeys[12],
            user: pubkeys[13],
            token_program: pubkeys[14],
        }
    }
}
impl<'info> From<DepositAccounts<'_, 'info>>
for [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN] {
    fn from(accounts: DepositAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.reserve_x.clone(),
            accounts.reserve_y.clone(),
            accounts.user_token_account_x.clone(),
            accounts.user_token_account_y.clone(),
            accounts.mint_sslp_x.clone(),
            accounts.mint_sslp_y.clone(),
            accounts.user_sslp_x.clone(),
            accounts.user_sslp_y.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN]>
for DepositAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            reserve_x: &arr[3],
            reserve_y: &arr[4],
            user_token_account_x: &arr[5],
            user_token_account_y: &arr[6],
            mint_sslp_x: &arr[7],
            mint_sslp_y: &arr[8],
            user_sslp_x: &arr[9],
            user_sslp_y: &arr[10],
            x_price_feed: &arr[11],
            y_price_feed: &arr[12],
            user: &arr[13],
            token_program: &arr[14],
        }
    }
}
pub const DEPOSIT_IX_DISCM: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct DepositIxArgs {
    pub input_x: u64,
    pub input_y: u64,
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
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        DEPOSIT_IX_DISCM, maybe_discm
                    ),
                ),
            );
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
pub fn deposit_ix(
    keys: DepositKeys,
    args: DepositIxArgs,
) -> std::io::Result<Instruction> {
    deposit_ix_with_program_id(super::ID, keys, args)
}
pub fn deposit_invoke_with_program_id(
    program_id: Pubkey,
    accounts: DepositAccounts<'_, '_>,
    args: DepositIxArgs,
) -> ProgramResult {
    let keys: DepositKeys = accounts.into();
    let ix = deposit_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn deposit_invoke(
    accounts: DepositAccounts<'_, '_>,
    args: DepositIxArgs,
) -> ProgramResult {
    deposit_invoke_with_program_id(super::ID, accounts, args)
}
pub fn deposit_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: DepositAccounts<'_, '_>,
    args: DepositIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: DepositKeys = accounts.into();
    let ix = deposit_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn deposit_invoke_signed(
    accounts: DepositAccounts<'_, '_>,
    args: DepositIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    deposit_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn deposit_verify_account_keys(
    accounts: DepositAccounts<'_, '_>,
    keys: DepositKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.reserve_x.key, keys.reserve_x),
        (*accounts.reserve_y.key, keys.reserve_y),
        (*accounts.user_token_account_x.key, keys.user_token_account_x),
        (*accounts.user_token_account_y.key, keys.user_token_account_y),
        (*accounts.mint_sslp_x.key, keys.mint_sslp_x),
        (*accounts.mint_sslp_y.key, keys.mint_sslp_y),
        (*accounts.user_sslp_x.key, keys.user_sslp_x),
        (*accounts.user_sslp_y.key, keys.user_sslp_y),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
        (*accounts.token_program.key, keys.token_program),
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
        accounts.trading_pair,
        accounts.reserve_x,
        accounts.reserve_y,
        accounts.user_token_account_x,
        accounts.user_token_account_y,
        accounts.mint_sslp_x,
        accounts.mint_sslp_y,
        accounts.user_sslp_x,
        accounts.user_sslp_y,
        accounts.user,
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
    for should_be_signer in [accounts.user] {
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
pub const WITHDRAW_IX_ACCOUNTS_LEN: usize = 15;
#[derive(Copy, Clone, Debug)]
pub struct WithdrawAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub reserve_x: &'me AccountInfo<'info>,
    pub reserve_y: &'me AccountInfo<'info>,
    pub user_token_account_x: &'me AccountInfo<'info>,
    pub user_token_account_y: &'me AccountInfo<'info>,
    pub mint_sslp_x: &'me AccountInfo<'info>,
    pub mint_sslp_y: &'me AccountInfo<'info>,
    pub user_sslp_x: &'me AccountInfo<'info>,
    pub user_sslp_y: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WithdrawKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub user_token_account_x: Pubkey,
    pub user_token_account_y: Pubkey,
    pub mint_sslp_x: Pubkey,
    pub mint_sslp_y: Pubkey,
    pub user_sslp_x: Pubkey,
    pub user_sslp_y: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
    pub token_program: Pubkey,
}
impl From<WithdrawAccounts<'_, '_>> for WithdrawKeys {
    fn from(accounts: WithdrawAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            reserve_x: *accounts.reserve_x.key,
            reserve_y: *accounts.reserve_y.key,
            user_token_account_x: *accounts.user_token_account_x.key,
            user_token_account_y: *accounts.user_token_account_y.key,
            mint_sslp_x: *accounts.mint_sslp_x.key,
            mint_sslp_y: *accounts.mint_sslp_y.key,
            user_sslp_x: *accounts.user_sslp_x.key,
            user_sslp_y: *accounts.user_sslp_y.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<WithdrawKeys> for [AccountMeta; WITHDRAW_IX_ACCOUNTS_LEN] {
    fn from(keys: WithdrawKeys) -> Self {
        [
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
                pubkey: keys.mint_sslp_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint_sslp_y,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_sslp_x,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_sslp_y,
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
        ]
    }
}
impl From<[Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]> for WithdrawKeys {
    fn from(pubkeys: [Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            reserve_x: pubkeys[3],
            reserve_y: pubkeys[4],
            user_token_account_x: pubkeys[5],
            user_token_account_y: pubkeys[6],
            mint_sslp_x: pubkeys[7],
            mint_sslp_y: pubkeys[8],
            user_sslp_x: pubkeys[9],
            user_sslp_y: pubkeys[10],
            x_price_feed: pubkeys[11],
            y_price_feed: pubkeys[12],
            user: pubkeys[13],
            token_program: pubkeys[14],
        }
    }
}
impl<'info> From<WithdrawAccounts<'_, 'info>>
for [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN] {
    fn from(accounts: WithdrawAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.reserve_x.clone(),
            accounts.reserve_y.clone(),
            accounts.user_token_account_x.clone(),
            accounts.user_token_account_y.clone(),
            accounts.mint_sslp_x.clone(),
            accounts.mint_sslp_y.clone(),
            accounts.user_sslp_x.clone(),
            accounts.user_sslp_y.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]>
for WithdrawAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            reserve_x: &arr[3],
            reserve_y: &arr[4],
            user_token_account_x: &arr[5],
            user_token_account_y: &arr[6],
            mint_sslp_x: &arr[7],
            mint_sslp_y: &arr[8],
            user_sslp_x: &arr[9],
            user_sslp_y: &arr[10],
            x_price_feed: &arr[11],
            y_price_feed: &arr[12],
            user: &arr[13],
            token_program: &arr[14],
        }
    }
}
pub const WITHDRAW_IX_DISCM: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct WithdrawIxArgs {
    pub lp_amount_x: u64,
    pub lp_amount_y: u64,
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
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        WITHDRAW_IX_DISCM, maybe_discm
                    ),
                ),
            );
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
pub fn withdraw_ix(
    keys: WithdrawKeys,
    args: WithdrawIxArgs,
) -> std::io::Result<Instruction> {
    withdraw_ix_with_program_id(super::ID, keys, args)
}
pub fn withdraw_invoke_with_program_id(
    program_id: Pubkey,
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
) -> ProgramResult {
    let keys: WithdrawKeys = accounts.into();
    let ix = withdraw_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn withdraw_invoke(
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
) -> ProgramResult {
    withdraw_invoke_with_program_id(super::ID, accounts, args)
}
pub fn withdraw_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: WithdrawKeys = accounts.into();
    let ix = withdraw_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn withdraw_invoke_signed(
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    withdraw_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn withdraw_verify_account_keys(
    accounts: WithdrawAccounts<'_, '_>,
    keys: WithdrawKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.reserve_x.key, keys.reserve_x),
        (*accounts.reserve_y.key, keys.reserve_y),
        (*accounts.user_token_account_x.key, keys.user_token_account_x),
        (*accounts.user_token_account_y.key, keys.user_token_account_y),
        (*accounts.mint_sslp_x.key, keys.mint_sslp_x),
        (*accounts.mint_sslp_y.key, keys.mint_sslp_y),
        (*accounts.user_sslp_x.key, keys.user_sslp_x),
        (*accounts.user_sslp_y.key, keys.user_sslp_y),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn withdraw_verify_writable_privileges<'me, 'info>(
    accounts: WithdrawAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.trading_pair,
        accounts.reserve_x,
        accounts.reserve_y,
        accounts.user_token_account_x,
        accounts.user_token_account_y,
        accounts.mint_sslp_x,
        accounts.mint_sslp_y,
        accounts.user_sslp_x,
        accounts.user_sslp_y,
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
    for should_be_signer in [accounts.user] {
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
pub const SWAP_X_TO_Y_IX_ACCOUNTS_LEN: usize = 12;
#[derive(Copy, Clone, Debug)]
pub struct SwapXToYAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub reserve_x: &'me AccountInfo<'info>,
    pub reserve_y: &'me AccountInfo<'info>,
    pub user_token_account_x: &'me AccountInfo<'info>,
    pub user_token_account_y: &'me AccountInfo<'info>,
    pub protocol_fee_y: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapXToYKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub user_token_account_x: Pubkey,
    pub user_token_account_y: Pubkey,
    pub protocol_fee_y: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
    pub token_program: Pubkey,
}
impl From<SwapXToYAccounts<'_, '_>> for SwapXToYKeys {
    fn from(accounts: SwapXToYAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            reserve_x: *accounts.reserve_x.key,
            reserve_y: *accounts.reserve_y.key,
            user_token_account_x: *accounts.user_token_account_x.key,
            user_token_account_y: *accounts.user_token_account_y.key,
            protocol_fee_y: *accounts.protocol_fee_y.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<SwapXToYKeys> for [AccountMeta; SWAP_X_TO_Y_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapXToYKeys) -> Self {
        [
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
                pubkey: keys.protocol_fee_y,
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
        ]
    }
}
impl From<[Pubkey; SWAP_X_TO_Y_IX_ACCOUNTS_LEN]> for SwapXToYKeys {
    fn from(pubkeys: [Pubkey; SWAP_X_TO_Y_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            reserve_x: pubkeys[3],
            reserve_y: pubkeys[4],
            user_token_account_x: pubkeys[5],
            user_token_account_y: pubkeys[6],
            protocol_fee_y: pubkeys[7],
            x_price_feed: pubkeys[8],
            y_price_feed: pubkeys[9],
            user: pubkeys[10],
            token_program: pubkeys[11],
        }
    }
}
impl<'info> From<SwapXToYAccounts<'_, 'info>>
for [AccountInfo<'info>; SWAP_X_TO_Y_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapXToYAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.reserve_x.clone(),
            accounts.reserve_y.clone(),
            accounts.user_token_account_x.clone(),
            accounts.user_token_account_y.clone(),
            accounts.protocol_fee_y.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_X_TO_Y_IX_ACCOUNTS_LEN]>
for SwapXToYAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SWAP_X_TO_Y_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            reserve_x: &arr[3],
            reserve_y: &arr[4],
            user_token_account_x: &arr[5],
            user_token_account_y: &arr[6],
            protocol_fee_y: &arr[7],
            x_price_feed: &arr[8],
            y_price_feed: &arr[9],
            user: &arr[10],
            token_program: &arr[11],
        }
    }
}
pub const SWAP_X_TO_Y_IX_DISCM: [u8; 8] = [226, 74, 41, 166, 87, 155, 41, 75];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapXToYIxArgs {
    pub input_x: u64,
    pub min_output_amt: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapXToYIxData(pub SwapXToYIxArgs);
impl From<SwapXToYIxArgs> for SwapXToYIxData {
    fn from(args: SwapXToYIxArgs) -> Self {
        Self(args)
    }
}
impl SwapXToYIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SWAP_X_TO_Y_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        SWAP_X_TO_Y_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(SwapXToYIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SWAP_X_TO_Y_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn swap_x_to_y_ix_with_program_id(
    program_id: Pubkey,
    keys: SwapXToYKeys,
    args: SwapXToYIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SWAP_X_TO_Y_IX_ACCOUNTS_LEN] = keys.into();
    let data: SwapXToYIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn swap_x_to_y_ix(
    keys: SwapXToYKeys,
    args: SwapXToYIxArgs,
) -> std::io::Result<Instruction> {
    swap_x_to_y_ix_with_program_id(super::ID, keys, args)
}
pub fn swap_x_to_y_invoke_with_program_id(
    program_id: Pubkey,
    accounts: SwapXToYAccounts<'_, '_>,
    args: SwapXToYIxArgs,
) -> ProgramResult {
    let keys: SwapXToYKeys = accounts.into();
    let ix = swap_x_to_y_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn swap_x_to_y_invoke(
    accounts: SwapXToYAccounts<'_, '_>,
    args: SwapXToYIxArgs,
) -> ProgramResult {
    swap_x_to_y_invoke_with_program_id(super::ID, accounts, args)
}
pub fn swap_x_to_y_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: SwapXToYAccounts<'_, '_>,
    args: SwapXToYIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: SwapXToYKeys = accounts.into();
    let ix = swap_x_to_y_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn swap_x_to_y_invoke_signed(
    accounts: SwapXToYAccounts<'_, '_>,
    args: SwapXToYIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    swap_x_to_y_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn swap_x_to_y_verify_account_keys(
    accounts: SwapXToYAccounts<'_, '_>,
    keys: SwapXToYKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.reserve_x.key, keys.reserve_x),
        (*accounts.reserve_y.key, keys.reserve_y),
        (*accounts.user_token_account_x.key, keys.user_token_account_x),
        (*accounts.user_token_account_y.key, keys.user_token_account_y),
        (*accounts.protocol_fee_y.key, keys.protocol_fee_y),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn swap_x_to_y_verify_writable_privileges<'me, 'info>(
    accounts: SwapXToYAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.trading_pair,
        accounts.reserve_x,
        accounts.reserve_y,
        accounts.user_token_account_x,
        accounts.user_token_account_y,
        accounts.protocol_fee_y,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn swap_x_to_y_verify_signer_privileges<'me, 'info>(
    accounts: SwapXToYAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn swap_x_to_y_verify_account_privileges<'me, 'info>(
    accounts: SwapXToYAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    swap_x_to_y_verify_writable_privileges(accounts)?;
    swap_x_to_y_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SWAP_Y_TO_X_IX_ACCOUNTS_LEN: usize = 12;
#[derive(Copy, Clone, Debug)]
pub struct SwapYToXAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub reserve_x: &'me AccountInfo<'info>,
    pub reserve_y: &'me AccountInfo<'info>,
    pub user_token_account_x: &'me AccountInfo<'info>,
    pub user_token_account_y: &'me AccountInfo<'info>,
    pub protocol_fee_x: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapYToXKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub user_token_account_x: Pubkey,
    pub user_token_account_y: Pubkey,
    pub protocol_fee_x: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
    pub token_program: Pubkey,
}
impl From<SwapYToXAccounts<'_, '_>> for SwapYToXKeys {
    fn from(accounts: SwapYToXAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            reserve_x: *accounts.reserve_x.key,
            reserve_y: *accounts.reserve_y.key,
            user_token_account_x: *accounts.user_token_account_x.key,
            user_token_account_y: *accounts.user_token_account_y.key,
            protocol_fee_x: *accounts.protocol_fee_x.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<SwapYToXKeys> for [AccountMeta; SWAP_Y_TO_X_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapYToXKeys) -> Self {
        [
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
                pubkey: keys.protocol_fee_x,
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
        ]
    }
}
impl From<[Pubkey; SWAP_Y_TO_X_IX_ACCOUNTS_LEN]> for SwapYToXKeys {
    fn from(pubkeys: [Pubkey; SWAP_Y_TO_X_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            reserve_x: pubkeys[3],
            reserve_y: pubkeys[4],
            user_token_account_x: pubkeys[5],
            user_token_account_y: pubkeys[6],
            protocol_fee_x: pubkeys[7],
            x_price_feed: pubkeys[8],
            y_price_feed: pubkeys[9],
            user: pubkeys[10],
            token_program: pubkeys[11],
        }
    }
}
impl<'info> From<SwapYToXAccounts<'_, 'info>>
for [AccountInfo<'info>; SWAP_Y_TO_X_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapYToXAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.reserve_x.clone(),
            accounts.reserve_y.clone(),
            accounts.user_token_account_x.clone(),
            accounts.user_token_account_y.clone(),
            accounts.protocol_fee_x.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_Y_TO_X_IX_ACCOUNTS_LEN]>
for SwapYToXAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SWAP_Y_TO_X_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            reserve_x: &arr[3],
            reserve_y: &arr[4],
            user_token_account_x: &arr[5],
            user_token_account_y: &arr[6],
            protocol_fee_x: &arr[7],
            x_price_feed: &arr[8],
            y_price_feed: &arr[9],
            user: &arr[10],
            token_program: &arr[11],
        }
    }
}
pub const SWAP_Y_TO_X_IX_DISCM: [u8; 8] = [219, 168, 219, 174, 169, 221, 164, 95];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapYToXIxArgs {
    pub input_y: u64,
    pub min_output_amt: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapYToXIxData(pub SwapYToXIxArgs);
impl From<SwapYToXIxArgs> for SwapYToXIxData {
    fn from(args: SwapYToXIxArgs) -> Self {
        Self(args)
    }
}
impl SwapYToXIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SWAP_Y_TO_X_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        SWAP_Y_TO_X_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(SwapYToXIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SWAP_Y_TO_X_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn swap_y_to_x_ix_with_program_id(
    program_id: Pubkey,
    keys: SwapYToXKeys,
    args: SwapYToXIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SWAP_Y_TO_X_IX_ACCOUNTS_LEN] = keys.into();
    let data: SwapYToXIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn swap_y_to_x_ix(
    keys: SwapYToXKeys,
    args: SwapYToXIxArgs,
) -> std::io::Result<Instruction> {
    swap_y_to_x_ix_with_program_id(super::ID, keys, args)
}
pub fn swap_y_to_x_invoke_with_program_id(
    program_id: Pubkey,
    accounts: SwapYToXAccounts<'_, '_>,
    args: SwapYToXIxArgs,
) -> ProgramResult {
    let keys: SwapYToXKeys = accounts.into();
    let ix = swap_y_to_x_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn swap_y_to_x_invoke(
    accounts: SwapYToXAccounts<'_, '_>,
    args: SwapYToXIxArgs,
) -> ProgramResult {
    swap_y_to_x_invoke_with_program_id(super::ID, accounts, args)
}
pub fn swap_y_to_x_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: SwapYToXAccounts<'_, '_>,
    args: SwapYToXIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: SwapYToXKeys = accounts.into();
    let ix = swap_y_to_x_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn swap_y_to_x_invoke_signed(
    accounts: SwapYToXAccounts<'_, '_>,
    args: SwapYToXIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    swap_y_to_x_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn swap_y_to_x_verify_account_keys(
    accounts: SwapYToXAccounts<'_, '_>,
    keys: SwapYToXKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.reserve_x.key, keys.reserve_x),
        (*accounts.reserve_y.key, keys.reserve_y),
        (*accounts.user_token_account_x.key, keys.user_token_account_x),
        (*accounts.user_token_account_y.key, keys.user_token_account_y),
        (*accounts.protocol_fee_x.key, keys.protocol_fee_x),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn swap_y_to_x_verify_writable_privileges<'me, 'info>(
    accounts: SwapYToXAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.trading_pair,
        accounts.reserve_x,
        accounts.reserve_y,
        accounts.user_token_account_x,
        accounts.user_token_account_y,
        accounts.protocol_fee_x,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn swap_y_to_x_verify_signer_privileges<'me, 'info>(
    accounts: SwapYToXAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn swap_y_to_x_verify_account_privileges<'me, 'info>(
    accounts: SwapYToXAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    swap_y_to_x_verify_writable_privileges(accounts)?;
    swap_y_to_x_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SWAP_IX_ACCOUNTS_LEN: usize = 12;
#[derive(Copy, Clone, Debug)]
pub struct SwapAccounts<'me, 'info> {
    pub trading_pair: &'me AccountInfo<'info>,
    pub mint_x: &'me AccountInfo<'info>,
    pub mint_y: &'me AccountInfo<'info>,
    pub reserve_x: &'me AccountInfo<'info>,
    pub reserve_y: &'me AccountInfo<'info>,
    pub user_token_account_x: &'me AccountInfo<'info>,
    pub user_token_account_y: &'me AccountInfo<'info>,
    pub protocol_fee: &'me AccountInfo<'info>,
    pub x_price_feed: &'me AccountInfo<'info>,
    pub y_price_feed: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapKeys {
    pub trading_pair: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub user_token_account_x: Pubkey,
    pub user_token_account_y: Pubkey,
    pub protocol_fee: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub user: Pubkey,
    pub token_program: Pubkey,
}
impl From<SwapAccounts<'_, '_>> for SwapKeys {
    fn from(accounts: SwapAccounts) -> Self {
        Self {
            trading_pair: *accounts.trading_pair.key,
            mint_x: *accounts.mint_x.key,
            mint_y: *accounts.mint_y.key,
            reserve_x: *accounts.reserve_x.key,
            reserve_y: *accounts.reserve_y.key,
            user_token_account_x: *accounts.user_token_account_x.key,
            user_token_account_y: *accounts.user_token_account_y.key,
            protocol_fee: *accounts.protocol_fee.key,
            x_price_feed: *accounts.x_price_feed.key,
            y_price_feed: *accounts.y_price_feed.key,
            user: *accounts.user.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<SwapKeys> for [AccountMeta; SWAP_IX_ACCOUNTS_LEN] {
    fn from(keys: SwapKeys) -> Self {
        [
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
        ]
    }
}
impl From<[Pubkey; SWAP_IX_ACCOUNTS_LEN]> for SwapKeys {
    fn from(pubkeys: [Pubkey; SWAP_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: pubkeys[0],
            mint_x: pubkeys[1],
            mint_y: pubkeys[2],
            reserve_x: pubkeys[3],
            reserve_y: pubkeys[4],
            user_token_account_x: pubkeys[5],
            user_token_account_y: pubkeys[6],
            protocol_fee: pubkeys[7],
            x_price_feed: pubkeys[8],
            y_price_feed: pubkeys[9],
            user: pubkeys[10],
            token_program: pubkeys[11],
        }
    }
}
impl<'info> From<SwapAccounts<'_, 'info>>
for [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN] {
    fn from(accounts: SwapAccounts<'_, 'info>) -> Self {
        [
            accounts.trading_pair.clone(),
            accounts.mint_x.clone(),
            accounts.mint_y.clone(),
            accounts.reserve_x.clone(),
            accounts.reserve_y.clone(),
            accounts.user_token_account_x.clone(),
            accounts.user_token_account_y.clone(),
            accounts.protocol_fee.clone(),
            accounts.x_price_feed.clone(),
            accounts.y_price_feed.clone(),
            accounts.user.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]>
for SwapAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            trading_pair: &arr[0],
            mint_x: &arr[1],
            mint_y: &arr[2],
            reserve_x: &arr[3],
            reserve_y: &arr[4],
            user_token_account_x: &arr[5],
            user_token_account_y: &arr[6],
            protocol_fee: &arr[7],
            x_price_feed: &arr[8],
            y_price_feed: &arr[9],
            user: &arr[10],
            token_program: &arr[11],
        }
    }
}
pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapIxArgs {
    pub is_x_to_y: bool,
    pub input_amt: u64,
    pub min_output_amt: u64,
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
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        SWAP_IX_DISCM, maybe_discm
                    ),
                ),
            );
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
pub fn swap_ix_with_program_id(
    program_id: Pubkey,
    keys: SwapKeys,
    args: SwapIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SWAP_IX_ACCOUNTS_LEN] = keys.into();
    let data: SwapIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn swap_ix(keys: SwapKeys, args: SwapIxArgs) -> std::io::Result<Instruction> {
    swap_ix_with_program_id(super::ID, keys, args)
}
pub fn swap_invoke_with_program_id(
    program_id: Pubkey,
    accounts: SwapAccounts<'_, '_>,
    args: SwapIxArgs,
) -> ProgramResult {
    let keys: SwapKeys = accounts.into();
    let ix = swap_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn swap_invoke(accounts: SwapAccounts<'_, '_>, args: SwapIxArgs) -> ProgramResult {
    swap_invoke_with_program_id(super::ID, accounts, args)
}
pub fn swap_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: SwapAccounts<'_, '_>,
    args: SwapIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: SwapKeys = accounts.into();
    let ix = swap_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn swap_invoke_signed(
    accounts: SwapAccounts<'_, '_>,
    args: SwapIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    swap_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
}
pub fn swap_verify_account_keys(
    accounts: SwapAccounts<'_, '_>,
    keys: SwapKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.trading_pair.key, keys.trading_pair),
        (*accounts.mint_x.key, keys.mint_x),
        (*accounts.mint_y.key, keys.mint_y),
        (*accounts.reserve_x.key, keys.reserve_x),
        (*accounts.reserve_y.key, keys.reserve_y),
        (*accounts.user_token_account_x.key, keys.user_token_account_x),
        (*accounts.user_token_account_y.key, keys.user_token_account_y),
        (*accounts.protocol_fee.key, keys.protocol_fee),
        (*accounts.x_price_feed.key, keys.x_price_feed),
        (*accounts.y_price_feed.key, keys.y_price_feed),
        (*accounts.user.key, keys.user),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn swap_verify_writable_privileges<'me, 'info>(
    accounts: SwapAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.trading_pair,
        accounts.reserve_x,
        accounts.reserve_y,
        accounts.user_token_account_x,
        accounts.user_token_account_y,
        accounts.protocol_fee,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn swap_verify_signer_privileges<'me, 'info>(
    accounts: SwapAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.user] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn swap_verify_account_privileges<'me, 'info>(
    accounts: SwapAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    swap_verify_writable_privileges(accounts)?;
    swap_verify_signer_privileges(accounts)?;
    Ok(())
}
