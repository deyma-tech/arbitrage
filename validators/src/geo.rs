use crate::constants::{BLOXROUTE_PATHS, JITO_PATHS};
use crate::validators::get_validators_metadata;
use ahash::AHashMap;
use std::cmp::Ordering;

pub fn get_coordinates_bloxroute() -> AHashMap<String, (f64, f64)> {
    let mut coord_map = AHashMap::new();
    for (url, coordinates) in BLOXROUTE_PATHS.iter() {
        let splits = coordinates.split(",").collect::<Vec<_>>();
        let latitude: f64 = splits[0].parse().unwrap();
        let longitude: f64 = splits[1].parse().unwrap();
        coord_map.insert(url.to_string(), (latitude, longitude));
    }
    coord_map
}

pub fn get_coordinates_jito() -> AHashMap<String, (f64, f64)> {
    let mut coord_map = AHashMap::new();
    for (_jito_quicknode, jito, coordinates) in JITO_PATHS.iter() {
        let splits = coordinates.split(",").collect::<Vec<_>>();
        let latitude: f64 = splits[0].parse().unwrap();
        let longitude: f64 = splits[1].parse().unwrap();
        coord_map.insert(jito.to_string(), (latitude, longitude));
    }
    coord_map
}

pub fn get_coordinates_jito_quicknode() -> AHashMap<String, (f64, f64)> {
    let mut coord_map = AHashMap::new();
    for (jito_quicknode, _jito, coordinates) in JITO_PATHS.iter() {
        let splits = coordinates.split(",").collect::<Vec<_>>();
        let latitude: f64 = splits[0].parse().unwrap();
        let longitude: f64 = splits[1].parse().unwrap();
        coord_map.insert(jito_quicknode.to_string(), (latitude, longitude));
    }
    coord_map
}

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

pub fn get_optimal_bloxroute() -> AHashMap<String, String> {
    let metadata = get_validators_metadata();
    let bloxroute = get_coordinates_bloxroute();
    let mut node_to_bloxroute = AHashMap::new();
    for item in metadata.validators {
        let mut distances = vec![];
        for (url, (latitude, longitude)) in bloxroute.iter() {
            let distance = harvesine_distance(item.latitude, item.longitude, *latitude, *longitude);
            distances.push((url.to_string(), distance));
        }
        distances.sort_by(|(_, x), (_, y)| if x >= y { Ordering::Less } else { Ordering::Greater });
        let (bloxroute_url, _distance) = distances.pop().unwrap();
        // println!("optimal path: {} {} {} {}", item.node_key, item.city, bloxroute_url, distance)
        node_to_bloxroute.insert(item.node_key, bloxroute_url);
    }
    node_to_bloxroute
}

pub fn get_optimal_jito() -> AHashMap<String, String> {
    let metadata = get_validators_metadata();
    let jito = get_coordinates_jito();
    let mut node_to_jito = AHashMap::new();
    for item in metadata.validators {
        let mut distances = vec![];
        for (url, (latitude, longitude)) in jito.iter() {
            let distance = harvesine_distance(item.latitude, item.longitude, *latitude, *longitude);
            distances.push((url.to_string(), distance));
        }
        distances.sort_by(|(_, x), (_, y)| if x >= y { Ordering::Less } else { Ordering::Greater });
        let (jito, _distance) = distances.pop().unwrap();
        // println!("optimal path: {} {} {} {}", item.node_key, item.city, jito, _distance);
        node_to_jito.insert(item.node_key, jito);
    }
    node_to_jito
}

pub fn get_optimal_jito_quicknode() -> AHashMap<String, String> {
    let metadata = get_validators_metadata();
    let jito = get_coordinates_jito_quicknode();
    let mut node_to_jito = AHashMap::new();
    for item in metadata.validators {
        let mut distances = vec![];
        for (url, (latitude, longitude)) in jito.iter() {
            let distance = harvesine_distance(item.latitude, item.longitude, *latitude, *longitude);
            distances.push((url.to_string(), distance));
        }
        distances.sort_by(|(_, x), (_, y)| if x >= y { Ordering::Less } else { Ordering::Greater });
        let (jito, _distance) = distances.pop().unwrap();
        // println!("optimal path: {} {} {} {}", item.node_key, item.city, jito, _distance);
        node_to_jito.insert(item.node_key, jito);
    }
    node_to_jito
}
