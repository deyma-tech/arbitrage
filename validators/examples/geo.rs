use std::cmp::Ordering;
use validators::geo::{get_coordinates_bloxroute, get_optimal_bloxroute};
use validators::validators::get_validators_metadata;

pub fn harvesine_distance(
    latitude0_degrees: f64,
    longitude0_degrees: f64,
    latitude1_degrees: f64,
    longitude1_degrees: f64,
) -> f64 {
    let earth_radius_kilometer = 6371.0_f64;

    let latitude0 = latitude0_degrees.to_radians();
    let latitude1 = latitude1_degrees.to_radians();

    let delta_latitude = (latitude0_degrees - latitude1_degrees).to_radians();
    let delta_longitude = (longitude0_degrees - longitude1_degrees).to_radians();

    let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
        + latitude0.cos() * latitude1.cos() * (delta_longitude / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    earth_radius_kilometer * central_angle
}
pub fn main() {
    let metadata = get_validators_metadata();
    let bloxroute = get_coordinates_bloxroute();
    for item in metadata.validators {
        println!("{:?} {:?}", item.latitude, item.longitude);
        let mut distances = vec![];
        for (url, (latitude, longitude)) in bloxroute.iter() {
            let distance = harvesine_distance(item.latitude, item.longitude, *latitude, *longitude);
            distances.push((url.to_string(), distance));
        }
        distances.sort_by(|(_, x), (_, y)| if x >= y { Ordering::Less } else { Ordering::Greater });
        let (bloxroute_url, distance) = distances.pop().unwrap();
        println!(
            "optimal path: {} {} {} {}",
            item.node_key, item.city, bloxroute_url, distance
        )
    }
    let optimal = get_optimal_bloxroute();
    println!("{:?}", optimal);

    //let optimal = get_optimal_jito();
    // println!("{:?}", optimal);
}
