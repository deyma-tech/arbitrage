use crate::pump_amm::{BuyIxArgs, BuyIxData, SellIxArgs, SellIxData, SellKeys};
use ahash::AHashSet;
use anyhow::Context;
use borsh::BorshDeserialize;
use bytemuck::{Pod, Zeroable};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_sdk::pubkey;
use spl_associated_token_account::get_associated_token_address;
use utils::quote::QuoteResult;

pub const POOL_DISCRIMINATOR: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];
pub const GLOBAL_CONFIG_DISCRIMINATOR: [u8; 8] = [149, 8, 156, 202, 160, 252, 176, 217];

pub const EVENT_AUTHORITY: Pubkey = pubkey!("GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR");

pub const FEE_CONFIG: Pubkey = pubkey!("5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");
pub const PUMP_FEE_PROGRAM: Pubkey = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");

pub const GLOBAL_VOLUME_ACUMULATOR: Pubkey = pubkey!("C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw");
pub const FEE_DISCRIMINATOR: [u8; 8] = [143, 52, 146, 187, 219, 123, 76, 155];

pub const PUMP_FUN_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Pool {
    pub discriminator: [u8; 8],           // 8
    pub pool_bump: u8,                    // 1
    pub index: u16,                       // 2
    pub creator: Pubkey,                  // 32
    pub base_mint: Pubkey,                // 32
    pub quote_mint: Pubkey,               // 32
    pub lp_mint: Pubkey,                  // 32
    pub pool_base_token_account: Pubkey,  // 32
    pub pool_quote_token_account: Pubkey, // 32
    pub lp_supply: u64,                   // 8
    pub coin_creator: Pubkey,             // 32
    /// New PumpSwap fee ABI flag at byte 244 of the pool account.
    pub is_mayhem: bool,
    pub is_cashback: bool,
    // total size: 8 + 1 + 2 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 32 =  256
}

impl Pool {
    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let pool = Pool {
            discriminator: data[0..8].try_into().context("Pump AMM deser failed: discriminator")?,
            pool_bump: data[8],
            index: u16::from_le_bytes(data[9..11].try_into().context("Pump AMM deser failed: index")?),
            creator: Pubkey::new_from_array(data[11..43].try_into().context("Pump AMM deser failed: creator")?),
            base_mint: Pubkey::new_from_array(data[43..75].try_into().context("Pump AMM deser failed: base_mint")?),
            quote_mint: Pubkey::new_from_array(data[75..107].try_into().context("Pump AMM deser failed: quote_mint")?),
            lp_mint: Pubkey::new_from_array(data[107..139].try_into().context("Pump AMM deser failed: lp_mint")?),
            pool_base_token_account: Pubkey::new_from_array(
                data[139..171]
                    .try_into()
                    .context("Pump AMM deser failed: pool_base_token_account")?,
            ),
            pool_quote_token_account: Pubkey::new_from_array(
                data[171..203]
                    .try_into()
                    .context("Pump AMM deser failed: pool_quote_token_account")?,
            ),
            lp_supply: u64::from_le_bytes(data[203..211].try_into().context("Pump AMM deser failed: lp_supply")?),
            coin_creator: if data.len() < 211 + 32 {
                pubkey!("11111111111111111111111111111111")
            } else {
                Pubkey::try_from_slice(
                    data[211..211 + 32]
                        .try_into()
                        .context("Pump AMM deser failed: pool_quote_token_account")?,
                )
                .unwrap_or(pubkey!("11111111111111111111111111111111"))
            },
            is_cashback: data.get(244).copied().unwrap_or(0) != 0,
            is_mayhem: data.get(243).copied().unwrap_or(0) != 0,
        };
        Ok(pool)
    }
}

#[derive(Debug)]
pub struct QuoteInput {
    pub base: u64,
    pub quote: u64, // coin = a
    pub fee: u64,
    //pub fees: Fees,
}

