pub fn generate(max_volume: u64, min_volume: u64, steps: usize) -> Vec<u64> {
    if steps < 2 {
        return vec![max_volume];
    }

    let max_volume = max_volume - 1; // Adjust to ensure we don't include max_volume itself

    let mut volumes = Vec::with_capacity(steps);
    let step_size = (max_volume - min_volume) as f64 / (steps - 1) as f64;

    for i in 0..steps {
        let vol = max_volume as f64 - i as f64 * step_size;
        volumes.push(vol.round() as u64);
    }

    volumes
}

pub fn generate_decreasing(max_volume: u64, min_volume: u64, steps: usize, factor: f64) -> Vec<u64> {
    if steps < 2 {
        return vec![max_volume];
    }

    let max_volume = max_volume - 1; // Adjust to ensure we don't include max_volume itself

    let mut volumes = Vec::with_capacity(steps);
    let mut current_value = max_volume as f64;

    for _ in 0..steps {
        if current_value < min_volume as f64 {
            break;
        }
        volumes.push(current_value.round() as u64);
        current_value *= factor;
    }

    volumes
}
