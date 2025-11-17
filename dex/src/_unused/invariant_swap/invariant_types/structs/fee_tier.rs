use crate::invariant_swap::decimals::FixedPoint;

#[repr(packed)]
#[derive(PartialEq, Default, Debug)]
pub struct FeeTier {
    pub fee: FixedPoint,
    pub tick_spacing: u16,
    pub bump: u8,
}
//size!(FeeTier);
