use utils::rnd::rnd_range;

use crate::questdb::ExecutionProviderType;

pub const TRANSACTION_FEE: u64 = 5_000;
pub const TRANSACTION_FEE_I64: i64 = 5_000;
pub const MIN_COMPUTE_UNIT_PRICE: u64 = 1;

#[derive(Debug, Clone)]
pub struct TipInput {
    pub diff: i64,
    pub max_tip: u64,
    // In percent
    pub priority_fee_pct: u64,
    // In percent, lower bound
    pub min_ratio: u64,
    // In percent, upper bound
    pub max_ratio: u64,
    pub provider: ExecutionProviderType,
    pub compute_unit_limit: u64,
    pub max_priority_fee: Option<u64>,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct TipResult {
    pub total_tip: u64,
    pub provider_tip: u64,
    pub compute_unit_price: u64,
    pub compute_unit_limit: u64,
    pub priority_fee: u64,
}

impl TipResult {
    /// Recompute the priority component after simulation changes the final CU
    /// limit. Solana charges priority fees from the requested limit, so the
    /// net-profit guard must use this final value.
    pub fn set_final_compute_unit_limit(&mut self, compute_unit_limit: u64) {
        self.compute_unit_limit = compute_unit_limit;
        self.priority_fee = self
            .compute_unit_price
            .saturating_mul(compute_unit_limit)
            .div_ceil(1_000_000);
        self.total_tip = self
            .provider_tip
            .saturating_add(self.priority_fee)
            .saturating_add(TRANSACTION_FEE);
    }
}

pub fn compute_tip(input: &TipInput) -> anyhow::Result<TipResult> {
    if input.provider == ExecutionProviderType::BloxroutePaladin {
        return compute_tip_bloxroute_paladin(input);
    }

    let profit = input.diff; // - TRANSACTION_FEE_I64;
    if profit <= 0 {
        return Err(anyhow::format_err!("ProfitTooLow"));
    }

    let profit = profit as u64;
    let min_tip = input.provider.get_min_tip() + TRANSACTION_FEE;

    if profit <= min_tip {
        return Err(anyhow::format_err!("ProfitLowerThanMinTip"));
    }

    let tip_percent = rnd_range(input.min_ratio, input.max_ratio);
    let mut tag_tip_percent = (tip_percent / 10) * 10;
    if tag_tip_percent == 100 {
        tag_tip_percent = 0;
    }

    let mut tip_result = TipResult::default();

    let total_tip = (((profit - min_tip) / 100 * tip_percent) + min_tip).min(input.max_tip);

    let mut priority_fee = (total_tip * input.priority_fee_pct / 100) as u64;
    if let Some(max_priority_fee) = input.max_priority_fee {
        priority_fee = priority_fee.min(max_priority_fee);
    }

    if priority_fee > total_tip {
        return Err(anyhow::format_err!("PriorityFeeTooHigh"));
    }

    let compute_unit_price = ((priority_fee * 1_000_000) / input.compute_unit_limit).max(MIN_COMPUTE_UNIT_PRICE);
    let mut compute_unit_limit = (input.compute_unit_limit / 100) * 100;

    compute_unit_limit += input.provider.get_tag();
    compute_unit_limit += tag_tip_percent;

    let priority_fee = compute_unit_price * compute_unit_limit / 1_000_000;
    let provider_tip = total_tip;

    if provider_tip + TRANSACTION_FEE < min_tip {
        return Err(anyhow::format_err!("ProviderTipTooLow"));
    }

    tip_result.compute_unit_limit = compute_unit_limit;
    tip_result.total_tip = provider_tip + priority_fee + TRANSACTION_FEE;
    tip_result.provider_tip = provider_tip;
    tip_result.compute_unit_price = compute_unit_price;
    tip_result.priority_fee = priority_fee;
    Ok(tip_result)
}

fn compute_tip_bloxroute_paladin(input: &TipInput) -> anyhow::Result<TipResult> {
    let profit = input.diff - TRANSACTION_FEE_I64;
    if profit <= 0 {
        return Err(anyhow::format_err!("ProfitTooLow"));
    }

    let profit = profit as u64;
    let min_tip = input.provider.get_min_tip() + ((input.provider.get_min_tip() / 100) * 80) + TRANSACTION_FEE;

    if profit <= min_tip {
        return Err(anyhow::format_err!("ProfitLowerThanMinTip"));
    }

    let tip_percent = rnd_range(input.min_ratio, input.max_ratio);
    let mut tag_tip_percent = (tip_percent / 10) * 10;
    if tag_tip_percent == 100 {
        tag_tip_percent = 0;
    }

    let mut tip_result = TipResult::default();

    let total_tip = (((profit - min_tip) * tip_percent / 100) + min_tip).max(input.max_tip);

    let mut priority_fee = total_tip * input.priority_fee_pct / 100;

    priority_fee = priority_fee.max((input.provider.get_min_tip() / 100) * 80);

    if priority_fee > total_tip {
        return Err(anyhow::format_err!("PriorityFeeTooHigh"));
    }

    let compute_unit_price = ((priority_fee * 1_000_000) / input.compute_unit_limit).max(MIN_COMPUTE_UNIT_PRICE);
    let mut compute_unit_limit = (input.compute_unit_limit / 100) * 100;

    compute_unit_limit += input.provider.get_tag();
    compute_unit_limit += tag_tip_percent;

    let priority_fee = compute_unit_price * compute_unit_limit / 1_000_000;
    let provider_tip = total_tip - priority_fee - TRANSACTION_FEE;

    if provider_tip + TRANSACTION_FEE + priority_fee < min_tip {
        return Err(anyhow::format_err!("ProviderTipTooLow"));
    }
    tip_result.compute_unit_limit = compute_unit_limit;
    tip_result.total_tip = total_tip;
    tip_result.provider_tip = provider_tip;
    tip_result.compute_unit_price = compute_unit_price;
    tip_result.priority_fee = priority_fee;
    Ok(tip_result)
}
