use std::time::Instant;

use rand::Rng;
use utils::queue::Queue;

fn main() {
    const Q_LEN: usize = 10_000;
    const I_LEN: usize = 1_000_000;

    let start = Instant::now();
    let mut queue = Queue::new(Q_LEN);
    for _ in 0..I_LEN {
        let range1: u64 = rand::thread_rng().gen_range(10_000..100_000);
        let range2: u64 = rand::thread_rng().gen_range(10_000..1_000_000_000);

        let random_range = rand::thread_rng().gen_range(0..=5);
        queue.push(if random_range == 5 { range2 } else { range1 });

        let _avg: u64 = queue.average();
    }
    let elapsed = start.elapsed();
    println!("Avg {:>8?}, took {:?}", queue.average(), elapsed);

    // let start = Instant::now();
    // let mut queue = QueueBM::new(Q_LEN);
    // for _ in 0..I_LEN {
    //     let range1: u64 = rand::thread_rng().gen_range(10_000..100_000);
    //     let range2: u64 = rand::thread_rng().gen_range(10_000..1_000_000_000);

    //     let random_range = rand::thread_rng().gen_range(0..=5);
    //     queue.push(if random_range == 5 { range2 } else { range1 });

    //     let _avg: u64 = queue.average();
    // }
    // let elapsed = start.elapsed();
    // println!("AvgBM {:>8?}, took {:?}", queue.average(), elapsed);
}
