use utils::volumes::generate_decreasing;

pub fn main() {
    let volumes = generate_decreasing(500_000_000_000, 1_000_000, 20, 0.80);
    for volume in volumes {
        println!("{volume},");
    }
}
