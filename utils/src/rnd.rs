use rand::{distributions::uniform::SampleUniform, Rng};

#[inline(always)]
pub fn rnd_index<T>(slice: &[T]) -> usize {
    if slice.is_empty() {
        panic!("Cannot get random index from empty slice");
    }
    rand::thread_rng().gen_range(0..slice.len())
}

#[inline(always)]
pub fn rnd_element<T>(slice: &[T]) -> &T {
    &slice[rnd_index(slice)]
}

#[inline(always)]
pub fn rnd_element_mut<T>(slice: &mut [T]) -> &mut T {
    let index = rnd_index(slice);
    &mut slice[index]
}

#[inline(always)]
pub fn rnd_vec_index<T>(vec: &Vec<T>) -> usize {
    rnd_index(vec.as_slice())
}

#[inline(always)]
pub fn rnd_vec_element<T>(vec: &Vec<T>) -> &T {
    rnd_element(vec.as_slice())
}

#[inline(always)]
pub fn rnd_range<T>(min: T, max: T) -> T
where
    T: SampleUniform + PartialOrd + Copy,
{
    if min >= max {
        panic!("Invalid range for random number generation");
    }
    rand::thread_rng().gen_range(min..max)
}

#[inline(always)]
pub fn rnd_bool() -> bool {
    rand::thread_rng().gen_bool(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_with_non_empty_slice() {
        let data = [1, 2, 3, 4, 5];
        for _ in 0..100 {
            let idx = rnd_index(&data);
            assert!(idx < data.len(), "Index should be within bounds");
        }
    }

    #[test]
    #[should_panic(expected = "Cannot get random index from empty slice")]
    fn test_index_with_empty_slice() {
        let empty: &[i32] = &[];
        rnd_index(empty);
    }

    #[test]
    fn test_element_returns_valid_reference() {
        let data = [10, 20, 30, 40, 50];
        for _ in 0..50 {
            let elem = rnd_element(&data);
            assert!(data.contains(elem), "Element should be from the slice");
        }
    }

    #[test]
    #[should_panic(expected = "Cannot get random index from empty slice")]
    fn test_element_with_empty_slice() {
        let empty: &[i32] = &[];
        rnd_element(empty);
    }

    #[test]
    fn test_element_mut_returns_valid_mutable_reference() {
        let mut data = [1, 2, 3, 4, 5];
        let original_sum: i32 = data.iter().sum();

        for _ in 0..10 {
            let elem = rnd_element_mut(&mut data);
            let old_value = *elem;
            *elem = 99;
            assert_eq!(*elem, 99, "Mutable reference should allow modification");
            *elem = old_value; // restore original value
        }

        let final_sum: i32 = data.iter().sum();
        assert_eq!(original_sum, final_sum, "Data should be restored to original state");
    }

    #[test]
    #[should_panic(expected = "Cannot get random index from empty slice")]
    fn test_element_mut_with_empty_slice() {
        let empty: &mut [i32] = &mut [];
        rnd_element_mut(empty);
    }

    #[test]
    fn test_vec_index_with_non_empty_vec() {
        let data = vec![1, 2, 3, 4, 5];
        for _ in 0..100 {
            let idx = rnd_vec_index(&data);
            assert!(idx < data.len(), "Index should be within bounds");
        }
    }

    #[test]
    #[should_panic(expected = "Cannot get random index from empty slice")]
    fn test_vec_index_with_empty_vec() {
        let empty: Vec<i32> = vec![];
        rnd_vec_index(&empty);
    }

    #[test]
    fn test_vec_element_returns_valid_reference() {
        let data = vec!["a", "b", "c", "d"];
        for _ in 0..50 {
            let elem = rnd_vec_element(&data);
            assert!(data.contains(elem), "Element should be from the vector");
        }
    }

    #[test]
    #[should_panic(expected = "Cannot get random index from empty slice")]
    fn test_vec_element_with_empty_vec() {
        let empty: Vec<String> = vec![];
        rnd_vec_element(&empty);
    }

    #[test]
    fn test_range_with_valid_bounds() {
        let min = 5;
        let max = 15;

        for _ in 0..100 {
            let value = rnd_range(min, max);
            assert!(
                value >= min && value < max,
                "Value {value} should be in range [{min}..{max})"
            );
        }
    }

    #[test]
    #[should_panic(expected = "Invalid range for random number generation")]
    fn test_range_with_equal_bounds() {
        rnd_range(5, 5);
    }

    #[test]
    #[should_panic(expected = "Invalid range for random number generation")]
    fn test_range_with_inverted_bounds() {
        rnd_range(10, 5);
    }

    #[test]
    fn test_range_with_different_types() {
        // Test with f64
        for _ in 0..100 {
            let value = rnd_range(1.0f64, 10.0f64);
            assert!(
                (1.0..10.0).contains(&value),
                "Float value {value} should be in range [1.0..10.0)"
            );
        }

        // Test with u32
        for _ in 0..100 {
            let value = rnd_range(100u32, 200u32);
            assert!(
                (100..200).contains(&value),
                "u32 value {value} should be in range [100..200)"
            );
        }

        // Test with i64
        for _ in 0..100 {
            let value = rnd_range(-50i64, 50i64);
            assert!(
                (-50..50).contains(&value),
                "i64 value {value} should be in range [-50..50)"
            );
        }
    }

    #[test]
    fn test_range_edge_cases() {
        // Test with very small range
        for _ in 0..100 {
            let value = rnd_range(0, 2);
            assert!(value == 0 || value == 1, "Value {value} should be 0 or 1");
        }

        // Test with negative numbers
        for _ in 0..100 {
            let value = rnd_range(-10, -5);
            assert!((-10..-5).contains(&value), "Value {value} should be in range [-10..-5)");
        }

        // Test single value range (max = min + 1)
        for _ in 0..100 {
            let value = rnd_range(42, 43);
            assert_eq!(value, 42, "Value should always be 42 in range [42..43)");
        }
    }

    #[test]
    fn test_range_distribution() {
        // Test that rnd_range produces reasonably distributed results
        let min = 0;
        let max = 5;
        let mut counts = [0; 5];
        let iterations = 1000;

        for _ in 0..iterations {
            let value = rnd_range(min, max);
            counts[value as usize] += 1;
        }

        // Each value should appear at least once in 1000 iterations
        for (i, &count) in counts.iter().enumerate() {
            assert!(count > 0, "Value {i} should appear at least once");
        }

        // No single value should dominate (less than 80% of all selections)
        for (i, &count) in counts.iter().enumerate() {
            let ratio = count as f64 / iterations as f64;
            assert!(ratio < 0.8, "Value {} appears too frequently: {}%", i, ratio * 100.0);
        }
    }

    #[test]
    fn test_bool_returns_valid_boolean() {
        let mut true_count = 0;
        let mut false_count = 0;
        let iterations = 1000;

        for _ in 0..iterations {
            if rnd_bool() {
                true_count += 1;
            } else {
                false_count += 1;
            }
        }

        assert!(true_count > 0, "Should generate some true values");
        assert!(false_count > 0, "Should generate some false values");
        assert_eq!(
            true_count + false_count,
            iterations,
            "All iterations should be accounted for"
        );

        // With 1000 iterations and 50% probability, we expect roughly equal distribution
        // Allow for some variance (between 30% and 70%)
        let true_ratio = true_count as f64 / iterations as f64;
        assert!(
            (0.3..=0.7).contains(&true_ratio),
            "Boolean distribution should be roughly balanced, got {}% true",
            true_ratio * 100.0
        );
    }

    #[test]
    fn test_distribution_randomness() {
        // Test that index function produces reasonably distributed results
        let data = [0, 1, 2, 3, 4];
        let mut counts = [0; 5];
        let iterations = 1000;

        for _ in 0..iterations {
            let idx = rnd_index(&data);
            counts[idx] += 1;
        }

        // Each index should appear at least once in 1000 iterations
        for (i, &count) in counts.iter().enumerate() {
            assert!(count > 0, "Index {i} should appear at least once");
        }

        // No single index should dominate (less than 80% of all selections)
        for (i, &count) in counts.iter().enumerate() {
            let ratio = count as f64 / iterations as f64;
            assert!(ratio < 0.8, "Index {} appears too frequently: {}%", i, ratio * 100.0);
        }
    }
}
