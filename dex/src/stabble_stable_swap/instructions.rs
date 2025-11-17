use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    // account_info::AccountInfo,
    // instruction::{AccountMeta, Instruction},
    // program_error::ProgramError,
    pubkey::Pubkey,
};
use std::io::Read;

#[derive(Clone, Debug, PartialEq)]
pub enum StableSwapProgramIx {
    // AcceptOwner,
    // ApproveStrategy,
    // ChangeAmpFactor(ChangeAmpFactorIxArgs),
    // ChangeMaxSupply(ChangeMaxSupplyIxArgs),
    // ChangeSwapFee(ChangeSwapFeeIxArgs),
    // CreateStrategy(CreateStrategyIxArgs),
    // Deposit(DepositIxArgs),
    // ExecStrategy(ExecStrategyIxArgs),
    // Initialize(InitializeIxArgs),
    // Pause,
    // RejectOwner,
    // Shutdown,
    // Swap(SwapIxArgs),
    SwapV2(SwapV2IxArgs),
    // TransferOwner(TransferOwnerIxArgs),
    // Unpause,
    // Withdraw(WithdrawIxArgs),
}

impl StableSwapProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            // ACCEPT_OWNER_IX_DISCM => Ok(Self::AcceptOwner),
            // APPROVE_STRATEGY_IX_DISCM => Ok(Self::ApproveStrategy),
            // CHANGE_AMP_FACTOR_IX_DISCM => Ok(Self::ChangeAmpFactor(ChangeAmpFactorIxArgs::deserialize(&mut reader)?)),
            // CHANGE_MAX_SUPPLY_IX_DISCM => Ok(Self::ChangeMaxSupply(ChangeMaxSupplyIxArgs::deserialize(&mut reader)?)),
            // CHANGE_SWAP_FEE_IX_DISCM => Ok(Self::ChangeSwapFee(ChangeSwapFeeIxArgs::deserialize(&mut reader)?)),
            // CREATE_STRATEGY_IX_DISCM => Ok(Self::CreateStrategy(CreateStrategyIxArgs::deserialize(&mut reader)?)),
            // DEPOSIT_IX_DISCM => Ok(Self::Deposit(DepositIxArgs::deserialize(&mut reader)?)),
            // EXEC_STRATEGY_IX_DISCM => Ok(Self::ExecStrategy(ExecStrategyIxArgs::deserialize(&mut reader)?)),
            // INITIALIZE_IX_DISCM => Ok(Self::Initialize(InitializeIxArgs::deserialize(&mut reader)?)),
            // PAUSE_IX_DISCM => Ok(Self::Pause),
            // REJECT_OWNER_IX_DISCM => Ok(Self::RejectOwner),
            // SHUTDOWN_IX_DISCM => Ok(Self::Shutdown),
            // SWAP_IX_DISCM => Ok(Self::Swap(SwapIxArgs::deserialize(&mut reader)?)),
            SWAP_V2_IX_DISCM => Ok(Self::SwapV2(SwapV2IxArgs::deserialize(&mut reader)?)),
            // TRANSFER_OWNER_IX_DISCM => Ok(Self::TransferOwner(TransferOwnerIxArgs::deserialize(&mut reader)?)),
            // UNPAUSE_IX_DISCM => Ok(Self::Unpause),
            // WITHDRAW_IX_DISCM => Ok(Self::Withdraw(WithdrawIxArgs::deserialize(&mut reader)?)),
            _ => Err(std::io::Error::other(format!("discm {maybe_discm:?} not found"))),
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            // Self::AcceptOwner => writer.write_all(&ACCEPT_OWNER_IX_DISCM),
            // Self::ApproveStrategy => writer.write_all(&APPROVE_STRATEGY_IX_DISCM),
            // Self::ChangeAmpFactor(args) => {
            //     writer.write_all(&CHANGE_AMP_FACTOR_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            // Self::ChangeMaxSupply(args) => {
            //     writer.write_all(&CHANGE_MAX_SUPPLY_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            // Self::ChangeSwapFee(args) => {
            //     writer.write_all(&CHANGE_SWAP_FEE_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            // Self::CreateStrategy(args) => {
            //     writer.write_all(&CREATE_STRATEGY_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            // Self::Deposit(args) => {
            //     writer.write_all(&DEPOSIT_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            // Self::ExecStrategy(args) => {
            //     writer.write_all(&EXEC_STRATEGY_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            // Self::Initialize(args) => {
            //     writer.write_all(&INITIALIZE_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            // Self::Pause => writer.write_all(&PAUSE_IX_DISCM),
            // Self::RejectOwner => writer.write_all(&REJECT_OWNER_IX_DISCM),
            // Self::Shutdown => writer.write_all(&SHUTDOWN_IX_DISCM),
            // Self::Swap(args) => {
            //     writer.write_all(&SWAP_IX_DISCM)?;
            //     args.serialize(&mut writer)
            // }
            Self::SwapV2(args) => {
                writer.write_all(&SWAP_V2_IX_DISCM)?;
                args.serialize(&mut writer)
            } // Self::TransferOwner(args) => {
              //     writer.write_all(&TRANSFER_OWNER_IX_DISCM)?;
              //     args.serialize(&mut writer)
              // }
              // Self::Unpause => writer.write_all(&UNPAUSE_IX_DISCM),
              // Self::Withdraw(args) => {
              //     writer.write_all(&WITHDRAW_IX_DISCM)?;
              //     args.serialize(&mut writer)
              // }
        }
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

