use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
pub const NUMERAIRE_CONFIG_ACCOUNT_DISCM: [u8; 8] = [230, 62, 124, 43, 102, 101, 88, 63];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct NumeraireConfig {
    pub owner: Pubkey,
    pub status: u32,
    pub rate_mints: [Pubkey; 10],
    pub rate_nums: [u32; 10],
    pub rate_denoms: [u32; 10],
    pub padding0: [u8; 12],
    pub padding1: [u8; 1024],
}
#[derive(Clone, Debug, PartialEq)]
pub struct NumeraireConfigAccount(pub NumeraireConfig);
impl NumeraireConfigAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != NUMERAIRE_CONFIG_ACCOUNT_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    NUMERAIRE_CONFIG_ACCOUNT_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(NumeraireConfig::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&NUMERAIRE_CONFIG_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const STABLE_POOL_ACCOUNT_DISCM: [u8; 8] = [239, 91, 93, 162, 171, 14, 42, 66];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct StablePool {
    pub pool_seed: Pubkey,
    pub lp_mint: Pubkey,
    pub whitelisted_adder: Pubkey,
    pub owner: Pubkey,
    pub inv_t: u64,
    pub inv_t_max: u64,
    pub pairs: [VirtualStablePair; 10],
    pub weights: [u32; 10],
    pub total_weight: u64,
    pub status: u32,
    pub fee_num: u32,
    pub fee_denom: u32,
    pub decimals: u8,
    pub num_stables: u8,
    pub padding0: [u8; 2],
    pub padding1: [u8; 128],
}
#[derive(Clone, Debug, PartialEq)]
pub struct StablePoolAccount(pub StablePool);
impl StablePoolAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != STABLE_POOL_ACCOUNT_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    STABLE_POOL_ACCOUNT_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(StablePool::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&STABLE_POOL_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const VIRTUAL_STABLE_PAIR_ACCOUNT_DISCM: [u8; 8] = [112, 153, 135, 223, 53, 247, 129, 101];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct VirtualStablePair {
    pub pair_authority: Pubkey,
    pub x_reserve_amount: u64,
    pub y_reserve: u64,
    pub curve_amp: u128,
    pub curve_a: u128,
    pub curve_b: u128,
    pub inv_l: u128,
    pub owner: Pubkey,
    pub x_mint: Pubkey,
    pub x_vault: Pubkey,
    pub curve_alpha: u64,
    pub curve_beta: u64,
    pub newest_rate_num: u32,
    pub newest_rate_denom: u32,
    pub decimals: u8,
    pub pair_index: u8,
    pub x_is_2022: u8,
    pub padding0: [u8; 5],
    pub padding1: [u8; 128],
}
#[derive(Clone, Debug, PartialEq)]
pub struct VirtualStablePairAccount(pub VirtualStablePair);
impl VirtualStablePairAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != VIRTUAL_STABLE_PAIR_ACCOUNT_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    VIRTUAL_STABLE_PAIR_ACCOUNT_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(VirtualStablePair::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&VIRTUAL_STABLE_PAIR_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
