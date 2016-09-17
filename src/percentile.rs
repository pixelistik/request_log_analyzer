pub fn percentile (data: &Vec<i64>, percent: f32) -> i64 {
    let mut data_sorted = data.clone();
    data_sorted.sort();

    let index = (data_sorted.len() as f32 * percent - 1.0).ceil() as usize;

    data_sorted[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_simple() {
        let data = vec![0, 1, 2, 3];

        let result = percentile(&data, 0.5);

        assert_eq!(result, 1);
    }

    #[test]
    fn test_percentile_simple_uneven() {
        let data = vec![4, 5, 7];

        let result = percentile(&data, 0.5);

        assert_eq!(result, 5);
    }

    #[test]
    fn test_percentile_unsorted() {
        let data = vec![5, 7, 3, 2, 8, 0, 4, 1, 9, 5];

        let result = percentile(&data, 0.5);

        assert_eq!(result, 4);
    }
}