// pub const ACCEPT_OWNER_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct AcceptOwnerAccounts<'me, 'info> {
//     pub pending_owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct AcceptOwnerKeys {
//     pub pending_owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<AcceptOwnerAccounts<'_, '_>> for AcceptOwnerKeys {
//     fn from(accounts: AcceptOwnerAccounts) -> Self {
//         Self {
//             pending_owner: *accounts.pending_owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<AcceptOwnerKeys> for [AccountMeta; ACCEPT_OWNER_IX_ACCOUNTS_LEN] {
//     fn from(keys: AcceptOwnerKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.pending_owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; ACCEPT_OWNER_IX_ACCOUNTS_LEN]> for AcceptOwnerKeys {
//     fn from(pubkeys: [Pubkey; ACCEPT_OWNER_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pending_owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<AcceptOwnerAccounts<'_, 'info>> for [AccountInfo<'info>; ACCEPT_OWNER_IX_ACCOUNTS_LEN] {
//     fn from(accounts: AcceptOwnerAccounts<'_, 'info>) -> Self {
//         [accounts.pending_owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; ACCEPT_OWNER_IX_ACCOUNTS_LEN]> for AcceptOwnerAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; ACCEPT_OWNER_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pending_owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const ACCEPT_OWNER_IX_DISCM: [u8; 8] = [176, 23, 41, 28, 23, 111, 8, 4];
// #[derive(Clone, Debug, PartialEq)]
// pub struct AcceptOwnerIxData;
// impl AcceptOwnerIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != ACCEPT_OWNER_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {ACCEPT_OWNER_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self)
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&ACCEPT_OWNER_IX_DISCM)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn accept_owner_ix_with_program_id(program_id: Pubkey, keys: AcceptOwnerKeys) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; ACCEPT_OWNER_IX_ACCOUNTS_LEN] = keys.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: AcceptOwnerIxData.try_to_vec()?,
//     })
// }
// pub fn accept_owner_ix(keys: AcceptOwnerKeys) -> std::io::Result<Instruction> {
//     accept_owner_ix_with_program_id(super::ID, keys)
// }
// pub fn accept_owner_verify_account_keys(
//     accounts: AcceptOwnerAccounts<'_, '_>,
//     keys: AcceptOwnerKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.pending_owner.key, keys.pending_owner),
//         (*accounts.pool.key, keys.pool),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn accept_owner_verify_writable_privileges<'me, 'info>(
//     accounts: AcceptOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn accept_owner_verify_signer_privileges<'me, 'info>(
//     accounts: AcceptOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.pending_owner] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn accept_owner_verify_account_privileges<'me, 'info>(
//     accounts: AcceptOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     accept_owner_verify_writable_privileges(accounts)?;
//     accept_owner_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const APPROVE_STRATEGY_IX_ACCOUNTS_LEN: usize = 4;
// #[derive(Copy, Clone, Debug)]
// pub struct ApproveStrategyAccounts<'me, 'info> {
//     pub pool: &'me AccountInfo<'info>,
//     pub vault: &'me AccountInfo<'info>,
//     pub admin: &'me AccountInfo<'info>,
//     pub strategy: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct ApproveStrategyKeys {
//     pub pool: Pubkey,
//     pub vault: Pubkey,
//     pub admin: Pubkey,
//     pub strategy: Pubkey,
// }
// impl From<ApproveStrategyAccounts<'_, '_>> for ApproveStrategyKeys {
//     fn from(accounts: ApproveStrategyAccounts) -> Self {
//         Self {
//             pool: *accounts.pool.key,
//             vault: *accounts.vault.key,
//             admin: *accounts.admin.key,
//             strategy: *accounts.strategy.key,
//         }
//     }
// }
// impl From<ApproveStrategyKeys> for [AccountMeta; APPROVE_STRATEGY_IX_ACCOUNTS_LEN] {
//     fn from(keys: ApproveStrategyKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.vault,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.admin,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.strategy,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; APPROVE_STRATEGY_IX_ACCOUNTS_LEN]> for ApproveStrategyKeys {
//     fn from(pubkeys: [Pubkey; APPROVE_STRATEGY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pool: pubkeys[0],
//             vault: pubkeys[1],
//             admin: pubkeys[2],
//             strategy: pubkeys[3],
//         }
//     }
// }
// impl<'info> From<ApproveStrategyAccounts<'_, 'info>> for [AccountInfo<'info>; APPROVE_STRATEGY_IX_ACCOUNTS_LEN] {
//     fn from(accounts: ApproveStrategyAccounts<'_, 'info>) -> Self {
//         [
//             accounts.pool.clone(),
//             accounts.vault.clone(),
//             accounts.admin.clone(),
//             accounts.strategy.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; APPROVE_STRATEGY_IX_ACCOUNTS_LEN]>
//     for ApproveStrategyAccounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; APPROVE_STRATEGY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pool: &arr[0],
//             vault: &arr[1],
//             admin: &arr[2],
//             strategy: &arr[3],
//         }
//     }
// }
// pub const APPROVE_STRATEGY_IX_DISCM: [u8; 8] = [7, 141, 162, 60, 71, 115, 26, 146];
// #[derive(Clone, Debug, PartialEq)]
// pub struct ApproveStrategyIxData;
// impl ApproveStrategyIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != APPROVE_STRATEGY_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {APPROVE_STRATEGY_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self)
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&APPROVE_STRATEGY_IX_DISCM)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn approve_strategy_ix_with_program_id(
//     program_id: Pubkey,
//     keys: ApproveStrategyKeys,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; APPROVE_STRATEGY_IX_ACCOUNTS_LEN] = keys.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: ApproveStrategyIxData.try_to_vec()?,
//     })
// }
// pub fn approve_strategy_ix(keys: ApproveStrategyKeys) -> std::io::Result<Instruction> {
//     approve_strategy_ix_with_program_id(super::ID, keys)
// }
// pub fn approve_strategy_verify_account_keys(
//     accounts: ApproveStrategyAccounts<'_, '_>,
//     keys: ApproveStrategyKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.pool.key, keys.pool),
//         (*accounts.vault.key, keys.vault),
//         (*accounts.admin.key, keys.admin),
//         (*accounts.strategy.key, keys.strategy),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn approve_strategy_verify_writable_privileges<'me, 'info>(
//     accounts: ApproveStrategyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool, accounts.strategy] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn approve_strategy_verify_signer_privileges<'me, 'info>(
//     accounts: ApproveStrategyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.admin] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn approve_strategy_verify_account_privileges<'me, 'info>(
//     accounts: ApproveStrategyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     approve_strategy_verify_writable_privileges(accounts)?;
//     approve_strategy_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN: usize = 3;
// #[derive(Copy, Clone, Debug)]
// pub struct ChangeAmpFactorAccounts<'me, 'info> {
//     pub pool: &'me AccountInfo<'info>,
//     pub vault: &'me AccountInfo<'info>,
//     pub admin: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct ChangeAmpFactorKeys {
//     pub pool: Pubkey,
//     pub vault: Pubkey,
//     pub admin: Pubkey,
// }
// impl From<ChangeAmpFactorAccounts<'_, '_>> for ChangeAmpFactorKeys {
//     fn from(accounts: ChangeAmpFactorAccounts) -> Self {
//         Self {
//             pool: *accounts.pool.key,
//             vault: *accounts.vault.key,
//             admin: *accounts.admin.key,
//         }
//     }
// }
// impl From<ChangeAmpFactorKeys> for [AccountMeta; CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN] {
//     fn from(keys: ChangeAmpFactorKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.vault,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.admin,
//                 is_signer: true,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN]> for ChangeAmpFactorKeys {
//     fn from(pubkeys: [Pubkey; CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pool: pubkeys[0],
//             vault: pubkeys[1],
//             admin: pubkeys[2],
//         }
//     }
// }
// impl<'info> From<ChangeAmpFactorAccounts<'_, 'info>> for [AccountInfo<'info>; CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN] {
//     fn from(accounts: ChangeAmpFactorAccounts<'_, 'info>) -> Self {
//         [accounts.pool.clone(), accounts.vault.clone(), accounts.admin.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN]>
//     for ChangeAmpFactorAccounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pool: &arr[0],
//             vault: &arr[1],
//             admin: &arr[2],
//         }
//     }
// }
// pub const CHANGE_AMP_FACTOR_IX_DISCM: [u8; 8] = [56, 238, 189, 35, 200, 157, 42, 66];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct ChangeAmpFactorIxArgs {
//     pub new_amp_factor: u16,
//     pub ramp_duration: u32,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct ChangeAmpFactorIxData(pub ChangeAmpFactorIxArgs);
// impl From<ChangeAmpFactorIxArgs> for ChangeAmpFactorIxData {
//     fn from(args: ChangeAmpFactorIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl ChangeAmpFactorIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != CHANGE_AMP_FACTOR_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {CHANGE_AMP_FACTOR_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(ChangeAmpFactorIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&CHANGE_AMP_FACTOR_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn change_amp_factor_ix_with_program_id(
//     program_id: Pubkey,
//     keys: ChangeAmpFactorKeys,
//     args: ChangeAmpFactorIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; CHANGE_AMP_FACTOR_IX_ACCOUNTS_LEN] = keys.into();
//     let data: ChangeAmpFactorIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn change_amp_factor_ix(keys: ChangeAmpFactorKeys, args: ChangeAmpFactorIxArgs) -> std::io::Result<Instruction> {
//     change_amp_factor_ix_with_program_id(super::ID, keys, args)
// }
// pub fn change_amp_factor_verify_account_keys(
//     accounts: ChangeAmpFactorAccounts<'_, '_>,
//     keys: ChangeAmpFactorKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.pool.key, keys.pool),
//         (*accounts.vault.key, keys.vault),
//         (*accounts.admin.key, keys.admin),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn change_amp_factor_verify_writable_privileges<'me, 'info>(
//     accounts: ChangeAmpFactorAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn change_amp_factor_verify_signer_privileges<'me, 'info>(
//     accounts: ChangeAmpFactorAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.admin] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn change_amp_factor_verify_account_privileges<'me, 'info>(
//     accounts: ChangeAmpFactorAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     change_amp_factor_verify_writable_privileges(accounts)?;
//     change_amp_factor_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct ChangeMaxSupplyAccounts<'me, 'info> {
//     pub owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct ChangeMaxSupplyKeys {
//     pub owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<ChangeMaxSupplyAccounts<'_, '_>> for ChangeMaxSupplyKeys {
//     fn from(accounts: ChangeMaxSupplyAccounts) -> Self {
//         Self {
//             owner: *accounts.owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<ChangeMaxSupplyKeys> for [AccountMeta; CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN] {
//     fn from(keys: ChangeMaxSupplyKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN]> for ChangeMaxSupplyKeys {
//     fn from(pubkeys: [Pubkey; CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<ChangeMaxSupplyAccounts<'_, 'info>> for [AccountInfo<'info>; CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN] {
//     fn from(accounts: ChangeMaxSupplyAccounts<'_, 'info>) -> Self {
//         [accounts.owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN]>
//     for ChangeMaxSupplyAccounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const CHANGE_MAX_SUPPLY_IX_DISCM: [u8; 8] = [93, 176, 0, 205, 69, 63, 87, 80];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct ChangeMaxSupplyIxArgs {
//     pub new_max_supply: u64,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct ChangeMaxSupplyIxData(pub ChangeMaxSupplyIxArgs);
// impl From<ChangeMaxSupplyIxArgs> for ChangeMaxSupplyIxData {
//     fn from(args: ChangeMaxSupplyIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl ChangeMaxSupplyIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != CHANGE_MAX_SUPPLY_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {CHANGE_MAX_SUPPLY_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(ChangeMaxSupplyIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&CHANGE_MAX_SUPPLY_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn change_max_supply_ix_with_program_id(
//     program_id: Pubkey,
//     keys: ChangeMaxSupplyKeys,
//     args: ChangeMaxSupplyIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; CHANGE_MAX_SUPPLY_IX_ACCOUNTS_LEN] = keys.into();
//     let data: ChangeMaxSupplyIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn change_max_supply_ix(keys: ChangeMaxSupplyKeys, args: ChangeMaxSupplyIxArgs) -> std::io::Result<Instruction> {
//     change_max_supply_ix_with_program_id(super::ID, keys, args)
// }
// pub fn change_max_supply_verify_account_keys(
//     accounts: ChangeMaxSupplyAccounts<'_, '_>,
//     keys: ChangeMaxSupplyKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [(*accounts.owner.key, keys.owner), (*accounts.pool.key, keys.pool)] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn change_max_supply_verify_writable_privileges<'me, 'info>(
//     accounts: ChangeMaxSupplyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn change_max_supply_verify_signer_privileges<'me, 'info>(
//     accounts: ChangeMaxSupplyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.owner] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn change_max_supply_verify_account_privileges<'me, 'info>(
//     accounts: ChangeMaxSupplyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     change_max_supply_verify_writable_privileges(accounts)?;
//     change_max_supply_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct ChangeSwapFeeAccounts<'me, 'info> {
//     pub owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct ChangeSwapFeeKeys {
//     pub owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<ChangeSwapFeeAccounts<'_, '_>> for ChangeSwapFeeKeys {
//     fn from(accounts: ChangeSwapFeeAccounts) -> Self {
//         Self {
//             owner: *accounts.owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<ChangeSwapFeeKeys> for [AccountMeta; CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN] {
//     fn from(keys: ChangeSwapFeeKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN]> for ChangeSwapFeeKeys {
//     fn from(pubkeys: [Pubkey; CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<ChangeSwapFeeAccounts<'_, 'info>> for [AccountInfo<'info>; CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN] {
//     fn from(accounts: ChangeSwapFeeAccounts<'_, 'info>) -> Self {
//         [accounts.owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN]>
//     for ChangeSwapFeeAccounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const CHANGE_SWAP_FEE_IX_DISCM: [u8; 8] = [231, 15, 132, 51, 132, 165, 64, 170];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct ChangeSwapFeeIxArgs {
//     pub new_swap_fee: u64,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct ChangeSwapFeeIxData(pub ChangeSwapFeeIxArgs);
// impl From<ChangeSwapFeeIxArgs> for ChangeSwapFeeIxData {
//     fn from(args: ChangeSwapFeeIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl ChangeSwapFeeIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != CHANGE_SWAP_FEE_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {CHANGE_SWAP_FEE_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(ChangeSwapFeeIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&CHANGE_SWAP_FEE_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn change_swap_fee_ix_with_program_id(
//     program_id: Pubkey,
//     keys: ChangeSwapFeeKeys,
//     args: ChangeSwapFeeIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; CHANGE_SWAP_FEE_IX_ACCOUNTS_LEN] = keys.into();
//     let data: ChangeSwapFeeIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn change_swap_fee_ix(keys: ChangeSwapFeeKeys, args: ChangeSwapFeeIxArgs) -> std::io::Result<Instruction> {
//     change_swap_fee_ix_with_program_id(super::ID, keys, args)
// }

// pub fn change_swap_fee_verify_account_keys(
//     accounts: ChangeSwapFeeAccounts<'_, '_>,
//     keys: ChangeSwapFeeKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [(*accounts.owner.key, keys.owner), (*accounts.pool.key, keys.pool)] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn change_swap_fee_verify_writable_privileges<'me, 'info>(
//     accounts: ChangeSwapFeeAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn change_swap_fee_verify_signer_privileges<'me, 'info>(
//     accounts: ChangeSwapFeeAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.owner] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn change_swap_fee_verify_account_privileges<'me, 'info>(
//     accounts: ChangeSwapFeeAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     change_swap_fee_verify_writable_privileges(accounts)?;
//     change_swap_fee_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const CREATE_STRATEGY_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct CreateStrategyAccounts<'me, 'info> {
//     pub owner_only: &'me AccountInfo<'info>,
//     pub strategy: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct CreateStrategyKeys {
//     pub owner_only: Pubkey,
//     pub strategy: Pubkey,
// }
// impl From<CreateStrategyAccounts<'_, '_>> for CreateStrategyKeys {
//     fn from(accounts: CreateStrategyAccounts) -> Self {
//         Self {
//             owner_only: *accounts.owner_only.key,
//             strategy: *accounts.strategy.key,
//         }
//     }
// }
// impl From<CreateStrategyKeys> for [AccountMeta; CREATE_STRATEGY_IX_ACCOUNTS_LEN] {
//     fn from(keys: CreateStrategyKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner_only,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.strategy,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; CREATE_STRATEGY_IX_ACCOUNTS_LEN]> for CreateStrategyKeys {
//     fn from(pubkeys: [Pubkey; CREATE_STRATEGY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner_only: pubkeys[0],
//             strategy: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<CreateStrategyAccounts<'_, 'info>> for [AccountInfo<'info>; CREATE_STRATEGY_IX_ACCOUNTS_LEN] {
//     fn from(accounts: CreateStrategyAccounts<'_, 'info>) -> Self {
//         [accounts.owner_only.clone(), accounts.strategy.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_STRATEGY_IX_ACCOUNTS_LEN]>
//     for CreateStrategyAccounts<'me, 'info>
// {
//     fn from(arr: &'me [AccountInfo<'info>; CREATE_STRATEGY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner_only: &arr[0],
//             strategy: &arr[1],
//         }
//     }
// }
// pub const CREATE_STRATEGY_IX_DISCM: [u8; 8] = [152, 160, 107, 148, 245, 190, 127, 224];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct CreateStrategyIxArgs {
//     pub amp_min_factor: u16,
//     pub amp_max_factor: u16,
//     pub ramp_min_step: u16,
//     pub ramp_max_step: u16,
//     pub ramp_min_duration: u32,
//     pub ramp_max_duration: u32,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct CreateStrategyIxData(pub CreateStrategyIxArgs);
// impl From<CreateStrategyIxArgs> for CreateStrategyIxData {
//     fn from(args: CreateStrategyIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl CreateStrategyIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != CREATE_STRATEGY_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {CREATE_STRATEGY_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(CreateStrategyIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&CREATE_STRATEGY_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn create_strategy_ix_with_program_id(
//     program_id: Pubkey,
//     keys: CreateStrategyKeys,
//     args: CreateStrategyIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; CREATE_STRATEGY_IX_ACCOUNTS_LEN] = keys.into();
//     let data: CreateStrategyIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn create_strategy_ix(keys: CreateStrategyKeys, args: CreateStrategyIxArgs) -> std::io::Result<Instruction> {
//     create_strategy_ix_with_program_id(super::ID, keys, args)
// }

// pub fn create_strategy_verify_account_keys(
//     accounts: CreateStrategyAccounts<'_, '_>,
//     keys: CreateStrategyKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.owner_only.key, keys.owner_only),
//         (*accounts.strategy.key, keys.strategy),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn create_strategy_verify_writable_privileges<'me, 'info>(
//     accounts: CreateStrategyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.strategy] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn create_strategy_verify_account_privileges<'me, 'info>(
//     accounts: CreateStrategyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     create_strategy_verify_writable_privileges(accounts)?;
//     Ok(())
// }
// pub const DEPOSIT_IX_ACCOUNTS_LEN: usize = 9;
// #[derive(Copy, Clone, Debug)]
// pub struct DepositAccounts<'me, 'info> {
//     pub user: &'me AccountInfo<'info>,
//     pub user_pool_token: &'me AccountInfo<'info>,
//     pub mint: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
//     pub pool_authority: &'me AccountInfo<'info>,
//     pub vault: &'me AccountInfo<'info>,
//     pub vault_authority: &'me AccountInfo<'info>,
//     pub token_program: &'me AccountInfo<'info>,
//     pub token_program_2022: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct DepositKeys {
//     pub user: Pubkey,
//     pub user_pool_token: Pubkey,
//     pub mint: Pubkey,
//     pub pool: Pubkey,
//     pub pool_authority: Pubkey,
//     pub vault: Pubkey,
//     pub vault_authority: Pubkey,
//     pub token_program: Pubkey,
//     pub token_program_2022: Pubkey,
// }
// impl From<DepositAccounts<'_, '_>> for DepositKeys {
//     fn from(accounts: DepositAccounts) -> Self {
//         Self {
//             user: *accounts.user.key,
//             user_pool_token: *accounts.user_pool_token.key,
//             mint: *accounts.mint.key,
//             pool: *accounts.pool.key,
//             pool_authority: *accounts.pool_authority.key,
//             vault: *accounts.vault.key,
//             vault_authority: *accounts.vault_authority.key,
//             token_program: *accounts.token_program.key,
//             token_program_2022: *accounts.token_program_2022.key,
//         }
//     }
// }
// impl From<DepositKeys> for [AccountMeta; DEPOSIT_IX_ACCOUNTS_LEN] {
//     fn from(keys: DepositKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.user,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.user_pool_token,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.mint,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program_2022,
//                 is_signer: false,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; DEPOSIT_IX_ACCOUNTS_LEN]> for DepositKeys {
//     fn from(pubkeys: [Pubkey; DEPOSIT_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: pubkeys[0],
//             user_pool_token: pubkeys[1],
//             mint: pubkeys[2],
//             pool: pubkeys[3],
//             pool_authority: pubkeys[4],
//             vault: pubkeys[5],
//             vault_authority: pubkeys[6],
//             token_program: pubkeys[7],
//             token_program_2022: pubkeys[8],
//         }
//     }
// }
// impl<'info> From<DepositAccounts<'_, 'info>> for [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN] {
//     fn from(accounts: DepositAccounts<'_, 'info>) -> Self {
//         [
//             accounts.user.clone(),
//             accounts.user_pool_token.clone(),
//             accounts.mint.clone(),
//             accounts.pool.clone(),
//             accounts.pool_authority.clone(),
//             accounts.vault.clone(),
//             accounts.vault_authority.clone(),
//             accounts.token_program.clone(),
//             accounts.token_program_2022.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN]> for DepositAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; DEPOSIT_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: &arr[0],
//             user_pool_token: &arr[1],
//             mint: &arr[2],
//             pool: &arr[3],
//             pool_authority: &arr[4],
//             vault: &arr[5],
//             vault_authority: &arr[6],
//             token_program: &arr[7],
//             token_program_2022: &arr[8],
//         }
//     }
// }
// pub const DEPOSIT_IX_DISCM: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct DepositIxArgs {
//     pub amounts: Vec<u64>,
//     pub minimum_amount_out: u64,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct DepositIxData(pub DepositIxArgs);
// impl From<DepositIxArgs> for DepositIxData {
//     fn from(args: DepositIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl DepositIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != DEPOSIT_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {DEPOSIT_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(DepositIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&DEPOSIT_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn deposit_ix_with_program_id(
//     program_id: Pubkey,
//     keys: DepositKeys,
//     args: DepositIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; DEPOSIT_IX_ACCOUNTS_LEN] = keys.into();
//     let data: DepositIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn deposit_ix(keys: DepositKeys, args: DepositIxArgs) -> std::io::Result<Instruction> {
//     deposit_ix_with_program_id(super::ID, keys, args)
// }
// pub fn deposit_verify_account_keys(
//     accounts: DepositAccounts<'_, '_>,
//     keys: DepositKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.user.key, keys.user),
//         (*accounts.user_pool_token.key, keys.user_pool_token),
//         (*accounts.mint.key, keys.mint),
//         (*accounts.pool.key, keys.pool),
//         (*accounts.pool_authority.key, keys.pool_authority),
//         (*accounts.vault.key, keys.vault),
//         (*accounts.vault_authority.key, keys.vault_authority),
//         (*accounts.token_program.key, keys.token_program),
//         (*accounts.token_program_2022.key, keys.token_program_2022),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn deposit_verify_writable_privileges<'me, 'info>(
//     accounts: DepositAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.user_pool_token, accounts.mint, accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn deposit_verify_signer_privileges<'me, 'info>(
//     accounts: DepositAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.user] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn deposit_verify_account_privileges<'me, 'info>(
//     accounts: DepositAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     deposit_verify_writable_privileges(accounts)?;
//     deposit_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const EXEC_STRATEGY_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct ExecStrategyAccounts<'me, 'info> {
//     pub strategy: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct ExecStrategyKeys {
//     pub strategy: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<ExecStrategyAccounts<'_, '_>> for ExecStrategyKeys {
//     fn from(accounts: ExecStrategyAccounts) -> Self {
//         Self {
//             strategy: *accounts.strategy.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<ExecStrategyKeys> for [AccountMeta; EXEC_STRATEGY_IX_ACCOUNTS_LEN] {
//     fn from(keys: ExecStrategyKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.strategy,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; EXEC_STRATEGY_IX_ACCOUNTS_LEN]> for ExecStrategyKeys {
//     fn from(pubkeys: [Pubkey; EXEC_STRATEGY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             strategy: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<ExecStrategyAccounts<'_, 'info>> for [AccountInfo<'info>; EXEC_STRATEGY_IX_ACCOUNTS_LEN] {
//     fn from(accounts: ExecStrategyAccounts<'_, 'info>) -> Self {
//         [accounts.strategy.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; EXEC_STRATEGY_IX_ACCOUNTS_LEN]> for ExecStrategyAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; EXEC_STRATEGY_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             strategy: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const EXEC_STRATEGY_IX_DISCM: [u8; 8] = [249, 46, 55, 57, 31, 38, 61, 27];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct ExecStrategyIxArgs {
//     pub ramp_step: u16,
//     pub ramp_duration: u32,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct ExecStrategyIxData(pub ExecStrategyIxArgs);
// impl From<ExecStrategyIxArgs> for ExecStrategyIxData {
//     fn from(args: ExecStrategyIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl ExecStrategyIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != EXEC_STRATEGY_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {EXEC_STRATEGY_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(ExecStrategyIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&EXEC_STRATEGY_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn exec_strategy_verify_account_keys(
//     accounts: ExecStrategyAccounts<'_, '_>,
//     keys: ExecStrategyKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [(*accounts.strategy.key, keys.strategy), (*accounts.pool.key, keys.pool)] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn exec_strategy_verify_writable_privileges<'me, 'info>(
//     accounts: ExecStrategyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn exec_strategy_verify_account_privileges<'me, 'info>(
//     accounts: ExecStrategyAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     exec_strategy_verify_writable_privileges(accounts)?;
//     Ok(())
// }
// pub const INITIALIZE_IX_ACCOUNTS_LEN: usize = 6;
// #[derive(Copy, Clone, Debug)]
// pub struct InitializeAccounts<'me, 'info> {
//     pub owner: &'me AccountInfo<'info>,
//     pub mint: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
//     pub pool_authority: &'me AccountInfo<'info>,
//     pub withdraw_authority: &'me AccountInfo<'info>,
//     pub vault: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct InitializeKeys {
//     pub owner: Pubkey,
//     pub mint: Pubkey,
//     pub pool: Pubkey,
//     pub pool_authority: Pubkey,
//     pub withdraw_authority: Pubkey,
//     pub vault: Pubkey,
// }
// impl From<InitializeAccounts<'_, '_>> for InitializeKeys {
//     fn from(accounts: InitializeAccounts) -> Self {
//         Self {
//             owner: *accounts.owner.key,
//             mint: *accounts.mint.key,
//             pool: *accounts.pool.key,
//             pool_authority: *accounts.pool_authority.key,
//             withdraw_authority: *accounts.withdraw_authority.key,
//             vault: *accounts.vault.key,
//         }
//     }
// }
// impl From<InitializeKeys> for [AccountMeta; INITIALIZE_IX_ACCOUNTS_LEN] {
//     fn from(keys: InitializeKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.mint,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.withdraw_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault,
//                 is_signer: false,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; INITIALIZE_IX_ACCOUNTS_LEN]> for InitializeKeys {
//     fn from(pubkeys: [Pubkey; INITIALIZE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: pubkeys[0],
//             mint: pubkeys[1],
//             pool: pubkeys[2],
//             pool_authority: pubkeys[3],
//             withdraw_authority: pubkeys[4],
//             vault: pubkeys[5],
//         }
//     }
// }
// impl<'info> From<InitializeAccounts<'_, 'info>> for [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN] {
//     fn from(accounts: InitializeAccounts<'_, 'info>) -> Self {
//         [
//             accounts.owner.clone(),
//             accounts.mint.clone(),
//             accounts.pool.clone(),
//             accounts.pool_authority.clone(),
//             accounts.withdraw_authority.clone(),
//             accounts.vault.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN]> for InitializeAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: &arr[0],
//             mint: &arr[1],
//             pool: &arr[2],
//             pool_authority: &arr[3],
//             withdraw_authority: &arr[4],
//             vault: &arr[5],
//         }
//     }
// }
// pub const INITIALIZE_IX_DISCM: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct InitializeIxArgs {
//     pub amp_factor: u16,
//     pub swap_fee: u64,
//     pub max_caps: Vec<u64>,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct InitializeIxData(pub InitializeIxArgs);
// impl From<InitializeIxArgs> for InitializeIxData {
//     fn from(args: InitializeIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl InitializeIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != INITIALIZE_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {INITIALIZE_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(InitializeIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&INITIALIZE_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn initialize_ix_with_program_id(
//     program_id: Pubkey,
//     keys: InitializeKeys,
//     args: InitializeIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; INITIALIZE_IX_ACCOUNTS_LEN] = keys.into();
//     let data: InitializeIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn initialize_verify_account_keys(
//     accounts: InitializeAccounts<'_, '_>,
//     keys: InitializeKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.owner.key, keys.owner),
//         (*accounts.mint.key, keys.mint),
//         (*accounts.pool.key, keys.pool),
//         (*accounts.pool_authority.key, keys.pool_authority),
//         (*accounts.withdraw_authority.key, keys.withdraw_authority),
//         (*accounts.vault.key, keys.vault),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn initialize_verify_writable_privileges<'me, 'info>(
//     accounts: InitializeAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn initialize_verify_signer_privileges<'me, 'info>(
//     accounts: InitializeAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.owner] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn initialize_verify_account_privileges<'me, 'info>(
//     accounts: InitializeAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     initialize_verify_writable_privileges(accounts)?;
//     initialize_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const PAUSE_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct PauseAccounts<'me, 'info> {
//     pub owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct PauseKeys {
//     pub owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<PauseAccounts<'_, '_>> for PauseKeys {
//     fn from(accounts: PauseAccounts) -> Self {
//         Self {
//             owner: *accounts.owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<PauseKeys> for [AccountMeta; PAUSE_IX_ACCOUNTS_LEN] {
//     fn from(keys: PauseKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; PAUSE_IX_ACCOUNTS_LEN]> for PauseKeys {
//     fn from(pubkeys: [Pubkey; PAUSE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<PauseAccounts<'_, 'info>> for [AccountInfo<'info>; PAUSE_IX_ACCOUNTS_LEN] {
//     fn from(accounts: PauseAccounts<'_, 'info>) -> Self {
//         [accounts.owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; PAUSE_IX_ACCOUNTS_LEN]> for PauseAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; PAUSE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const PAUSE_IX_DISCM: [u8; 8] = [211, 22, 221, 251, 74, 121, 193, 47];
// #[derive(Clone, Debug, PartialEq)]
// pub struct PauseIxData;
// impl PauseIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != PAUSE_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {PAUSE_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self)
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&PAUSE_IX_DISCM)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn pause_ix_with_program_id(program_id: Pubkey, keys: PauseKeys) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; PAUSE_IX_ACCOUNTS_LEN] = keys.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: PauseIxData.try_to_vec()?,
//     })
// }
// pub fn pause_ix(keys: PauseKeys) -> std::io::Result<Instruction> {
//     pause_ix_with_program_id(super::ID, keys)
// }
// pub fn pause_verify_account_keys(accounts: PauseAccounts<'_, '_>, keys: PauseKeys) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [(*accounts.owner.key, keys.owner), (*accounts.pool.key, keys.pool)] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn pause_verify_writable_privileges<'me, 'info>(
//     accounts: PauseAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn pause_verify_signer_privileges<'me, 'info>(
//     accounts: PauseAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.owner] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn pause_verify_account_privileges<'me, 'info>(
//     accounts: PauseAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     pause_verify_writable_privileges(accounts)?;
//     pause_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const REJECT_OWNER_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct RejectOwnerAccounts<'me, 'info> {
//     pub pending_owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct RejectOwnerKeys {
//     pub pending_owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<RejectOwnerAccounts<'_, '_>> for RejectOwnerKeys {
//     fn from(accounts: RejectOwnerAccounts) -> Self {
//         Self {
//             pending_owner: *accounts.pending_owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<RejectOwnerKeys> for [AccountMeta; REJECT_OWNER_IX_ACCOUNTS_LEN] {
//     fn from(keys: RejectOwnerKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.pending_owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; REJECT_OWNER_IX_ACCOUNTS_LEN]> for RejectOwnerKeys {
//     fn from(pubkeys: [Pubkey; REJECT_OWNER_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pending_owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<RejectOwnerAccounts<'_, 'info>> for [AccountInfo<'info>; REJECT_OWNER_IX_ACCOUNTS_LEN] {
//     fn from(accounts: RejectOwnerAccounts<'_, 'info>) -> Self {
//         [accounts.pending_owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; REJECT_OWNER_IX_ACCOUNTS_LEN]> for RejectOwnerAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; REJECT_OWNER_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             pending_owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const REJECT_OWNER_IX_DISCM: [u8; 8] = [238, 206, 198, 215, 51, 178, 133, 228];
// #[derive(Clone, Debug, PartialEq)]
// pub struct RejectOwnerIxData;
// impl RejectOwnerIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != REJECT_OWNER_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {REJECT_OWNER_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self)
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&REJECT_OWNER_IX_DISCM)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn reject_owner_ix_with_program_id(program_id: Pubkey, keys: RejectOwnerKeys) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; REJECT_OWNER_IX_ACCOUNTS_LEN] = keys.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: RejectOwnerIxData.try_to_vec()?,
//     })
// }
// pub fn reject_owner_verify_account_keys(
//     accounts: RejectOwnerAccounts<'_, '_>,
//     keys: RejectOwnerKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.pending_owner.key, keys.pending_owner),
//         (*accounts.pool.key, keys.pool),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn reject_owner_verify_writable_privileges<'me, 'info>(
//     accounts: RejectOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn reject_owner_verify_signer_privileges<'me, 'info>(
//     accounts: RejectOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.pending_owner] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn reject_owner_verify_account_privileges<'me, 'info>(
//     accounts: RejectOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     reject_owner_verify_writable_privileges(accounts)?;
//     reject_owner_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const SHUTDOWN_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct ShutdownAccounts<'me, 'info> {
//     pub owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct ShutdownKeys {
//     pub owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<ShutdownAccounts<'_, '_>> for ShutdownKeys {
//     fn from(accounts: ShutdownAccounts) -> Self {
//         Self {
//             owner: *accounts.owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<ShutdownKeys> for [AccountMeta; SHUTDOWN_IX_ACCOUNTS_LEN] {
//     fn from(keys: ShutdownKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; SHUTDOWN_IX_ACCOUNTS_LEN]> for ShutdownKeys {
//     fn from(pubkeys: [Pubkey; SHUTDOWN_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<ShutdownAccounts<'_, 'info>> for [AccountInfo<'info>; SHUTDOWN_IX_ACCOUNTS_LEN] {
//     fn from(accounts: ShutdownAccounts<'_, 'info>) -> Self {
//         [accounts.owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; SHUTDOWN_IX_ACCOUNTS_LEN]> for ShutdownAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; SHUTDOWN_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const SHUTDOWN_IX_DISCM: [u8; 8] = [146, 204, 241, 213, 86, 21, 253, 211];
// #[derive(Clone, Debug, PartialEq)]
// pub struct ShutdownIxData;
// impl ShutdownIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != SHUTDOWN_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {SHUTDOWN_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self)
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&SHUTDOWN_IX_DISCM)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn shutdown_ix_with_program_id(program_id: Pubkey, keys: ShutdownKeys) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; SHUTDOWN_IX_ACCOUNTS_LEN] = keys.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: ShutdownIxData.try_to_vec()?,
//     })
// }
// pub fn shutdown_ix(keys: ShutdownKeys) -> std::io::Result<Instruction> {
//     shutdown_ix_with_program_id(super::ID, keys)
// }
// pub fn shutdown_verify_account_keys(
//     accounts: ShutdownAccounts<'_, '_>,
//     keys: ShutdownKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [(*accounts.owner.key, keys.owner), (*accounts.pool.key, keys.pool)] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn shutdown_verify_writable_privileges<'me, 'info>(
//     accounts: ShutdownAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.owner, accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn shutdown_verify_account_privileges<'me, 'info>(
//     accounts: ShutdownAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     shutdown_verify_writable_privileges(accounts)?;
//     Ok(())
// }
// pub const SWAP_IX_ACCOUNTS_LEN: usize = 12;
// #[derive(Copy, Clone, Debug)]
// pub struct SwapAccounts<'me, 'info> {
//     pub user: &'me AccountInfo<'info>,
//     pub user_token_in: &'me AccountInfo<'info>,
//     pub user_token_out: &'me AccountInfo<'info>,
//     pub vault_token_in: &'me AccountInfo<'info>,
//     pub vault_token_out: &'me AccountInfo<'info>,
//     pub beneficiary_token_out: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
//     pub withdraw_authority: &'me AccountInfo<'info>,
//     pub vault: &'me AccountInfo<'info>,
//     pub vault_authority: &'me AccountInfo<'info>,
//     pub vault_program: &'me AccountInfo<'info>,
//     pub token_program: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct SwapKeys {
//     pub user: Pubkey,
//     pub user_token_in: Pubkey,
//     pub user_token_out: Pubkey,
//     pub vault_token_in: Pubkey,
//     pub vault_token_out: Pubkey,
//     pub beneficiary_token_out: Pubkey,
//     pub pool: Pubkey,
//     pub withdraw_authority: Pubkey,
//     pub vault: Pubkey,
//     pub vault_authority: Pubkey,
//     pub vault_program: Pubkey,
//     pub token_program: Pubkey,
// }
// impl From<SwapAccounts<'_, '_>> for SwapKeys {
//     fn from(accounts: SwapAccounts) -> Self {
//         Self {
//             user: *accounts.user.key,
//             user_token_in: *accounts.user_token_in.key,
//             user_token_out: *accounts.user_token_out.key,
//             vault_token_in: *accounts.vault_token_in.key,
//             vault_token_out: *accounts.vault_token_out.key,
//             beneficiary_token_out: *accounts.beneficiary_token_out.key,
//             pool: *accounts.pool.key,
//             withdraw_authority: *accounts.withdraw_authority.key,
//             vault: *accounts.vault.key,
//             vault_authority: *accounts.vault_authority.key,
//             vault_program: *accounts.vault_program.key,
//             token_program: *accounts.token_program.key,
//         }
//     }
// }
// impl From<SwapKeys> for [AccountMeta; SWAP_IX_ACCOUNTS_LEN] {
//     fn from(keys: SwapKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.user,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.user_token_in,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.user_token_out,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_token_in,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_token_out,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.beneficiary_token_out,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.withdraw_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; SWAP_IX_ACCOUNTS_LEN]> for SwapKeys {
//     fn from(pubkeys: [Pubkey; SWAP_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: pubkeys[0],
//             user_token_in: pubkeys[1],
//             user_token_out: pubkeys[2],
//             vault_token_in: pubkeys[3],
//             vault_token_out: pubkeys[4],
//             beneficiary_token_out: pubkeys[5],
//             pool: pubkeys[6],
//             withdraw_authority: pubkeys[7],
//             vault: pubkeys[8],
//             vault_authority: pubkeys[9],
//             vault_program: pubkeys[10],
//             token_program: pubkeys[11],
//         }
//     }
// }
// impl<'info> From<SwapAccounts<'_, 'info>> for [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN] {
//     fn from(accounts: SwapAccounts<'_, 'info>) -> Self {
//         [
//             accounts.user.clone(),
//             accounts.user_token_in.clone(),
//             accounts.user_token_out.clone(),
//             accounts.vault_token_in.clone(),
//             accounts.vault_token_out.clone(),
//             accounts.beneficiary_token_out.clone(),
//             accounts.pool.clone(),
//             accounts.withdraw_authority.clone(),
//             accounts.vault.clone(),
//             accounts.vault_authority.clone(),
//             accounts.vault_program.clone(),
//             accounts.token_program.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]> for SwapAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; SWAP_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: &arr[0],
//             user_token_in: &arr[1],
//             user_token_out: &arr[2],
//             vault_token_in: &arr[3],
//             vault_token_out: &arr[4],
//             beneficiary_token_out: &arr[5],
//             pool: &arr[6],
//             withdraw_authority: &arr[7],
//             vault: &arr[8],
//             vault_authority: &arr[9],
//             vault_program: &arr[10],
//             token_program: &arr[11],
//         }
//     }
// }
// pub const SWAP_IX_DISCM: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct SwapIxArgs {
//     pub amount_in: Option<u64>,
//     pub minimum_amount_out: u64,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct SwapIxData(pub SwapIxArgs);
// impl From<SwapIxArgs> for SwapIxData {
//     fn from(args: SwapIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl SwapIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != SWAP_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {SWAP_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(SwapIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&SWAP_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn swap_ix_with_program_id(program_id: Pubkey, keys: SwapKeys, args: SwapIxArgs) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; SWAP_IX_ACCOUNTS_LEN] = keys.into();
//     let data: SwapIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn swap_ix(keys: SwapKeys, args: SwapIxArgs) -> std::io::Result<Instruction> {
//     swap_ix_with_program_id(super::ID, keys, args)
// }
// pub fn swap_verify_account_keys(accounts: SwapAccounts<'_, '_>, keys: SwapKeys) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.user.key, keys.user),
//         (*accounts.user_token_in.key, keys.user_token_in),
//         (*accounts.user_token_out.key, keys.user_token_out),
//         (*accounts.vault_token_in.key, keys.vault_token_in),
//         (*accounts.vault_token_out.key, keys.vault_token_out),
//         (*accounts.beneficiary_token_out.key, keys.beneficiary_token_out),
//         (*accounts.pool.key, keys.pool),
//         (*accounts.withdraw_authority.key, keys.withdraw_authority),
//         (*accounts.vault.key, keys.vault),
//         (*accounts.vault_authority.key, keys.vault_authority),
//         (*accounts.vault_program.key, keys.vault_program),
//         (*accounts.token_program.key, keys.token_program),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn swap_verify_writable_privileges<'me, 'info>(
//     accounts: SwapAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [
//         accounts.user_token_in,
//         accounts.user_token_out,
//         accounts.vault_token_in,
//         accounts.vault_token_out,
//         accounts.beneficiary_token_out,
//         accounts.pool,
//     ] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn swap_verify_signer_privileges<'me, 'info>(
//     accounts: SwapAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.user] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn swap_verify_account_privileges<'me, 'info>(
//     accounts: SwapAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     swap_verify_writable_privileges(accounts)?;
//     swap_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const SWAP_V2_IX_ACCOUNTS_LEN: usize = 15;
// #[derive(Copy, Clone, Debug)]
// pub struct SwapV2Accounts<'me, 'info> {
//     pub user: &'me AccountInfo<'info>,
//     pub mint_in: &'me AccountInfo<'info>,
//     pub mint_out: &'me AccountInfo<'info>,
//     pub user_token_in: &'me AccountInfo<'info>,
//     pub user_token_out: &'me AccountInfo<'info>,
//     pub vault_token_in: &'me AccountInfo<'info>,
//     pub vault_token_out: &'me AccountInfo<'info>,
//     pub beneficiary_token_out: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
//     pub withdraw_authority: &'me AccountInfo<'info>,
//     pub vault: &'me AccountInfo<'info>,
//     pub vault_authority: &'me AccountInfo<'info>,
//     pub vault_program: &'me AccountInfo<'info>,
//     pub token_program: &'me AccountInfo<'info>,
//     pub token_2022program: &'me AccountInfo<'info>,
// }

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwapV2Keys {
    pub user: Pubkey,
    pub mint_in: Pubkey,
    pub mint_out: Pubkey,
    pub user_token_in: Pubkey,
    pub user_token_out: Pubkey,
    pub vault_token_in: Pubkey,
    pub vault_token_out: Pubkey,
    pub beneficiary_token_out: Pubkey,
    pub pool: Pubkey,
    pub withdraw_authority: Pubkey,
    pub vault: Pubkey,
    pub vault_authority: Pubkey,
    pub vault_program: Pubkey,
    pub token_program: Pubkey,
    pub token_2022program: Pubkey,
}