impl utils::pool::Pool<QuoteInput> for Pool {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.pool_quote_token_account, &self.pool_base_token_account]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.pool_quote_token_account, self.pool_base_token_account]
    }

    fn get_a_mint<'a>(&self) -> &Pubkey {
        &self.base_mint
    }

    fn get_b_mint<'a>(&self) -> &Pubkey {
        &self.quote_mint
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        super::calculator::quote(a_to_b, exact_in, amount, input)
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    // TODO
    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        // if a to b false - quote to base - buy
        if exact_in && !a_to_b {
            return BuyIxData(BuyIxArgs {
                // debug!("base amount out: {} max quote: {}", amount, amount_threshold);
                base_amount_out: amount_threshold,
                max_quote_amount_in: amount,
            })
            .try_to_vec()
            .context("Pump AMM: BuyIxData");
        } else if exact_in && a_to_b {
            // base to quote
            return SellIxData(SellIxArgs {
                base_amount_in: amount,
                min_quote_amount_out: amount_threshold,
            })
            .try_to_vec()
            .context("Pump AMM: SellIxData");
        }
        Ok(vec![])
    }

    // TODO ...
    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        // TODO add accounts
        let config_pubkey = optional_account.context("Pump AMM: config_pubkey")?;
        let binding = optional_accounts.context("Pump AMM: binding")?;
        let recipient = binding.first().context("Pump AMM: recipient")?;

        let (coin_creator_vault_authority, _) =
            Pubkey::find_program_address(&["creator_vault".as_ref(), self.coin_creator.as_ref()], &super::ID);
        let coin_creator_vault_ata = get_associated_token_address(&coin_creator_vault_authority, &self.quote_mint);

        let keys = SellKeys {
            pool: pool_pubkey,
            user: signer,
            global_config: config_pubkey,
            base_mint: self.base_mint,
            quote_mint: self.quote_mint,
            user_base_token_account: if a_to_b {
                source_token_account
            } else {
                destination_token_account
            },
            user_quote_token_account: if a_to_b {
                destination_token_account
            } else {
                source_token_account
            },
            pool_base_token_account: self.pool_base_token_account,
            pool_quote_token_account: self.pool_quote_token_account,
            protocol_fee_recipient: *recipient,
            protocol_fee_recipient_token_account: get_associated_token_address(recipient, &self.quote_mint),
            base_token_program: if !allowed_token2022.contains(&self.base_mint) {
                spl_token::ID
            } else {
                spl_token_2022::ID
            },
            quote_token_program: if !allowed_token2022.contains(&self.quote_mint) {
                spl_token::ID
            } else {
                spl_token_2022::ID
            },
            system_program: solana_program::system_program::ID,
            associated_token_program: spl_associated_token_account::ID,
            event_authority: EVENT_AUTHORITY,
            program: super::ID,
            // Coin Creator Vault Ata Writable
            // Coin Creator Vault Authority 8N3GDaZ2iwN65oxVatKTLPNooAVUJTbfiVJ1ahyqwjSk
        };
        // exact_in == true
        if a_to_b {
            Ok(vec![
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
                    is_writable: false,
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
                AccountMeta {
                    pubkey: coin_creator_vault_ata,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: coin_creator_vault_authority,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: FEE_CONFIG,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: PUMP_FEE_PROGRAM,
                    is_signer: false,
                    is_writable: false,
                },
            ])
        } else {
            Ok(vec![
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
                    is_writable: false,
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
                AccountMeta {
                    pubkey: coin_creator_vault_ata,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: coin_creator_vault_authority,
                    is_signer: false,
                    is_writable: false,
                },
                //
                AccountMeta {
                    pubkey: GLOBAL_VOLUME_ACUMULATOR,
                    is_signer: false,
                    is_writable: true,
                },
                // TODO: remove derive (derive lazy)
                AccountMeta {
                    pubkey: {
                        let (user_volume_accumulator, _) =
                            Pubkey::find_program_address(&[b"user_volume_accumulator", signer.as_ref()], &super::ID);
                        user_volume_accumulator
                    },
                    is_signer: false,
                    is_writable: true,
                },
                //
                AccountMeta {
                    pubkey: FEE_CONFIG,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: PUMP_FEE_PROGRAM,
                    is_signer: false,
                    is_writable: false,
                },
            ])
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalConfig {
    pub discriminator: [u8; 8],
    pub admin: Pubkey,
    pub lp_fee_basis_points: u64,
    pub protocol_fee_basis_points: u64,
    pub disable_flags: u8,
    pub protocol_fee_recipients: [Pubkey; 8],
    pub reserved_protocol_fee_recipients: [Pubkey; 8],
}

impl GlobalConfig {
    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let mut pubkeys = vec![];
        for i in 0..8 {
            let start = 57 + (i * 32);
            let end = start + 32;
            let pubkey = Pubkey::new_from_array(
                data[start..end]
                    .try_into()
                    .context("Pump AMM GlobalConfig deser failed: protocol_fee_recipients")?,
            );
            pubkeys.push(pubkey);
        }

        let recipients = TryInto::<[Pubkey; 8]>::try_into(pubkeys);
        let recipients =
            recipients.map_err(|_| anyhow::anyhow!("Pump AMM GlobalConfig deser failed: protocol_fee_recipients."))?;

        let mut reserved_recipients = [Pubkey::default(); 8];
        if data.len() >= 385 + 32 {
            reserved_recipients[0] = Pubkey::new_from_array(
                data[385..417]
                    .try_into()
                    .context("Pump AMM GlobalConfig deser failed: reserved protocol recipient")?,
            );
            for (index, recipient) in reserved_recipients.iter_mut().enumerate().skip(1) {
                let start = 418 + (index - 1) * 32;
                let end = start + 32;
                if end <= data.len() {
                    *recipient = Pubkey::new_from_array(
                        data[start..end]
                            .try_into()
                            .context("Pump AMM GlobalConfig deser failed: reserved protocol recipients")?,
                    );
                }
            }
        }

        let cfg = GlobalConfig {
            discriminator: data[0..8]
                .try_into()
                .context("Pump AMM GlobalConfig deser failed: discriminator")?,
            admin: Pubkey::new_from_array(
                data[8..40]
                    .try_into()
                    .context("Pump AMM GlobalConfig deser failed: admin")?,
            ),
            lp_fee_basis_points: u64::from_le_bytes(
                data[40..48]
                    .try_into()
                    .context("Pump AMM GlobalConfig deser failed: lp_fee_basis_points")?,
            ),
            protocol_fee_basis_points: u64::from_le_bytes(
                data[48..56]
                    .try_into()
                    .context("Pump AMM GlobalConfig deser failed: protocol_fee_basis_points")?,
            ),
            disable_flags: data[56],
            protocol_fee_recipients: recipients,
            reserved_protocol_fee_recipients: reserved_recipients,
        };
        Ok(cfg)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Default)]
