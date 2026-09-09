use ahash::AHashSet;
use anyhow::Context;
use bloxroute_client::instruction::{tip_ix, tip_ix_with_key};
use jito_searcher_client::tip::get_jito_random_address;
use log::warn;
use nextblock_client::get_next_block_random_address;
use record::instruction::RecordInstruction;
use solana_program::example_mocks::solana_sdk::system_instruction;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash::Hash;
use solana_sdk::message::{v0, AddressLookupTableAccount, VersionedMessage};
use solana_sdk::pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use spl_token::instruction::close_account;
use utils::instructions::create_ata_token_or_not;

use utils::constants::{ALLOWED_TOKEN_2022, FLASHLOAN_ID, SWAP_PROGRAM_ID};
use utils::transaction::check_transaction_size;

#[derive(Debug, Clone, Default)]
pub struct IxBuilder {
    ixs: Vec<Instruction>,
    pub owner: Pubkey,
}

#[derive(Debug, Clone, Default)]
pub struct ArbitrageCompressedInstructionInput {
    pub tag: u8,
    pub ixs_data: Vec<Vec<u8>>,
    pub accounts: Vec<AccountMeta>,
    pub accounts_indices: Vec<Vec<u8>>,
    direct_instruction: Option<Instruction>,
}

impl ArbitrageCompressedInstructionInput {
    pub fn new(account: Pubkey) -> Self {
        let accounts = vec![AccountMeta::new(account, false)];
        Self {
            tag: 4_u8,
            ixs_data: vec![],
            accounts,
            accounts_indices: vec![],
            direct_instruction: None,
        }
    }

    /// Stores an already-built instruction for an isolated executor program.
    /// The legacy compressed ABI must not rewrite its program ID or payload.
    pub fn from_direct_instruction(instruction: Instruction) -> Self {
        Self {
            tag: 0,
            ixs_data: vec![],
            accounts: instruction.accounts.clone(),
            accounts_indices: vec![],
            direct_instruction: Some(instruction),
        }
    }
    pub fn push(&mut self, ix: Instruction) {
        self.ixs_data.push(ix.data);
        let indices = self.get_indices(ix.accounts);
        self.accounts_indices.push(indices);
    }