// impl From<SwapV2Accounts<'_, '_>> for SwapV2Keys {
//     fn from(accounts: SwapV2Accounts) -> Self {
//         Self {
//             user: *accounts.user.key,
//             mint_in: *accounts.mint_in.key,
//             mint_out: *accounts.mint_out.key,
//             user_token_in: *accounts.user_token_in.key,
//             user_token_out: *accounts.user_token_out.key,
//             vault_token_in: *accounts.vault_token_in.key,
//             vault_token_out: *accounts.vault_token_out.key,
//             beneficiary_token_out: *accounts.beneficiary_token_out.key,
//             pool: *accounts.pool.key,
//             withdraw_authority: *accounts.withdraw_authority.key,
//             vault: *accounts.vault.key,
//             vault_authority: *accounts.vault_authority.key,
//             vault_program: *accounts.vault_program.key,
//             token_program: *accounts.token_program.key,
//             token_2022program: *accounts.token_2022program.key,
//         }
//     }
// }
// impl From<SwapV2Keys> for [AccountMeta; SWAP_V2_IX_ACCOUNTS_LEN] {
//     fn from(keys: SwapV2Keys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.user,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.mint_in,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.mint_out,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.user_token_in,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.user_token_out,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_token_in,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_token_out,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.beneficiary_token_out,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.withdraw_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_2022program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; SWAP_V2_IX_ACCOUNTS_LEN]> for SwapV2Keys {
//     fn from(pubkeys: [Pubkey; SWAP_V2_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: pubkeys[0],
//             mint_in: pubkeys[1],
//             mint_out: pubkeys[2],
//             user_token_in: pubkeys[3],
//             user_token_out: pubkeys[4],
//             vault_token_in: pubkeys[5],
//             vault_token_out: pubkeys[6],
//             beneficiary_token_out: pubkeys[7],
//             pool: pubkeys[8],
//             withdraw_authority: pubkeys[9],
//             vault: pubkeys[10],
//             vault_authority: pubkeys[11],
//             vault_program: pubkeys[12],
//             token_program: pubkeys[13],
//             token_2022program: pubkeys[14],
//         }
//     }
// }
// impl<'info> From<SwapV2Accounts<'_, 'info>> for [AccountInfo<'info>; SWAP_V2_IX_ACCOUNTS_LEN] {
//     fn from(accounts: SwapV2Accounts<'_, 'info>) -> Self {
//         [
//             accounts.user.clone(),
//             accounts.mint_in.clone(),
//             accounts.mint_out.clone(),
//             accounts.user_token_in.clone(),
//             accounts.user_token_out.clone(),
//             accounts.vault_token_in.clone(),
//             accounts.vault_token_out.clone(),
//             accounts.beneficiary_token_out.clone(),
//             accounts.pool.clone(),
//             accounts.withdraw_authority.clone(),
//             accounts.vault.clone(),
//             accounts.vault_authority.clone(),
//             accounts.vault_program.clone(),
//             accounts.token_program.clone(),
//             accounts.token_2022program.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; SWAP_V2_IX_ACCOUNTS_LEN]> for SwapV2Accounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; SWAP_V2_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: &arr[0],
//             mint_in: &arr[1],
//             mint_out: &arr[2],
//             user_token_in: &arr[3],
//             user_token_out: &arr[4],
//             vault_token_in: &arr[5],
//             vault_token_out: &arr[6],
//             beneficiary_token_out: &arr[7],
//             pool: &arr[8],
//             withdraw_authority: &arr[9],
//             vault: &arr[10],
//             vault_authority: &arr[11],
//             vault_program: &arr[12],
//             token_program: &arr[13],
//             token_2022program: &arr[14],
//         }
//     }
// }

