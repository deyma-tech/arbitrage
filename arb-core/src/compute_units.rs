use crate::calculator::ComputeUnitsPerPoolType;

pub const CU_MAX: u64 = 1_400_000;
// pub const CU_PROGRAM_RECORD: u64 = 200;
// pub const CU_PROGRAM_CHECK: u64 = 200;
pub const CU_CREATE_ATA_IDEMPOTENT: u64 = 22_000;
pub const CU_CLOSE_ACCOUNT: u64 = 3_000;
pub const CU_SWAP_CPI: u64 = 3_000;
pub const CU_FLASH_CPI: u64 = 14_000;
pub const CU_RESERVE: u64 = 10_000;

pub const CU_MAX_UNITS_PER_SWAP: u64 = 400_000;

//pub const CU_FLASHLOAN_BORROW_PLUS_REPAY: u64 = CU_FLASHLOAN_BORROW + CU_FLASHLOAN_REPAY;

/// let cu_per_pool_type = calculators
///     .iter()
///     .map(|calculator| calculator.get_compute_units())
///     .collect::<Vec<_>>();
pub fn calculate_compute_units(cu_per_pool_type: &[ComputeUnitsPerPoolType], volume: u64, flash_loan: bool) -> u64 {
    let len = cu_per_pool_type.len() as u64;

    let is_constant = cu_per_pool_type.iter().all(|cu_type| cu_type.is_constant());

    let mut sum = 0;

    if is_constant {
        sum = cu_per_pool_type.iter().map(|e| e.get_value()).sum();
    } else {
        for cu_type in cu_per_pool_type.iter() {
            sum += cu_type.apply_volume(volume);
        }
    }
    sum += (len - 1) * CU_CREATE_ATA_IDEMPOTENT;
    sum += (len - 1) * CU_CLOSE_ACCOUNT;
    sum += CU_SWAP_CPI;

    if flash_loan {
        sum += CU_FLASH_CPI;
    }

    sum += len * CU_RESERVE;

    ((sum / 1000) * 1000).min(CU_MAX)
}
