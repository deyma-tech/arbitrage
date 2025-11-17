use bytemuck::{Pod, Zeroable};

use crate::invariant_swap::decimals::*;

const SIZE: u16 = 256; // UPDATE IN ARRAYS AS WELL!

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Oracle {
    pub data: [Record; 256],
    pub head: u16,
    pub amount: u16,
    pub size: u16,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Record {
    pub timestamp: u64,
    pub price: Price,
}

impl Oracle {
    pub fn add_record(&mut self, timestamp: u64, price: Price) {
        let record = Record { timestamp, price };

        self.head = (self.head + 1) % self.size;
        self.data[self.head as usize] = record;

        if self.amount < self.size {
            self.amount += 1;
        }
    }

    pub fn init(&mut self) {
        self.size = SIZE;
        self.head = SIZE - 1;
    }

    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {        
        let mut buf = &buf[8..];
        let res = bytemuck::try_from_bytes::<Oracle>(&mut buf).map_err(|_| {
            anyhow::anyhow!("FailedToDeserializeInvariantSwapOracle")
        })?;
        Ok(*res)
    }

}