pub const SWAP_V2_IX_DISCM: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct SwapV2IxArgs {
    pub amount_in: Option<u64>,
    pub minimum_amount_out: u64,
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct SwapV2IxData(pub SwapV2IxArgs);
// impl From<SwapV2IxArgs> for SwapV2IxData {
//     fn from(args: SwapV2IxArgs) -> Self {
//         Self(args)
//     }
// }

// impl SwapV2IxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != SWAP_V2_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {SWAP_V2_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(SwapV2IxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&SWAP_V2_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }

// pub fn swap_v2_ix_with_program_id(
//     program_id: Pubkey,
//     keys: SwapV2Keys,
//     args: SwapV2IxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; SWAP_V2_IX_ACCOUNTS_LEN] = keys.into();
//     let data: SwapV2IxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn swap_v2_ix(keys: SwapV2Keys, args: SwapV2IxArgs) -> std::io::Result<Instruction> {
//     swap_v2_ix_with_program_id(super::ID, keys, args)
// }

// pub fn swap_v2_verify_account_keys(accounts: SwapV2Accounts<'_, '_>, keys: SwapV2Keys) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.user.key, keys.user),
//         (*accounts.mint_in.key, keys.mint_in),
//         (*accounts.mint_out.key, keys.mint_out),
//         (*accounts.user_token_in.key, keys.user_token_in),
//         (*accounts.user_token_out.key, keys.user_token_out),
//         (*accounts.vault_token_in.key, keys.vault_token_in),
//         (*accounts.vault_token_out.key, keys.vault_token_out),
//         (*accounts.beneficiary_token_out.key, keys.beneficiary_token_out),
//         (*accounts.pool.key, keys.pool),
//         (*accounts.withdraw_authority.key, keys.withdraw_authority),
//         (*accounts.vault.key, keys.vault),
//         (*accounts.vault_authority.key, keys.vault_authority),
//         (*accounts.vault_program.key, keys.vault_program),
//         (*accounts.token_program.key, keys.token_program),
//         (*accounts.token_2022program.key, keys.token_2022program),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// // pub fn swap_v2_verify_writable_privileges<'me, 'info>(
// //     accounts: SwapV2Accounts<'me, 'info>,
// // ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
// //     for should_be_writable in [
// //         accounts.user_token_in,
// //         accounts.user_token_out,
// //         accounts.vault_token_in,
// //         accounts.vault_token_out,
// //         accounts.beneficiary_token_out,
// //         accounts.pool,
// //     ] {
// //         if !should_be_writable.is_writable {
// //             return Err((should_be_writable, ProgramError::InvalidAccountData));
// //         }
// //     }
// //     Ok(())
// // }
// // pub fn swap_v2_verify_signer_privileges<'me, 'info>(
// //     accounts: SwapV2Accounts<'me, 'info>,
// // ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
// //     for should_be_signer in [accounts.user] {
// //         if !should_be_signer.is_signer {
// //             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
// //         }
// //     }
// //     Ok(())
// // }
// // pub fn swap_v2_verify_account_privileges<'me, 'info>(
// //     accounts: SwapV2Accounts<'me, 'info>,
// // ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
// //     swap_v2_verify_writable_privileges(accounts)?;
// //     swap_v2_verify_signer_privileges(accounts)?;
// //     Ok(())
// // }
// pub const TRANSFER_OWNER_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct TransferOwnerAccounts<'me, 'info> {
//     pub owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct TransferOwnerKeys {
//     pub owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<TransferOwnerAccounts<'_, '_>> for TransferOwnerKeys {
//     fn from(accounts: TransferOwnerAccounts) -> Self {
//         Self {
//             owner: *accounts.owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<TransferOwnerKeys> for [AccountMeta; TRANSFER_OWNER_IX_ACCOUNTS_LEN] {
//     fn from(keys: TransferOwnerKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; TRANSFER_OWNER_IX_ACCOUNTS_LEN]> for TransferOwnerKeys {
//     fn from(pubkeys: [Pubkey; TRANSFER_OWNER_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<TransferOwnerAccounts<'_, 'info>> for [AccountInfo<'info>; TRANSFER_OWNER_IX_ACCOUNTS_LEN] {
//     fn from(accounts: TransferOwnerAccounts<'_, 'info>) -> Self {
//         [accounts.owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; TRANSFER_OWNER_IX_ACCOUNTS_LEN]> for TransferOwnerAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; TRANSFER_OWNER_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const TRANSFER_OWNER_IX_DISCM: [u8; 8] = [245, 25, 221, 175, 106, 229, 225, 45];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct TransferOwnerIxArgs {
//     pub new_owner: Pubkey,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct TransferOwnerIxData(pub TransferOwnerIxArgs);
// impl From<TransferOwnerIxArgs> for TransferOwnerIxData {
//     fn from(args: TransferOwnerIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl TransferOwnerIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != TRANSFER_OWNER_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {TRANSFER_OWNER_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(TransferOwnerIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&TRANSFER_OWNER_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn transfer_owner_ix_with_program_id(
//     program_id: Pubkey,
//     keys: TransferOwnerKeys,
//     args: TransferOwnerIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; TRANSFER_OWNER_IX_ACCOUNTS_LEN] = keys.into();
//     let data: TransferOwnerIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn transfer_owner_ix(keys: TransferOwnerKeys, args: TransferOwnerIxArgs) -> std::io::Result<Instruction> {
//     transfer_owner_ix_with_program_id(super::ID, keys, args)
// }
// pub fn transfer_owner_verify_account_keys(
//     accounts: TransferOwnerAccounts<'_, '_>,
//     keys: TransferOwnerKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [(*accounts.owner.key, keys.owner), (*accounts.pool.key, keys.pool)] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn transfer_owner_verify_writable_privileges<'me, 'info>(
//     accounts: TransferOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn transfer_owner_verify_signer_privileges<'me, 'info>(
//     accounts: TransferOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.owner] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn transfer_owner_verify_account_privileges<'me, 'info>(
//     accounts: TransferOwnerAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     transfer_owner_verify_writable_privileges(accounts)?;
//     transfer_owner_verify_signer_privileges(accounts)?;
//     Ok(())
// }
// pub const UNPAUSE_IX_ACCOUNTS_LEN: usize = 2;
// #[derive(Copy, Clone, Debug)]
// pub struct UnpauseAccounts<'me, 'info> {
//     pub owner: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct UnpauseKeys {
//     pub owner: Pubkey,
//     pub pool: Pubkey,
// }
// impl From<UnpauseAccounts<'_, '_>> for UnpauseKeys {
//     fn from(accounts: UnpauseAccounts) -> Self {
//         Self {
//             owner: *accounts.owner.key,
//             pool: *accounts.pool.key,
//         }
//     }
// }
// impl From<UnpauseKeys> for [AccountMeta; UNPAUSE_IX_ACCOUNTS_LEN] {
//     fn from(keys: UnpauseKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.owner,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; UNPAUSE_IX_ACCOUNTS_LEN]> for UnpauseKeys {
//     fn from(pubkeys: [Pubkey; UNPAUSE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: pubkeys[0],
//             pool: pubkeys[1],
//         }
//     }
// }
// impl<'info> From<UnpauseAccounts<'_, 'info>> for [AccountInfo<'info>; UNPAUSE_IX_ACCOUNTS_LEN] {
//     fn from(accounts: UnpauseAccounts<'_, 'info>) -> Self {
//         [accounts.owner.clone(), accounts.pool.clone()]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; UNPAUSE_IX_ACCOUNTS_LEN]> for UnpauseAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; UNPAUSE_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             owner: &arr[0],
//             pool: &arr[1],
//         }
//     }
// }
// pub const UNPAUSE_IX_DISCM: [u8; 8] = [169, 144, 4, 38, 10, 141, 188, 255];
// #[derive(Clone, Debug, PartialEq)]
// pub struct UnpauseIxData;
// impl UnpauseIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != UNPAUSE_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {UNPAUSE_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self)
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&UNPAUSE_IX_DISCM)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn unpause_ix_with_program_id(program_id: Pubkey, keys: UnpauseKeys) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; UNPAUSE_IX_ACCOUNTS_LEN] = keys.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: UnpauseIxData.try_to_vec()?,
//     })
// }
// pub fn unpause_ix(keys: UnpauseKeys) -> std::io::Result<Instruction> {
//     unpause_ix_with_program_id(super::ID, keys)
// }

