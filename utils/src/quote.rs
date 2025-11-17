use questdb::ingress::Buffer;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuoteResult {
    pub amount_a_in: u64,
    pub amount_b_in: u64,
    pub amount_a_out: u64,
    pub amount_b_out: u64,
    pub indices: Option<Vec<i32>>,
    pub compute_units: u32,
}

impl QuoteResult {
    pub fn update_buffer(&self, buffer: &mut Buffer) {
        buffer
            .column_i64("amount_a_in", self.amount_a_in as i64)
            .unwrap()
            .column_i64("amount_b_in", self.amount_b_in as i64)
            .unwrap()
            .column_i64("amount_a_out", self.amount_b_in as i64)
            .unwrap()
            .column_i64("amount_b_out", self.amount_b_in as i64)
            .unwrap();
    }
}
