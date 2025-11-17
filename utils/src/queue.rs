use std::collections::BTreeSet;

const TOP_PERCENTILE: usize = 99;

#[derive(Debug, Clone)]
pub struct Queue {
    queue: Vec<u64>,
    size: usize,
    sorted: BTreeSet<u64>,
    average: u64,
}

impl Queue {
    pub fn new(size: usize) -> Self {
        Queue {
            queue: Vec::with_capacity(size),
            size,
            sorted: BTreeSet::new(),
            average: 0,
        }
    }

    pub fn push(&mut self, value: u64) {
        if self.queue.len() == self.size {
            let removed = self.queue.remove(0);
            self.sorted.remove(&removed);
            self.average = ((self.average * self.size as u64) - removed) / (self.size as u64 - 1);
        }
        if self.average != 0 {
            self.average = ((self.average * self.queue.len() as u64) + value) / (self.queue.len() as u64 + 1);
        } else {
            self.average = value;
        }
        self.queue.push(value);
        self.sorted.insert(value);
    }

    #[inline(always)]
    pub fn is_filled(&self) -> bool {
        self.queue.len() == self.size
    }

    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        !self.queue.is_empty()
    }

    #[inline(always)]
    pub fn max(&self) -> u64 {
        //self.queue.iter().copied().max()
        self.sorted.iter().last().copied().unwrap_or(0)
    }

    #[inline(always)]
    pub fn average(&self) -> u64 {
        self.average
    }

    #[inline(always)]
    pub fn true_average(&self) -> u64 {
        if !self.sorted.is_empty() {
            return self.sorted.iter().sum::<u64>() / self.sorted.len() as u64;
        }
        0
    }

    #[inline]
    pub fn take_min_for_top_percentile(&self) -> u64 {
        if self.sorted.is_empty() {
            return 0;
        }
        let len = self.sorted.len();
        let percentile_index = len / 100 * TOP_PERCENTILE;
        self.sorted.iter().nth(percentile_index).copied().unwrap_or(0)
    }
}
