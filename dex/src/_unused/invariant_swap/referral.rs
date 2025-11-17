// pub mod whitelist {
//     use anchor_lang::prelude::Pubkey;
//     #[cfg(feature = "jupiter")]
//     use std::str::FromStr;

//     #[allow(unreachable_code)]
//     pub fn contains_owner(_ref_owner: Pubkey) -> bool {
//         #[cfg(feature = "jupiter")]
//         {
//             let jup2 = Pubkey::from_str(&"BUX7s2ef2htTGb2KKoPHWkmzxPj4nTWMWRgs5CSbQxf9").unwrap();
//             return jup2 == _ref_owner;
//         }

//         #[cfg(feature = "all")]
//         return true;

//         #[cfg(feature = "none")]
//         return false;
//     }
// }
