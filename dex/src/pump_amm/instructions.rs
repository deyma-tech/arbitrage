use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};
use std::io::Read;
#[derive(Clone, Debug, PartialEq)]
pub enum PumpAmmProgramIx {
    Buy(BuyIxArgs),   // from quote to base
    Sell(SellIxArgs), // from base to quote
}
impl PumpAmmProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            BUY_IX_DISCM => Ok(Self::Buy(BuyIxArgs::deserialize(&mut reader)?)),
            SELL_IX_DISCM => Ok(Self::Sell(SellIxArgs::deserialize(&mut reader)?)),
            _ => Err(std::io::Error::other(format!("discm {maybe_discm:?} not found"))),
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::Buy(args) => {
                writer.write_all(&BUY_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Sell(args) => {
                writer.write_all(&SELL_IX_DISCM)?;
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
// fn invoke_instruction<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
//     ix: &Instruction,
//     accounts: A,
// ) -> ProgramResult {
//     let account_info: [AccountInfo<'info>; N] = accounts.into();
//     invoke(ix, &account_info)
// }
// fn invoke_instruction_signed<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
//     ix: &Instruction,
//     accounts: A,
//     seeds: &[&[&[u8]]],
// ) -> ProgramResult {
//     let account_info: [AccountInfo<'info>; N] = accounts.into();
//     invoke_signed(ix, &account_info, seeds)
// }
pub const BUY_IX_ACCOUNTS_LEN: usize = 17;
#[derive(Copy, Clone, Debug)]
pub struct BuyAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub global_config: &'me AccountInfo<'info>,
    pub base_mint: &'me AccountInfo<'info>,
    pub quote_mint: &'me AccountInfo<'info>,
    pub user_base_token_account: &'me AccountInfo<'info>,
    pub user_quote_token_account: &'me AccountInfo<'info>,
    pub pool_base_token_account: &'me AccountInfo<'info>,
    pub pool_quote_token_account: &'me AccountInfo<'info>,
    pub protocol_fee_recipient: &'me AccountInfo<'info>,
    pub protocol_fee_recipient_token_account: &'me AccountInfo<'info>,
    pub base_token_program: &'me AccountInfo<'info>,
    pub quote_token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub associated_token_program: &'me AccountInfo<'info>,
    pub event_authority: &'me AccountInfo<'info>,
    pub program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BuyKeys {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub global_config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub system_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
}
impl From<BuyAccounts<'_, '_>> for BuyKeys {
    fn from(accounts: BuyAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            user: *accounts.user.key,
            global_config: *accounts.global_config.key,
            base_mint: *accounts.base_mint.key,
            quote_mint: *accounts.quote_mint.key,
            user_base_token_account: *accounts.user_base_token_account.key,
            user_quote_token_account: *accounts.user_quote_token_account.key,
            pool_base_token_account: *accounts.pool_base_token_account.key,
            pool_quote_token_account: *accounts.pool_quote_token_account.key,
            protocol_fee_recipient: *accounts.protocol_fee_recipient.key,
            protocol_fee_recipient_token_account: *accounts.protocol_fee_recipient_token_account.key,
            base_token_program: *accounts.base_token_program.key,
            quote_token_program: *accounts.quote_token_program.key,
            system_program: *accounts.system_program.key,
            associated_token_program: *accounts.associated_token_program.key,
            event_authority: *accounts.event_authority.key,
            program: *accounts.program.key,
        }
    }
}
impl From<BuyKeys> for [AccountMeta; BUY_IX_ACCOUNTS_LEN] {
    fn from(keys: BuyKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.global_config,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.base_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.quote_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user_base_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_quote_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_base_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_quote_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_recipient,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_recipient_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.base_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.quote_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.associated_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.event_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; BUY_IX_ACCOUNTS_LEN]> for BuyKeys {
    fn from(pubkeys: [Pubkey; BUY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            user: pubkeys[1],
            global_config: pubkeys[2],
            base_mint: pubkeys[3],
            quote_mint: pubkeys[4],
            user_base_token_account: pubkeys[5],
            user_quote_token_account: pubkeys[6],
            pool_base_token_account: pubkeys[7],
            pool_quote_token_account: pubkeys[8],
            protocol_fee_recipient: pubkeys[9],
            protocol_fee_recipient_token_account: pubkeys[10],
            base_token_program: pubkeys[11],
            quote_token_program: pubkeys[12],
            system_program: pubkeys[13],
            associated_token_program: pubkeys[14],
            event_authority: pubkeys[15],
            program: pubkeys[16],
        }
    }
}
impl<'info> From<BuyAccounts<'_, 'info>> for [AccountInfo<'info>; BUY_IX_ACCOUNTS_LEN] {
    fn from(accounts: BuyAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.user.clone(),
            accounts.global_config.clone(),
            accounts.base_mint.clone(),
            accounts.quote_mint.clone(),
            accounts.user_base_token_account.clone(),
            accounts.user_quote_token_account.clone(),
            accounts.pool_base_token_account.clone(),
            accounts.pool_quote_token_account.clone(),
            accounts.protocol_fee_recipient.clone(),
            accounts.protocol_fee_recipient_token_account.clone(),
            accounts.base_token_program.clone(),
            accounts.quote_token_program.clone(),
            accounts.system_program.clone(),
            accounts.associated_token_program.clone(),
            accounts.event_authority.clone(),
            accounts.program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; BUY_IX_ACCOUNTS_LEN]> for BuyAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; BUY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            user: &arr[1],
            global_config: &arr[2],
            base_mint: &arr[3],
            quote_mint: &arr[4],
            user_base_token_account: &arr[5],
            user_quote_token_account: &arr[6],
            pool_base_token_account: &arr[7],
            pool_quote_token_account: &arr[8],
            protocol_fee_recipient: &arr[9],
            protocol_fee_recipient_token_account: &arr[10],
            base_token_program: &arr[11],
            quote_token_program: &arr[12],
            system_program: &arr[13],
            associated_token_program: &arr[14],
            event_authority: &arr[15],
            program: &arr[16],
        }
    }
}
pub const BUY_IX_DISCM: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct BuyIxArgs {
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct BuyIxData(pub BuyIxArgs);
impl From<BuyIxArgs> for BuyIxData {
    fn from(args: BuyIxArgs) -> Self {
        Self(args)
    }
}
impl BuyIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != BUY_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {BUY_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(BuyIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&BUY_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
// pub fn buy_ix_with_program_id(program_id: Pubkey, keys: BuyKeys, args: BuyIxArgs) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; BUY_IX_ACCOUNTS_LEN] = keys.into();
//     let data: BuyIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn buy_ix(keys: BuyKeys, args: BuyIxArgs) -> std::io::Result<Instruction> {
//     buy_ix_with_program_id(super::ID, keys, args)
// }
// pub fn buy_invoke_with_program_id(program_id: Pubkey, accounts: BuyAccounts<'_, '_>, args: BuyIxArgs) -> ProgramResult {
//     let keys: BuyKeys = accounts.into();
//     let ix = buy_ix_with_program_id(program_id, keys, args)?;
//     invoke_instruction(&ix, accounts)
// }
// pub fn buy_invoke(accounts: BuyAccounts<'_, '_>, args: BuyIxArgs) -> ProgramResult {
//     buy_invoke_with_program_id(super::ID, accounts, args)
// }
// pub fn buy_invoke_signed_with_program_id(
//     program_id: Pubkey,
//     accounts: BuyAccounts<'_, '_>,
//     args: BuyIxArgs,
//     seeds: &[&[&[u8]]],
// ) -> ProgramResult {
//     let keys: BuyKeys = accounts.into();
//     let ix = buy_ix_with_program_id(program_id, keys, args)?;
//     invoke_instruction_signed(&ix, accounts, seeds)
// }
// pub fn buy_invoke_signed(accounts: BuyAccounts<'_, '_>, args: BuyIxArgs, seeds: &[&[&[u8]]]) -> ProgramResult {
//     buy_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
// }
// pub fn buy_verify_account_keys(accounts: BuyAccounts<'_, '_>, keys: BuyKeys) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.pool.key, keys.pool),
//         (*accounts.user.key, keys.user),
//         (*accounts.global_config.key, keys.global_config),
//         (*accounts.base_mint.key, keys.base_mint),
//         (*accounts.quote_mint.key, keys.quote_mint),
//         (*accounts.user_base_token_account.key, keys.user_base_token_account),
//         (*accounts.user_quote_token_account.key, keys.user_quote_token_account),
//         (*accounts.pool_base_token_account.key, keys.pool_base_token_account),
//         (*accounts.pool_quote_token_account.key, keys.pool_quote_token_account),
//         (*accounts.protocol_fee_recipient.key, keys.protocol_fee_recipient),
//         (
//             *accounts.protocol_fee_recipient_token_account.key,
//             keys.protocol_fee_recipient_token_account,
//         ),
//         (*accounts.base_token_program.key, keys.base_token_program),
//         (*accounts.quote_token_program.key, keys.quote_token_program),
//         (*accounts.system_program.key, keys.system_program),
//         (*accounts.associated_token_program.key, keys.associated_token_program),
//         (*accounts.event_authority.key, keys.event_authority),
//         (*accounts.program.key, keys.program),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn buy_verify_writable_privileges<'me, 'info>(
//     accounts: BuyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [
//         accounts.pool,
//         accounts.user,
//         accounts.user_base_token_account,
//         accounts.user_quote_token_account,
//         accounts.pool_base_token_account,
//         accounts.pool_quote_token_account,
//         accounts.protocol_fee_recipient,
//         accounts.protocol_fee_recipient_token_account,
//     ] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn buy_verify_signer_privileges<'me, 'info>(
//     accounts: BuyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.user] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn buy_verify_account_privileges<'me, 'info>(
//     accounts: BuyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     buy_verify_writable_privileges(accounts)?;
//     buy_verify_signer_privileges(accounts)?;
//     Ok(())
// }
pub const SELL_IX_ACCOUNTS_LEN: usize = 17;
#[derive(Copy, Clone, Debug)]
pub struct SellAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub user: &'me AccountInfo<'info>,
    pub global_config: &'me AccountInfo<'info>,
    pub base_mint: &'me AccountInfo<'info>,
    pub quote_mint: &'me AccountInfo<'info>,
    pub user_base_token_account: &'me AccountInfo<'info>,
    pub user_quote_token_account: &'me AccountInfo<'info>,
    pub pool_base_token_account: &'me AccountInfo<'info>,
    pub pool_quote_token_account: &'me AccountInfo<'info>,
    pub protocol_fee_recipient: &'me AccountInfo<'info>,
    pub protocol_fee_recipient_token_account: &'me AccountInfo<'info>,
    pub base_token_program: &'me AccountInfo<'info>,
    pub quote_token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub associated_token_program: &'me AccountInfo<'info>,
    pub event_authority: &'me AccountInfo<'info>,
    pub program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SellKeys {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub global_config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub system_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
}
impl From<SellAccounts<'_, '_>> for SellKeys {
    fn from(accounts: SellAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            user: *accounts.user.key,
            global_config: *accounts.global_config.key,
            base_mint: *accounts.base_mint.key,
            quote_mint: *accounts.quote_mint.key,
            user_base_token_account: *accounts.user_base_token_account.key,
            user_quote_token_account: *accounts.user_quote_token_account.key,
            pool_base_token_account: *accounts.pool_base_token_account.key,
            pool_quote_token_account: *accounts.pool_quote_token_account.key,
            protocol_fee_recipient: *accounts.protocol_fee_recipient.key,
            protocol_fee_recipient_token_account: *accounts.protocol_fee_recipient_token_account.key,
            base_token_program: *accounts.base_token_program.key,
            quote_token_program: *accounts.quote_token_program.key,
            system_program: *accounts.system_program.key,
            associated_token_program: *accounts.associated_token_program.key,
            event_authority: *accounts.event_authority.key,
            program: *accounts.program.key,
        }
    }
}
impl From<SellKeys> for [AccountMeta; SELL_IX_ACCOUNTS_LEN] {
    fn from(keys: SellKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.global_config,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.base_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.quote_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user_base_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_quote_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_base_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_quote_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_recipient,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_recipient_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.base_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.quote_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.associated_token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.event_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; SELL_IX_ACCOUNTS_LEN]> for SellKeys {
    fn from(pubkeys: [Pubkey; SELL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            user: pubkeys[1],
            global_config: pubkeys[2],
            base_mint: pubkeys[3],
            quote_mint: pubkeys[4],
            user_base_token_account: pubkeys[5],
            user_quote_token_account: pubkeys[6],
            pool_base_token_account: pubkeys[7],
            pool_quote_token_account: pubkeys[8],
            protocol_fee_recipient: pubkeys[9],
            protocol_fee_recipient_token_account: pubkeys[10],
            base_token_program: pubkeys[11],
            quote_token_program: pubkeys[12],
            system_program: pubkeys[13],
            associated_token_program: pubkeys[14],
            event_authority: pubkeys[15],
            program: pubkeys[16],
        }
    }
}
impl<'info> From<SellAccounts<'_, 'info>> for [AccountInfo<'info>; SELL_IX_ACCOUNTS_LEN] {
    fn from(accounts: SellAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.user.clone(),
            accounts.global_config.clone(),
            accounts.base_mint.clone(),
            accounts.quote_mint.clone(),
            accounts.user_base_token_account.clone(),
            accounts.user_quote_token_account.clone(),
            accounts.pool_base_token_account.clone(),
            accounts.pool_quote_token_account.clone(),
            accounts.protocol_fee_recipient.clone(),
            accounts.protocol_fee_recipient_token_account.clone(),
            accounts.base_token_program.clone(),
            accounts.quote_token_program.clone(),
            accounts.system_program.clone(),
            accounts.associated_token_program.clone(),
            accounts.event_authority.clone(),
            accounts.program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SELL_IX_ACCOUNTS_LEN]> for SellAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SELL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            user: &arr[1],
            global_config: &arr[2],
            base_mint: &arr[3],
            quote_mint: &arr[4],
            user_base_token_account: &arr[5],
            user_quote_token_account: &arr[6],
            pool_base_token_account: &arr[7],
            pool_quote_token_account: &arr[8],
            protocol_fee_recipient: &arr[9],
            protocol_fee_recipient_token_account: &arr[10],
            base_token_program: &arr[11],
            quote_token_program: &arr[12],
            system_program: &arr[13],
            associated_token_program: &arr[14],
            event_authority: &arr[15],
            program: &arr[16],
        }
    }
}
pub const SELL_IX_DISCM: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SellIxArgs {
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SellIxData(pub SellIxArgs);
impl From<SellIxArgs> for SellIxData {
    fn from(args: SellIxArgs) -> Self {
        Self(args)
    }
}
impl SellIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SELL_IX_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {SELL_IX_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(SellIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SELL_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
// pub fn sell_ix_with_program_id(program_id: Pubkey, keys: SellKeys, args: SellIxArgs) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; SELL_IX_ACCOUNTS_LEN] = keys.into();
//     let data: SellIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn sell_ix(keys: SellKeys, args: SellIxArgs) -> std::io::Result<Instruction> {
//     sell_ix_with_program_id(super::ID, keys, args)
// }
// pub fn sell_invoke_with_program_id(
//     program_id: Pubkey,
//     accounts: SellAccounts<'_, '_>,
//     args: SellIxArgs,
// ) -> ProgramResult {
//     let keys: SellKeys = accounts.into();
//     let ix = sell_ix_with_program_id(program_id, keys, args)?;
//     invoke_instruction(&ix, accounts)
// }
// pub fn sell_invoke(accounts: SellAccounts<'_, '_>, args: SellIxArgs) -> ProgramResult {
//     sell_invoke_with_program_id(super::ID, accounts, args)
// }
// pub fn sell_invoke_signed_with_program_id(
//     program_id: Pubkey,
//     accounts: SellAccounts<'_, '_>,
//     args: SellIxArgs,
//     seeds: &[&[&[u8]]],
// ) -> ProgramResult {
//     let keys: SellKeys = accounts.into();
//     let ix = sell_ix_with_program_id(program_id, keys, args)?;
//     invoke_instruction_signed(&ix, accounts, seeds)
// }
// pub fn sell_invoke_signed(accounts: SellAccounts<'_, '_>, args: SellIxArgs, seeds: &[&[&[u8]]]) -> ProgramResult {
//     sell_invoke_signed_with_program_id(super::ID, accounts, args, seeds)
// }
// pub fn sell_verify_account_keys(accounts: SellAccounts<'_, '_>, keys: SellKeys) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.pool.key, keys.pool),
//         (*accounts.user.key, keys.user),
//         (*accounts.global_config.key, keys.global_config),
//         (*accounts.base_mint.key, keys.base_mint),
//         (*accounts.quote_mint.key, keys.quote_mint),
//         (*accounts.user_base_token_account.key, keys.user_base_token_account),
//         (*accounts.user_quote_token_account.key, keys.user_quote_token_account),
//         (*accounts.pool_base_token_account.key, keys.pool_base_token_account),
//         (*accounts.pool_quote_token_account.key, keys.pool_quote_token_account),
//         (*accounts.protocol_fee_recipient.key, keys.protocol_fee_recipient),
//         (
//             *accounts.protocol_fee_recipient_token_account.key,
//             keys.protocol_fee_recipient_token_account,
//         ),
//         (*accounts.base_token_program.key, keys.base_token_program),
//         (*accounts.quote_token_program.key, keys.quote_token_program),
//         (*accounts.system_program.key, keys.system_program),
//         (*accounts.associated_token_program.key, keys.associated_token_program),
//         (*accounts.event_authority.key, keys.event_authority),
//         (*accounts.program.key, keys.program),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn sell_verify_writable_privileges<'me, 'info>(
//     accounts: SellAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [
//         accounts.pool,
//         accounts.user,
//         accounts.user_base_token_account,
//         accounts.user_quote_token_account,
//         accounts.pool_base_token_account,
//         accounts.pool_quote_token_account,
//         accounts.protocol_fee_recipient,
//         accounts.protocol_fee_recipient_token_account,
//     ] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn sell_verify_signer_privileges<'me, 'info>(
//     accounts: SellAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.user] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn sell_verify_account_privileges<'me, 'info>(
//     accounts: SellAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     sell_verify_writable_privileges(accounts)?;
//     sell_verify_signer_privileges(accounts)?;
//     Ok(())
// }