// pub const WITHDRAW_IX_ACCOUNTS_LEN: usize = 10;
// #[derive(Copy, Clone, Debug)]
// pub struct WithdrawAccounts<'me, 'info> {
//     pub user: &'me AccountInfo<'info>,
//     pub user_pool_token: &'me AccountInfo<'info>,
//     pub mint: &'me AccountInfo<'info>,
//     pub pool: &'me AccountInfo<'info>,
//     pub withdraw_authority: &'me AccountInfo<'info>,
//     pub vault: &'me AccountInfo<'info>,
//     pub vault_authority: &'me AccountInfo<'info>,
//     pub vault_program: &'me AccountInfo<'info>,
//     pub token_program: &'me AccountInfo<'info>,
//     pub token_program_2022: &'me AccountInfo<'info>,
// }
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct WithdrawKeys {
//     pub user: Pubkey,
//     pub user_pool_token: Pubkey,
//     pub mint: Pubkey,
//     pub pool: Pubkey,
//     pub withdraw_authority: Pubkey,
//     pub vault: Pubkey,
//     pub vault_authority: Pubkey,
//     pub vault_program: Pubkey,
//     pub token_program: Pubkey,
//     pub token_program_2022: Pubkey,
// }
// impl From<WithdrawAccounts<'_, '_>> for WithdrawKeys {
//     fn from(accounts: WithdrawAccounts) -> Self {
//         Self {
//             user: *accounts.user.key,
//             user_pool_token: *accounts.user_pool_token.key,
//             mint: *accounts.mint.key,
//             pool: *accounts.pool.key,
//             withdraw_authority: *accounts.withdraw_authority.key,
//             vault: *accounts.vault.key,
//             vault_authority: *accounts.vault_authority.key,
//             vault_program: *accounts.vault_program.key,
//             token_program: *accounts.token_program.key,
//             token_program_2022: *accounts.token_program_2022.key,
//         }
//     }
// }
// impl From<WithdrawKeys> for [AccountMeta; WITHDRAW_IX_ACCOUNTS_LEN] {
//     fn from(keys: WithdrawKeys) -> Self {
//         [
//             AccountMeta {
//                 pubkey: keys.user,
//                 is_signer: true,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.user_pool_token,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.mint,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.pool,
//                 is_signer: false,
//                 is_writable: true,
//             },
//             AccountMeta {
//                 pubkey: keys.withdraw_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_authority,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.vault_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program,
//                 is_signer: false,
//                 is_writable: false,
//             },
//             AccountMeta {
//                 pubkey: keys.token_program_2022,
//                 is_signer: false,
//                 is_writable: false,
//             },
//         ]
//     }
// }
// impl From<[Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]> for WithdrawKeys {
//     fn from(pubkeys: [Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: pubkeys[0],
//             user_pool_token: pubkeys[1],
//             mint: pubkeys[2],
//             pool: pubkeys[3],
//             withdraw_authority: pubkeys[4],
//             vault: pubkeys[5],
//             vault_authority: pubkeys[6],
//             vault_program: pubkeys[7],
//             token_program: pubkeys[8],
//             token_program_2022: pubkeys[9],
//         }
//     }
// }
// impl<'info> From<WithdrawAccounts<'_, 'info>> for [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN] {
//     fn from(accounts: WithdrawAccounts<'_, 'info>) -> Self {
//         [
//             accounts.user.clone(),
//             accounts.user_pool_token.clone(),
//             accounts.mint.clone(),
//             accounts.pool.clone(),
//             accounts.withdraw_authority.clone(),
//             accounts.vault.clone(),
//             accounts.vault_authority.clone(),
//             accounts.vault_program.clone(),
//             accounts.token_program.clone(),
//             accounts.token_program_2022.clone(),
//         ]
//     }
// }
// impl<'me, 'info> From<&'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]> for WithdrawAccounts<'me, 'info> {
//     fn from(arr: &'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
//         Self {
//             user: &arr[0],
//             user_pool_token: &arr[1],
//             mint: &arr[2],
//             pool: &arr[3],
//             withdraw_authority: &arr[4],
//             vault: &arr[5],
//             vault_authority: &arr[6],
//             vault_program: &arr[7],
//             token_program: &arr[8],
//             token_program_2022: &arr[9],
//         }
//     }
// }
// pub const WITHDRAW_IX_DISCM: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
// #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
// pub struct WithdrawIxArgs {
//     pub amount: u64,
//     pub minimum_amounts_out: Vec<u64>,
// }
// #[derive(Clone, Debug, PartialEq)]
// pub struct WithdrawIxData(pub WithdrawIxArgs);
// impl From<WithdrawIxArgs> for WithdrawIxData {
//     fn from(args: WithdrawIxArgs) -> Self {
//         Self(args)
//     }
// }
// impl WithdrawIxData {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != WITHDRAW_IX_DISCM {
//             return Err(std::io::Error::other(format!(
//                 "discm does not match. Expected: {WITHDRAW_IX_DISCM:?}. Received: {maybe_discm:?}"
//             )));
//         }
//         Ok(Self(WithdrawIxArgs::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&WITHDRAW_IX_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }
// pub fn withdraw_ix_with_program_id(
//     program_id: Pubkey,
//     keys: WithdrawKeys,
//     args: WithdrawIxArgs,
// ) -> std::io::Result<Instruction> {
//     let metas: [AccountMeta; WITHDRAW_IX_ACCOUNTS_LEN] = keys.into();
//     let data: WithdrawIxData = args.into();
//     Ok(Instruction {
//         program_id,
//         accounts: Vec::from(metas),
//         data: data.try_to_vec()?,
//     })
// }
// pub fn withdraw_ix(keys: WithdrawKeys, args: WithdrawIxArgs) -> std::io::Result<Instruction> {
//     withdraw_ix_with_program_id(super::ID, keys, args)
// }
// pub fn withdraw_verify_account_keys(
//     accounts: WithdrawAccounts<'_, '_>,
//     keys: WithdrawKeys,
// ) -> Result<(), (Pubkey, Pubkey)> {
//     for (actual, expected) in [
//         (*accounts.user.key, keys.user),
//         (*accounts.user_pool_token.key, keys.user_pool_token),
//         (*accounts.mint.key, keys.mint),
//         (*accounts.pool.key, keys.pool),
//         (*accounts.withdraw_authority.key, keys.withdraw_authority),
//         (*accounts.vault.key, keys.vault),
//         (*accounts.vault_authority.key, keys.vault_authority),
//         (*accounts.vault_program.key, keys.vault_program),
//         (*accounts.token_program.key, keys.token_program),
//         (*accounts.token_program_2022.key, keys.token_program_2022),
//     ] {
//         if actual != expected {
//             return Err((actual, expected));
//         }
//     }
//     Ok(())
// }
// pub fn withdraw_verify_writable_privileges<'me, 'info>(
//     accounts: WithdrawAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_writable in [accounts.user_pool_token, accounts.mint, accounts.pool] {
//         if !should_be_writable.is_writable {
//             return Err((should_be_writable, ProgramError::InvalidAccountData));
//         }
//     }
//     Ok(())
// }
// pub fn withdraw_verify_signer_privileges<'me, 'info>(
//     accounts: WithdrawAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     for should_be_signer in [accounts.user] {
//         if !should_be_signer.is_signer {
//             return Err((should_be_signer, ProgramError::MissingRequiredSignature));
//         }
//     }
//     Ok(())
// }
// pub fn withdraw_verify_account_privileges<'me, 'info>(
//     accounts: WithdrawAccounts<'me, 'info>,
// ) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
//     withdraw_verify_writable_privileges(accounts)?;
//     withdraw_verify_signer_privileges(accounts)?;
//     Ok(())
// }
