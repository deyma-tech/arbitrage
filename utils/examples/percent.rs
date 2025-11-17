use utils::math::Percent;

fn main() {
    let percent = 7.25;

    let value = 100;
    let result = value.percent(percent);
    println!("{}% of {} is {}", percent, value, result);

    let tips_i = [0, 3, 30, 300, 3_000, 30_000, 300_000, 3_000_000, 30_000_000];

    let tips_f = [
        0.003,
        0.03,
        0.3,
        0.0,
        3.0,
        30.0,
        300.0,
        3_000.0,
        30_000.0,
        300_000.0,
        3_000_000.0,
        30_000_000.0,
    ];

    println!();

    for tip in tips_i {
        let res1 = tip.percent(percent);
        let res2 = (tip / 100) * (100 + percent as u64);
        println!("{}% of {:>8} is: {:>8} {:>8}", percent, tip, res1, res2);
    }

    println!();

    for tip in tips_f {
        let res1 = tip.percent(percent);
        let res2 = (tip / 100_f64) * (100_f64 + percent);
        println!("{}% of {:>8} is: {:>8} {:>8}", percent, tip, res1, res2);
    }
}
