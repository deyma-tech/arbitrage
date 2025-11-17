use solana_program::instruction::{AccountMeta, Instruction};
use {
    record::state::RecordData,
    solana_program_test::*,
    solana_sdk::{
        account::Account,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    },
};

use record::instruction::RecordInstruction;
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program_option::COption;
use spl_token::state::Account as TokenAccount;

async fn initialize_storage_account(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    account: &Keypair,
    read_account: &Pubkey,
    data: &[u8],
) {
    let account_length = std::mem::size_of::<RecordData>().checked_add(data.len()).unwrap();
    let custom_program_id = Pubkey::new_from_array(record::ID);
    let data = RecordInstruction::Initialize.pack();
    let ix = Instruction {
        program_id: custom_program_id,
        accounts: vec![
            AccountMeta::new(account.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), false),
        ],
        data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                1.max(Rent::default().minimum_balance(account_length)),
                account_length as u64,
                &custom_program_id,
            ),
            ix,
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, account],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(transaction).await.unwrap();

    let data = RecordInstruction::WriteU64 { offset: 64 }.pack();
    let ix = Instruction {
        program_id: custom_program_id,
        accounts: vec![
            AccountMeta::new(account.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(*read_account, false),
        ],
        data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(transaction).await.unwrap();
}

#[tokio::test]
async fn initialize_success() {
    let custom_program_id = Pubkey::new_from_array(record::ID);
    let mut program_test = ProgramTest::new("record", custom_program_id, None);

    let read_account_pubkey = Pubkey::new_unique();
    let read_account = TokenAccount {
        mint: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        amount: 999,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
        delegate: COption::None,
    };

    let mut read_account_data: [u8; 165] = [0; 165];
    TokenAccount::pack(read_account, &mut read_account_data).unwrap();
    program_test.add_account(
        read_account_pubkey,
        Account {
            lamports: 100,
            data: read_account_data.to_vec(), //bytemuck::bytes_of(&read_account),
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context: ProgramTestContext = program_test.start_with_context().await;

    let authority = Keypair::new();
    let account = Keypair::new();
    let data = &[0u8; 8];
    initialize_storage_account(&mut context, &authority, &account, &read_account_pubkey, data).await;

    let record_account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let record_value = u64::from_le_bytes(
        record_account.data[RecordData::WRITABLE_START_INDEX..(RecordData::WRITABLE_START_INDEX + 8)]
            .try_into()
            .unwrap(),
    );
    assert_eq!(record_value, 999);
}

#[tokio::test]
async fn check_add_success() {
    let custom_program_id = Pubkey::new_from_array(record::ID);
    let mut program_test = ProgramTest::new("record", custom_program_id, None);

    let read_account_pubkey = Pubkey::new_unique();
    let read_account = TokenAccount {
        mint: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        amount: 999,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
        delegate: COption::None,
    };

    let mut read_account_data: [u8; 165] = [0; 165];
    TokenAccount::pack(read_account, &mut read_account_data).unwrap();
    program_test.add_account(
        read_account_pubkey,
        Account {
            lamports: 100,
            data: read_account_data.to_vec(),
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context: ProgramTestContext = program_test.start_with_context().await;

    let authority = Keypair::new();
    let account = Keypair::new();
    let data = &[0u8; 8];
    initialize_storage_account(&mut context, &authority, &account, &read_account_pubkey, data).await;

    let record_account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let record_value = u64::from_le_bytes(
        record_account.data[RecordData::WRITABLE_START_INDEX..(RecordData::WRITABLE_START_INDEX + 8)]
            .try_into()
            .unwrap(),
    );
    assert_eq!(record_value, 999);

    let data = RecordInstruction::CheckAdd {
        offset: 64,
        addition: 0,
    }
    .pack();
    let ix = Instruction {
        program_id: custom_program_id,
        accounts: vec![
            AccountMeta::new(account.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(read_account_pubkey, false),
        ],
        data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        //&[&context.payer],
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    assert!(context.banks_client.process_transaction(transaction).await.is_ok());
}

#[tokio::test]
async fn check_add_fail() {
    let custom_program_id = Pubkey::new_from_array(record::ID);

    let mut program_test = ProgramTest::new("record", custom_program_id, None);

    let read_account_pubkey = Pubkey::new_unique();
    let read_account = TokenAccount {
        mint: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        amount: 999,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
        delegate: COption::None,
    };

    let mut read_account_data: [u8; 165] = [0; 165];
    TokenAccount::pack(read_account, &mut read_account_data).unwrap();
    program_test.add_account(
        read_account_pubkey,
        Account {
            lamports: 100,
            data: read_account_data.to_vec(),
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context: ProgramTestContext = program_test.start_with_context().await;

    let authority = Keypair::new();
    let account = Keypair::new();
    let data = &[0u8; 8];
    initialize_storage_account(&mut context, &authority, &account, &read_account_pubkey, data).await;

    let record_account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let record_value = u64::from_le_bytes(
        record_account.data[RecordData::WRITABLE_START_INDEX..(RecordData::WRITABLE_START_INDEX + 8)]
            .try_into()
            .unwrap(),
    );
    assert_eq!(record_value, 999);

    let data = RecordInstruction::CheckAdd {
        offset: 64,
        addition: 1,
    }
    .pack();
    let ix = Instruction {
        program_id: custom_program_id,
        accounts: vec![
            AccountMeta::new(account.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(read_account_pubkey, false),
        ],
        data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        //&[&context.payer],
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    assert!(context.banks_client.process_transaction(transaction).await.is_err());
}

#[tokio::test]
async fn set_authority_success() {
    let custom_program_id = Pubkey::new_from_array(record::ID);
    let mut program_test = ProgramTest::new("record", custom_program_id, None);

    let read_account_pubkey = Pubkey::new_unique();
    let read_account = TokenAccount {
        mint: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        amount: 999,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
        delegate: COption::None,
    };

    let mut read_account_data: [u8; 165] = [0; 165];
    TokenAccount::pack(read_account, &mut read_account_data).unwrap();
    program_test.add_account(
        read_account_pubkey,
        Account {
            lamports: 100,
            data: read_account_data.to_vec(),
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context: ProgramTestContext = program_test.start_with_context().await;

    let authority = Keypair::new();
    let account = Keypair::new();
    let new_authority = Keypair::new();

    let data = &[0u8; 8];
    initialize_storage_account(&mut context, &authority, &account, &read_account_pubkey, data).await;

    let mut record_account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let old_account_data =
        bytemuck::try_from_bytes_mut::<RecordData>(&mut record_account.data[..RecordData::WRITABLE_START_INDEX])
            .unwrap();

    let data = RecordInstruction::SetAuthority.pack();
    let ix = Instruction {
        program_id: custom_program_id,
        accounts: vec![
            AccountMeta::new(account.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(new_authority.pubkey(), false),
        ],
        data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        //&[&context.payer],
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    assert!(context.banks_client.process_transaction(transaction).await.is_ok());

    let mut record_account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let new_account_data =
        bytemuck::try_from_bytes_mut::<RecordData>(&mut record_account.data[..RecordData::WRITABLE_START_INDEX])
            .unwrap();
    assert_eq!(old_account_data.authority, authority.pubkey().to_bytes());
    assert_eq!(new_account_data.authority, new_authority.pubkey().to_bytes());
}

#[tokio::test]
async fn close_account_success() {
    let custom_program_id = Pubkey::new_from_array(record::ID);
    let mut program_test = ProgramTest::new("record", custom_program_id, None);

    let read_account_pubkey = Pubkey::new_unique();
    let read_account = TokenAccount {
        mint: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        amount: 999,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
        delegate: COption::None,
    };

    let mut read_account_data: [u8; 165] = [0; 165];
    TokenAccount::pack(read_account, &mut read_account_data).unwrap();
    program_test.add_account(
        read_account_pubkey,
        Account {
            lamports: 100,
            data: read_account_data.to_vec(),
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context: ProgramTestContext = program_test.start_with_context().await;

    let authority = Keypair::new();
    let account = Keypair::new();

    let data = &[0u8; 8];
    initialize_storage_account(&mut context, &authority, &account, &read_account_pubkey, data).await;

    let record_account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap()
        .unwrap();

    assert!(record_account.lamports > 0);
    assert!(!record_account.data.is_empty());
    let data = RecordInstruction::CloseAccount.pack();
    let ix = Instruction {
        program_id: custom_program_id,
        accounts: vec![
            AccountMeta::new(account.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new(authority.pubkey(), false),
        ],
        data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        //&[&context.payer],
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    assert!(context.banks_client.process_transaction(transaction).await.is_ok());

    let record_account = context.banks_client.get_account(account.pubkey()).await.unwrap();
    assert!(record_account.is_none());
}