#[repr(C, packed)]
pub struct Fees {
    pub lp_fee_bps: u64,       // 8
    pub protocol_fee_bps: u64, // 8
    pub creator_fee_bps: u64,  // 8
                               // total size: 24
}

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Default)]
#[repr(C, packed)]
pub struct FeeTier {
    pub market_cap_lamports_threshold: u128, // 16
    pub fees: Fees,                          // 24
                                             // total size: 40
}

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C, packed)]
pub struct FeeConfig {
    // discriminator: [u8; 8], // 8
    // pub bump: u8,        // 1
    // pub admin: Pubkey,   // 32
    pub flat_fees: Fees,   // 24
    pub padding0: [u8; 4], // 4
    pub fee_tiers: [FeeTier; 25], // 1000
                           // total size: 8 + 1 + 32 + 24 + 1000 = 1065
}

impl FeeConfig {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        // let fees = bytemuck::try_from_bytes::<Fees>(&buf[41..65])
        //     .map_err(|e| anyhow::anyhow!("Pump AMM FeeConfig deser failed: {:?}", e))?;
        // // padding?
        // let chunks = buf[69..1069].chunks_exact(40);
        // let fee_tiers: Vec<FeeTier> = chunks
        //     .map(|chunk| {
        //         let fees = Fees {
        //             lp_fee_bps: u64::from_le_bytes(chunk[16..24].try_into().unwrap()),
        //             protocol_fee_bps: u64::from_le_bytes(chunk[24..32].try_into().unwrap()),
        //             creator_fee_bps: u64::from_le_bytes(chunk[32..40].try_into().unwrap()),
        //         };
        //         let market_cap_lamports_threshold = u128::from_le_bytes(chunk[0..16].try_into().unwrap());
        //         FeeTier {
        //             market_cap_lamports_threshold,
        //             fees,
        //         }
        //     })
        //     .collect::<_>();
        // println!("fee_tiers: {:?}", fee_tiers);
        let fee_config = bytemuck::try_from_bytes::<FeeConfig>(&buf[41..1069])
            .map_err(|e| anyhow::anyhow!("Pump AMM FeeConfig deser failed: {:?}", e))?;
        Ok(*fee_config)
    }
}
