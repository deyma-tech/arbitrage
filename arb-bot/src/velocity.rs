use std::collections::BTreeMap;

pub struct Velocity {
    pub timestamp_to_n_opportunities: BTreeMap<u64, u64>,
    pub size: usize,
}

impl Velocity {
    pub fn new(size: usize) -> Self {
        Velocity {
            timestamp_to_n_opportunities: Default::default(),
            size,
        }
    }

    pub fn insert(&mut self, timestamp_in_millis: u64, number_of_opportunities: u64) {
        self.timestamp_to_n_opportunities
            .insert(timestamp_in_millis, number_of_opportunities);
        if self.timestamp_to_n_opportunities.len() > self.size {
            // throw away oldest timestamp ...
            self.timestamp_to_n_opportunities.pop_first();
        }
    }

    pub fn get_millis_difference(&self) -> Option<u64> {
        let first = self.timestamp_to_n_opportunities.first_key_value();
        let last = self.timestamp_to_n_opportunities.last_key_value();
        if let (Some((first_timestamp, _)), Some((last_timestamp, _))) = (first, last) {
            let seconds = last_timestamp - first_timestamp;
            return Some(seconds);
        }
        None
    }

    /// Returns average of the number of vectors (opportunities) per second
    pub fn average_vec_per_second(&self) -> u64 {
        let millis = self.get_millis_difference();
        if let Some(millis) = millis {
            if !self.timestamp_to_n_opportunities.is_empty() && millis > 1_000 {
                let seconds = millis as f64 / 1_000.0;
                let vec_total = self.timestamp_to_n_opportunities.len();
                let vec_per_seconds = vec_total as f64 / seconds;
                return vec_per_seconds as u64;
            }
        }
        0
    }

    // pub fn average_opportunity_per_second(&self) -> u64 {
    //     let millis = self.get_millis_difference();
    //     if let Some(millis) = millis {
    //         let sum = self.timestamp_to_n_opportunities.iter().map(|(_, val)|  *val).sum();
    //         let seconds = millis/1_000;
    //         sum/seconds + 1_u64
    //     }
    //     0
    // }

    pub fn calculate_take(&self, limit: u64) -> u64 {
        let vec_per_seconds = self.average_vec_per_second();
        if vec_per_seconds > 0 {
            (limit / vec_per_seconds) + 1
        } else {
            limit
        }
    }
}
