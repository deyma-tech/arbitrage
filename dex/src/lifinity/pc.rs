use bytemuck::{Pod, Zeroable};

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct AccKey {
    pub val: [u8; 32],
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C)]
#[allow(dead_code)]
pub enum PriceStatus {
    Unknown,
    #[default]
    Trading,
    Halted,
    Auction,
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C)]
pub enum CorpAction {
    #[default]
    NoCorpAct,
}

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct PriceInfo {
    pub price: i64,           // 8bytes
    pub conf: u64,            // 8bytes
    pub status: PriceStatus,  // 1byte
    pub corp_act: CorpAction, // 1byte
    pub pub_slot: u64,        // 8bytes
}
// total bytes: 26

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct PriceComp {
    publisher: AccKey, // 32 bytes
    agg: PriceInfo,
    latest: PriceInfo,
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C)]
#[allow(dead_code, clippy::upper_case_acronyms)]
pub enum PriceType {
    Unknown,
    #[default]
    Price,
    TWAP,
    Volatility,
}

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct Price {
    pub magic: u32,            // Pyth magic number. 4bytes
    pub ver: u32,              // Program version. 4bytes
    pub atype: u32,            // Account type. 4bytes
    pub size: u32,             // Price account size. 4bytes
    pub ptype: PriceType,      // Price or calculation type. 4bytes
    pub expo: i32,             // Price exponent. 4bytes
    pub num: u32,              // Number of component prices. 4bytes
    pub unused: u32,           // Unused. 4bytes
    pub curr_slot: u64,        // Currently accumulating price slot. 8bytes
    pub valid_slot: u64,       // Valid slot-time of agg. price. 8bytes
    pub twap: i64,             // Time-weighted average price. 8bytes
    pub avol: u64,             // Annualized price volatility. 8bytes
    pub drv0: i64,             // Space for future derived values. 8bytes
    pub drv1: i64,             // Space for future derived values. 8bytes
    pub drv2: i64,             // Space for future derived values. 8bytes
    pub drv3: i64,             // Space for future derived values. 8bytes
    pub drv4: i64,             // Space for future derived values. 8bytes
    pub drv5: i64,             // Space for future derived values. 8bytes
    pub prod: AccKey,          // Product account key.  8bytes
    pub next: AccKey,          // Next Price account in linked list. 8bytes
    pub agg_pub: AccKey,       // Quoter who computed last aggregate price. 32 bytes
    pub agg: PriceInfo,        // Aggregate price info. 26bytes
    pub comp: [PriceComp; 32], // Price components one per quoter. 26bytes * 32 = 832 bytes
}
// total bytes: 1024

// impl Price {
//     #[inline]
//     pub fn load<'a>(
//         price_feed: &'a AccountInfo,
//     ) -> std::result::Result<RefMut<'a, Price>, ProgramError> {
//         let account_data: RefMut<'a, [u8]> =
//             RefMut::map(price_feed.try_borrow_mut_data().un_wrap(), |data| *data);
//
//         let state: RefMut<'a, Self> = RefMut::map(account_data, |data| {
//             from_bytes_mut(cast_slice_mut::<u8, u8>(try_cast_slice_mut(data).un_wrap()))
//         });
//         Ok(state)
//     }
// }

#[cfg(target_endian = "little")]
unsafe impl Zeroable for Price {}

#[cfg(target_endian = "little")]
unsafe impl Pod for Price {}