    pub fn get_indices(&mut self, accounts: Vec<AccountMeta>) -> Vec<u8> {
        let mut indices = vec![];
        'outer: for account in accounts.into_iter() {
            for (idx, indexed_account) in self.accounts.iter_mut().enumerate() {
                if indexed_account.pubkey == account.pubkey {
                    indices.push(idx as u8);
                    if account.is_writable && !indexed_account.is_writable {
                        indexed_account.is_writable = true;
                    }
                    if account.is_signer && !indexed_account.is_signer {
                        indexed_account.is_signer = true;
                    }
                    continue 'outer;
                }
            }
            self.accounts.push(account);
            indices.push(self.accounts.len() as u8 - 1)
        }
        indices
    }

    pub fn to_instruction(&self, amount: u64) -> Instruction {
        if let Some(instruction) = &self.direct_instruction {
            return instruction.clone();
        }
        let mut data = vec![4_u8];
        data.extend(amount.to_le_bytes());
        for (idx, ix_data) in self.ixs_data.iter().enumerate() {
            // accounts
            let indices = self.accounts_indices[idx].clone();
            data.push(indices.len() as u8);
            // data
            data.push(ix_data.len() as u8);
            data.extend(ix_data);
            data.extend(indices);
        }
        Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts: self.accounts.clone(),
            data,
        }
    }

    pub fn to_floashloan_ix(
        &self,
        flashloan_amount: u64,
        amount: u64,
        pool_ata: Pubkey,
        pool: Pubkey,
        ata_wsol: Pubkey,
    ) -> Instruction {
        assert!(
            self.direct_instruction.is_none(),
            "direct executor instructions cannot be wrapped in a flashloan"
        );
        let ix = self.to_instruction(amount);
        let mut accounts = vec![
            AccountMeta::new(pool_ata, false),
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new(ata_wsol, false),
            // AccountMeta::new_readonly(signer, true),
            AccountMeta::new_readonly(ix.program_id, false),
        ];
        accounts.extend(ix.accounts);
        if accounts.len() > 64 {
            warn!("{} accounts in cpi instructions ...", accounts.len());
        }
        let mut data = vec![8];
        data.extend(flashloan_amount.to_le_bytes());
        data.extend(ix.data);
        Instruction {
            program_id: FLASHLOAN_ID,
            accounts,
            data,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ArbitrageInstructionInput {
    pub tag: u8,
    pub account: Pubkey,
    pub ixs: Vec<(Vec<u8>, Vec<AccountMeta>)>,
}
impl ArbitrageInstructionInput {
    pub fn new(account: Pubkey) -> Self {
        Self {
            tag: 3_u8,
            account,
            ixs: vec![],
        }
    }

    pub fn to_floashloan_ix(
        &self,
        flashloan_amount: u64,
        amount: u64,
        pool_ata: Pubkey,
        pool: Pubkey,
        ata_wsol: Pubkey,
    ) -> Instruction {
        let ix = self.to_instruction(amount);
        let mut accounts = vec![
            AccountMeta::new(pool_ata, false),
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new(ata_wsol, false),
            // AccountMeta::new_readonly(signer, true),
            AccountMeta::new_readonly(ix.program_id, false),
        ];
        accounts.extend(ix.accounts);
        if accounts.len() > 64 {
            warn!("{} accounts in cpi instructions ...", accounts.len());
        }
        let mut data = vec![8];
        data.extend(flashloan_amount.to_le_bytes());
        data.extend(ix.data);
        Instruction {
            program_id: FLASHLOAN_ID,
            accounts,
            data,
        }
    }

    pub fn to_floashloan_reduced_ix(
        &self,
        flashloan_amount: u64,
        amount: u64,
        pool_ata: Pubkey,
        pool: Pubkey,
    ) -> Instruction {
        let ix = self.to_instruction(amount);
        let mut accounts = vec![
            AccountMeta::new(pool_ata, false),
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new_readonly(ix.program_id, false),
        ];
        accounts.extend(ix.accounts);
        let mut data = vec![6];
        data.extend(flashloan_amount.to_le_bytes());
        data.extend(ix.data);
        Instruction {
            program_id: FLASHLOAN_ID,
            accounts,
            data,
        }
    }

    pub fn push(&mut self, ix: Instruction) {
        self.ixs.push((ix.data, ix.accounts));
    }

    pub fn to_instruction(&self, amount: u64) -> Instruction {
        let mut data = vec![3_u8];
        data.extend(amount.to_le_bytes());
        let mut accounts = vec![AccountMeta::new(self.account, false)];
        for ix in self.ixs.clone().into_iter() {
            // accounts
            data.push(ix.1.len() as u8);
            // data
            data.push(ix.0.len() as u8);
            data.extend(ix.0);
            accounts.extend(ix.1);
        }
        //accounts.push(AccountMeta::new_readonly(SWAP_PROGRAM_ID, false));
        Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

impl IxBuilder {
    pub fn new(owner: Pubkey) -> Self {
        Self { ixs: vec![], owner }
    }

    pub fn add_compute_accounts_data_size_limit(&mut self, data_size: u32) {
        let ix = ComputeBudgetInstruction::set_loaded_accounts_data_size_limit(data_size);
        self.add_custom_ix(ix.program_id, ix.data, ix.accounts)
    }

    pub fn add_compute_unit_limit(&mut self, unit_limit: u32) {
        let ix = ComputeBudgetInstruction::set_compute_unit_limit(unit_limit);
        let mut accounts = ix.accounts;
        accounts.push(AccountMeta {
            pubkey: pubkey!("jitodontfront111111111111111111111111111123"),
            is_signer: false,
            is_writable: false,
        });
        self.add_custom_ix(ix.program_id, ix.data, accounts)
    }

    pub fn add_compute_unit_price(&mut self, micro_lamports: u64) {
        self.ixs
            .push(ComputeBudgetInstruction::set_compute_unit_price(micro_lamports));
    }

    // pub fn add_memo(&mut self, msg: &str) {
    //     self.ixs.push(build_memo(msg.as_bytes(), &[&self.owner]));
    // }

    pub fn add_compute_unit_limit_on_start(&mut self, unit_limit: u32) {
        self.ixs
            .insert(0, ComputeBudgetInstruction::set_compute_unit_limit(unit_limit));
    }

    pub fn add_compute_unit_price_on_start(&mut self, micro_lamports: u64) {
        self.ixs
            .insert(0, ComputeBudgetInstruction::set_compute_unit_price(micro_lamports));
    }

    pub fn add_create_ata_if_not_exists(&mut self, mint: &Pubkey) {
        self.ixs.push(create_ata_token_or_not(
            &self.owner,
            mint,
            &self.owner,
            Some(&spl_token::ID),
        ))
    }

    pub fn add_create_ata_2022_if_not_exists(&mut self, mint: &Pubkey) {
        self.ixs.push(create_ata_token_or_not(
            &self.owner,
            mint,
            &self.owner,
            Some(&spl_token_2022::ID),
        ))
    }

    pub fn get_create_ata_if_not_exists(&mut self, mint: &Pubkey) -> Instruction {
        create_ata_token_or_not(
            &self.owner,
            mint,
            &self.owner,
            if !ALLOWED_TOKEN_2022.contains(mint) {
                Some(&spl_token::ID)
            } else {
                Some(&spl_token_2022::ID)
            },
        )
    }

    pub fn get_create_ata_if_not_exists_v2(
        &mut self,
        mint: &Pubkey,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> Instruction {
        create_ata_token_or_not(
            &self.owner,
            mint,
            &self.owner,
            if !allowed_token2022.contains(mint) {
                Some(&spl_token::ID)
            } else {
                Some(&spl_token_2022::ID)
            },
        )
    }

    pub fn get_read_token_account(&self, token_account: &Pubkey) -> Instruction {
        Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts: vec![AccountMeta::new_readonly(*token_account, false)],
            data: vec![5],
        }
    }

    pub fn add_read_token_account(&mut self, token_account: &Pubkey) {
        self.ixs.push(self.get_read_token_account(token_account));
    }

    pub fn add_record_write_u64(&mut self, record_account: Pubkey, ta_account: Pubkey, add_to_start: bool) {
        let custom_program_id = Pubkey::new_from_array(record::ID);
        let record_data = RecordInstruction::WriteU64 { offset: 64 }.pack();
        let instruction = Instruction {
            program_id: custom_program_id,
            accounts: vec![
                AccountMeta::new(record_account, false),
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new_readonly(ta_account, false),
            ],
            data: record_data,
        };
        if add_to_start {
            self.ixs.insert(0, instruction);
        } else {
            self.ixs.push(instruction);
        }
    }

    pub fn add_custom_ix(&mut self, program_id: Pubkey, data: Vec<u8>, accounts: Vec<AccountMeta>) {
        self.ixs.push(Instruction {
            program_id,
            accounts,
            data,
        });
    }

    pub fn push_ix(&mut self, ix: Instruction) {
        self.ixs.push(ix);
    }

    pub fn close_ata_ix(&mut self, ata: &Pubkey) -> anyhow::Result<()> {
        self.ixs.push(
            close_account(&spl_token::id(), ata, &self.owner, &self.owner, &[&self.owner])
                .context("Failed to create close account instruction")?,
        );
        Ok(())
    }

    pub fn close_ata_2022_ix(&mut self, ata: &Pubkey) -> anyhow::Result<()> {
        let ix = spl_token_2022::instruction::close_account(&spl_token_2022::ID, ata, &self.owner, &self.owner, &[])?;
        self.ixs.push(ix);
        Ok(())
    }

    pub fn get_close_ata_ix(&mut self, ata: &Pubkey) -> Instruction {
        Instruction {
            program_id: spl_token::ID,
            accounts: vec![
                AccountMeta::new(*ata, false),
                AccountMeta::new(self.owner, false),
                AccountMeta::new(self.owner, true),
            ],
            data: vec![9],
        }
    }

    pub fn get_close_ata_2022_ix(&mut self, ata: &Pubkey) -> Instruction {
        spl_token_2022::instruction::close_account(&spl_token_2022::ID, ata, &self.owner, &self.owner, &[]).unwrap()
    }

    // record account pubkey!("2Xd9sUd9psiJSBF8XaYRqG8kc7dSG77QcDseQfLYoHTU")
    pub fn add_record_check_add(&mut self, record_account: Pubkey, ta_account: Pubkey, threshold: u64) {
        let custom_program_id = Pubkey::new_from_array(record::ID);
        let data = RecordInstruction::CheckAdd {
            offset: 64,
            addition: threshold,
        }
        .pack();
        self.ixs.push(Instruction {
            program_id: custom_program_id,
            accounts: vec![
                AccountMeta::new(record_account, false),
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new_readonly(ta_account, false),
                // this is just to not have sandwich ...
                // AccountMeta::new_readonly(pubkey!("CatzoSMUkTRidT5DwBxAC2pEtnwMBTpkCepHkFgZDiqb"), false),
            ],
            data,
        });
    }

    pub fn add_record_check_add_with_va(&mut self, record_account: Pubkey, ta_account: Pubkey, threshold: u64) {
        let custom_program_id = Pubkey::new_from_array(record::ID);
        let data = RecordInstruction::CheckAdd {
            offset: 64,
            addition: threshold,
        }
        .pack();
        self.ixs.push(Instruction {
            program_id: custom_program_id,
            accounts: vec![
                AccountMeta::new(record_account, false),
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new_readonly(ta_account, false),
                // this is just to not have sandwich ...
                AccountMeta::new_readonly(pubkey!("CatzoSMUkTRidT5DwBxAC2pEtnwMBTpkCepHkFgZDiqb"), false),
            ],
            data,
        });
    }
    pub fn add_bloxroute_tip_ix(&mut self, tip: u64) {
        self.ixs.push(tip_ix(tip, &self.owner));
    }

    pub fn add_bloxroute_tip_ix_with_key(&mut self, tip: u64, bloxroute_key: &Pubkey) {
        self.ixs.push(tip_ix_with_key(tip, &self.owner, bloxroute_key));
    }

    pub fn add_sync_native(&mut self, ata_wsol: &Pubkey) {
        match spl_token::instruction::sync_native(&spl_token::ID, ata_wsol) {
            Ok(ix) => self.ixs.push(ix),
            Err(_) => warn!("Failed to create sync native instruction"),
        };
    }

    pub fn add_system_transfer_ix(&mut self, amount: u64, to: &Pubkey) {
        self.ixs.push(system_instruction::transfer(&self.owner, to, amount));
    }

    pub fn add_jito_tip_ix(&mut self, tip: u64) {
        self.ixs.push(system_instruction::transfer(
            &self.owner,
            &get_jito_random_address(),
            tip,
        ));
    }

    pub fn add_nextblock_tip_ix(&mut self, tip: u64) {
        self.ixs.push(system_instruction::transfer(
            &self.owner,
            &get_next_block_random_address(),
            tip,
        ));
    }

    pub fn add_flashloan_init_pool(&mut self, mint: &Pubkey) {
        let (pool_authority, bump) = Pubkey::find_program_address(
            &[
                b"pa",
                // mint.as_ref(),
                mint.as_ref(),
                self.owner.as_ref(),
            ],
            &FLASHLOAN_ID,
        );
        let data = vec![0, bump];
        self.ixs.push(Instruction {
            program_id: FLASHLOAN_ID,
            accounts: vec![
                AccountMeta::new(pool_authority, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(&pool_authority, mint),
                    false,
                ),
                AccountMeta::new(self.owner, true),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            ],
            data,
        });
    }

    pub fn add_flashloan_deposit(&mut self, mint: &Pubkey, pool: &Pubkey, deposit: u64) {
        let mut data = vec![1];
        data.extend(deposit.to_le_bytes());
        self.ixs.push(Instruction {
            program_id: FLASHLOAN_ID,
            accounts: vec![
                AccountMeta::new_readonly(*pool, false),
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(&self.owner, mint),
                    false,
                ),
                AccountMeta::new(self.owner, true),
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(pool, mint),
                    false,
                ),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data,
        });
    }

    pub fn add_flashloan_dummy(&mut self) {
        let data = vec![7];
        self.ixs.push(Instruction {
            program_id: FLASHLOAN_ID,
            accounts: vec![
                AccountMeta::new_readonly(pubkey!("CatzoSMUkTRidT5DwBxAC2pEtnwMBTpkCepHkFgZDiqb"), false),
                AccountMeta::new_readonly(pubkey!("jitodontfront11111111SentWithNextBLoCKdotio"), false),
            ],
            data,
        });
    }

    pub fn add_flashloan_withdraw(&mut self, mint: &Pubkey, pool: &Pubkey, withdraw: u64) {
        let mut data = vec![2];
        data.extend(withdraw.to_le_bytes());
        self.ixs.push(Instruction {
            program_id: FLASHLOAN_ID,
            accounts: vec![
                AccountMeta::new_readonly(*pool, false),
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(&self.owner, mint),
                    false,
                ),
                AccountMeta::new(self.owner, true),
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(pool, mint),
                    false,
                ),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data,
        });
    }

    // TODO - remove generating of ATA ...
    pub fn add_flashloan_borrow(&mut self, mint: &Pubkey, pool: &Pubkey, borrow: u64, index: u8) {
        let mut data = vec![3];
        data.extend(borrow.to_le_bytes());
        data.push(index);
        self.ixs.push(Instruction {
            program_id: FLASHLOAN_ID,
            accounts: vec![
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(pool, mint),
                    false,
                ),
                AccountMeta::new_readonly(*pool, false),
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(&self.owner, mint),
                    false,
                ),
                AccountMeta::new(self.owner, true),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(
                    pubkey!("Sysvar1nstructions1111111111111111111111111"), //Sysvar1111111111111111111111111111111111111
                    false,
                ),
            ],
            data,
        });
    }

    pub fn add_flashloan_borrow_with_ata_start(
        &mut self,
        pool: &Pubkey,
        ata_pool: &Pubkey,
        ata: &Pubkey,
        borrow: u64,
        index: u8,
    ) {
        let mut data = vec![3];
        data.extend(borrow.to_le_bytes());
        data.push(index);
        self.ixs.insert(
            0,
            Instruction {
                program_id: FLASHLOAN_ID,
                accounts: vec![
                    AccountMeta::new(
                        // spl_associated_token_account::get_associated_token_address(
                        //     &pool,
                        //     &mint,
                        // ),
                        *ata_pool, false,
                    ),
                    AccountMeta::new_readonly(*pool, false),
                    AccountMeta::new(
                        // spl_associated_token_account::get_associated_token_address(
                        //     &self.owner,
                        //     &mint,
                        // ),
                        *ata, false,
                    ),
                    AccountMeta::new(self.owner, true),
                    AccountMeta::new_readonly(spl_token::id(), false),
                    AccountMeta::new_readonly(
                        pubkey!("Sysvar1nstructions1111111111111111111111111"), //Sysvar1111111111111111111111111111111111111
                        false,
                    ),
                ],
                data,
            },
        );
    }

    // TODO - remove genrating ata
    pub fn add_flashloan_repay(&mut self, mint: &Pubkey, pool: &Pubkey, repay: u64, index: u8) {
        let mut data = vec![4];
        data.extend(repay.to_le_bytes());
        data.push(index);
        self.ixs.push(Instruction {
            program_id: FLASHLOAN_ID,
            accounts: vec![
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(pool, mint),
                    false,
                ),
                AccountMeta::new_readonly(*pool, false),
                AccountMeta::new(
                    spl_associated_token_account::get_associated_token_address(&self.owner, mint),
                    false,
                ),
                AccountMeta::new(self.owner, true),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(
                    pubkey!("Sysvar1nstructions1111111111111111111111111"), //Sysvar1111111111111111111111111111111111111
                    false,
                ),
            ],
            data,
        });
    }

    pub fn add_flashloan_repay_with_ata(
        &mut self,
        pool: &Pubkey,
        ata_pool: &Pubkey,
        ata_owner: &Pubkey,
        repay: u64,
        index: u8,
    ) {
        let mut data = vec![4];
        data.extend(repay.to_le_bytes());
        data.push(index);
        self.ixs.push(Instruction {
            program_id: FLASHLOAN_ID,
            accounts: vec![
                AccountMeta::new(
                    // spl_associated_token_account::get_associated_token_address(
                    //     &pool,
                    //     &mint,
                    // ),
                    *ata_pool, false,
                ),
                AccountMeta::new_readonly(*pool, false),
                AccountMeta::new(
                    *ata_owner,
                    // spl_associated_token_account::get_associated_token_address(
                    //     &self.owner,
                    //     &mint,
                    // ),
                    false,
                ),
                AccountMeta::new(self.owner, true),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(
                    pubkey!("Sysvar1nstructions1111111111111111111111111"), //Sysvar1111111111111111111111111111111111111
                    false,
                ),
            ],
            data,
        });
    }

    pub fn get_swap_ix(
        &mut self,
        program_id: Pubkey,
        mut data: Vec<u8>,
        mut accounts: Vec<AccountMeta>,
        token_account: Pubkey,
        offset: u16,
    ) -> Instruction {
        accounts.insert(0, AccountMeta::new_readonly(token_account, false));
        accounts.insert(1, AccountMeta::new_readonly(program_id, false));
        let offset = offset.to_le_bytes();
        data.insert(0_usize, offset[0]);
        data.insert(1_usize, offset[1]);
        data.insert(0_usize, 0); // tag
        Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts,
            data,
        }
    }

    // TODO: use get_swap_ix
    pub fn add_swap_ix(
        &mut self,
        program_id: Pubkey,
        mut data: Vec<u8>,
        mut accounts: Vec<AccountMeta>,
        token_account: Pubkey,
        offset: u16,
    ) {
        accounts.insert(0, AccountMeta::new_readonly(token_account, false));
        accounts.insert(1, AccountMeta::new_readonly(program_id, false));
        let offset = offset.to_le_bytes();
        data.insert(0_usize, offset[0]);
        data.insert(1_usize, offset[1]);
        data.insert(0_usize, 0); // tag
        self.ixs.push(Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts,
            data,
        });
    }

    pub fn get_swap_with_close_ata_ix(
        &mut self,
        program_id: Pubkey,
        mut data: Vec<u8>,
        mut accounts: Vec<AccountMeta>,
        token_account: Pubkey,
        offset: u16,
    ) -> Instruction {
        accounts.insert(0, AccountMeta::new(token_account, false));
        accounts.insert(1, AccountMeta::new_readonly(program_id, false));
        let offset = offset.to_le_bytes();
        data.insert(0_usize, offset[0]);
        data.insert(1_usize, offset[1]);
        data.insert(0_usize, 2); // tag

        Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts,
            data,
        }
    }

    // TODO: use get_swap_with_close_ata_ix
    pub fn add_swap_with_close_ata_ix(
        &mut self,
        program_id: Pubkey,
        mut data: Vec<u8>,
        mut accounts: Vec<AccountMeta>,
        token_account: Pubkey,
        offset: u16,
    ) {
        accounts.insert(0, AccountMeta::new(token_account, false));
        accounts.insert(1, AccountMeta::new_readonly(program_id, false));
        let offset = offset.to_le_bytes();
        data.insert(0_usize, offset[0]);
        data.insert(1_usize, offset[1]);
        data.insert(0_usize, 2); // tag
        self.ixs.push(Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts,
            data,
        });
    }

    pub fn get_simple_swap_ix(
        &mut self,
        program_id: Pubkey,
        mut data: Vec<u8>,
        mut accounts: Vec<AccountMeta>,
    ) -> Instruction {
        accounts.insert(0, AccountMeta::new_readonly(program_id, false));
        data.reverse();
        data.insert(0_usize, 1); // tag
        Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts,
            data,
        }
    }

    // TODO: use get_simple_swap_ix
    pub fn add_simple_swap_ix(&mut self, program_id: Pubkey, mut data: Vec<u8>, mut accounts: Vec<AccountMeta>) {
        accounts.insert(0, AccountMeta::new_readonly(program_id, false));
        data.reverse();
        data.insert(0_usize, 1); // tag
        self.ixs.push(Instruction {
            program_id: SWAP_PROGRAM_ID,
            accounts,
            data,
        });
    }

    #[inline]
    pub fn ixs(&mut self) -> &[Instruction] {
        &self.ixs
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ixs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ixs.is_empty()
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Instruction> {
        self.ixs.pop()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.ixs = vec![];
    }

    pub fn prepare_tx_old(
        &mut self,
        keypair: &Keypair,
        alts: &[AddressLookupTableAccount],
        blockhash: Hash,
    ) -> anyhow::Result<VersionedTransaction> {
        let msg = v0::Message::try_compile(&keypair.pubkey(), self.ixs(), alts, blockhash);
        if let Ok(msg) = msg {
            let priority_txn = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&keypair]);
            if let Ok(priority_txn) = priority_txn {
                return Ok(priority_txn);
            }
        }
        Err(anyhow::anyhow!("FailedToCreateTransaction"))
    }

    pub fn prepare_tx(
        &mut self,
        keypair: &Keypair,
        alts: &[AddressLookupTableAccount],
        blockhash: Hash,
    ) -> anyhow::Result<VersionedTransaction> {
        if alts.is_empty() {
            return self.prepare_tx_old(keypair, alts, blockhash);
        }
        for i in 0..alts.len() {
            let msg = v0::Message::try_compile(&keypair.pubkey(), self.ixs(), &alts[0..i], blockhash);
            if let Ok(msg) = msg {
                let priority_txn = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&keypair]);
                if let Ok(priority_txn) = priority_txn {
                    let res = check_transaction_size(&priority_txn);
                    if res.is_ok() {
                        return Ok(priority_txn);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("FailedToCreateTransaction"))
    }
}
