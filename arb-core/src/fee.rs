/// If we have 1SOL, we will pay max (amount / 100 * RATIO)
const RATIO: u64 = 80;

/// 20 SOL
const TWENTY_SOL: u64 = 20_000_000_000;
const ONE_SOL: f64 = 1_000_000_000_f64;

pub fn calculate_max_fee(amount: u64) -> u64 {
    if amount <= TWENTY_SOL {
        return (amount / 100) * RATIO;
    }

    let one_sol_log = ONE_SOL.ln();
    let amount_log = ((amount - TWENTY_SOL) as f64).ln();
    let parcial_fee = amount_log - one_sol_log;
    if parcial_fee.is_sign_negative() {
        return (amount / 100) * RATIO;
    }
    let log_fee = (parcial_fee * ONE_SOL) as u64;
    (log_fee * 2) + (TWENTY_SOL / 100 * RATIO)
}

// pub fn calculate_max_fee_v2(amount_to_borrow: u64) -> u64 {
//     if amount_to_borrow < 527_000_000_000 {
//         ((527_000_000_000 - amount_to_borrow) / 100) * RATIO
//     } else {
//         1
//     }
// }
